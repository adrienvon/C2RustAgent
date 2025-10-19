//! 演示如何使用 LLM 推断外部 API 语义
//!
//! 运行方式：
//! ```bash
//! cargo run --example llm_external_api_demo
//! ```

use c2rust_agent::llm_assists;

#[tokio::main]
async fn main() {
    println!("=== LLM 外部 API 语义推断演示 ===\n");

    // 示例 1: malloc
    println!("1. 分析 malloc:");
    let malloc_header = "void* malloc(size_t size);";
    let malloc_semantics = llm_assists::infer_external_api_semantics("malloc", malloc_header).await;
    for tag in &malloc_semantics {
        println!("   {}", tag);
    }

    // 示例 2: fopen
    println!("\n2. 分析 fopen:");
    let fopen_header = r#"
        FILE* fopen(const char* filename, const char* mode);
        /* Opens a file and returns a file pointer */
    "#;
    let fopen_semantics = llm_assists::infer_external_api_semantics("fopen", fopen_header).await;
    for tag in &fopen_semantics {
        println!("   {}", tag);
    }

    // 示例 3: strlen
    println!("\n3. 分析 strlen:");
    let strlen_header = "size_t strlen(const char* str);";
    let strlen_semantics = llm_assists::infer_external_api_semantics("strlen", strlen_header).await;
    for tag in &strlen_semantics {
        println!("   {}", tag);
    }

    // 示例 4: 未知函数
    println!("\n4. 分析未知函数 (custom_func):");
    let custom_header = "void custom_func(void* data);";
    let custom_semantics =
        llm_assists::infer_external_api_semantics("custom_func", custom_header).await;
    for tag in &custom_semantics {
        println!("   {}", tag);
    }

    println!("\n=== 演示完成 ===");
    println!("\n说明：");
    println!("- 当前实现使用基于规则的模拟推断");
    println!("- 在生产环境中，将调用实际的 LLM API");
    println!("- 推断结果将被添加到 MIR Statement::Call 节点作为注解");
}
