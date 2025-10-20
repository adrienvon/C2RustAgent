# 使用 translate_hybrid 翻译 chibicc 项目指南

## 快速开始

### 1. 进入 Docker 容器（如果使用 Docker）

```bash
# 在宿主机上
docker exec -it c2rust-test /bin/bash

# 或者在 PowerShell 中
docker exec -it c2rust-test /bin/bash
```

### 2. 进入 translate_hybrid 目录

```bash
cd translate_hybrid
```

### 3. 初始化配置（首次运行）

```bash
# 创建配置文件
cargo run -- init

# 编辑配置文件，设置 API Key
nano config/hybrid_config.toml
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

### 4. 测试 LLM 连接

```bash
cargo run -- test-llm --prompt "Hello, can you translate C code to Rust?"
```

如果看到流式输出和成功消息，说明配置正确。

## 翻译方式

### 方式 A：使用 PowerShell 脚本（Windows/Docker 推荐）

#### 翻译所有文件

```powershell
.\translate_chibicc.ps1
```

#### 翻译单个文件

```powershell
.\translate_chibicc.ps1 -SingleFile -File unicode.c
```

### 方式 B：使用 Bash 脚本（Linux/Mac）

```bash
chmod +x translate_chibicc.sh
./translate_chibicc.sh
```

### 方式 C：手动逐个翻译

```bash
# 翻译 unicode.c
cargo run -- translate \
    --input ../translate_chibicc/src/unicode.c \
    --output ./rust_output_chibicc/unicode.rs

# 翻译 strings.c
cargo run -- translate \
    --input ../translate_chibicc/src/strings.c \
    --output ./rust_output_chibicc/strings.rs

# 翻译 hashmap.c
cargo run -- translate \
    --input ../translate_chibicc/src/hashmap.c \
    --output ./rust_output_chibicc/hashmap.rs

# ...以此类推
```

## 验证和修复

### 1. 创建 Cargo 项目

```bash
cd rust_output_chibicc
cargo init --lib
```

### 2. 编译验证

```bash
cargo build
```

### 3. 如果有编译错误

```bash
# 保存错误信息
cargo check 2> errors.txt

# 使用 LLM 修复
cd ..
cargo run -- fix \
    --file ./rust_output_chibicc/unicode.rs \
    --errors ./rust_output_chibicc/errors.txt

# 重新验证
cd rust_output_chibicc
cargo check
```

### 4. 优化 unsafe 代码

```bash
# 查看当前 unsafe 占比（在翻译时会显示）

# 优化特定文件
cd ..
cargo run -- optimize-unsafe \
    --file ./rust_output_chibicc/unicode.rs

# 验证优化结果
cd rust_output_chibicc
cargo build
```

## 完整工作流示例

```bash
# 1. 进入 Docker 容器
docker exec -it c2rust-test /bin/bash

# 2. 进入项目目录
cd /workspace/translate_hybrid

# 3. 检查配置
cat config/hybrid_config.toml

# 4. 测试连接
cargo run -- test-llm

# 5. 批量翻译
./translate_chibicc.ps1

# 6. 进入输出目录
cd rust_output_chibicc

# 7. 初始化 Cargo 项目
cargo init --lib

# 8. 编译验证
cargo build

# 9. 运行测试（如果有）
cargo test

# 10. 检查代码质量
cargo clippy
```

## CLI 命令参考

### 查看帮助

```bash
cargo run -- --help
```

### 子命令列表

- `init` - 初始化配置文件
- `version` - 显示版本信息
- `test-llm` - 测试 LLM 连接
- `translate` - 翻译 C 文件到 Rust
- `fix` - 修复 Rust 语法错误
- `optimize-unsafe` - 优化 unsafe 代码

### translate 命令详细参数

```bash
cargo run -- translate --help

# 基本用法
cargo run -- translate \
    --input <C_FILE> \
    --output <RUST_FILE>

# 使用自定义提示词
cargo run -- translate \
    --input <C_FILE> \
    --output <RUST_FILE> \
    --prompt-file ./config/prompts/custom_translate.txt
```

### fix 命令详细参数

```bash
cargo run -- fix --help

# 基本用法
cargo run -- fix \
    --file <RUST_FILE> \
    --errors <ERROR_FILE>
```

### optimize-unsafe 命令详细参数

```bash
cargo run -- optimize-unsafe --help

# 覆盖原文件
cargo run -- optimize-unsafe --file <RUST_FILE>

# 输出到新文件
cargo run -- optimize-unsafe \
    --file <RUST_FILE> \
    --output <OUTPUT_FILE>
```

## 文件列表

chibicc 项目的 C 文件（按推荐翻译顺序）：

1. `unicode.c` - Unicode 处理（基础模块）
2. `strings.c` - 字符串工具（基础模块）
3. `hashmap.c` - 哈希表实现（基础模块）
4. `type.c` - 类型系统
5. `tokenize.c` - 词法分析
6. `parse.c` - 语法分析
7. `codegen.c` - 代码生成
8. `preprocess.c` - 预处理器
9. `main.c` - 主程序

建议先翻译基础模块（unicode, strings, hashmap），验证成功后再翻译其他模块。

## 预期结果

基于之前的成功经验（`rust_output_final/`）：

- ✅ **unicode.rs**: ~300 行，12/12 测试通过
- ✅ **strings.rs**: ~200 行
- ✅ **hashmap.rs**: ~250 行
- ✅ **type.rs**: ~195 行

总计：945 行 Rust 代码，编译成功，测试通过。

## 常见问题

### Q: LLM 连接超时

A: 增加 `config/hybrid_config.toml` 中的 `timeout` 值（默认 120 秒）：
```toml
timeout = 300  # 5 分钟
```

### Q: unsafe 占比过高

A: 使用 `optimize-unsafe` 命令多次优化：
```bash
cargo run -- optimize-unsafe --file <file>
# 检查结果
cargo build
# 如果还有问题，再次优化
```

### Q: 编译错误无法自动修复

A: 
1. 查看具体错误信息
2. 手动修改代码或调整提示词
3. 参考 `rust_output_final/` 中的成功案例

### Q: API 配额用完

A: 
1. 使用更便宜的模型（如 `gpt-4o-mini`）
2. 减少 `max_tokens` 值
3. 分批次翻译

## 性能优化建议

1. **并行翻译**: 如果 API 支持，可以修改脚本并行翻译多个文件
2. **缓存结果**: 成功翻译的文件建议备份
3. **渐进式验证**: 每翻译一个文件立即验证，避免累积错误
4. **使用快速模型**: 初次翻译用快速模型，优化时用更智能的模型

## 参考文档

- [translate_hybrid README](README.md)
- [translate_hybrid QUICKSTART](QUICKSTART.md)
- [成功案例](../rust_output_final/)
- [转换报告](../docs/reports/TRANSLATION_SUCCESS_SUMMARY.md)
