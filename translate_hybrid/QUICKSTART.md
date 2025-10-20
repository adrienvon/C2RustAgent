# Translate Hybrid - 快速开始指南

## 安装和配置

### 1. 进入子项目目录

```pwsh
cd translate_hybrid
```

### 2. 初始化配置文件

```pwsh
# 创建配置文件
cargo run -- init

# 编辑配置文件，设置你的 API Key
notepad config\hybrid_config.toml
```

配置文件示例：

```toml
[llm]
base_url = "https://router.shengsuanyun.com/api/v1"
api_key = "your-api-key-here"  # 替换为你的 API Key
model = "google/gemini-2.5-pro:discount"
temperature = 0.6
top_p = 0.7
max_tokens = 4000
stream = true
timeout = 120
```

### 3. 测试 LLM 连接

```pwsh
cargo run -- test-llm --prompt "Which number is larger, 9.11 or 9.8?"
```

如果看到流式输出并且最后显示 "✓ LLM 连接测试成功！"，说明配置正确。

## 使用示例

### 基础功能测试

```pwsh
# 查看版本
cargo run -- version

# 测试自定义提示词
cargo run -- test-llm --prompt "用 Rust 实现快速排序"
```

### 下一步开发

项目框架已搭建完成，包含：

1. ✅ **LLM 客户端** (`src/llm_client.rs`)
   - 支持自定义 API 端点
   - 流式响应
   - UTF-8 输出（无乱码）
   - 内置翻译、修复、优化方法

2. ✅ **工具函数** (`src/utils.rs`)
   - 彩色控制台输出
   - 代码块提取
   - unsafe 占比计算

3. ✅ **Prompt 模板** (`config/prompts/`)
   - 翻译模板
   - 语法修复模板
   - unsafe 优化模板

4. 🚧 **待实现模块**：
   - C 代码预处理器（集成主项目的 AST 解析）
   - 语法检查器（cargo check 集成）
   - unsafe 优化器
   - 项目构建器

## 开发工作流

### 添加新功能

1. 在 `src/` 创建新模块
2. 在 `src/lib.rs` 中导出
3. 编写单元测试
4. 更新 `README.md`

### 运行测试

```pwsh
cargo test
```

### 代码检查

```pwsh
cargo clippy
cargo fmt
```

## 故障排除

### Windows 控制台乱码

项目已集成 `console` crate 自动处理 UTF-8 输出。如果仍有问题：

```pwsh
# 设置控制台为 UTF-8
chcp 65001
```

### API 连接失败

1. 检查 `config/hybrid_config.toml` 中的 API Key 是否正确
2. 检查网络连接
3. 确认 API 端点 URL 正确
4. 查看日志：`cargo run -- --log-level debug test-llm`

### 依赖问题

```pwsh
# 清理并重新构建
cargo clean
cargo build
```

## 参考资料

- 主项目文档：`../README.md`
- API 文档：`cargo doc --open`
- 示例配置：`config/hybrid_config.toml.example`
