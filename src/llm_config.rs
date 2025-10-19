//! LLM 配置管理模块
//!
//! 支持从配置文件和环境变量读取 LLM API 配置

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// LLM API 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// API 提供商类型（openai, azure, custom）
    #[serde(default = "default_provider")]
    pub provider: String,

    /// OpenAI API Key
    pub api_key: Option<String>,

    /// 自定义 API URL（可选，用于代理或自定义端点）
    pub api_url: Option<String>,

    /// 使用的模型名称
    #[serde(default = "default_model")]
    pub model: String,

    /// 温度参数（0.0-2.0）
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// 最大 token 数
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// 是否使用 Mock 模式
    #[serde(default)]
    pub use_mock: bool,
}

fn default_provider() -> String {
    "openai".to_string()
}

fn default_model() -> String {
    "gpt-4o-mini".to_string()
}

fn default_temperature() -> f32 {
    0.3
}

fn default_max_tokens() -> u32 {
    1000
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            api_key: None,
            api_url: None,
            model: default_model(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            use_mock: false,
        }
    }
}

impl LlmConfig {
    /// 从配置文件和环境变量加载配置
    ///
    /// 优先级（从高到低）：
    /// 1. 环境变量
    /// 2. 用户配置文件 (~/.c2rust-agent/config.toml)
    /// 3. 项目配置文件 (./c2rust-agent.toml)
    /// 4. 默认值
    pub fn load() -> Result<Self> {
        let mut config = config::Config::builder();

        // 1. 加载默认配置
        config = config.add_source(config::Config::try_from(&Self::default())?);

        // 2. 尝试加载项目配置文件
        let project_config = PathBuf::from("c2rust-agent.toml");
        if project_config.exists() {
            config = config.add_source(
                config::File::from(project_config)
                    .required(false)
                    .format(config::FileFormat::Toml),
            );
        }

        // 3. 尝试加载用户配置文件
        if let Some(user_config_path) = Self::user_config_path() {
            if user_config_path.exists() {
                config = config.add_source(
                    config::File::from(user_config_path)
                        .required(false)
                        .format(config::FileFormat::Toml),
                );
            }
        }

        // 4. 从环境变量覆盖
        config = config.add_source(
            config::Environment::with_prefix("C2RUST_AGENT")
                .separator("_")
                .try_parsing(true),
        );

        // 5. 特殊处理常见的环境变量
        let mut settings: LlmConfig = config
            .build()
            .context("构建配置失败")?
            .try_deserialize()
            .context("解析配置失败")?;

        // 环境变量兼容性：OPENAI_API_KEY 和 USE_MOCK_LLM
        if settings.api_key.is_none() {
            settings.api_key = std::env::var("OPENAI_API_KEY").ok();
        }

        if std::env::var("USE_MOCK_LLM").unwrap_or_default() == "true" {
            settings.use_mock = true;
        }

        Ok(settings)
    }

    /// 获取用户配置文件路径
    pub fn user_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("c2rust-agent").join("config.toml"))
    }

    /// 创建示例配置文件
    pub fn create_example_config() -> Result<String> {
        let example = r#"# C2RustAgent LLM 配置文件
# 配置文件位置：
# - 项目目录：./c2rust-agent.toml
# - 用户目录：~/.config/c2rust-agent/config.toml (Linux/macOS)
# - 用户目录：%APPDATA%\c2rust-agent\config.toml (Windows)

# API 提供商：openai, azure, custom
provider = "openai"

# OpenAI API Key
# 也可以通过环境变量 OPENAI_API_KEY 设置
api_key = "sk-your-api-key-here"

# 自定义 API URL（可选）
# 用于代理或自定义 OpenAI 兼容端点
# api_url = "https://api.openai.com/v1"
# api_url = "https://your-proxy.com/v1"

# 使用的模型
# 可选：gpt-4o-mini, gpt-4o, gpt-4-turbo, gpt-3.5-turbo
model = "gpt-4o-mini"

# 温度参数（0.0-2.0）
# 越低越确定，越高越随机
temperature = 0.3

# 最大 token 数
max_tokens = 1000

# 是否使用 Mock 模式（用于测试）
use_mock = false
"#;
        Ok(example.to_string())
    }

    /// 保存配置到用户配置文件
    pub fn save_to_user_config(&self) -> Result<()> {
        let config_path = Self::user_config_path().context("无法确定用户配置目录")?;

        // 创建配置目录
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).context("创建配置目录失败")?;
        }

        // 序列化配置
        let toml = toml::to_string_pretty(self).context("序列化配置失败")?;

        // 写入文件
        std::fs::write(&config_path, toml)
            .context(format!("写入配置文件失败: {:?}", config_path))?;

        Ok(())
    }

    /// 验证配置是否有效
    pub fn validate(&self) -> Result<()> {
        if !self.use_mock && self.api_key.is_none() {
            anyhow::bail!(
                "未设置 API Key。请通过以下方式之一设置：\n\
                1. 配置文件：{:?}\n\
                2. 环境变量：OPENAI_API_KEY\n\
                3. 或设置 use_mock = true 使用测试模式",
                Self::user_config_path()
            );
        }

        if self.temperature < 0.0 || self.temperature > 2.0 {
            anyhow::bail!("temperature 必须在 0.0-2.0 之间");
        }

        if self.max_tokens == 0 {
            anyhow::bail!("max_tokens 必须大于 0");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LlmConfig::default();
        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-4o-mini");
        assert_eq!(config.temperature, 0.3);
        assert_eq!(config.max_tokens, 1000);
        assert!(!config.use_mock);
    }

    #[test]
    fn test_create_example_config() {
        let example = LlmConfig::create_example_config().unwrap();
        assert!(example.contains("api_key"));
        assert!(example.contains("model"));
        assert!(example.contains("temperature"));
    }

    #[test]
    fn test_validate_config() {
        let mut config = LlmConfig::default();
        config.use_mock = true;
        assert!(config.validate().is_ok());

        config.use_mock = false;
        assert!(config.validate().is_err()); // 缺少 API key

        config.api_key = Some("sk-test".to_string());
        assert!(config.validate().is_ok());

        config.temperature = 3.0;
        assert!(config.validate().is_err()); // temperature 超出范围
    }
}
