# 快速使用指南：LLM 辅助外部 API 语义推断

## 快速开始

### 1. 运行演示

```bash
cargo run --example llm_external_api_demo
```

**输出示例：**
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
```

### 2. 在代码中使用

```rust
use c2rust_agent::llm_assists;

#[tokio::main]
async fn main() {
    // 推断外部函数语义
    let semantics = llm_assists::infer_external_api_semantics(
        "fopen",  // 函数名
        "FILE* fopen(const char*, const char*);"  // 头文件内容
    ).await;
    
    // 使用推断结果
    for tag in semantics {
        println!("{}", tag);
    }
}
```

### 3. 集成到转换流程

```rust
// 在 AST→MIR 转换时
match entity.get_kind() {
    EntityKind::CallExpr => {
        let func_name = get_function_name(entity);
        
        // 检查是否为外部函数
        if is_external_function(&func_name) {
            // 读取头文件
            let header = read_header_for_function(&func_name)?;
            
            // 异步推断语义
            let semantics = llm_assists::infer_external_api_semantics(
                &func_name, 
                &header
            ).await;
            
            // 将语义添加到 MIR Call 语句
            let call_stmt = Statement::Call(target, func_name, args);
            let annotated_stmt = Statement::Annotated {
                stmt: Box::new(call_stmt),
                annotations: semantics,
            };
        }
    }
    _ => {}
}
```

## 语义标注参考

| 标注 | 含义 | Rust 对应 |
|------|------|-----------|
| `[ReturnsNewResource(fn)]` | 返回值需要特定函数释放 | Drop trait, RAII |
| `[TakesOwnership(param)]` | 函数接管参数所有权 | move semantics |
| `[HasSideEffects]` | 有副作用（I/O、全局状态） | unsafe 块候选 |
| `[Pure]` | 纯函数，无副作用 | const fn 候选 |
| `[RequiresNonNull(param)]` | 参数不能为 NULL | Option<NonNull<T>> |
| `[RequiresValidPointer(param)]` | 参数必须是有效指针 | 借用检查 |
| `[ReturnsValidUntil(cond)]` | 返回值有生命周期限制 | 'a 注解 |

## 当前支持的函数

### 内存管理
- `malloc`, `calloc`, `realloc` → `[ReturnsNewResource(free)]`
- `free` → `[TakesOwnership(ptr)]`

### 文件 I/O
- `fopen`, `fdopen`, `freopen` → `[ReturnsNewResource(fclose)]`, `[HasSideEffects]`
- `fclose` → `[TakesOwnership(stream)]`, `[HasSideEffects]`
- `printf`, `fprintf`, `sprintf`, `scanf`, `fscanf` → `[HasSideEffects]`

### 字符串操作
- `strlen`, `strcmp`, `strchr` → `[Pure]`, `[RequiresNonNull(str)]`
- `strcpy`, `strcat`, `memcpy` → `[HasSideEffects]`, `[RequiresNonNull(...)]`

### 环境变量
- `getenv` → `[HasSideEffects]`, `[ReturnsValidUntil(next_getenv_call)]`

### 未知函数
- 其他函数 → `[HasSideEffects]`, `[Unknown]`

## 运行测试

```bash
# 所有测试
cargo test

# 仅 LLM 相关测试
cargo test llm_assists
```

## 未来扩展

当前实现使用基于规则的模拟推断。集成真实 LLM API 后，将能够：

1. **处理任意外部函数**：不限于标准库
2. **理解注释和文档**：利用头文件中的注释
3. **推断复杂语义**：如回调函数的生命周期
4. **生成详细文档**：为转译后的 Rust 代码添加说明

### 集成 OpenAI API 示例

```rust
async fn call_llm_api(prompt: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(std::env::var("OPENAI_API_KEY")?)
        .json(&serde_json::json!({
            "model": "gpt-4",
            "messages": [{"role": "user", "content": prompt}],
        }))
        .send()
        .await?;
    
    let result: serde_json::Value = response.json().await?;
    Ok(result["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string())
}
```

## 相关文档

- `docs/phase3_analysis_and_llm.md` - 完整技术文档
- `docs/phase3_summary.md` - 实现总结
- `examples/llm_external_api_demo.rs` - 示例代码

## 问题排查

### 编译错误

确保 `Cargo.toml` 包含：
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

### 运行时错误

异步函数需要在 `tokio` 运行时中执行：
```rust
#[tokio::main]
async fn main() {
    // 你的代码
}
```

或在测试中：
```rust
#[tokio::test]
async fn test_something() {
    // 测试代码
}
```
