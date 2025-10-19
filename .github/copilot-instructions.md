# Copilot Instructions for C2RustAgent

## 项目架构与核心流程

- **多阶段管道**：C2RustAgent 采用分阶段架构，主流程为：
  1. C 源码 → Clang AST 解析（`clang` crate，AST 遍历）
  2. AST → MIR（`ast_to_mir.rs`，结构降级、控制流重建）
  3. MIR 静态分析（`analysis/`，如活性分析、生命周期推断）
  4. MIR → Rust 代码生成（规划中）
  5. LLM 语义增强（多阶段注入注释、推断所有权/用途）
- **关键数据结构**：MIR 结构定义于 `src/mir.rs`，AST 到 MIR 转换逻辑在 `src/ast_to_mir.rs`。
- **静态分析**：分析相关代码位于 `src/analysis/`，如 `liveness.rs`。
- **主入口**：`src/main.rs` 负责 orchestrate 各阶段。

## 主要开发/运行流程

- **构建**：
  ```pwsh
  cargo build
  ```
- **运行**：
  ```pwsh
  cargo run
  ```
- **测试**：
  ```pwsh
  cargo test
  ```
- **代码风格**：提交前请运行 `cargo fmt` 和 `cargo clippy`。
- **依赖**：需本地安装 LLVM/Clang（libclang），详见 `README.md`。

## 约定与模式

- **MIR 设计**：采用基本块（basic block）+ 显式控制流，左值/右值区分，支持 JSON 序列化（`serde`）。
- **LLM 集成点**：所有权、用途、命名等语义注释通过 LLM 注入，相关接口预留在 MIR 层。
- **错误处理**：统一使用 `anyhow`/`thiserror`。
- **目录结构**：
  - `src/ast_to_mir.rs`：AST→MIR 转换主逻辑
  - `src/mir.rs`：MIR 结构与序列化
  - `src/analysis/`：静态分析模块
  - `docs/`：阶段设计与实现文档

## 典型代码片段

- **MIR 示例**（见 `README.md`）：
  ```json
  {
    "name": "add",
    "parameters": [ ... ],
    "basic_blocks": [ ... ],
    "annotations": [ ... ]
  }
  ```
- **AST→MIR 转换**：
  - 入口函数通常为 `ast_to_mir::convert_*`，以函数/表达式为粒度递归处理。
- **静态分析调用**：
  - 以 MIR 为输入，分析结果可用于 LLM 注释或后续代码生成。

## 其他说明

- **贡献/分支**：遵循 `README.md` 贡献流程。
- **文档**：详细设计与阶段说明见 `docs/`，如 `phase2_mir.md`。
- **变更频繁**：架构和 API 仍在快速演进，注意同步主分支。

---

如需更详细的开发约定或遇到不明确的结构，请优先查阅 `README.md` 和 `docs/`，或在 PR/Issue 中提出。
