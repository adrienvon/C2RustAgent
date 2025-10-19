//! Rust 代码生成器演示（带 LLM 增强）
//!
//! 展示如何使用 LLM 增强的代码生成器生成带有详细文档和 unsafe 注释的 Rust 项目

use anyhow::Result;
use c2rust_agent::analysis::{AnalysisManager, ProjectAnalysisResults};
use c2rust_agent::codegen::CodeGenerator;
use c2rust_agent::mir::*;
use std::collections::HashMap;
use tempfile::TempDir;
use tokio;

/// 创建示例 MIR（包含 unsafe 操作）
fn create_example_mir_with_unsafe() -> ProjectMIR {
    let mut project = ProjectMIR {
        functions: HashMap::new(),
        globals: HashMap::new(),
    };

    // 示例 1: malloc/free 函数
    let malloc_func = Function {
        name: "allocate_memory".to_string(),
        parameters: vec![Parameter {
            name: "size".to_string(),
            param_type: Type::Int,
            var_id: 0,
        }],
        return_type: Some(Type::Pointer(Box::new(Type::Int))),
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
            "[RequiresNonNull(size)]".to_string(),
        ],
        is_static: false,
        is_public: true,
    };

    // 示例 2: 指针操作函数
    let pointer_func = Function {
        name: "pointer_arithmetic".to_string(),
        parameters: vec![
            Parameter {
                name: "ptr".to_string(),
                param_type: Type::Pointer(Box::new(Type::Int)),
                var_id: 0,
            },
            Parameter {
                name: "offset".to_string(),
                param_type: Type::Int,
                var_id: 1,
            },
        ],
        return_type: Some(Type::Pointer(Box::new(Type::Int))),
        basic_blocks: vec![BasicBlock {
            id: 0,
            statements: vec![Statement::Assign(
                LValue::Variable(2),
                RValue::AddressOf(Box::new(LValue::Variable(0))),
            )],
            terminator: Terminator::Return(Some(RValue::Use(Box::new(LValue::Variable(2))))),
        }],
        annotations: vec![
            "[RequiresValidPointer(ptr)]".to_string(),
            "[HasSideEffects]".to_string(),
        ],
        is_static: false,
        is_public: true,
    };

    // 示例 3: 简单的数学函数（不需要 unsafe）
    let safe_func = Function {
        name: "add_numbers".to_string(),
        parameters: vec![
            Parameter {
                name: "a".to_string(),
                param_type: Type::Int,
                var_id: 0,
            },
            Parameter {
                name: "b".to_string(),
                param_type: Type::Int,
                var_id: 1,
            },
        ],
        return_type: Some(Type::Int),
        basic_blocks: vec![BasicBlock {
            id: 0,
            statements: vec![],
            terminator: Terminator::Return(Some(RValue::BinaryOp(
                BinOp::Add,
                Box::new(RValue::Use(Box::new(LValue::Variable(0)))),
                Box::new(RValue::Use(Box::new(LValue::Variable(1)))),
            ))),
        }],
        annotations: vec!["[Pure]".to_string()],
        is_static: false,
        is_public: true,
    };

    project
        .functions
        .insert("allocate_memory".to_string(), malloc_func);
    project
        .functions
        .insert("pointer_arithmetic".to_string(), pointer_func);
    project
        .functions
        .insert("add_numbers".to_string(), safe_func);

    // 添加全局变量
    project.globals.insert(
        "GLOBAL_COUNTER".to_string(),
        GlobalVar {
            name: "GLOBAL_COUNTER".to_string(),
            var_type: Type::Int,
            is_static: false,
            is_public: true,
        },
    );

    project
}

/// 显示生成的文件内容
fn display_generated_files(output_dir: &std::path::Path) -> Result<()> {
    println!("\n=== 生成的 Rust 项目结构 ===\n");

    // Cargo.toml
    if let Ok(content) = std::fs::read_to_string(output_dir.join("Cargo.toml")) {
        println!("--- Cargo.toml ---");
        println!("{}\n", content);
    }

    // lib.rs
    if let Ok(content) = std::fs::read_to_string(output_dir.join("src/lib.rs")) {
        println!("--- src/lib.rs ---");
        println!("{}\n", content);
    }

    // globals.rs
    if let Ok(content) = std::fs::read_to_string(output_dir.join("src/globals.rs")) {
        println!("--- src/globals.rs ---");
        println!("{}\n", content);
    }

    // generated.rs
    if let Ok(content) = std::fs::read_to_string(output_dir.join("src/generated.rs")) {
        println!("--- src/generated.rs (部分) ---");
        // 只显示前 100 行以便阅读
        let lines: Vec<&str> = content.lines().collect();
        for (idx, line) in lines.iter().take(100).enumerate() {
            println!("{:4}: {}", idx + 1, line);
        }
        if lines.len() > 100 {
            println!("... ({} 行省略)", lines.len() - 100);
        }
        println!();
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Rust 代码生成器演示（带 LLM 增强）===\n");

    // 创建临时输出目录
    let temp_dir = TempDir::new()?;
    println!("输出目录: {:?}\n", temp_dir.path());

    // 创建示例 MIR
    println!("正在创建示例 MIR（包含 unsafe 操作）...");
    let project_mir = create_example_mir_with_unsafe();
    println!("✅ 创建了 {} 个函数", project_mir.functions.len());
    println!("✅ 创建了 {} 个全局变量\n", project_mir.globals.len());

    // 运行静态分析
    println!("正在运行静态分析...");
    let manager = AnalysisManager::new(&project_mir);
    let analysis_results: ProjectAnalysisResults = manager.run_all_passes();
    println!("✅ 分析完成\n");

    // 创建代码生成器并生成项目（使用 LLM）
    println!("正在生成 Rust 项目（带 LLM 增强的文档和 unsafe 注释）...");
    let mut generator = CodeGenerator::new(temp_dir.path(), "unsafe_demo_rs".to_string());

    // 使用 LLM 增强版本
    generator
        .generate_with_llm(&project_mir, &analysis_results)
        .await?;

    println!("✅ 代码生成成功！\n");

    // 显示生成的文件
    display_generated_files(temp_dir.path())?;

    println!("\n=== LLM 集成点说明 ===");
    println!("1. 模块级文档：每个 .rs 文件顶部包含 LLM 生成的模块级文档（//!）");
    println!("2. unsafe 注释：涉及 unsafe 操作的代码包含详细的 SAFETY 注释");
    println!("3. 注释内容：");
    println!("   - 为什么需要 unsafe");
    println!("   - 不变量要求");
    println!("   - 潜在风险");
    println!("   - 正确性论证");
    println!("\n查看 src/generated.rs 以查看完整的 LLM 增强效果！");

    // 保持临时目录以便检查
    println!("\n💡 提示：临时目录将在程序退出后自动删除");
    println!("如需保留，请将以下目录复制到其他位置：");
    println!("{:?}", temp_dir.path());

    Ok(())
}
