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
async fn infer_external_api_semantics_mock(
    function_name: &str,
    _prompt: &str,
) -> Vec<String> {
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
            vec![
                "[Pure]".to_string(),
                "[RequiresNonNull(str)]".to_string(),
            ]
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
    Ok(format!("LLM 响应（占位）：{}", &prompt[..50.min(prompt.len())]))
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
        let result = infer_external_api_semantics("fopen", "FILE* fopen(const char*, const char*);").await;
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
}
