use anyhow::{Result, anyhow};
use c2rust_agent::{ast_to_mir, project_loader};
use clang::{Clang, Entity, Index};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

const C_CODE: &str = "int add(int a, int b) { return a + b; }";

/// 命令行参数
#[derive(Parser, Debug)]
#[command(name = "C2RustAgent", version, about = "C 项目加载与转译原型", long_about = None)]
struct Cli {
    /// C 项目的根目录（包含 compile_commands.json）
    #[arg(value_name = "PROJECT_ROOT")]
    project_root: Option<PathBuf>,
}

fn main() -> Result<()> {
    // CLI：若提供项目根目录则执行项目级加载与两阶段转换
    let cli = Cli::parse();
    if let Some(root) = cli.project_root {
        let cc = root.join("compile_commands.json");
        if !cc.exists() {
            return Err(anyhow!(
                "未找到 {}\n提示: 使用 'bear -- make' 或 'cmake -DCMAKE_EXPORT_COMPILE_COMMANDS=ON .' 生成该文件",
                cc.display()
            ));
        }
        let project = project_loader::CProject::load(&root)?;
        println!("已解析 C 源文件数量: {}", project.units.len());

        // 两阶段转换（占位实现 - 函数体转换尚未完整）
        let proj_mir = ast_to_mir::Converter::convert_project(&project)?;
        println!(
            "项目级 MIR：函数数={}, 全局变量数={}",
            proj_mir.functions.len(),
            proj_mir.globals.len()
        );
        return Ok(());
    }

    // 若未提供项目路径，运行内置示例与 AST/MIR 演示
    let clang = Clang::new().map_err(|e| anyhow!("无法初始化 Clang: {}", e))?;
    let index = Index::new(&clang, false, false);
    let temp_file = PathBuf::from("temp_test.c");
    fs::write(&temp_file, C_CODE)?;
    let translation_unit = index
        .parser(&temp_file)
        .arguments(&["-std=c11"])
        .parse()
        .map_err(|e| anyhow!("无法解析 C 代码: {:?}", e))?;

    println!("成功解析 C 代码:\n{}\n", C_CODE);
    println!("AST 结构:\n{}", "=".repeat(60));
    let root_entity = translation_unit.get_entity();
    traverse_ast(&root_entity, 0);

    println!("\n{}\nMIR 表示示例:\n{}", "=".repeat(60), "=".repeat(60));
    demonstrate_mir();

    println!(
        "\n{}\nAST 到 MIR 自动转换:\n{}",
        "=".repeat(60),
        "=".repeat(60)
    );
    if let Ok(converted_func) = ast_to_mir::Converter::convert_from_entity(&root_entity) {
        println!("\n✓ 成功转换函数: {}", converted_func.name);
        println!("  参数数量: {}", converted_func.parameters.len());
        println!("  基本块数量: {}", converted_func.basic_blocks.len());
        if let Ok(json) = serde_json::to_string_pretty(&converted_func) {
            println!("\nMIR JSON 表示:\n{}", json);
        }
    }

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
    use c2rust_agent::mir::*;

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
