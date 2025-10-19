# OpenAI API 集成指南

## 概述

C2RustAgent 现已集成 `async-openai` crate，支持使用真实的 OpenAI API 来生成：
- 模块级文档注释
- 详细的 `unsafe` 代码安全性解释
- 外部 C API 的语义标注

## 快速开始

### 1. 获取 OpenAI API Key

访问 [OpenAI Platform](https://platform.openai.com/api-keys) 创建 API key。

### 2. 设置环境变量

**Windows PowerShell:**
```powershell
$env:OPENAI_API_KEY="sk-your-api-key-here"
```

**Linux/macOS:**
```bash
export OPENAI_API_KEY="sk-your-api-key-here"
```

**持久化配置（可选）：**

Windows: 添加到系统环境变量
```powershell
[System.Environment]::SetEnvironmentVariable('OPENAI_API_KEY', 'sk-your-key', 'User')
```

Linux/macOS: 添加到 `~/.bashrc` 或 `~/.zshrc`
```bash
echo 'export OPENAI_API_KEY="sk-your-key"' >> ~/.bashrc
source ~/.bashrc
```

### 3. 运行演示

**使用真实 API：**
```bash
cargo run --example codegen_with_real_llm
```

**使用 Mock（不需要 API key）：**
```bash
$env:USE_MOCK_LLM="true"
cargo run --example codegen_llm_demo
```

## API 配置

### 模型选择

当前默认使用 `gpt-4o-mini`（性价比高）。可以修改 `src/llm_assists.rs` 中的模型：

```rust
let request = CreateChatCompletionRequestArgs::default()
    .model("gpt-4o-mini")  // 可改为 gpt-4o, gpt-4-turbo 等
    .messages(messages)
    .temperature(0.3)      // 0.0-2.0，越低越确定
    .max_tokens(1000u32)   // 最大响应长度
    .build()?;
```

### 可用模型

| 模型 | 特点 | 价格（每百万 tokens）|
|------|------|---------------------|
| gpt-4o-mini | 快速、便宜 | $0.15/$0.60 (输入/输出) |
| gpt-4o | 平衡性能 | $2.50/$10.00 |
| gpt-4-turbo | 高性能 | $10/$30 |

推荐使用 `gpt-4o-mini` 进行开发和测试。

## 使用方式

### 在代码中使用

```rust
use c2rust_agent::codegen::CodeGenerator;
use c2rust_agent::analysis::AnalysisManager;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建 MIR 和分析结果
    let project_mir = /* ... */;
    let manager = AnalysisManager::new(&project_mir);
    let analysis_results = manager.run_all_passes();

    // 创建代码生成器
    let mut generator = CodeGenerator::new(
        output_path,
        "my_project_rs".to_string()
    );

    // 使用 LLM 增强生成（需要 OPENAI_API_KEY）
    generator.generate_with_llm(&project_mir, &analysis_results).await?;

    // 或使用无 LLM 版本（不需要 API key）
    // generator.generate(&project_mir, &analysis_results)?;

    Ok(())
}
```

### 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `OPENAI_API_KEY` | OpenAI API 密钥 | 必需 |
| `USE_MOCK_LLM` | 设为 "true" 使用 mock | false |

## LLM 集成点

### 1. 模块文档生成

**调用位置：** `generate_module_file_async` in `src/codegen.rs`

**提示词：**
```
这是一个从 C 文件 `tokenize.c` 自动翻译过来的 Rust 模块 `tokenize`。

项目背景：
- 项目名称：chibicc
- 项目功能：A C compiler

请为这个 Rust 模块生成一段高级别的文档注释（使用 `//!` 格式），包括：
1. 简要说明此模块的作用（基于文件名推断）
2. 提醒开发者这是从 C 自动翻译的，可能包含不安全模式
3. 建议在使用前进行安全性审查
4. 简短地说明可能需要特别注意的地方（如指针操作、内存管理等）

请直接返回 Rust 文档注释内容，每行以 `//!` 开头，不超过 10 行。
```

**系统提示：** "你是一个 Rust 编程专家，擅长编写清晰、专业的文档注释。"

### 2. Unsafe 代码注释

**调用位置：** `generate_statement_with_llm` in `src/codegen.rs`

**提示词：**
```
在 C 项目 `chibicc` 的文件 `parse.c` 的 `parse_expr` 函数中，
以下代码片段因为 [使用原始指针和 FFI 调用] 被翻译为了 `unsafe` Rust。

原始 C 代码：
```c
char *p = malloc(10);
```

生成的 Rust 代码：
```rust
let p = libc::malloc(10);
```

请撰写一段详尽的 `// SAFETY:` 注释，包含：
1. **为什么需要 unsafe**：明确说明哪些操作违反了 Rust 的安全保证
2. **不变量要求**：调用者必须维护哪些前置条件和后置条件
3. **潜在风险**：如果违反不变量会导致什么问题（如 UB、内存泄漏等）
4. **正确性论证**：在满足不变量的情况下，为什么这段代码是安全的

请直接返回注释内容，每行以 `//` 开头，简洁但全面（不超过 15 行）。
```

**系统提示：** "你是一个 Rust 安全专家，擅长分析和解释 unsafe 代码的安全性要求。"

### 3. API 语义推断

**调用位置：** `infer_external_api_semantics` in `src/llm_assists.rs`

**提示词：**
```
分析以下 C 头文件内容，并专注于函数 fopen。
根据其命名、参数和可能的注释，推断其资源管理行为。

头文件内容：
```c
FILE* fopen(const char* filename, const char* mode);
```

请回答以下问题并以简洁的标签形式返回推断结果：
1. 返回的指针是否需要调用特定函数来释放？
   - 如果是，返回：[ReturnsNewResource(释放函数名)]
2. 某个参数指针指向的内存是否会被函数接管所有权？
   - 如果是，返回：[TakesOwnership(参数名)]
3. 这个函数是否有副作用？
   - 如果有副作用，返回：[HasSideEffects]
   - 如果是纯函数，返回：[Pure]
4. 参数是否需要满足特定前置条件？
   - 例如：[RequiresNonNull(param1)]

请仅返回标签列表，每行一个，不需要额外解释。
```

**系统提示：** "你是一个 C 语言和系统编程专家，擅长分析 C API 的语义和资源管理行为。"

## 错误处理

所有 LLM 调用都有错误回退机制：

```rust
match call_llm_api(&prompt, Some(system_prompt)).await {
    Ok(response) => {
        // 使用 LLM 响应
    }
    Err(_) => {
        // 回退到 mock 实现
        generate_mock_response()
    }
}
```

**常见错误：**

1. **API Key 无效**
   ```
   Error: OpenAI API 调用失败，请检查 OPENAI_API_KEY 环境变量
   ```
   解决：检查 API key 是否正确设置

2. **网络连接问题**
   ```
   Error: request timed out
   ```
   解决：检查网络连接，可能需要代理

3. **速率限制**
   ```
   Error: Rate limit exceeded
   ```
   解决：等待一段时间或升级 API 计划

4. **Token 超限**
   ```
   Error: maximum context length exceeded
   ```
   解决：减少提示词长度或增加 max_tokens

## 成本估算

### 典型使用场景

假设生成一个包含 10 个函数的 Rust 项目：

- 模块文档生成：1 次调用，~500 tokens
- Unsafe 注释：每个函数 ~300 tokens × 5 个 unsafe 函数 = 1500 tokens
- API 语义推断：每个外部函数 ~200 tokens × 3 个 = 600 tokens

**总计：** ~2600 tokens

**使用 gpt-4o-mini：**
- 输入：~2000 tokens × $0.15/1M = $0.0003
- 输出：~600 tokens × $0.60/1M = $0.00036
- **总成本：** ~$0.00066 (不到 1 美分)

### 优化建议

1. **缓存结果**：相同的函数不重复调用
2. **批量处理**：合并多个小请求
3. **使用 mock 模式**：开发时使用 `USE_MOCK_LLM=true`
4. **选择合适模型**：日常开发使用 gpt-4o-mini

## 高级配置

### 自定义提示词

修改 `src/llm_assists.rs` 中的提示词模板：

```rust
let prompt = format!(
    r#"你的自定义提示词
    
    C 代码：
    {}
    
    请生成..."#,
    c_code
);
```

### 添加新的 LLM 提供商

支持其他 LLM（如 Anthropic Claude、本地模型）：

```rust
// 在 call_llm_api 中添加新的客户端
match env::var("LLM_PROVIDER").unwrap_or_default().as_str() {
    "anthropic" => {
        // 使用 Anthropic API
    }
    "local" => {
        // 使用本地模型
    }
    _ => {
        // 默认使用 OpenAI
    }
}
```

## 测试

### 运行测试（Mock 模式）

```bash
$env:USE_MOCK_LLM="true"
cargo test --lib llm_assists
```

### 测试真实 API

```bash
$env:OPENAI_API_KEY="sk-your-key"
cargo test --lib llm_assists --ignored
```

## 故障排查

### 问题：API 调用很慢

**可能原因：**
- 网络延迟
- 模型响应时间

**解决方案：**
- 使用更快的模型（gpt-4o-mini）
- 实现并发调用
- 添加超时机制

### 问题：生成的注释质量不佳

**可能原因：**
- 提示词不够明确
- 上下文信息不足

**解决方案：**
- 改进提示词
- 增加示例
- 调整温度参数

### 问题：成本过高

**解决方案：**
- 使用 gpt-4o-mini
- 实现结果缓存
- 减少不必要的调用
- 优化提示词长度

## 相关资源

- [OpenAI API 文档](https://platform.openai.com/docs)
- [async-openai Crate](https://docs.rs/async-openai)
- [OpenAI 定价](https://openai.com/pricing)
- [C2RustAgent 文档](../README.md)

## 总结

OpenAI API 集成为 C2RustAgent 提供了强大的语义理解能力：

✅ **优势：**
- 生成高质量的文档和注释
- 理解 C 代码的语义和意图
- 提供详细的安全性论证
- 支持多种编程场景

⚠️ **注意：**
- 需要有效的 API key
- 产生少量使用成本
- 依赖网络连接
- 响应时间可能较长

💡 **建议：**
- 开发时使用 mock 模式
- 生产部署前测试 API 集成
- 实现结果缓存以降低成本
- 定期审查生成的注释质量
