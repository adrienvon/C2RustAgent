mod mir;

use anyhow::{Result, anyhow};
use clang::{Clang, Entity, Index};
use std::fs;
use std::path::PathBuf;

const C_CODE: &str = "int add(int a, int b) { return a + b; }";

fn main() -> Result<()> {
    // 初始化 Clang
    let clang = Clang::new().map_err(|e| anyhow!("无法初始化 Clang: {}", e))?;
    let index = Index::new(&clang, false, false);

    // 创建临时 C 文件
    let temp_file = PathBuf::from("temp_test.c");
    fs::write(&temp_file, C_CODE)?;

    // 解析 C 代码文件
    let translation_unit = index
        .parser(&temp_file)
        .arguments(&["-std=c11"])
        .parse()
        .map_err(|e| anyhow!("无法解析 C 代码: {:?}", e))?;

    println!("成功解析 C 代码:");
    println!("{}\n", C_CODE);
    println!("AST 结构:");
    println!("{}", "=".repeat(60));

    // 获取根实体并遍历 AST
    let root_entity = translation_unit.get_entity();
    traverse_ast(&root_entity, 0);

    println!("\n{}", "=".repeat(60));
    println!("MIR 表示示例:");
    println!("{}", "=".repeat(60));

    // 展示 MIR 结构
    demonstrate_mir();

    // 清理临时文件
    let _ = fs::remove_file(&temp_file);

    Ok(())
}

/// 递归遍历 AST 节点
fn traverse_ast(entity: &Entity, depth: usize) {
    let indent = "  ".repeat(depth);
    let kind = entity.get_kind();
    let name = entity
        .get_display_name()
        .unwrap_or_else(|| "<unnamed>".to_string());

    // 打印节点信息
    println!("{}{:?}: {}", indent, kind, name);

    // 递归遍历子节点
    let children = entity.get_children();
    for child in children {
        traverse_ast(&child, depth + 1);
    }
}

/// 展示如何使用 MIR 数据结构表示 C 函数
fn demonstrate_mir() {
    use mir::*;

    // 手动构建 MIR 表示: int add(int a, int b) { return a + b; }
    let mut func = Function::new("add".to_string(), Some(Type::Int));

    // 添加参数
    func.add_parameter("a".to_string(), Type::Int, 0);
    func.add_parameter("b".to_string(), Type::Int, 1);

    // 添加 LLM 注释
    func.add_annotation("Function takes ownership of parameters".to_string());
    func.add_annotation("Returns sum of two integers".to_string());

    // 创建基本块 0: 计算 a + b 并返回
    let bb0 = BasicBlock::new(
        0,
        Terminator::Return(Some(RValue::BinaryOp(
            BinOp::Add,
            Box::new(RValue::Use(Box::new(LValue::Variable(0)))),
            Box::new(RValue::Use(Box::new(LValue::Variable(1)))),
        ))),
    );

    func.add_basic_block(bb0);

    // 打印 MIR 结构
    println!("\n函数: {}", func.name);
    println!("返回类型: {:?}", func.return_type);
    println!("\n参数:");
    for param in &func.parameters {
        println!(
            "  - {} (id: {}, type: {:?})",
            param.name, param.var_id, param.param_type
        );
    }

    println!("\nLLM 注释:");
    for annotation in &func.annotations {
        println!("  • {}", annotation);
    }

    println!("\n基本块:");
    for bb in &func.basic_blocks {
        println!("  BB{}:", bb.id);
        for (i, stmt) in bb.statements.iter().enumerate() {
            println!("    [{}] {:?}", i, stmt);
        }
        println!("    终结符: {:?}", bb.terminator);
    }

    // 序列化为 JSON
    match serde_json::to_string_pretty(&func) {
        Ok(json) => {
            println!("\n{}", "=".repeat(60));
            println!("MIR JSON 序列化:");
            println!("{}", "=".repeat(60));
            println!("{}", json);
        }
        Err(e) => eprintln!("JSON 序列化失败: {}", e),
    }
}
