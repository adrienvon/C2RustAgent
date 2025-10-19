//! Rust 代码生成演示
//!
//! 运行方式：
//! ```bash
//! cargo run --example codegen_demo
//! ```

use c2rust_agent::analysis::AnalysisManager;
use c2rust_agent::codegen::CodeGenerator;
use c2rust_agent::mir::{
    BasicBlock, BinOp, Function, GlobalVar, LValue, Parameter, ProjectMIR, RValue, Terminator, Type,
};
use tempfile::TempDir;

fn main() {
    println!("=== Rust 代码生成器演示 ===\n");

    // 创建临时输出目录
    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let output_path = temp_dir.path();

    println!("输出目录: {}\n", output_path.display());

    // 1. 创建示例 MIR
    let project_mir = create_example_mir();

    // 2. 运行静态分析（目前为空）
    let manager = AnalysisManager::new(&project_mir);
    let analysis_results = manager.run_all_passes();

    // 3. 生成 Rust 代码
    let mut generator = CodeGenerator::new(output_path, "example_c_project_rs".to_string());

    println!("正在生成 Rust 项目...");
    generator
        .generate(&project_mir, &analysis_results)
        .expect("代码生成失败");

    println!("✅ 代码生成成功！\n");

    // 4. 展示生成的文件
    println!("生成的文件：");
    display_generated_files(output_path);

    println!("\n=== 演示完成 ===");
    println!("提示：临时目录会在程序退出后自动清理");
}

/// 创建示例 MIR
fn create_example_mir() -> ProjectMIR {
    let mut project_mir = ProjectMIR::new();

    // 示例 1: 简单的加法函数
    // C 代码: int add(int a, int b) { return a + b; }
    let add_func = Function {
        name: "add".to_string(),
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
        annotations: vec!["纯函数".to_string(), "无副作用".to_string()],
        is_static: false,
        is_public: true,
    };

    // 示例 2: 带条件的函数
    // C 代码: int max(int a, int b) { return (a > b) ? a : b; }
    let max_func = Function {
        name: "max".to_string(),
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
            terminator: Terminator::If {
                condition: RValue::BinaryOp(
                    BinOp::Gt,
                    Box::new(RValue::Use(Box::new(LValue::Variable(0)))),
                    Box::new(RValue::Use(Box::new(LValue::Variable(1)))),
                ),
                then_block: 1,
                else_block: 2,
            },
        }],
        annotations: vec![],
        is_static: false,
        is_public: true,
    };

    // 示例 3: static 函数
    // C 代码: static int helper(int x) { return x * 2; }
    let helper_func = Function {
        name: "helper".to_string(),
        parameters: vec![Parameter {
            name: "x".to_string(),
            param_type: Type::Int,
            var_id: 0,
        }],
        return_type: Some(Type::Int),
        basic_blocks: vec![BasicBlock {
            id: 0,
            statements: vec![],
            terminator: Terminator::Return(Some(RValue::BinaryOp(
                BinOp::Mul,
                Box::new(RValue::Use(Box::new(LValue::Variable(0)))),
                Box::new(RValue::Constant(2)),
            ))),
        }],
        annotations: vec![],
        is_static: true,
        is_public: false,
    };

    // 添加函数到项目 MIR
    project_mir.functions.insert("add".to_string(), add_func);
    project_mir.functions.insert("max".to_string(), max_func);
    project_mir
        .functions
        .insert("helper".to_string(), helper_func);

    // 添加全局变量
    let counter_var = GlobalVar {
        name: "counter".to_string(),
        var_type: Type::Int,
        is_static: false,
        is_public: true,
    };

    let internal_state_var = GlobalVar {
        name: "internal_state".to_string(),
        var_type: Type::Int,
        is_static: true,
        is_public: false,
    };

    project_mir
        .globals
        .insert("counter".to_string(), counter_var);
    project_mir
        .globals
        .insert("internal_state".to_string(), internal_state_var);

    project_mir
}

/// 显示生成的文件内容
fn display_generated_files(output_path: &std::path::Path) {
    use std::fs;

    // 显示 Cargo.toml
    println!("\n--- Cargo.toml ---");
    if let Ok(content) = fs::read_to_string(output_path.join("Cargo.toml")) {
        println!("{}", content);
    }

    // 显示 lib.rs
    println!("\n--- src/lib.rs ---");
    if let Ok(content) = fs::read_to_string(output_path.join("src/lib.rs")) {
        println!("{}", content);
    }

    // 显示 globals.rs
    println!("\n--- src/globals.rs ---");
    if let Ok(content) = fs::read_to_string(output_path.join("src/globals.rs")) {
        println!("{}", content);
    }

    // 显示 generated.rs
    println!("\n--- src/generated.rs ---");
    if let Ok(content) = fs::read_to_string(output_path.join("src/generated.rs")) {
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() > 50 {
            // 如果太长，只显示前50行
            for line in &lines[..50] {
                println!("{}", line);
            }
            println!("... ({} 行省略)", lines.len() - 50);
        } else {
            println!("{}", content);
        }
    }
}
