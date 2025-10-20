# Docker 翻译测试指南

使用 Docker 容器测试 chibicc 项目的 C 到 Rust 翻译，利用大模型的 1049K 上下文能力。

## 快速开始

### Windows 用户

```powershell
# 1. 确保 Docker Desktop 正在运行

# 2. 进入项目目录
cd C:\Users\baoba\Desktop\C2RustAgent

# 3. 运行 Docker 测试
.\scripts\docker_run.ps1
```

### Linux/Mac 用户

```bash
# 1. 进入项目目录
cd /path/to/C2RustAgent

# 2. 运行 Docker 测试
bash scripts/docker_run.sh
```

## 详细步骤

### 1. 首次运行（自动测试）

脚本会自动执行以下操作：

1. ✅ 构建 Docker 镜像（包含 Rust、Clang、LLVM）
2. ✅ 生成 `compile_commands.json`
3. ✅ 分析 chibicc 源代码
4. ✅ 运行 AST 到 MIR 转换
5. ✅ 检查 LLM 配置
6. ⏸️  等待你配置 API Key（如需使用 LLM）

### 2. 配置 LLM API（可选但推荐）

容器启动后，你会看到一个交互式 shell。配置 API：

```bash
# 在容器内执行
vi /workspace/translate_hybrid/config/hybrid_config.toml

# 或使用 nano（更简单）
nano /workspace/translate_hybrid/config/hybrid_config.toml
```

修改以下部分：

```toml
[llm]
base_url = "https://router.shengsuanyun.com/api/v1"
api_key = "你的-API-Key"  # 替换这里
model = "google/gemini-2.5-pro:discount"
temperature = 0.6
top_p = 0.7
max_tokens = 4000
stream = true
```

保存后（vi: `:wq`，nano: `Ctrl+X -> Y -> Enter`）

### 3. 测试 LLM 连接

```bash
cd /workspace/translate_hybrid
cargo run --release -- test-llm --prompt "Which number is larger, 9.11 or 9.8?"
```

如果看到流式输出和成功消息，配置正确！

### 4. 翻译单个文件（推荐先测试）

```bash
# 翻译 tokenize.c（较小，适合测试）
/workspace/scripts/translate_single_file.sh \
    /workspace/translate_chibicc/src/tokenize.c \
    /tmp/tokenize.rs

# 翻译 main.c
/workspace/scripts/translate_single_file.sh \
    /workspace/translate_chibicc/src/main.c \
    /tmp/main.rs
```

脚本会：
- ✅ 读取 C 源码
- ✅ 包含相关头文件作为上下文（利用 1049K 上下文）
- ✅ 调用 LLM 翻译
- ✅ 保存 Rust 代码
- ✅ 自动尝试编译
- ✅ 显示 unsafe 占比统计

### 5. 查看翻译结果

```bash
# 查看生成的 Rust 代码
cat /tmp/tokenize.rs

# 统计代码质量
echo "总行数: $(wc -l < /tmp/tokenize.rs)"
echo "unsafe 出现次数: $(grep -c 'unsafe' /tmp/tokenize.rs || echo 0)"

# 手动编译测试
rustc --crate-type lib /tmp/tokenize.rs 2>&1 | head -50
```

### 6. 迭代修复编译错误

如果编译失败，可以让 LLM 修复：

```bash
# 收集编译错误
ERRORS=$(rustc --crate-type lib /tmp/tokenize.rs 2>&1)

# 使用 LLM 修复（在 translate_hybrid 目录）
cd /workspace/translate_hybrid

# 创建修复 Prompt
cat > /tmp/fix_prompt.txt << EOF
以下 Rust 代码有编译错误，请修复：

\`\`\`rust
$(cat /tmp/tokenize.rs)
\`\`\`

编译错误：
\`\`\`
$ERRORS
\`\`\`

请返回修复后的完整代码。
EOF

# 调用 LLM 修复
cargo run --release -- test-llm --prompt "$(cat /tmp/fix_prompt.txt)" > /tmp/tokenize_v2.rs

# 再次测试
rustc --crate-type lib /tmp/tokenize_v2.rs
```

### 7. 翻译整个项目（需要时间）

```bash
# chibicc 包含约 8000 行代码
# 建议分批翻译

cd /workspace/translate_chibicc/src

# 列出所有 C 文件
ls *.c

# 逐个翻译
for file in tokenize.c parse.c codegen.c type.c; do
    echo "翻译 $file..."
    /workspace/scripts/translate_single_file.sh \
        "/workspace/translate_chibicc/src/$file" \
        "/tmp/rust_output/${file%.c}.rs"
done
```

## 性能与成本估算

### chibicc 项目规模

- **源文件**: 9 个 .c 文件
- **代码行数**: 约 8000 行
- **函数数量**: 约 200-300 个

### LLM 使用估算（基于 1049K 上下文）

单文件翻译：
- **输入 tokens**: 约 5K-15K（源码 + 上下文）
- **输出 tokens**: 约 5K-15K（Rust 代码）
- **单次调用**: 可翻译 500-1000 行 C 代码

全项目翻译：
- **总调用次数**: 约 10-15 次
- **总 tokens**: 约 200K-300K（输入+输出）
- **预计成本**: 根据 API 定价计算

### 时间估算

- **AST 解析**: < 1 分钟
- **单文件翻译**: 30 秒 - 2 分钟
- **全项目翻译**: 15-30 分钟
- **编译测试**: 每个文件 5-10 秒

## 常见问题

### Q: Docker 构建失败？

```bash
# 清理并重试
docker system prune -a
docker build -t c2rust-agent-translate -f Dockerfile.translate .
```

### Q: 容器内无法访问网络？

检查 Docker Desktop 的网络设置。

### Q: LLM API 调用失败？

1. 检查 API Key 是否正确
2. 检查网络连接
3. 查看详细日志：`cargo run --release -- --log-level debug test-llm`

### Q: Rust 编译错误太多？

1. 使用迭代修复功能
2. 调整 Prompt（降低 temperature）
3. 分批翻译，每次翻译更小的代码块

### Q: unsafe 占比过高（>5%）？

```bash
# 使用 unsafe 优化功能
cd /workspace/translate_hybrid

cargo run --release -- test-llm --prompt "
请优化以下 Rust 代码中的 unsafe 块：

\`\`\`rust
$(cat /tmp/tokenize.rs)
\`\`\`

要求：尽可能使用安全抽象替换裸指针操作。
"
```

## 查看测试报告

```bash
# 在容器内
cat /workspace/translation_report.txt
```

## 退出容器

```bash
# 在容器内
exit

# 容器会保留，可以重新进入
docker start -ai c2rust-test
```

## 清理

```bash
# 删除容器
docker rm c2rust-test

# 删除镜像
docker rmi c2rust-agent-translate

# 完全清理（谨慎！）
docker system prune -a
```

## 高级用法

### 保存翻译结果到宿主机

翻译结果自动保存在项目目录（通过 volume 挂载）：

```powershell
# Windows 上查看
cd C:\Users\baoba\Desktop\C2RustAgent
dir tmp\rust_output\
```

### 批量翻译脚本

创建自定义批量翻译脚本（在容器内）：

```bash
#!/bin/bash
# batch_translate.sh

OUTPUT_DIR="/workspace/rust_output"
mkdir -p "$OUTPUT_DIR"

for file in /workspace/translate_chibicc/src/*.c; do
    basename=$(basename "$file" .c)
    echo "翻译 $basename.c..."
    
    /workspace/scripts/translate_single_file.sh \
        "$file" \
        "$OUTPUT_DIR/${basename}.rs"
    
    # 等待避免 API 限流
    sleep 5
done

echo "全部翻译完成！输出目录: $OUTPUT_DIR"
```

## 下一步

1. ✅ 成功翻译单个文件
2. ✅ 验证编译通过
3. ✅ 优化 unsafe 占比
4. 🚧 翻译整个项目
5. 🚧 创建完整的 Cargo 项目
6. 🚧 运行原有测试用例
7. 🚧 性能对比

祝你翻译成功！🎉
