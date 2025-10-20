//! 增强的 LLM 客户端
//!
//! 支持自定义 base_url、流式响应和 UTF-8 输出（解决 Windows 乱码）

use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tokio_stream::StreamExt;
use tracing::{debug, info};

/// LLM 客户端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// API 基础 URL
    pub base_url: String,
    /// API 密钥
    pub api_key: String,
    /// 模型名称
    pub model: String,
    /// 温度参数 (0.0-1.0)
    pub temperature: f32,
    /// Top-p 采样
    pub top_p: f32,
    /// 最大 token 数
    pub max_tokens: u32,
    /// 是否使用流式响应
    pub stream: bool,
    /// 超时时间（秒）
    pub timeout: u64,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4o-mini".to_string(),
            temperature: 0.6,
            top_p: 0.7,
            max_tokens: 4000,
            stream: true,
            timeout: 120,
        }
    }
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// LLM 客户端
pub struct LlmClient {
    config: LlmConfig,
    client: Client,
}

impl LlmClient {
    /// 创建新的 LLM 客户端
    pub fn new(config: LlmConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .build()
            .context("创建 HTTP 客户端失败")?;

        Ok(Self { config, client })
    }

    /// 从配置文件加载
    pub fn from_toml(path: &str) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("读取配置文件失败: {}", path))?;
        let config: LlmConfig = toml::from_str(&content).context("解析配置文件失败")?;
        Self::new(config)
    }

    /// 发送聊天完成请求（非流式）
    pub async fn chat_completion(&self, messages: Vec<ChatMessage>) -> Result<String> {
        info!("发送 LLM 请求 (非流式), 模型: {}", self.config.model);
        debug!("消息数量: {}", messages.len());

        let url = format!("{}/chat/completions", self.config.base_url);

        let request_body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "temperature": self.config.temperature,
            "top_p": self.config.top_p,
            "max_tokens": self.config.max_tokens,
            "stream": false,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("发送请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API 请求失败: {} - {}", status, error_text));
        }

        let response_json: Value = response.json().await.context("解析响应 JSON 失败")?;

        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("响应中未找到内容"))?
            .to_string();

        info!("收到响应, 长度: {} 字符", content.len());
        Ok(content)
    }

    /// 发送聊天完成请求（流式）
    ///
    /// 返回完整响应，并通过回调函数实时输出每个 chunk
    pub async fn chat_completion_stream<F>(
        &self,
        messages: Vec<ChatMessage>,
        mut on_chunk: F,
    ) -> Result<String>
    where
        F: FnMut(&str),
    {
        info!("发送 LLM 请求 (流式), 模型: {}", self.config.model);
        debug!("消息数量: {}", messages.len());

        let url = format!("{}/chat/completions", self.config.base_url);

        let request_body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "temperature": self.config.temperature,
            "top_p": self.config.top_p,
            "max_tokens": self.config.max_tokens,
            "stream": true,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("发送请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API 请求失败: {} - {}", status, error_text));
        }

        let mut stream = response.bytes_stream();
        let mut full_response = String::new();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.context("读取响应流失败")?;
            let text = String::from_utf8_lossy(&chunk);
            buffer.push_str(&text);

            // 处理 SSE (Server-Sent Events) 格式
            for line in buffer.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];

                    if data == "[DONE]" {
                        continue;
                    }

                    if let Ok(json) = serde_json::from_str::<Value>(data) {
                        if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                            full_response.push_str(content);
                            on_chunk(content);
                        }
                    }
                }
            }

            // 保留最后一行（可能不完整）
            if let Some(last_newline) = buffer.rfind('\n') {
                buffer = buffer[last_newline + 1..].to_string();
            }
        }

        info!("流式响应完成, 总长度: {} 字符", full_response.len());
        Ok(full_response)
    }

    /// 简化的翻译接口
    pub async fn translate_code(
        &self,
        c_code: &str,
        context: &str,
        system_prompt: &str,
    ) -> Result<String> {
        let user_message = format!(
            "{}

C 代码:
```c
{}
```

请生成对应的 Rust 代码。",
            context, c_code
        );

        let messages = vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(user_message),
        ];

        if self.config.stream {
            self.chat_completion_stream(messages, |chunk| {
                // UTF-8 安全输出（解决 Windows 乱码）
                crate::utils::print_utf8(chunk);
            })
            .await
        } else {
            self.chat_completion(messages).await
        }
    }

    /// 修复语法错误
    pub async fn fix_syntax_errors(&self, rust_code: &str, errors: &str) -> Result<String> {
        let messages = vec![
            ChatMessage::system("你是 Rust 编译器错误修复专家。"),
            ChatMessage::user(format!(
                "以下 Rust 代码有编译错误:

```rust
{}
```

编译错误:
```
{}
```

请修复这些错误并返回完整的修正后代码。",
                rust_code, errors
            )),
        ];

        if self.config.stream {
            self.chat_completion_stream(messages, |chunk| {
                crate::utils::print_utf8(chunk);
            })
            .await
        } else {
            self.chat_completion(messages).await
        }
    }

    /// 优化 unsafe 代码
    pub async fn optimize_unsafe(&self, rust_code: &str) -> Result<String> {
        let messages = vec![
            ChatMessage::system("你是 Rust 安全编程专家。"),
            ChatMessage::user(format!(
                "请分析以下 Rust 代码中的 unsafe 块，并尽可能将其改写为安全代码:

```rust
{}
```

要求:
1. 使用 Rust 的安全抽象（引用、Box、Vec 等）替换裸指针
2. 为 FFI 调用创建安全封装函数
3. 添加详细的安全性注释
4. 如果某些 unsafe 无法消除，解释原因

请返回优化后的完整代码。",
                rust_code
            )),
        ];

        if self.config.stream {
            self.chat_completion_stream(messages, |chunk| {
                crate::utils::print_utf8(chunk);
            })
            .await
        } else {
            self.chat_completion(messages).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = LlmConfig::default();
        assert_eq!(config.temperature, 0.6);
        assert_eq!(config.stream, true);
    }

    #[test]
    fn test_chat_message_creation() {
        let msg = ChatMessage::user("test");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "test");
    }

    #[tokio::test]
    async fn test_client_creation() {
        let config = LlmConfig::default();
        let client = LlmClient::new(config);
        assert!(client.is_ok());
    }
}
