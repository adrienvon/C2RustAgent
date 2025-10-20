# Copilot Instructions for C2RustAgent

**C2RustAgent** 将 C 代码转换为地道 Rust，通过形式化静态分析保证正确性，用 LLM 增强语义理解。

## 核心架构认知

### 转换管道（两阶段）

```
C源码 → [1]Clang AST → [2]MIR → [3]静态分析 → [4]Rust代码生成
             ↓            ↓          ↓             ↓
         LLM语义协处理（辅助角色，贯穿全流程）
```

**设计哲学**：形式化方法保证正确性（借用检查、类型安全），LLM 提升可读性（注释、命名、所有权推断）

**关键实现模式**：两阶段 AST→MIR 转换避免前向引用问题，所有 LLM 函数支持 Mock 降级保证可测试性

### 关键模块与数据流

1. **项目加载** (`src/project_loader.rs`)

   - 输入：`compile_commands.json` (由 `bear -- make` 或 CMake 生成)
   - 解析编译单元 → `CProject{units: Vec<UnitSpec>}`
   - 核心方法：`CProject::load(&path)` → `process_units(callback)`
   - 设计要点：按需创建 `TranslationUnit`，通过回调处理避免内存占用

2. **AST→MIR 转换** (`src/ast_to_mir.rs`) - 两阶段设计

   - **Pass 1**: `discover_symbols()` - 扫描函数签名/全局变量（不展开函数体）
   - **Pass 2**: `convert_bodies()` - 构建基本块、填充控制流
   - **关键技术**：维护 `var_map` 和 `next_var_id` 管理变量作用域
   - 输出：`ProjectMIR{functions: HashMap, globals: HashMap}`
   - **已知限制**：部分 C 类型未完整映射（见 `docs/P4提示词.md`）

3. **静态分析框架** (`src/analysis/`)

   - `AnalysisManager::run_all_passes()` - 统一编排所有分析 Pass
   - `PerFunctionResults` - 聚合单个函数的所有分析结果
   - `ProjectAnalysisResults` - 项目级分析结果（函数名 → 结果映射）
   - **扩展模式**：新增分析需在 `PerFunctionResults` 添加字段并在 `run_all_passes()` 调用
   - **现状**：`liveness.rs` 定义接口但算法待实现（占位返回空结果）

4. **代码生成器** (`src/codegen.rs`)

   - `generate()` - 基线版本（无 LLM），生成可编译但非地道的 Rust 代码
   - `generate_with_llm()` - 异步版本（LLM 增强模块文档、unsafe 注释）
   - **生成流程**：创建目录结构 → `Cargo.toml` → 模块文件 → `lib.rs`
   - **模块化策略**：按源文件分组，全局变量单独生成 `globals.rs`
   - **关键方法**：`build_source_module_mapping()` 确定模块划分

5. **LLM 集成** (`src/llm_assists.rs` + `src/llm_config.rs`)
   - **异步架构**：所有 LLM 函数为 `async fn`，依赖 `tokio` 运行时
   - **降级策略**：API 失败自动回退到基于规则的 Mock 实现（见 `infer_external_api_semantics_mock()`）
   - **核心功能**：`infer_external_api_semantics()` 推断 C API 资源管理语义
   - **配置优先级**：项目配置 > 用户配置 > 环境变量（通过 `LlmConfig::load()` 实现）
   - **标注系统**：`[ReturnsNewResource(free)]` → Rust `impl Drop`

## 开发工作流（Windows）

### 一次性环境配置

```pwsh
# 1. 安装 LLVM (libclang 依赖，必需)
# 下载: https://github.com/llvm/llvm-project/releases
# 安装后需设置环境变量 LIBCLANG_PATH（如 C:\Program Files\LLVM\bin）

# 2. 配置 LLM API (三选一，优先级递减)
# 方式1: 用户配置 (推荐 - 配置保存在 %APPDATA%\c2rust-agent\config.toml)
cargo run --bin c2rust-agent-config -- init
notepad $env:APPDATA\c2rust-agent\config.toml  # 编辑 api_key 和 model

# 方式2: 环境变量
$env:OPENAI_API_KEY="sk-your-key"

# 方式3: 项目配置文件 c2rust-agent.toml (不要提交包含 API Key 的文件!)
cargo run --bin c2rust-agent-config -- init-project
```

### 日常命令

```pwsh
cargo build                                           # 构建主程序
cargo run                                             # 演示模式 (内置 C 代码 → AST + MIR)
cargo run -- ./translate_littlefs_fuse               # 转换 C 项目 (需 compile_commands.json)
cargo test                                            # 运行测试（LLM 测试自动使用 Mock）
cargo fmt && cargo clippy                             # 提交前检查

# 配置管理工具 (独立二进制)
cargo run --bin c2rust-agent-config -- show          # 显示当前生效配置
cargo run --bin c2rust-agent-config -- show --verbose  # 显示配置来源
cargo run --bin c2rust-agent-config -- validate      # 验证配置有效性
cargo run --bin c2rust-agent-config -- path          # 显示用户配置文件路径
```

### 测试 C 项目转换

```pwsh
# 示例1：转换 littlefs-fuse (950 个函数，当前有编译错误)
cd translate_littlefs_fuse
# 检查 compile_commands.json 是否存在
cargo run -- .

# 示例2：自定义 C 项目
cd your_c_project
bear -- make                # 生成 compile_commands.json
# 或 cmake -DCMAKE_EXPORT_COMPILE_COMMANDS=ON .
cargo run -- /path/to/your_c_project
```

## 约定与模式

### MIR 设计原则

- **基本块**：每个 `BasicBlock` = `statements[]` + 单个 `Terminator`
- **左值/右值**：`LValue`(赋值目标) vs `RValue`(表达式)
- **序列化**：所有 MIR 结构实现 `serde::Serialize`，支持 JSON 导出
- **LLM 注释**：`Function.annotations: Vec<String>` 存储语义标注

### 语义标注系统

LLM 推断生成的标注及其 Rust 映射：

```rust
[ReturnsNewResource(free)]    → impl Drop
[TakesOwnership(param)]       → fn(param: Box<T>)
[HasSideEffects]              → unsafe block 候选
[Pure]                        → const fn 候选
[RequiresNonNull(param)]      → NonNull<T>
```

### 错误处理策略

- 使用 `anyhow::Result<T>`，添加上下文：`.with_context(|| "原因")`
- 生产代码避免 `unwrap()`（示例代码可用）

### 异步 LLM 约定

```rust
pub async fn infer_something(...) -> Vec<String> {
    match call_llm_api(...).await {
        Ok(resp) => parse_response(resp),
        Err(_) => fallback_to_mock(...),  // 优雅降级
    }
}
```

测试异步 LLM 函数使用 `#[tokio::test]` 属性宏

### 项目结构

```
src/
├── main.rs           # CLI: demo 或转换 C 项目
├── mir.rs            # MIR 数据结构
├── ast_to_mir.rs     # 两阶段转换: discover_symbols + convert_bodies
├── codegen.rs        # generate[_with_llm]()
├── llm_assists.rs    # LLM 语义推断
├── project_loader.rs # CProject::load() + process_units()
├── analysis/mod.rs   # AnalysisManager + 分析结果
└── bin/config.rs     # 配置 CLI
```

## 常见任务

### 添加静态分析 Pass

```rust
// 1. 创建 src/analysis/pointer_analysis.rs
pub struct PointerAnalysisResult { ... }
pub fn run_pointer_analysis(func: &Function) -> PointerAnalysisResult { ... }

// 2. 在 analysis/mod.rs 注册
pub struct PerFunctionResults {
    pub pointer_origins: PointerAnalysisResult,  // 新增字段
}
impl AnalysisManager<'_> {
    pub fn run_all_passes(&self) -> ProjectAnalysisResults {
        per_fn.pointer_origins = run_pointer_analysis(func);
    }
}
```

### 扩展 LLM 推断

```rust
// llm_assists.rs 中添加新函数
pub async fn infer_lifetime_annotations(func: &Function) -> Vec<String> {
    let prompt = format!("分析 MIR 推断生命周期:\n{:?}", func);
    match call_llm_api(&prompt, ...).await {
        Ok(resp) => parse_lifetime_tags(resp),
        Err(_) => mock_lifetime_inference(func),  // 必须提供 mock
    }
}
```

### 处理新 C 项目

```pwsh
# 1. 生成 compile_commands.json
cd your_c_project
bear -- make  # 或 cmake -DCMAKE_EXPORT_COMPILE_COMMANDS=ON .

# 2. 运行转换
cargo run -- /path/to/your_c_project
```

## 调试技巧

- **查看 MIR**：`serde_json::to_string_pretty(&function)?`
- **Mock LLM**：`$env:USE_MOCK_LLM="true"; cargo test`
- **查看 AST**：`main.rs` 中的 `traverse_ast()` 函数

## 参考文档

- 架构设计：`docs/phase*.md`
- 配置指南：`docs/QUICKSTART_CONFIG.md`
- 示例项目：`translate_littlefs_fuse/`、`translate_chibicc/`

## 注意事项

⚠️ **API 演进中** - 架构可能频繁变更  
⚠️ **安全配置** - 不要提交包含 API Key 的 `c2rust-agent.toml`  
⚠️ **LLVM 依赖** - Windows 需安装 LLVM 并配置 LIBCLANG_PATH
