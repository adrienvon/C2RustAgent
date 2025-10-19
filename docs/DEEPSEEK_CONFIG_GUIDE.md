# DeepSeek API 配置指南

本文档演示如何配置 C2RustAgent 使用 DeepSeek 大语言模型 API。

## ✅ 测试结果

**测试时间**: 2025年10月19日  
**测试状态**: ✅ 全部成功  
**API 地址**: https://api.deepseek.com  
**使用模型**: deepseek-coder  

## 📋 配置步骤

### 1. 创建配置文件

在项目根目录创建 `c2rust-agent.toml` 文件：

```toml
# C2RustAgent 配置 - DeepSeek API
# 本配置文件用于连接 DeepSeek 大语言模型

# ========================================
# DeepSeek API 设置
# ========================================

# API 提供商（保持为 openai，因为 DeepSeek 兼容 OpenAI API）
provider = "openai"

# DeepSeek API Key
api_key = "sk-7862c4aa401c403ea499e582eaf14f88"

# DeepSeek API 地址
api_url = "https://api.deepseek.com"

# DeepSeek 模型名称
# 可选模型：
# - deepseek-chat      (推荐，最新对话模型)
# - deepseek-coder     (代码专用模型，推荐用于 C2Rust)
model = "deepseek-coder"

# ========================================
# 模型参数
# ========================================

# 温度参数（0.0-2.0）
# DeepSeek 推荐：0.0-0.3 用于代码生成
temperature = 0.3

# 最大生成 token 数
# DeepSeek 支持更大的上下文窗口
max_tokens = 2000

# 是否使用 Mock 模式（用于测试）
use_mock = false
```

### 2. 验证配置

使用配置管理工具验证配置是否正确：

```powershell
# 查看当前配置
cargo run --bin c2rust-agent-config -- show --verbose
```

**预期输出**：
```
📋 当前有效配置：

  Provider:     openai
  Model:        deepseek-coder
  Temperature:  0.3
  Max Tokens:   2000
  Use Mock:     false
  API URL:      https://api.deepseek.com
  API Key:      sk-786...4f88

📍 配置来源（优先级从高到低）：
  1. 环境变量
  2. 用户配置：C:\Users\...\c2rust-agent\config.toml ❌
  3. 项目配置：c2rust-agent.toml ✅
  4. 默认值 ✅

✅ 配置有效
```

### 3. 运行测试

运行 DeepSeek API 测试示例：

```powershell
cargo run --example test_deepseek
```

**测试结果示例**：

#### 测试 1: 推断 malloc 函数语义
```
函数签名: void* malloc(size_t size)
✅ 成功获取语义信息:
   [ReturnsNewResource(free)]
```

#### 测试 2: 生成模块文档
```
✅ 成功生成文档:
//! 数学计算模块，提供基础的算术运算功能。
//!
//! 此模块是从 C 语言文件 `math.c` 自动翻译而来，可能包含不安全的代码模式。
//! 在使用前建议进行全面的安全性审查，特别是对以下方面：
//! - 指针操作和内存管理
//! - 边界检查和溢出处理
//! - 类型转换的安全性
//!
//! 请确保所有使用场景都经过充分测试。
```

#### 测试 3: 生成 unsafe 代码说明
```
✅ 成功生成说明:
// SAFETY: This block uses unsafe due to:
// - Raw pointer dereference violates Rust's memory safety guarantees
// - FFI call to libc::malloc bypasses Rust's ownership system
//
// Invariants required:
// - malloc(10) must return a valid, aligned pointer to 10-byte memory
// - Pointer must not be null and must be writable
// - Memory region must not overlap with other active references
//
// Potential risks:
// - UB if pointer is null, unaligned, or points to invalid memory
// - Memory leak if not paired with free()
// - Use-after-free if pointer escapes scope
//
// Safety argument:
// - malloc success checked implicitly by non-null assumption
// - Single-byte write stays within 10-byte allocated bounds
// - No concurrent access to this memory region
```

## 🎯 DeepSeek 特点

### 为什么选择 DeepSeek

1. **代码专用模型** - `deepseek-coder` 专门针对代码任务优化
2. **高性价比** - 相比 GPT-4，成本更低
3. **大上下文窗口** - 支持更长的代码片段
4. **OpenAI 兼容** - 无需修改代码，直接切换

### 模型选择建议

| 模型 | 用途 | 特点 |
|------|------|------|
| `deepseek-coder` | C 到 Rust 转换（推荐） | 专注代码理解和生成 |
| `deepseek-chat` | 通用对话任务 | 更好的自然语言理解 |

### 参数调优建议

```toml
# 代码生成（推荐）
temperature = 0.3   # 低温度，确保输出稳定
max_tokens = 2000   # 足够生成完整注释

# 创意性任务
temperature = 0.7   # 更高的随机性
max_tokens = 3000   # 更长的输出
```

## 💡 使用技巧

### 1. 配置文件管理

**项目配置** vs **用户配置**：

```powershell
# 项目配置（推荐用于团队协作）
./c2rust-agent.toml

# 用户配置（推荐用于个人开发）
cargo run --bin c2rust-agent-config -- init
# Windows: %APPDATA%\c2rust-agent\config.toml
# Linux/macOS: ~/.config/c2rust-agent/config.toml
```

**优先级顺序**：
1. 环境变量 ← 最高优先级
2. 用户配置文件
3. 项目配置文件
4. 默认值 ← 最低优先级

### 2. 切换不同 API 提供商

只需修改配置文件，无需更改代码：

```toml
# 使用 DeepSeek
provider = "openai"
api_url = "https://api.deepseek.com"
api_key = "sk-your-deepseek-key"
model = "deepseek-coder"

# 切换到 OpenAI
# provider = "openai"
# api_url = "https://api.openai.com/v1"  # 或省略使用默认值
# api_key = "sk-your-openai-key"
# model = "gpt-4o-mini"

# 使用本地模型（如 Ollama）
# provider = "openai"
# api_url = "http://localhost:11434/v1"
# model = "llama3"
# api_key = "dummy"  # 本地模型可能不需要
```

### 3. 安全最佳实践

**⚠️ 不要将 API Key 提交到 Git！**

```bash
# 添加到 .gitignore
echo 'c2rust-agent.toml' >> .gitignore
```

**推荐做法**：
- 开发环境：使用用户配置文件
- CI/CD：使用环境变量
- 团队协作：项目配置文件不包含 Key，每个成员设置自己的环境变量

```powershell
# 使用环境变量（优先级最高）
$env:OPENAI_API_KEY="sk-your-key"
$env:C2RUST_AGENT_API_URL="https://api.deepseek.com"
```

## 📊 成本对比

| 提供商 | 模型 | 输入成本 | 输出成本 | 适用场景 |
|--------|------|---------|---------|---------|
| DeepSeek | deepseek-coder | $0.14/1M tokens | $0.28/1M tokens | 代码转换（推荐） |
| OpenAI | gpt-4o-mini | $0.15/1M tokens | $0.60/1M tokens | 通用任务 |
| OpenAI | gpt-4o | $2.50/1M tokens | $10.00/1M tokens | 高质量要求 |

**估算**（基于 DeepSeek）：
- 转换 1000 行 C 代码：约 $0.05 - $0.15
- 生成 100 个函数注释：约 $0.02 - $0.05

## 🔍 故障排查

### 问题 1: API 调用失败

**症状**：
```
⚠️  未获取到语义信息（可能是 API 调用失败或网络问题）
```

**排查步骤**：
1. 检查 API Key 是否正确
2. 检查网络连接
3. 验证 API URL 是否正确（`https://api.deepseek.com`）
4. 查看 DeepSeek 账户余额

```powershell
# 验证配置
cargo run --bin c2rust-agent-config -- validate
```

### 问题 2: 模型名称错误

**错误信息**：
```
Model not found: deepseek-coder
```

**解决方案**：
- 确认使用正确的模型名称
- 参考 DeepSeek 官方文档获取最新模型列表

### 问题 3: 配置文件不生效

**排查步骤**：
```powershell
# 查看配置来源
cargo run --bin c2rust-agent-config -- show --verbose
```

检查 "配置来源" 部分，确认配置文件路径正确且文件存在（✅）。

## 🚀 下一步

配置成功后，您可以：

1. **运行主程序**：
   ```powershell
   cargo run
   ```

2. **运行完整测试**：
   ```powershell
   cargo test
   ```

3. **开始转换 C 代码**：
   参考 [README.md](../README.md) 了解如何使用 C2RustAgent 转换项目

## 📚 相关资源

- [DeepSeek 官方文档](https://platform.deepseek.com/docs)
- [DeepSeek API 参考](https://platform.deepseek.com/api-docs)
- [C2RustAgent 配置文档](./QUICKSTART_CONFIG.md)
- [OpenAI API 集成指南](./openai_api_integration.md)

---

**配置演示成功！** 🎉

如有问题，请查看 [GitHub Issues](https://github.com/your-repo/issues) 或联系技术支持。
