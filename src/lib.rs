//! C2RustAgent 库
//!
//! 提供 C 到 Rust 的转译功能，结合静态分析与 LLM 语义增强

pub mod analysis;
pub mod ast_to_mir;
pub mod codegen;
pub mod llm_assists;
pub mod llm_config;
pub mod mir;
pub mod project_loader;
