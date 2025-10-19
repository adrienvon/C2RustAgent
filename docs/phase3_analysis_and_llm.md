# 阶段三：静态分析管道与 LLM 集成

## 概述

阶段三实现了静态分析管道的调度系统，并集成了 LLM 辅助功能用于外部 API 语义推断。

## 架构组件

### 1. 分析管理器 (`src/analysis/mod.rs`)

#### `AnalysisManager`
- 持有 `ProjectMIR` 的引用
- 对所有函数调度各种分析算法
- 收集和汇总分析结果

#### `PerFunctionResults`
- 存储单个函数的所有分析结果
- 当前包含：活跃变量分析结果
- 可扩展：指针来源分析、生命周期推断等

#### `ProjectAnalysisResults`
- 项目级分析结果集合
- `HashMap<String, PerFunctionResults>` 映射函数名到结果

### 2. 活跃变量分析 (`src/analysis/liveness.rs`)

#### `LivenessResult`
```rust
pub struct LivenessResult {
    pub block_live_vars: HashMap<BasicBlockId, HashSet<VarId>>,
}
```

#### `run_liveness_analysis(func: &Function) -> LivenessResult`
- 函数内分析（Intra-procedural）
- 计算每个基本块的活跃变量集合
- 用于后续生命周期推断和借用检查

### 3. LLM 辅助 (`src/llm_assists.rs`)

#### 核心功能：外部 API 语义推断

```rust
pub async fn infer_external_api_semantics(
    function_name: &str,
    header_file_content: &str,
) -> Vec<String>
```

**推断的语义标注类型：**

| 标注 | 含义 | 示例 |
|------|------|------|
| `[ReturnsNewResource(释放函数)]` | 返回需要特定函数释放的资源 | `malloc` → `[ReturnsNewResource(free)]` |
| `[TakesOwnership(参数名)]` | 接管参数的所有权 | `fclose` → `[TakesOwnership(stream)]` |
| `[HasSideEffects]` | 有副作用（I/O、全局状态等） | `printf` → `[HasSideEffects]` |
| `[Pure]` | 纯函数，无副作用 | `strlen` → `[Pure]` |
| `[RequiresNonNull(参数)]` | 参数不能为 NULL | `strlen` → `[RequiresNonNull(str)]` |
| `[RequiresValidPointer(参数)]` | 参数必须是有效指针 | `memcpy` → `[RequiresValidPointer(dest)]` |
| `[ReturnsValidUntil(条件)]` | 返回值的有效期限制 | `getenv` → `[ReturnsValidUntil(next_getenv_call)]` |

## 使用流程

### 1. 运行分析管道

```rust
use c2rust_agent::analysis::AnalysisManager;
use c2rust_agent::mir::ProjectMIR;

let project_mir = /* 从 AST 转换得到 */;
let manager = AnalysisManager::new(&project_mir);
let results = manager.run_all_passes();

// 访问特定函数的分析结果
if let Some(func_results) = results.results.get("my_function") {
    let live_vars = &func_results.liveness.block_live_vars;
    // 使用活跃变量信息...
}
```

### 2. 推断外部函数语义

在 AST→MIR 转换过程中，当遇到外部函数调用时：

```rust
use c2rust_agent::llm_assists;

// 检测到外部函数调用（如 fopen）
let function_name = "fopen";
let header_content = read_header_file("stdio.h")?;

// 异步推断语义
let semantics = llm_assists::infer_external_api_semantics(
    function_name, 
    &header_content
).await;

// 结果示例: ["[ReturnsNewResource(fclose)]", "[HasSideEffects]"]

// 将语义标注添加到 MIR Statement::Call
let call_stmt = Statement::Call(target, function_name, args)
    .with_annotations(semantics);
```

### 3. 在代码生成中使用语义标注

```rust
match statement {
    Statement::Call(target, func_name, args) => {
        // 检查是否有 LLM 推断的语义
        if let Statement::Annotated { stmt, annotations } = statement {
            for annotation in annotations {
                if annotation.contains("ReturnsNewResource") {
                    // 生成自动释放代码或 Drop trait
                }
                if annotation.contains("TakesOwnership") {
                    // 标记所有权转移
                }
                if annotation.contains("RequiresNonNull") {
                    // 添加空指针检查
                }
            }
        }
    }
    // ...
}
```

## 当前实现状态

### ✅ 已实现

- [x] `AnalysisManager` 结构与调度逻辑
- [x] `PerFunctionResults` 和 `ProjectAnalysisResults`
- [x] `LivenessResult` 数据结构（算法为占位）
- [x] `infer_external_api_semantics` 函数框架
- [x] 基于规则的模拟推断（常见 C 标准库函数）
- [x] 完整的测试套件
- [x] 示例程序 (`examples/llm_external_api_demo.rs`)

### 🚧 待完善

- [ ] 实际活跃变量分析算法实现（数据流分析）
- [ ] 指针来源分析
- [ ] 生命周期推断
- [ ] 集成真实 LLM API（OpenAI/Anthropic/本地）
- [ ] 从编译数据库中提取头文件内容
- [ ] 缓存 LLM 推断结果以提高性能

## 测试与验证

### 运行测试
```bash
# 所有测试
cargo test

# 仅 LLM 辅助测试
cargo test llm_assists

# 仅分析模块测试
cargo test analysis
```

### 运行示例
```bash
cargo run --example llm_external_api_demo
```

**预期输出：**
```
=== LLM 外部 API 语义推断演示 ===

1. 分析 malloc:
   [ReturnsNewResource(free)]

2. 分析 fopen:
   [ReturnsNewResource(fclose)]
   [HasSideEffects]

3. 分析 strlen:
   [Pure]
   [RequiresNonNull(str)]

4. 分析未知函数 (custom_func):
   [HasSideEffects]
   [Unknown]
```

## 集成到主流程

在 `src/main.rs` 中集成分析管道：

```rust
use c2rust_agent::{analysis::AnalysisManager, ast_to_mir, project_loader};

fn main() -> Result<()> {
    let project = project_loader::CProject::load(&root)?;
    
    // 阶段一：AST 解析
    // ... (已实现)
    
    // 阶段二：AST → MIR 转换
    let proj_mir = ast_to_mir::Converter::convert_project(&project)?;
    
    // 阶段三：静态分析管道
    let manager = AnalysisManager::new(&proj_mir);
    let analysis_results = manager.run_all_passes();
    
    println!("分析完成：{} 个函数", analysis_results.results.len());
    
    // 阶段四：LLM 语义增强（在转换时调用）
    // 阶段五：Rust 代码生成（待实现）
    
    Ok(())
}
```

## 未来扩展

### 新增分析类型

在 `PerFunctionResults` 中添加新字段：

```rust
pub struct PerFunctionResults {
    pub liveness: LivenessResult,
    pub pointer_origins: PointerOriginResult,  // 新增
    pub lifetimes: LifetimeResult,             // 新增
    pub borrowck: BorrowCheckResult,           // 新增
}
```

在 `AnalysisManager::run_all_passes` 中调用：

```rust
pub fn run_all_passes(&self) -> ProjectAnalysisResults {
    let mut results = HashMap::new();
    for (name, func) in &self.project_mir.functions {
        let liveness = run_liveness_analysis(func);
        let pointer_origins = run_pointer_origin_analysis(func, &liveness);
        let lifetimes = infer_lifetimes(func, &pointer_origins);
        
        let per_func = PerFunctionResults {
            liveness,
            pointer_origins,
            lifetimes,
        };
        results.insert(name.clone(), per_func);
    }
    ProjectAnalysisResults { results }
}
```

### LLM API 集成

替换 `infer_external_api_semantics_mock` 为真实调用：

```rust
pub async fn infer_external_api_semantics(
    function_name: &str,
    header_file_content: &str,
) -> Vec<String> {
    let prompt = build_prompt(function_name, header_file_content);
    
    // 实际 LLM 调用
    let response = call_openai_api(&prompt).await?;
    parse_llm_response(&response)
}
```

## 相关文档

- `docs/phase2_mir.md` - MIR 设计文档
- `docs/phase2_2_ast_to_mir.md` - AST→MIR 转换文档
- `README.md` - 项目总览

## 总结

阶段三实现了：
1. **静态分析管道架构**：可扩展的分析调度系统
2. **LLM 语义推断**：为外部 API 自动推断资源管理和所有权语义
3. **测试与示例**：完整的测试覆盖和可运行示例

这为后续的 Rust 代码生成奠定了坚实基础，使得转译结果能够：
- 正确管理资源（自动插入 Drop）
- 遵循 Rust 所有权规则
- 添加必要的安全检查
- 生成清晰的文档注释
