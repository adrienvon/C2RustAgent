//! 为 littlefs-fuse 项目生成 Rust 代码
//!
//! 运行方式：
//! ```bash
//! cargo run --example generate_littlefs_rust
//! ```

use anyhow::Result;
use c2rust_agent::{analysis::AnalysisManager, ast_to_mir, codegen::CodeGenerator, project_loader};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 littlefs-fuse Rust 代码生成器\n");
    println!("{}", "=".repeat(60));

    // 1. 加载 littlefs-fuse 项目
    let project_root = PathBuf::from("translate_littlefs_fuse");
    println!("📂 项目路径: {}", project_root.display());

    let project = project_loader::CProject::load(&project_root)?;
    println!("✅ 已解析 C 源文件数量: {}", project.units.len());

    // 2. 转换为 MIR
    println!("\n{}", "=".repeat(60));
    println!("🔄 正在转换为 MIR...");
    let project_mir = ast_to_mir::Converter::convert_project(&project)?;
    println!(
        "✅ MIR 转换完成：函数数={}, 全局变量数={}",
        project_mir.functions.len(),
        project_mir.globals.len()
    );

    // 3. 运行静态分析
    println!("\n{}", "=".repeat(60));
    println!("🔍 正在运行静态分析...");
    let manager = AnalysisManager::new(&project_mir);
    let analysis_results = manager.run_all_passes();
    println!("✅ 静态分析完成");

    // 4. 生成 Rust 代码（使用 LLM 增强）
    println!("\n{}", "=".repeat(60));
    println!("📝 正在生成 Rust 代码（LLM 增强版）...");
    println!("   使用 DeepSeek API 生成文档和安全注释...");

    let output_dir = PathBuf::from("translate_littlefs_fuse/generated_rust");
    let mut generator = CodeGenerator::new(&output_dir, "littlefs_fuse_rs".to_string());

    generator
        .generate_with_llm(&project_mir, &analysis_results)
        .await?;

    println!("✅ Rust 代码生成完成！");

    // 5. 显示生成结果
    println!("\n{}", "=".repeat(60));
    println!("📦 生成的 Rust 项目：");
    println!("   路径: {}", output_dir.display());
    println!("   项目名: littlefs_fuse_rs");
    println!("\n📁 项目结构：");
    println!("   {}/", output_dir.display());
    println!("   ├── Cargo.toml");
    println!("   ├── src/");
    println!("   │   ├── lib.rs");
    println!("   │   ├── globals.rs");
    println!("   │   └── [模块文件].rs");

    // 6. 尝试编译
    println!("\n{}", "=".repeat(60));
    println!("🔨 尝试编译生成的 Rust 项目...");
    println!("   运行: cd {} && cargo build", output_dir.display());

    let compile_result = std::process::Command::new("cargo")
        .args(&["build", "--manifest-path"])
        .arg(output_dir.join("Cargo.toml"))
        .output();

    match compile_result {
        Ok(output) => {
            if output.status.success() {
                println!("✅ 编译成功！");
            } else {
                println!("⚠️  编译遇到错误（这是预期的，因为函数体转换尚未完整）：");
                println!("\nstdout:");
                println!("{}", String::from_utf8_lossy(&output.stdout));
                println!("\nstderr:");
                println!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            println!("❌ 无法运行 cargo build: {}", e);
            println!("   请确保 cargo 在 PATH 中");
        }
    }

    // 7. 总结
    println!("\n{}", "=".repeat(60));
    println!("📊 转译总结：");
    println!("   C 源文件: {} 个", project.units.len());
    println!("   函数: {} 个", project_mir.functions.len());
    println!("   全局变量: {} 个", project_mir.globals.len());
    println!("   输出目录: {}", output_dir.display());

    println!("\n💡 下一步：");
    println!("   1. 查看生成的代码: code {}", output_dir.display());
    println!("   2. 手动编译: cd {} && cargo build", output_dir.display());
    println!(
        "   3. 查看编译错误: cd {} && cargo check 2>&1 | less",
        output_dir.display()
    );

    println!("\n⚠️  注意：");
    println!("   由于当前 AST→MIR 转换器尚未完整实现函数体转换，");
    println!("   生成的代码可能无法通过编译。这是正常的，需要后续完善。");

    println!("\n{}", "=".repeat(60));
    println!("🎉 转译流程完成！");

    Ok(())
}
