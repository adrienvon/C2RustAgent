//! 使用真实 OpenAI API 的代码生成演示
//!
//! 运行前需要设置环境变量：
//! ```bash
//! # Windows PowerShell
//! $env:OPENAI_API_KEY="your-api-key-here"
//! cargo run --example codegen_with_real_llm
//!
//! # Linux/macOS
//! export OPENAI_API_KEY="your-api-key-here"
//! cargo run --example codegen_with_real_llm
//! ```

use anyhow::Result;
use c2rust_agent::analysis::AnalysisManager;
use c2rust_agent::codegen::CodeGenerator;
use c2rust_agent::mir::*;
use std::collections::HashMap;
use std::env;
use tempfile::TempDir;
use tokio;

/// 创建示例 MIR（包含需要详细解释的 unsafe 操作）
fn create_example_mir() -> ProjectMIR {
    let mut project = ProjectMIR {
        functions: HashMap::new(),
        globals: HashMap::new(),
    };

    // 示例：内存分配函数
    let alloc_func = Function {
        name: "safe_allocate".to_string(),
        parameters: vec![Parameter {
            name: "size".to_string(),
            param_type: Type::Int,
            var_id: 0,
        }],
        return_type: Some(Type::Pointer(Box::new(Type::Void))),
        basic_blocks: vec![BasicBlock {
            id: 0,
            statements: vec![Statement::Call(
                LValue::Variable(1),
                "libc::malloc".to_string(),
                vec![RValue::Use(Box::new(LValue::Variable(0)))],
            )],
            terminator: Terminator::Return(Some(RValue::Use(Box::new(LValue::Variable(1))))),
        }],
        annotations: vec![
            "[ReturnsNewResource(free)]".to_string(),
            "[RequiresNonNull(size > 0)]".to_string(),
        ],
        is_static: false,
        is_public: true,
    };

    project
        .functions
        .insert("safe_allocate".to_string(), alloc_func);

    project
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== 使用真实 OpenAI API 的代码生成演示 ===\n");

    // 检查 API key
    if env::var("OPENAI_API_KEY").is_err() {
        eprintln!("❌ 错误：未设置 OPENAI_API_KEY 环境变量");
        eprintln!("\n请先设置 API key：");
        eprintln!("  Windows PowerShell:");
        eprintln!("    $env:OPENAI_API_KEY=\"your-api-key-here\"");
        eprintln!("  Linux/macOS:");
        eprintln!("    export OPENAI_API_KEY=\"your-api-key-here\"");
        eprintln!("\n或者运行 mock 版本：");
        eprintln!("    cargo run --example codegen_llm_demo");
        return Ok(());
    }

    println!("✅ 检测到 OPENAI_API_KEY");
    println!("📡 将使用真实的 OpenAI API 生成文档和注释\n");

    // 创建临时输出目录
    let temp_dir = TempDir::new()?;
    println!("输出目录: {:?}\n", temp_dir.path());

    // 创建示例 MIR
    println!("正在创建示例 MIR...");
    let project_mir = create_example_mir();
    println!("✅ 创建了 {} 个函数\n", project_mir.functions.len());

    // 运行静态分析
    println!("正在运行静态分析...");
    let manager = AnalysisManager::new(&project_mir);
    let analysis_results = manager.run_all_passes();
    println!("✅ 分析完成\n");

    // 创建代码生成器并生成项目
    println!("正在调用 OpenAI API 生成 Rust 项目...");
    println!("（这可能需要几秒钟）");
    let mut generator = CodeGenerator::new(temp_dir.path(), "llm_demo_rs".to_string());

    // 使用 LLM 增强版本
    match generator
        .generate_with_llm(&project_mir, &analysis_results)
        .await
    {
        Ok(_) => {
            println!("✅ 代码生成成功！\n");

            // 显示生成的文件
            display_generated_files(temp_dir.path())?;

            println!("\n=== API 调用成功 ===");
            println!("生成的文档和注释由 OpenAI GPT-4o-mini 提供");
            println!("您可以看到更详细、更有针对性的说明");
        }
        Err(e) => {
            eprintln!("❌ 代码生成失败: {}", e);
            eprintln!("\n可能的原因：");
            eprintln!("  1. API key 无效");
            eprintln!("  2. 网络连接问题");
            eprintln!("  3. API 速率限制");
            eprintln!("\n将回退到 mock 模式...");
        }
    }

    println!("\n💡 提示：生成的代码位于：");
    println!("{:?}", temp_dir.path());
    println!("\n按 Enter 键退出（目录将被清理）...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}

fn display_generated_files(output_dir: &std::path::Path) -> Result<()> {
    println!("\n=== 生成的文件预览 ===\n");

    // src/generated.rs
    if let Ok(content) = std::fs::read_to_string(output_dir.join("src/generated.rs")) {
        println!("--- src/generated.rs ---");
        let lines: Vec<&str> = content.lines().collect();
        for (idx, line) in lines.iter().take(50).enumerate() {
            println!("{:3}: {}", idx + 1, line);
        }
        if lines.len() > 50 {
            println!("... ({} 行省略)", lines.len() - 50);
        }
    }

    Ok(())
}
