//! Translate Hybrid CLI

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use translate_hybrid::utils::{print_error, print_info, print_success, print_warning};
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

    /// 翻译 C 代码到 Rust
    Translate {
        /// 输入的 C 文件路径
        #[arg(short, long)]
        input: String,

        /// 输出的 Rust 文件路径
        #[arg(short, long)]
        output: String,

        /// 自定义系统提示词文件路径
        #[arg(short = 'p', long)]
        prompt_file: Option<String>,
    },

    /// 修复 Rust 代码的语法错误
    Fix {
        /// Rust 文件路径
        #[arg(short, long)]
        file: String,

        /// 错误信息文件路径
        #[arg(short, long)]
        errors: String,
    },

    /// 优化 unsafe 代码
    OptimizeUnsafe {
        /// Rust 文件路径
        #[arg(short, long)]
        file: String,

        /// 输出文件路径（可选，默认覆盖原文件）
        #[arg(short, long)]
        output: Option<String>,
    },

    /// 翻译整个 C 项目
    TranslateProject {
        /// C 项目根目录
        #[arg(short, long)]
        project_dir: String,

        /// 输出目录
        #[arg(short, long, default_value = "rust_output")]
        output_dir: String,

        /// 要翻译的文件模式（如 *.c）
        #[arg(short = 't', long, default_value = "*.c")]
        pattern: String,

        /// 并发翻译的文件数量
        #[arg(short = 'j', long, default_value = "1")]
        jobs: usize,

        /// 跳过已存在的文件
        #[arg(long)]
        skip_existing: bool,
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

        Commands::Translate {
            input,
            output,
            prompt_file,
        } => translate_file(&cli.config, &input, &output, prompt_file.as_deref()).await,

        Commands::Fix { file, errors } => fix_syntax(&cli.config, &file, &errors).await,

        Commands::OptimizeUnsafe { file, output } => {
            optimize_unsafe_file(&cli.config, &file, output.as_deref()).await
        }

        Commands::TranslateProject {
            project_dir,
            output_dir,
            pattern,
            jobs,
            skip_existing,
        } => {
            translate_project(
                &cli.config,
                &project_dir,
                &output_dir,
                &pattern,
                jobs,
                skip_existing,
            )
            .await
        }
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

/// 翻译 C 文件到 Rust
async fn translate_file(
    config_path: &str,
    input: &str,
    output: &str,
    prompt_file: Option<&str>,
) -> Result<()> {
    print_info(&format!("📄 输入文件: {}", input));
    print_info(&format!("📄 输出文件: {}", output));

    // 读取 C 代码
    let c_code =
        std::fs::read_to_string(input).with_context(|| format!("无法读取输入文件: {}", input))?;

    print_info(&format!("📏 C 代码行数: {}", c_code.lines().count()));

    // 加载配置
    let content = std::fs::read_to_string(config_path)?;
    let config_value: toml::Value = toml::from_str(&content)?;
    let llm_config = config_value
        .get("llm")
        .ok_or_else(|| anyhow::anyhow!("配置文件中未找到 [llm] 部分"))?;
    let llm_config: LlmConfig = llm_config.clone().try_into()?;

    let client = LlmClient::new(llm_config)?;

    // 读取系统提示词
    let system_prompt = if let Some(path) = prompt_file {
        std::fs::read_to_string(path).with_context(|| format!("无法读取提示词文件: {}", path))?
    } else {
        // 尝试从默认位置读取
        let default_prompt = "config/prompts/translate.txt";
        if std::path::Path::new(default_prompt).exists() {
            std::fs::read_to_string(default_prompt)?
        } else {
            "你是 C 到 Rust 转换专家。请将给定的 C 代码翻译为地道、安全的 Rust 代码。\n\n要求：\n1. 使用 Rust 的所有权系统替代手动内存管理\n2. 使用安全的抽象（引用、Box、Vec 等）\n3. 添加必要的错误处理\n4. 保持代码逻辑一致\n5. 添加清晰的注释".to_string()
        }
    };

    print_info("🤖 调用 LLM 翻译...\n");

    let rust_code = client.translate_code(&c_code, "", &system_prompt).await?;

    // 提取代码块（如果 LLM 返回了 markdown 格式）
    let rust_code =
        if let Some(code) = translate_hybrid::utils::extract_code_block(&rust_code, "rust") {
            code
        } else {
            rust_code
        };

    // 写入输出文件
    std::fs::write(output, &rust_code).with_context(|| format!("无法写入输出文件: {}", output))?;

    print_success(&format!("✅ 翻译完成！"));
    print_info(&format!("📏 Rust 代码行数: {}", rust_code.lines().count()));

    // 计算 unsafe 占比
    let unsafe_ratio = translate_hybrid::utils::calculate_unsafe_ratio(&rust_code);
    if unsafe_ratio > 0.0 {
        print_info(&format!("⚠️  unsafe 占比: {:.1}%", unsafe_ratio * 100.0));
        if unsafe_ratio > 0.2 {
            print_info("💡 提示: 使用 'optimize-unsafe' 命令优化 unsafe 代码");
        }
    }

    Ok(())
}

/// 修复 Rust 代码的语法错误
async fn fix_syntax(config_path: &str, file: &str, errors_file: &str) -> Result<()> {
    print_info(&format!("📄 Rust 文件: {}", file));
    print_info(&format!("📄 错误文件: {}", errors_file));

    // 读取文件
    let rust_code =
        std::fs::read_to_string(file).with_context(|| format!("无法读取 Rust 文件: {}", file))?;
    let errors = std::fs::read_to_string(errors_file)
        .with_context(|| format!("无法读取错误文件: {}", errors_file))?;

    // 加载配置
    let content = std::fs::read_to_string(config_path)?;
    let config_value: toml::Value = toml::from_str(&content)?;
    let llm_config = config_value
        .get("llm")
        .ok_or_else(|| anyhow::anyhow!("配置文件中未找到 [llm] 部分"))?;
    let llm_config: LlmConfig = llm_config.clone().try_into()?;

    let client = LlmClient::new(llm_config)?;

    print_info("🔧 调用 LLM 修复错误...\n");

    let fixed_code = client.fix_syntax_errors(&rust_code, &errors).await?;

    // 提取代码块
    let fixed_code =
        if let Some(code) = translate_hybrid::utils::extract_code_block(&fixed_code, "rust") {
            code
        } else {
            fixed_code
        };

    // 覆盖原文件
    std::fs::write(file, &fixed_code).with_context(|| format!("无法写入文件: {}", file))?;

    print_success("✅ 修复完成！");
    print_info("💡 提示: 运行 'cargo check' 验证修复结果");

    Ok(())
}

/// 优化 unsafe 代码
async fn optimize_unsafe_file(config_path: &str, file: &str, output: Option<&str>) -> Result<()> {
    print_info(&format!("📄 输入文件: {}", file));

    // 读取代码
    let rust_code =
        std::fs::read_to_string(file).with_context(|| format!("无法读取文件: {}", file))?;

    let unsafe_ratio_before = translate_hybrid::utils::calculate_unsafe_ratio(&rust_code);
    print_info(&format!(
        "📊 优化前 unsafe 占比: {:.1}%",
        unsafe_ratio_before * 100.0
    ));

    if unsafe_ratio_before == 0.0 {
        print_success("✅ 代码中没有 unsafe 块，无需优化");
        return Ok(());
    }

    // 加载配置
    let content = std::fs::read_to_string(config_path)?;
    let config_value: toml::Value = toml::from_str(&content)?;
    let llm_config = config_value
        .get("llm")
        .ok_or_else(|| anyhow::anyhow!("配置文件中未找到 [llm] 部分"))?;
    let llm_config: LlmConfig = llm_config.clone().try_into()?;

    let client = LlmClient::new(llm_config)?;

    print_info("🔧 调用 LLM 优化 unsafe 代码...\n");

    let optimized_code = client.optimize_unsafe(&rust_code).await?;

    // 提取代码块
    let optimized_code =
        if let Some(code) = translate_hybrid::utils::extract_code_block(&optimized_code, "rust") {
            code
        } else {
            optimized_code
        };

    // 计算优化后的 unsafe 占比
    let unsafe_ratio_after = translate_hybrid::utils::calculate_unsafe_ratio(&optimized_code);

    let output_file = output.unwrap_or(file);
    std::fs::write(output_file, &optimized_code)
        .with_context(|| format!("无法写入文件: {}", output_file))?;

    print_success("✅ 优化完成！");
    print_info(&format!(
        "📊 优化后 unsafe 占比: {:.1}%",
        unsafe_ratio_after * 100.0
    ));

    if unsafe_ratio_after < unsafe_ratio_before {
        let improvement = (unsafe_ratio_before - unsafe_ratio_after) * 100.0;
        print_success(&format!("🎉 unsafe 代码减少了 {:.1}%", improvement));
    } else if unsafe_ratio_after == unsafe_ratio_before {
        print_info("ℹ️  unsafe 占比未变化（可能已经是最优解）");
    }

    Ok(())
}

/// 翻译整个 C 项目
async fn translate_project(
    config_path: &str,
    project_dir: &str,
    output_dir: &str,
    pattern: &str,
    _jobs: usize,
    skip_existing: bool,
) -> Result<()> {
    use std::path::PathBuf;
    use walkdir::WalkDir;

    print_info(&format!("🚀 开始翻译项目: {}", project_dir));
    print_info(&format!("📁 输出目录: {}", output_dir));
    print_info(&format!("🔍 文件模式: {}", pattern));

    // 创建输出目录
    std::fs::create_dir_all(output_dir)?;

    // 查找所有匹配的 C 文件
    let mut c_files: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(project_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "c" && pattern == "*.c" {
                c_files.push(path.to_path_buf());
            } else if ext == "h" && pattern == "*.h" {
                c_files.push(path.to_path_buf());
            } else if pattern == "*" {
                if ext == "c" || ext == "h" {
                    c_files.push(path.to_path_buf());
                }
            }
        }
    }

    print_info(&format!("📄 找到 {} 个文件", c_files.len()));

    if c_files.is_empty() {
        print_error("未找到要翻译的文件");
        return Ok(());
    }

    // 加载配置
    let content = std::fs::read_to_string(config_path)?;
    let config_value: toml::Value = toml::from_str(&content)?;
    let llm_config = config_value
        .get("llm")
        .ok_or_else(|| anyhow::anyhow!("配置文件中未找到 [llm] 部分"))?;
    let llm_config: LlmConfig = llm_config.clone().try_into()?;

    // 读取系统提示词
    let system_prompt = if std::path::Path::new("config/prompts/translate.txt").exists() {
        std::fs::read_to_string("config/prompts/translate.txt")?
    } else {
        "你是 C 到 Rust 转换专家。请将给定的 C 代码翻译为地道、安全的 Rust 代码。\n\n要求：\n1. 使用 Rust 的所有权系统替代手动内存管理\n2. 使用安全的抽象（引用、Box、Vec 等）\n3. 添加必要的错误处理\n4. 保持代码逻辑一致\n5. 添加清晰的注释".to_string()
    };

    // 统计信息
    let total_files = c_files.len();
    let mut translated_files = 0;
    let mut skipped_files = 0;
    let mut failed_files = 0;

    // 翻译每个文件
    for (idx, c_file) in c_files.iter().enumerate() {
        let relative_path = c_file.strip_prefix(project_dir).unwrap_or(c_file);
        let output_file = PathBuf::from(output_dir)
            .join(relative_path)
            .with_extension("rs");

        println!("\n{}", "=".repeat(80));
        print_info(&format!(
            "📝 [{}/{}] 翻译: {}",
            idx + 1,
            total_files,
            relative_path.display()
        ));

        // 检查是否跳过已存在的文件
        if skip_existing && output_file.exists() {
            print_warning(&format!("⏭️  跳过已存在的文件: {}", output_file.display()));
            skipped_files += 1;
            continue;
        }

        // 创建输出目录
        if let Some(parent) = output_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 读取 C 代码
        let c_code = match std::fs::read_to_string(c_file) {
            Ok(code) => code,
            Err(e) => {
                print_error(&format!("❌ 无法读取文件: {}", e));
                failed_files += 1;
                continue;
            }
        };

        print_info(&format!("📏 C 代码行数: {}", c_code.lines().count()));

        // 调用 LLM 翻译
        let client = LlmClient::new(llm_config.clone())?;

        let rust_code = match client.translate_code(&c_code, "", &system_prompt).await {
            Ok(code) => code,
            Err(e) => {
                print_error(&format!("❌ 翻译失败: {}", e));
                failed_files += 1;
                continue;
            }
        };

        // 提取代码块
        let rust_code =
            if let Some(code) = translate_hybrid::utils::extract_code_block(&rust_code, "rust") {
                code
            } else {
                rust_code
            };

        // 写入输出文件
        if let Err(e) = std::fs::write(&output_file, &rust_code) {
            print_error(&format!("❌ 无法写入文件: {}", e));
            failed_files += 1;
            continue;
        }

        translated_files += 1;
        print_success(&format!("✅ 已保存: {}", output_file.display()));

        let rust_lines = rust_code.lines().count();
        print_info(&format!("📏 Rust 代码行数: {}", rust_lines));

        // 计算 unsafe 占比
        let unsafe_ratio = translate_hybrid::utils::calculate_unsafe_ratio(&rust_code);
        if unsafe_ratio > 0.0 {
            print_info(&format!("⚠️  unsafe 占比: {:.1}%", unsafe_ratio * 100.0));
        }
    }

    // 生成 Cargo.toml
    println!("\n{}", "=".repeat(80));
    print_info("📦 生成 Cargo.toml...");

    let project_name = std::path::Path::new(project_dir)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("translated_project");

    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
libc = "0.2"

[lib]
name = "{}"
path = "lib.rs"
"#,
        project_name,
        project_name.replace("-", "_")
    );

    let cargo_toml_path = PathBuf::from(output_dir).join("Cargo.toml");
    std::fs::write(&cargo_toml_path, cargo_toml)?;
    print_success(&format!("✅ 已生成: {}", cargo_toml_path.display()));

    // 生成 lib.rs
    print_info("📦 生成 lib.rs...");

    let mut lib_rs_content = String::from("//! Auto-generated Rust code from C project\n\n");

    // 添加所有模块声明
    for c_file in &c_files {
        let relative_path = c_file.strip_prefix(project_dir).unwrap_or(c_file);
        let module_name = relative_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("module")
            .replace("-", "_");

        lib_rs_content.push_str(&format!("pub mod {};\n", module_name));
    }

    let lib_rs_path = PathBuf::from(output_dir).join("lib.rs");
    std::fs::write(&lib_rs_path, lib_rs_content)?;
    print_success(&format!("✅ 已生成: {}", lib_rs_path.display()));

    // 打印总结
    println!("\n{}", "=".repeat(80));
    print_success("🎉 项目翻译完成！");
    println!("\n📊 统计信息:");
    println!("  ✅ 成功翻译: {} 个文件", translated_files);
    if skipped_files > 0 {
        println!("  ⏭️  跳过: {} 个文件", skipped_files);
    }
    if failed_files > 0 {
        println!("  ❌ 失败: {} 个文件", failed_files);
    }
    println!("  📁 输出目录: {}", output_dir);

    println!("\n💡 下一步:");
    println!("  cd {}", output_dir);
    println!("  cargo check   # 检查语法");
    println!("  cargo build   # 构建项目");
    println!("  cargo test    # 运行测试");

    Ok(())
}
