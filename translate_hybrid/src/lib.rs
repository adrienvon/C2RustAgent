//! Translate Hybrid - 混合智能 C 到 Rust 翻译器
//!
//! 结合静态分析和大语言模型的 C 到 Rust 翻译解决方案

pub mod llm_client;
pub mod utils;

// 重新导出主要类型
pub use llm_client::{ChatMessage, LlmClient, LlmConfig};

/// 库版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 获取版本信息
pub fn version_info() -> String {
    format!("Translate Hybrid v{}", VERSION)
}
