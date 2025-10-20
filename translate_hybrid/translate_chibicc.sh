#!/bin/bash
# 使用 translate_hybrid 翻译 chibicc 项目的脚本

set -e

echo "================================"
echo "Chibicc C 到 Rust 翻译器"
echo "使用 translate_hybrid 子项目"
echo "================================"
echo ""

# 检查配置文件
CONFIG_FILE="config/hybrid_config.toml"
if [ ! -f "$CONFIG_FILE" ]; then
    echo "❌ 错误: 配置文件不存在: $CONFIG_FILE"
    echo "请先运行: cargo run -- init"
    exit 1
fi

# 检查 API Key
if ! grep -q "api_key.*=.*['\"]sk-" "$CONFIG_FILE" && ! grep -q "api_key.*=.*['\"].*[a-zA-Z0-9]" "$CONFIG_FILE"; then
    echo "⚠️  警告: 配置文件中可能未设置有效的 API Key"
    echo "请编辑 $CONFIG_FILE 并设置你的 API Key"
    exit 1
fi

# 源文件目录
SRC_DIR="../translate_chibicc/src"
OUTPUT_DIR="./rust_output_chibicc"

# 创建输出目录
mkdir -p "$OUTPUT_DIR"

# 要翻译的文件列表
FILES=(
    "unicode.c"
    "strings.c"
    "hashmap.c"
    "type.c"
    "tokenize.c"
    "parse.c"
    "codegen.c"
    "preprocess.c"
    "main.c"
)

echo "📁 源文件目录: $SRC_DIR"
echo "📁 输出目录: $OUTPUT_DIR"
echo "📝 待翻译文件: ${#FILES[@]} 个"
echo ""

# 逐个翻译文件
for c_file in "${FILES[@]}"; do
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📄 正在翻译: $c_file"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    input_file="$SRC_DIR/$c_file"
    output_file="$OUTPUT_DIR/${c_file%.c}.rs"
    
    if [ ! -f "$input_file" ]; then
        echo "⚠️  跳过: 文件不存在 $input_file"
        continue
    fi
    
    # 读取 C 代码
    c_code=$(cat "$input_file")
    
    # 构建提示词（读取 prompt 模板）
    if [ -f "config/prompts/translate.txt" ]; then
        system_prompt=$(cat "config/prompts/translate.txt")
    else
        system_prompt="你是 C 到 Rust 转换专家。请将给定的 C 代码翻译为地道的 Rust 代码。"
    fi
    
    # 使用 Rust CLI 调用翻译（我们需要添加这个命令）
    echo "🔄 调用 LLM 翻译..."
    cargo run --quiet -- translate \
        --input "$input_file" \
        --output "$output_file" \
        2>&1 || {
        echo "❌ 翻译失败: $c_file"
        continue
    }
    
    echo "✅ 翻译完成: $output_file"
    echo ""
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ 所有文件翻译完成！"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "下一步："
echo "1. 检查生成的代码: cd $OUTPUT_DIR"
echo "2. 创建 Cargo 项目: cargo init --lib"
echo "3. 编译验证: cargo build"
echo "4. 运行测试: cargo test"
echo ""
