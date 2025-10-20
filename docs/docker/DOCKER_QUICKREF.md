# Docker 翻译测试 - 快速参考

## 🚀 一键启动

```powershell
# Windows
.\scripts\docker_run.ps1

# Linux/Mac  
bash scripts/docker_run.sh
```

## 📝 容器内常用命令

### 配置 LLM

```bash
# 编辑配置文件
nano /workspace/translate_hybrid/config/hybrid_config.toml

# 设置：
#   api_key = "你的密钥"
#   model = "google/gemini-2.5-pro:discount"
```

### 测试连接

```bash
cd /workspace/translate_hybrid
cargo run --release -- test-llm --prompt "Hello"
```

### 翻译文件

```bash
# 基础用法
/workspace/scripts/translate_single_file.sh <输入.c> <输出.rs>

# 示例：翻译 tokenize.c
/workspace/scripts/translate_single_file.sh \
    /workspace/translate_chibicc/src/tokenize.c \
    /tmp/tokenize.rs

# 查看结果
cat /tmp/tokenize.rs | less

# 编译测试
rustc --crate-type lib /tmp/tokenize.rs
```

### 批量翻译

```bash
# 翻译所有 .c 文件
cd /workspace/translate_chibicc/src
for f in *.c; do
    /workspace/scripts/translate_single_file.sh \
        "$f" "/tmp/${f%.c}.rs"
done
```

### 查看状态

```bash
# 项目信息
cat /workspace/translation_report.txt

# chibicc 源码统计
find /workspace/translate_chibicc/src -name "*.c" | xargs wc -l

# 检查生成的文件
ls -lh /tmp/*.rs
```

## 🔧 常见任务

### 修复编译错误

```bash
# 1. 收集错误
ERRORS=$(rustc --crate-type lib /tmp/file.rs 2>&1)

# 2. 让 LLM 修复
cd /workspace/translate_hybrid
cargo run --release -- test-llm --prompt "
修复以下 Rust 代码的编译错误：
\`\`\`rust
$(cat /tmp/file.rs)
\`\`\`
错误：$ERRORS
"
```

### 优化 unsafe

```bash
cd /workspace/translate_hybrid
cargo run --release -- test-llm --prompt "
优化以下代码的 unsafe 块：
\`\`\`rust
$(cat /tmp/file.rs)
\`\`\`
"
```

### 分析代码质量

```bash
# unsafe 占比
echo "scale=2; $(grep -c unsafe /tmp/file.rs) * 100 / $(wc -l < /tmp/file.rs)" | bc

# 代码行数对比
echo "C 代码: $(wc -l < src/file.c) 行"
echo "Rust 代码: $(wc -l < /tmp/file.rs) 行"
```

## 💾 保存结果

```bash
# 创建输出目录
mkdir -p /workspace/rust_output

# 复制翻译结果
cp /tmp/*.rs /workspace/rust_output/

# 在宿主机查看（Windows）
# 路径：C:\Users\baoba\Desktop\C2RustAgent\rust_output\
```

## 🛠️ 故障排除

### API 连接失败
```bash
# 检查配置
cat /workspace/translate_hybrid/config/hybrid_config.toml | grep api_key

# 测试网络
curl -I https://router.shengsuanyun.com
```

### 编译失败
```bash
# 详细错误信息
rustc --crate-type lib /tmp/file.rs --explain E0277

# 增加调试输出
RUST_BACKTRACE=1 rustc --crate-type lib /tmp/file.rs
```

### 容器问题
```bash
# 重启容器
exit
docker start -ai c2rust-test

# 重新构建
docker build -t c2rust-agent-translate -f Dockerfile.translate .
```

## 📊 性能指标

| 项目 | chibicc |
|------|---------|
| C 文件 | 9 个 |
| 代码行数 | ~8000 |
| 预计翻译时间 | 15-30 分钟 |
| API 调用次数 | ~10-15 |

## 🎯 推荐流程

1. ✅ 启动容器
2. ✅ 配置 API Key
3. ✅ 测试连接
4. ✅ 翻译 1 个小文件（tokenize.c）
5. ✅ 验证编译通过
6. ✅ 优化 unsafe
7. 🚧 翻译全部文件
8. 🚧 创建 Cargo 项目
9. 🚧 运行测试

## 🔗 相关文档

- 完整指南：`/workspace/DOCKER_GUIDE.md`
- 项目文档：`/workspace/translate_hybrid/README.md`
- 配置说明：`/workspace/translate_hybrid/QUICKSTART.md`

---

**提示**: 按 `Ctrl+D` 或输入 `exit` 退出容器
