# chibicc 翻译测试 - 快速命令参考

## 🚀 启动测试

### 基础测试（推荐先运行）

```powershell
# Windows
.\scripts\docker_run.ps1

# 会执行:
# - 环境检查
# - 生成 compile_commands.json
# - AST 解析测试
# - 进入交互式 shell
```

### 完整翻译测试（翻译所有 9 个文件）

```powershell
# Windows
.\scripts\docker_run.ps1 -FullTranslation

# 会执行:
# - 所有基础测试
# - 翻译 9 个 C 文件到 Rust
# - 自动编译验证
# - 生成详细报告
# 预计耗时: 15-30 分钟
```

## 📝 配置 API（必需）

### 首次使用前配置

容器启动后，在 shell 中运行：

```bash
# 复制配置模板
cp /workspace/translate_hybrid/config/hybrid_config.toml.example \
   /workspace/translate_hybrid/config/hybrid_config.toml

# 编辑配置
nano /workspace/translate_hybrid/config/hybrid_config.toml
```

配置内容：

```toml
[llm]
base_url = "https://router.shengsuanyun.com/api/v1"
api_key = "你的API密钥"  # <-- 修改这里
model = "google/gemini-2.5-pro:discount"
temperature = 0.6
top_p = 0.7
max_tokens = 4000
stream = true
```

保存：`Ctrl+X` → `Y` → `Enter`

## 🧪 测试命令

### 测试 LLM 连接

```bash
cd /workspace/translate_hybrid
cargo run --release -- test-llm --prompt "Hello, test!"
```

✅ 成功标志：看到流式输出和 "✓ LLM 连接测试成功！"

### 手动翻译单个文件

```bash
# 翻译最简单的文件（unicode.c，~100 行）
/workspace/scripts/translate_single_file.sh \
    /workspace/translate_chibicc/src/unicode.c \
    /tmp/unicode.rs

# 翻译 tokenize.c（~1000 行）
/workspace/scripts/translate_single_file.sh \
    /workspace/translate_chibicc/src/tokenize.c \
    /tmp/tokenize.rs
```

### 运行完整翻译（在容器内）

```bash
# 如果你在交互式 shell 中
/workspace/scripts/translate_chibicc_full.sh
```

## 📊 查看结果

### 翻译结果

```bash
# 查看生成的 Rust 文件
ls -lh /workspace/rust_output/

# 查看特定文件
cat /workspace/rust_output/tokenize.rs | less

# 统计信息
wc -l /workspace/rust_output/*.rs
```

### 编译状态

```bash
# 查看编译错误（如有）
cat /workspace/rust_output/*.rs.errors

# 统计编译错误数
grep -c "error\[" /workspace/rust_output/*.rs.errors 2>/dev/null || echo 0
```

### 测试报告

```bash
# 完整翻译报告
cat /workspace/chibicc_translation_report.txt

# 基础测试报告
cat /workspace/translation_report.txt
```

## 🔧 编译测试

### 手动编译单个文件

```bash
# 创建测试文件
cat > /tmp/test.rs << 'EOF'
#![allow(unused)]
#![allow(dead_code)]

// 粘贴翻译的代码
EOF

# 编译为库
rustc --crate-type lib /tmp/test.rs

# 查看详细错误
rustc --crate-type lib /tmp/test.rs 2>&1 | less
```

### 批量编译测试

```bash
# 测试所有生成的文件
for f in /workspace/rust_output/*.rs; do
    echo "测试 $(basename $f)..."
    rustc --crate-type lib "$f" 2>&1 | head -5
done
```

## 🛠️ 迭代修复

### 使用 LLM 修复编译错误

```bash
# 收集错误
RUST_FILE="/workspace/rust_output/tokenize.rs"
ERRORS=$(rustc --crate-type lib "$RUST_FILE" 2>&1)

# 创建修复 Prompt
cat > /tmp/fix_prompt.txt << EOF
以下 Rust 代码有编译错误，请修复：

\`\`\`rust
$(cat "$RUST_FILE")
\`\`\`

编译错误：
\`\`\`
$ERRORS
\`\`\`

请返回修复后的完整代码，用 \`\`\`rust 包裹。
EOF

# 调用 LLM 修复
cd /workspace/translate_hybrid
cargo run --release -- test-llm --prompt "$(cat /tmp/fix_prompt.txt)"
```

## 📈 统计分析

### unsafe 占比分析

```bash
# 单个文件
RUST_FILE="/workspace/rust_output/tokenize.rs"
TOTAL_LINES=$(wc -l < "$RUST_FILE")
UNSAFE_LINES=$(grep -c "unsafe" "$RUST_FILE" || echo 0)
echo "unsafe 占比: $(echo "scale=2; $UNSAFE_LINES * 100 / $TOTAL_LINES" | bc)%"

# 所有文件
for f in /workspace/rust_output/*.rs; do
    TOTAL=$(wc -l < "$f")
    UNSAFE=$(grep -c "unsafe" "$f" || echo 0)
    RATIO=$(echo "scale=2; $UNSAFE * 100 / $TOTAL" | bc)
    echo "$(basename $f): ${RATIO}%"
done
```

### 代码量对比

```bash
# C 代码总行数
find /workspace/translate_chibicc/src -name "*.c" | xargs wc -l | tail -1

# Rust 代码总行数
wc -l /workspace/rust_output/*.rs | tail -1

# 对比
echo "C 代码: $(find /workspace/translate_chibicc/src -name "*.c" | xargs cat | wc -l) 行"
echo "Rust 代码: $(cat /workspace/rust_output/*.rs | wc -l) 行"
```

## 💾 保存结果

### 复制到宿主机

翻译结果已通过 Docker volume 自动同步：

```powershell
# Windows 上查看
cd C:\Users\baoba\Desktop\C2RustAgent\rust_output
dir
```

### 创建 Git 提交

```bash
# 在容器内
cd /workspace
git add rust_output/
git commit -m "Add translated Rust code from chibicc"
```

## 🎯 chibicc 文件清单

翻译顺序（从简单到复杂）：

| 文件 | 行数 | 复杂度 | 用途 |
|------|------|--------|------|
| unicode.c | ~100 | ⭐ | Unicode 处理 |
| strings.c | ~150 | ⭐ | 字符串工具 |
| hashmap.c | ~200 | ⭐⭐ | 哈希表 |
| tokenize.c | ~1000 | ⭐⭐⭐ | 词法分析器 |
| type.c | ~500 | ⭐⭐⭐ | 类型系统 |
| preprocess.c | ~1000 | ⭐⭐⭐⭐ | 预处理器 |
| parse.c | ~3000 | ⭐⭐⭐⭐⭐ | 语法分析器 |
| codegen.c | ~1500 | ⭐⭐⭐⭐ | 代码生成器 |
| main.c | ~700 | ⭐⭐⭐ | 主程序 |

**总计**: ~8,150 行 C 代码

## ⚡ 性能提示

### 减少 API 调用

```bash
# 只翻译小文件测试
for f in unicode.c strings.c hashmap.c; do
    /workspace/scripts/translate_single_file.sh \
        "/workspace/translate_chibicc/src/$f" \
        "/workspace/rust_output/${f%.c}.rs"
done
```

### 并行处理（需要更多 API 配额）

```bash
# 不推荐：可能触发限流
# 仅当你有足够配额时使用
```

## 🆘 故障排除

### API 调用失败

```bash
# 检查配置
cat /workspace/translate_hybrid/config/hybrid_config.toml | grep -E "(api_key|base_url|model)"

# 测试网络
curl -I https://router.shengsuanyun.com

# 增加日志
cd /workspace/translate_hybrid
cargo run --release -- --log-level debug test-llm
```

### 编译错误过多

```bash
# 降低复杂度：先翻译简单文件
# 调整 Prompt：增加示例
# 减少 temperature：提高确定性
```

### Docker 问题

```bash
# 重启容器
exit
docker start -ai c2rust-test

# 重新构建
docker build -t c2rust-agent-translate -f Dockerfile.translate .
```

## 📚 相关文档

- 完整指南: `cat /workspace/DOCKER_GUIDE.md | less`
- 项目总结: `cat /workspace/DOCKER_SUMMARY.md | less`
- 子项目文档: `cat /workspace/translate_hybrid/README.md | less`

---

**快速上手**:
1. `.\scripts\docker_run.ps1` - 启动容器
2. 配置 API Key
3. `./workspace/scripts/translate_chibicc_full.sh` - 开始翻译
4. 查看 `/workspace/rust_output/` 结果
