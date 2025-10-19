//! LLM 辅助模块
//!
//! 提供 LLM 增强功能，用于推断语义、所有权契约等

use anyhow::Result;

/// 为外部库函数推断语义标注
///
/// # 参数
/// - unction_name: 外部函数名称（如 "fopen"）
/// - header_file_content: 头文件内容（如 stdio.h）
///
/// # 返回值
/// 语义标注的向量，例如：
/// - [ReturnsNewResource(fclose)]: 返回需要 fclose 释放的资源
/// - [TakesOwnership(param1)]: 接管参数1的所有权
/// - [HasSideEffects]: 有副作用
/// - [Pure]: 纯函数
///
/// # 示例
/// `ignore
/// let semantics = infer_external_api_semantics("fopen", stdio_h_content).await;
/// // 结果可能是: vec!["[ReturnsNewResource(fclose)]".to_string()]
/// `
pub async fn infer_external_api_semantics(
    function_name: &str,
    header_file_content: &str,
) -> Vec<String> {
    // 构建 LLM 提示词
    let prompt = format!(
        r#"分析以下 C 头文件内容，并专注于函数 {}。根据其命名、参数和可能的注释，推断其资源管理行为。

头文件内容：
`c
{}
`

请回答以下问题并以简洁的标签形式返回推断结果：

1. 返回的指针是否需要调用特定函数（如 ree 或 close）来释放？
   - 如果是，返回：[ReturnsNewResource(释放函数名)]
   
2. 某个参数指针指向的内存是否会被函数接管所有权？
   - 如果是，返回：[TakesOwnership(参数名)]
   
3. 这个函数是否有副作用（如修改全局状态、I/O操作）？
   - 如果有副作用，返回：[HasSideEffects]
   - 如果是纯函数，返回：[Pure]

4. 参数是否需要满足特定前置条件？
   - 例如：[RequiresNonNull(param1)], [RequiresValidPointer(param2)]

请仅返回标签列表，每行一个，不需要额外解释。"#,
        function_name, header_file_content
    );

    // TODO: 实际调用 LLM API（如 OpenAI、Anthropic 等）
    // 这里先返回模拟结果
    infer_external_api_semantics_mock(function_name, &prompt).await
}

/// 模拟 LLM 调用（用于开发/测试）
///
/// 在实际 LLM API 集成前，此函数提供基于规则的简单推断
async fn infer_external_api_semantics_mock(function_name: &str, _prompt: &str) -> Vec<String> {
    // 基于常见 C 标准库函数的简单规则推断
    match function_name {
        "malloc" | "calloc" | "realloc" => {
            vec!["[ReturnsNewResource(free)]".to_string()]
        }
        "fopen" | "fdopen" | "freopen" => {
            vec![
                "[ReturnsNewResource(fclose)]".to_string(),
                "[HasSideEffects]".to_string(),
            ]
        }
        "fclose" => {
            vec![
                "[TakesOwnership(stream)]".to_string(),
                "[HasSideEffects]".to_string(),
            ]
        }
        "free" => {
            vec!["[TakesOwnership(ptr)]".to_string()]
        }
        "strlen" | "strcmp" | "strchr" => {
            vec!["[Pure]".to_string(), "[RequiresNonNull(str)]".to_string()]
        }
        "strcpy" | "strcat" | "memcpy" => {
            vec![
                "[HasSideEffects]".to_string(),
                "[RequiresNonNull(dest)]".to_string(),
                "[RequiresNonNull(src)]".to_string(),
            ]
        }
        "printf" | "fprintf" | "sprintf" | "scanf" | "fscanf" => {
            vec!["[HasSideEffects]".to_string()]
        }
        "getenv" => {
            vec![
                "[HasSideEffects]".to_string(),
                "[ReturnsValidUntil(next_getenv_call)]".to_string(),
            ]
        }
        _ => {
            // 未知函数，默认假设有副作用
            vec!["[HasSideEffects]".to_string(), "[Unknown]".to_string()]
        }
    }
}

/// 为模块生成文档注释
///
/// # 参数
/// - `module_name`: 模块名称（如 "tokenize"）
/// - `file_name`: 原始 C 文件名（如 "tokenize.c"）
/// - `project_name`: 项目名称
/// - `project_summary`: 从 README 或注释提取的项目摘要（可选）
///
/// # 返回值
/// 模块级文档字符串（`//!` 格式）
///
/// # 示例
/// ```ignore
/// let doc = generate_module_documentation("tokenize", "tokenize.c", "chibicc", Some("A C compiler")).await;
/// ```
pub async fn generate_module_documentation(
    module_name: &str,
    file_name: &str,
    project_name: &str,
    project_summary: Option<&str>,
) -> Result<String> {
    let summary_text = project_summary.unwrap_or("一个 C 项目");

    let prompt = format!(
        r#"这是一个从 C 文件 `{}` 自动翻译过来的 Rust 模块 `{}`。

项目背景：
- 项目名称：{}
- 项目功能：{}

请为这个 Rust 模块生成一段高级别的文档注释（使用 `//!` 格式），包括：

1. 简要说明此模块的作用（基于文件名推断）
2. 提醒开发者这是从 C 自动翻译的，可能包含不安全模式
3. 建议在使用前进行安全性审查
4. 简短地说明可能需要特别注意的地方（如指针操作、内存管理等）

请直接返回 Rust 文档注释内容，每行以 `//!` 开头，不超过 10 行。"#,
        file_name, module_name, project_name, summary_text
    );

    // TODO: 实际调用 LLM API
    generate_module_documentation_mock(module_name, file_name, &prompt).await
}

/// 模拟模块文档生成
async fn generate_module_documentation_mock(
    module_name: &str,
    file_name: &str,
    _prompt: &str,
) -> Result<String> {
    let doc = format!(
        r#"//! 模块：{}
//!
//! 此模块从 C 源文件 `{}` 自动翻译而来。
//!
//! ⚠️ **安全性注意事项**：
//! - 此代码可能包含从 C 转换的不安全模式
//! - 指针操作和内存管理需要特别小心
//! - 建议在生产环境使用前进行全面的安全性审查
//!
//! 请参考原始 C 代码以理解具体的实现逻辑和假设前提。
"#,
        module_name, file_name
    );

    Ok(doc)
}

/// 为 unsafe 代码块生成详细的安全注释
///
/// # 参数
/// - `project_name`: 项目名称
/// - `file_name`: C 文件名
/// - `function_name`: 函数名
/// - `c_code`: 原始 C 代码片段
/// - `rust_code`: 生成的 Rust 代码片段
/// - `reason`: 静态分析给出的需要 unsafe 的原因
///
/// # 返回值
/// `// SAFETY:` 注释内容
///
/// # 示例
/// ```ignore
/// let safety_comment = generate_unsafe_explanation(
///     "chibicc",
///     "parse.c",
///     "parse_expr",
///     "char *p = malloc(10);",
///     "let p = libc::malloc(10);",
///     "使用原始指针和 FFI 调用"
/// ).await?;
/// ```
pub async fn generate_unsafe_explanation(
    project_name: &str,
    file_name: &str,
    function_name: &str,
    c_code: &str,
    rust_code: &str,
    reason: &str,
) -> Result<String> {
    let prompt = format!(
        r#"在 C 项目 `{}` 的文件 `{}` 的 `{}` 函数中，以下代码片段因为 [{}] 被翻译为了 `unsafe` Rust。

原始 C 代码：
```c
{}
```

生成的 Rust 代码：
```rust
{}
```

请撰写一段详尽的 `// SAFETY:` 注释，包含：

1. **为什么需要 unsafe**：明确说明哪些操作违反了 Rust 的安全保证
2. **不变量要求**：调用者必须维护哪些前置条件和后置条件
3. **潜在风险**：如果违反不变量会导致什么问题（如 UB、内存泄漏等）
4. **正确性论证**：在满足不变量的情况下，为什么这段代码是安全的

请直接返回注释内容，每行以 `//` 开头，简洁但全面（不超过 15 行）。"#,
        project_name, file_name, function_name, reason, c_code, rust_code
    );

    // TODO: 实际调用 LLM API
    generate_unsafe_explanation_mock(reason, &prompt).await
}

/// 模拟 unsafe 注释生成
async fn generate_unsafe_explanation_mock(reason: &str, _prompt: &str) -> Result<String> {
    let comment = format!(
        r#"// SAFETY: {}
// 
// 不变量要求：
// - 所有指针参数必须是有效的、正确对齐的指针
// - 指针指向的内存必须在整个操作期间保持有效
// - 如果指针被解引用，必须确保没有数据竞争
// 
// 潜在风险：
// - 解引用无效指针会导致未定义行为
// - 并发访问可变数据可能导致数据竞争
// 
// 正确性论证：
// - 此代码从 C 直接翻译，假设 C 代码遵循其自身的内存安全约定
// - 调用方需确保满足 C API 的所有前置条件
"#,
        reason
    );

    Ok(comment)
}

/// 实际 LLM API 调用接口（占位）
///
/// 集成实际 LLM API 时实现此函数
#[allow(dead_code)]
async fn call_llm_api(prompt: &str) -> Result<String> {
    // TODO: 集成实际 LLM API
    // 例如：
    // - OpenAI API
    // - Anthropic Claude API
    // - 本地 LLM 服务

    // 示例返回格式
    Ok(format!(
        "LLM 响应（占位）：{}",
        &prompt[..50.min(prompt.len())]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_infer_malloc_semantics() {
        let result = infer_external_api_semantics("malloc", "void* malloc(size_t size);").await;
        assert!(result.contains(&"[ReturnsNewResource(free)]".to_string()));
    }

    #[tokio::test]
    async fn test_infer_fopen_semantics() {
        let result =
            infer_external_api_semantics("fopen", "FILE* fopen(const char*, const char*);").await;
        assert!(result.contains(&"[ReturnsNewResource(fclose)]".to_string()));
        assert!(result.contains(&"[HasSideEffects]".to_string()));
    }

    #[tokio::test]
    async fn test_infer_strlen_semantics() {
        let result = infer_external_api_semantics("strlen", "size_t strlen(const char*);").await;
        assert!(result.contains(&"[Pure]".to_string()));
        assert!(result.contains(&"[RequiresNonNull(str)]".to_string()));
    }

    #[tokio::test]
    async fn test_infer_unknown_function() {
        let result = infer_external_api_semantics("unknown_func", "void unknown_func();").await;
        assert!(result.contains(&"[HasSideEffects]".to_string()));
        assert!(result.contains(&"[Unknown]".to_string()));
    }

    #[tokio::test]
    async fn test_generate_module_documentation() {
        let result = generate_module_documentation(
            "tokenize",
            "tokenize.c",
            "chibicc",
            Some("A small C compiler"),
        )
        .await;

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert!(doc.contains("//!"));
        assert!(doc.contains("tokenize"));
        assert!(doc.contains("tokenize.c"));
        assert!(doc.contains("安全性"));
    }

    #[tokio::test]
    async fn test_generate_unsafe_explanation() {
        let result = generate_unsafe_explanation(
            "test_project",
            "test.c",
            "test_func",
            "char *p = malloc(10);",
            "let p = libc::malloc(10);",
            "使用原始指针和 FFI 调用",
        )
        .await;

        assert!(result.is_ok());
        let comment = result.unwrap();
        assert!(comment.contains("// SAFETY:"));
        assert!(comment.contains("不变量"));
        assert!(comment.contains("潜在风险"));
    }
}
