//! 配置管理工具
//! 提供命令行接口来管理 C2RustAgent 的配置文件

use anyhow::{Context, Result};
use c2rust_agent::llm_config::LlmConfig;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "c2rust-agent-config")]
#[command(about = "C2RustAgent 配置管理工具", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化用户配置文件（在 ~/.config/c2rust-agent/config.toml）
    Init {
        /// 是否覆盖已存在的配置文件
        #[arg(short, long)]
        force: bool,
    },

    /// 显示当前有效的配置
    Show {
        /// 显示详细信息（包括配置来源）
        #[arg(short, long)]
        verbose: bool,
    },

    /// 显示用户配置文件路径
    Path,

    /// 验证配置文件
    Validate,

    /// 创建项目配置文件模板（在当前目录）
    InitProject {
        /// 是否覆盖已存在的配置文件
        #[arg(short, long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { force } => init_user_config(force),
        Commands::Show { verbose } => show_config(verbose),
        Commands::Path => show_path(),
        Commands::Validate => validate_config(),
        Commands::InitProject { force } => init_project_config(force),
    }
}

/// 初始化用户配置文件
fn init_user_config(force: bool) -> Result<()> {
    let config_path = LlmConfig::user_config_path().context("无法确定用户配置文件路径")?;

    // 检查文件是否已存在
    if config_path.exists() && !force {
        println!("❌ 配置文件已存在：{}", config_path.display());
        println!("   如要覆盖，请使用 --force 选项");
        return Ok(());
    }

    // 创建配置目录
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("创建配置目录失败：{}", parent.display()))?;
    }

    // 创建示例配置
    let example_config = LlmConfig::create_example_config()?;

    // 保存到用户配置文件
    std::fs::write(&config_path, example_config)
        .with_context(|| format!("写入配置文件失败：{}", config_path.display()))?;

    println!("✅ 配置文件已创建：{}", config_path.display());
    println!("\n请编辑该文件，设置您的 API Key：");
    println!("   api_key = \"sk-your-api-key-here\"");
    println!("\n或者通过环境变量设置：");
    println!("   export OPENAI_API_KEY=sk-your-api-key-here  # Linux/macOS");
    println!("   $env:OPENAI_API_KEY=\"sk-your-api-key-here\" # Windows PowerShell");

    Ok(())
}

/// 初始化项目配置文件
fn init_project_config(force: bool) -> Result<()> {
    let config_path = PathBuf::from("c2rust-agent.toml");

    // 检查文件是否已存在
    if config_path.exists() && !force {
        println!("❌ 项目配置文件已存在：{}", config_path.display());
        println!("   如要覆盖，请使用 --force 选项");
        return Ok(());
    }

    // 创建示例配置
    let example_config = LlmConfig::create_example_config()?;

    // 保存到项目配置文件
    std::fs::write(&config_path, example_config)
        .with_context(|| format!("写入配置文件失败：{}", config_path.display()))?;

    println!("✅ 项目配置文件已创建：{}", config_path.display());
    println!("\n此配置仅对当前项目生效。");
    println!("请编辑该文件，设置项目特定的配置。");
    println!("\n提示：不要将包含真实 API Key 的配置文件提交到 Git！");
    println!("      建议将其添加到 .gitignore：");
    println!("      echo 'c2rust-agent.toml' >> .gitignore");

    Ok(())
}

/// 显示当前有效的配置
fn show_config(verbose: bool) -> Result<()> {
    let config = LlmConfig::load().context("加载配置失败")?;

    println!("📋 当前有效配置：\n");
    println!("  Provider:     {}", config.provider);
    println!("  Model:        {}", config.model);
    println!("  Temperature:  {}", config.temperature);
    println!("  Max Tokens:   {}", config.max_tokens);
    println!("  Use Mock:     {}", config.use_mock);

    if let Some(api_url) = &config.api_url {
        println!("  API URL:      {}", api_url);
    } else {
        println!("  API URL:      (default)");
    }

    if let Some(api_key) = &config.api_key {
        // 只显示 API Key 的前后几位
        let masked = if api_key.len() > 10 {
            format!("{}...{}", &api_key[..6], &api_key[api_key.len() - 4..])
        } else {
            "***".to_string()
        };
        println!("  API Key:      {}", masked);
    } else {
        println!("  API Key:      ❌ 未设置");
    }

    if verbose {
        println!("\n📍 配置来源（优先级从高到低）：");
        println!("  1. 环境变量");

        if let Some(user_path) = LlmConfig::user_config_path() {
            let exists = user_path.exists();
            println!(
                "  2. 用户配置：{} {}",
                user_path.display(),
                if exists { "✅" } else { "❌" }
            );
        }

        let project_path = PathBuf::from("c2rust-agent.toml");
        let exists = project_path.exists();
        println!(
            "  3. 项目配置：{} {}",
            project_path.display(),
            if exists { "✅" } else { "❌" }
        );

        println!("  4. 默认值 ✅");
    }

    // 验证配置
    if let Err(e) = config.validate() {
        println!("\n⚠️  配置验证失败：{}", e);
        println!("   提示：请使用 'init' 命令创建配置文件并设置 API Key");
    } else {
        println!("\n✅ 配置有效");
    }

    Ok(())
}

/// 显示用户配置文件路径
fn show_path() -> Result<()> {
    if let Some(path) = LlmConfig::user_config_path() {
        println!("📂 用户配置文件路径：");
        println!("   {}", path.display());

        if path.exists() {
            println!("   ✅ 文件存在");
        } else {
            println!("   ❌ 文件不存在");
            println!("\n   使用 'init' 命令创建：");
            println!("   c2rust-agent-config init");
        }
    } else {
        println!("❌ 无法确定用户配置文件路径");
    }

    println!("\n📂 项目配置文件路径：");
    println!("   ./c2rust-agent.toml");

    let project_path = PathBuf::from("c2rust-agent.toml");
    if project_path.exists() {
        println!("   ✅ 文件存在");
    } else {
        println!("   ❌ 文件不存在");
        println!("\n   使用 'init-project' 命令创建：");
        println!("   c2rust-agent-config init-project");
    }

    Ok(())
}

/// 验证配置文件
fn validate_config() -> Result<()> {
    println!("🔍 正在验证配置...\n");

    let config = LlmConfig::load().context("加载配置失败")?;

    match config.validate() {
        Ok(_) => {
            println!("✅ 配置验证通过！\n");
            println!("  Provider:     {}", config.provider);
            println!("  Model:        {}", config.model);
            println!("  Temperature:  {}", config.temperature);
            println!("  Max Tokens:   {}", config.max_tokens);
            println!("  Use Mock:     {}", config.use_mock);

            if config.api_key.is_some() {
                println!("  API Key:      ✅ 已设置");
            }

            Ok(())
        }
        Err(e) => {
            println!("❌ 配置验证失败：\n");
            println!("  错误：{}\n", e);

            if config.api_key.is_none() && !config.use_mock {
                println!("💡 解决方案：");
                println!("  1. 设置环境变量：");
                println!("     export OPENAI_API_KEY=sk-your-key  # Linux/macOS");
                println!("     $env:OPENAI_API_KEY=\"sk-your-key\" # Windows PowerShell");
                println!("  2. 或创建配置文件：");
                println!("     c2rust-agent-config init");
                println!("  3. 或使用 Mock 模式测试：");
                println!("     export USE_MOCK_LLM=true");
            }

            Err(anyhow::anyhow!("配置验证失败"))
        }
    }
}
