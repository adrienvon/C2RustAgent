//! DeepSeek API 测试示例
//!
//! 演示如何使用配置文件调用 DeepSeek API

use anyhow::Result;
use c2rust_agent::llm_assists;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 DeepSeek API 配置测试\n");

    // 测试 1: 推断 malloc 函数的语义
    println!("📝 测试 1: 推断 malloc 函数语义");
    println!("函数签名: void* malloc(size_t size)");

    let header_content = r#"
// 内存分配函数
void* malloc(size_t size);
void free(void* ptr);
    "#;

    let semantics = llm_assists::infer_external_api_semantics("malloc", header_content).await;

    if !semantics.is_empty() {
        println!("✅ 成功获取语义信息:");
        for tag in &semantics {
            println!("   {}", tag);
        }
    } else {
        println!("⚠️  未获取到语义信息（可能是 API 调用失败或网络问题）");
        println!("   提示: 请检查 API Key 和网络连接");
    }

    println!("\n{}", "=".repeat(60));

    // 测试 2: 生成模块文档
    println!("\n📝 测试 2: 生成模块文档");

    match llm_assists::generate_module_documentation(
        "math",
        "math.c",
        "calculator",
        Some("一个简单的计算器程序"),
    )
    .await
    {
        Ok(doc) => {
            println!("✅ 成功生成文档:");
            println!("{}", doc);
        }
        Err(e) => {
            println!("❌ 错误: {}", e);
        }
    }

    println!("\n{}", "=".repeat(60));

    // 测试 3: 生成 unsafe 代码说明
    println!("\n📝 测试 3: 生成 unsafe 代码说明");

    let c_code = "char *p = malloc(10); *p = 'A';";
    let rust_code = "let p = libc::malloc(10) as *mut i8; *p = b'A' as i8;";

    match llm_assists::generate_unsafe_explanation(
        "memory_demo",
        "demo.c",
        "allocate_and_write",
        c_code,
        rust_code,
        "使用原始指针和 FFI 调用",
    )
    .await
    {
        Ok(explanation) => {
            println!("✅ 成功生成说明:");
            println!("{}", explanation);
        }
        Err(e) => {
            println!("❌ 错误: {}", e);
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("\n✨ 测试完成！");
    println!("\n💡 配置信息:");
    println!("   配置文件: ./c2rust-agent.toml");
    println!("   API 地址: https://api.deepseek.com");
    println!("   模型: deepseek-coder");
    println!("   温度: 0.3");
    println!("   最大 tokens: 2000");

    Ok(())
}
