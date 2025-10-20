//! Translate Hybrid CLI

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use translate_hybrid::utils::{print_error, print_info, print_success};
use translate_hybrid::{ChatMessage, LlmClient, LlmConfig, version_info};

#[derive(Parser)]
#[command(name = "translate-hybrid")]
#[command(about = "混合智能 C 到 Rust 翻译器", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// 配置文件路径
    #[arg(short, long, default_value = "config/hybrid_config.toml")]
    config: String,

    /// 日志级别
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    /// 测试 LLM 连接
    TestLlm {
        /// 测试提示词
        #[arg(short, long, default_value = "Hello, world!")]
        prompt: String,
    },

    /// 显示版本信息
    Version,

    /// 初始化配置文件
    Init {
        /// 是否覆盖已存在的配置
        #[arg(short, long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| cli.log_level.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    match cli.command {
        Commands::Version => {
            println!("{}", version_info());
            Ok(())
        }

        Commands::Init { force } => {
            init_config(force)?;
            Ok(())
        }

        Commands::TestLlm { prompt } => test_llm_connection(&cli.config, &prompt).await,
    }
}

/// 初始化配置文件
fn init_config(force: bool) -> Result<()> {
    let config_path = "config/hybrid_config.toml";

    if std::path::Path::new(config_path).exists() && !force {
        print_error(&format!("配置文件已存在: {}", config_path));
        print_info("使用 --force 参数覆盖");
        return Ok(());
    }

    // 复制示例配置
    let example_path = "config/hybrid_config.toml.example";
    if !std::path::Path::new(example_path).exists() {
        print_error(&format!("示例配置文件不存在: {}", example_path));
        return Ok(());
    }

    std::fs::copy(example_path, config_path)?;
    print_success(&format!("配置文件已创建: {}", config_path));
    print_info("请编辑配置文件并设置你的 API Key");

    Ok(())
}

/// 测试 LLM 连接
async fn test_llm_connection(config_path: &str, prompt: &str) -> Result<()> {
    print_info(&format!("加载配置: {}", config_path));

    // 读取配置文件
    let content = std::fs::read_to_string(config_path)?;
    let config_value: toml::Value = toml::from_str(&content)?;

    // 提取 LLM 配置
    let llm_config = config_value
        .get("llm")
        .ok_or_else(|| anyhow::anyhow!("配置文件中未找到 [llm] 部分"))?;

    let llm_config: LlmConfig = llm_config.clone().try_into()?;

    print_info(&format!("使用模型: {}", llm_config.model));
    print_info(&format!("API 端点: {}", llm_config.base_url));

    let client = LlmClient::new(llm_config)?;

    print_info("发送测试请求...\n");

    let messages = vec![ChatMessage::user(prompt)];

    let response = client
        .chat_completion_stream(messages, |chunk| {
            print!("{}", chunk);
            std::io::Write::flush(&mut std::io::stdout()).ok();
        })
        .await?;

    println!("\n");
    print_success("LLM 连接测试成功！");
    print_info(&format!("响应长度: {} 字符", response.len()));

    Ok(())
}
