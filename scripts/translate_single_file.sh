#!/bin/bash

# 使用大模型翻译单个 C 文件的脚本
# 利用 1049K 上下文能力一次性翻译

set -e

if [ $# -lt 1 ]; then
    echo "用法: $0 <C源文件路径> [输出Rust文件路径]"
    echo "示例: $0 /workspace/translate_chibicc/src/tokenize.c tokenize.rs"
    exit 1
fi

INPUT_FILE=$1
OUTPUT_FILE=${2:-output.rs}

echo "================================"
echo "单文件翻译测试"
echo "================================"
echo "输入文件: $INPUT_FILE"
echo "输出文件: $OUTPUT_FILE"
echo ""

# 检查输入文件
if [ ! -f "$INPUT_FILE" ]; then
    echo "错误: 文件不存在: $INPUT_FILE"
    exit 1
fi

# 统计信息
LINES=$(wc -l < "$INPUT_FILE")
SIZE=$(du -h "$INPUT_FILE" | cut -f1)

echo "文件信息:"
echo "  行数: $LINES"
echo "  大小: $SIZE"
echo ""

# 读取文件内容
echo "读取 C 代码..."
C_CODE=$(cat "$INPUT_FILE")

# 构建完整的 Prompt
# 利用 1049K 上下文，可以包含大量上下文信息
echo "构建 Prompt (利用 1049K 上下文)..."

# 提取相关头文件
HEADER_DIR=$(dirname "$INPUT_FILE")
HEADER_FILE="${HEADER_DIR}/../chibicc.h"

CONTEXT=""
if [ -f "$HEADER_FILE" ]; then
    echo "  包含头文件: $HEADER_FILE"
    CONTEXT="头文件内容 (chibicc.h):\n\`\`\`c\n$(head -200 "$HEADER_FILE")\n\`\`\`\n\n"
fi

# 使用 translate_hybrid 的 LLM 客户端
cd /workspace/translate_hybrid

# 创建临时 Prompt 文件
PROMPT_FILE="/tmp/translate_prompt.txt"
cat > "$PROMPT_FILE" << EOF
你是一个精通 C 和 Rust 的系统编程专家。请将以下 C 代码翻译成地道的 Rust 代码。

要求:
1. 保持功能完全等价
2. 使用 Rust 的惯用法（idiomatic Rust）
3. 最小化 unsafe 使用（目标 <5%）
4. 添加必要的文档注释
5. 使用合适的 Rust 类型（Option, Result, Vec, Box 等）

$CONTEXT

C 代码 ($INPUT_FILE):
\`\`\`c
$C_CODE
\`\`\`

请直接返回 Rust 代码，格式如下:
\`\`\`rust
// 翻译后的 Rust 代码
\`\`\`
EOF

echo "Prompt 大小: $(wc -c < "$PROMPT_FILE") 字节"
echo ""

# 调用 LLM
echo "调用 LLM API (流式输出)..."
echo "================================"

RESPONSE=$(cargo run --release -- test-llm --prompt "$(cat "$PROMPT_FILE")" 2>&1)

echo ""
echo "================================"
echo ""

# 提取 Rust 代码块
echo "提取 Rust 代码..."
RUST_CODE=$(echo "$RESPONSE" | sed -n '/```rust/,/```/p' | sed '1d;$d')

if [ -n "$RUST_CODE" ]; then
    echo "$RUST_CODE" > "$OUTPUT_FILE"
    echo "✓ Rust 代码已保存至: $OUTPUT_FILE"
    echo ""
    
    # 统计生成的代码
    RUST_LINES=$(echo "$RUST_CODE" | wc -l)
    echo "生成的 Rust 代码:"
    echo "  行数: $RUST_LINES"
    echo "  原始 C 代码行数: $LINES"
    echo "  比例: $(echo "scale=2; $RUST_LINES / $LINES" | bc)"
    echo ""
    
    # 计算 unsafe 占比
    UNSAFE_LINES=$(echo "$RUST_CODE" | grep -c "unsafe" || echo 0)
    UNSAFE_RATIO=$(echo "scale=4; $UNSAFE_LINES / $RUST_LINES * 100" | bc)
    echo "  unsafe 占比: ${UNSAFE_RATIO}%"
    echo ""
    
    # 尝试编译
    echo "尝试编译 Rust 代码..."
    TEMP_DIR=$(mktemp -d)
    cat > "$TEMP_DIR/test.rs" << RUST_EOF
#![allow(unused)]
#![allow(dead_code)]

$RUST_CODE
RUST_EOF
    
    if rustc --crate-type lib "$TEMP_DIR/test.rs" -o "$TEMP_DIR/libtest.rlib" 2>&1 | head -50; then
        echo ""
        echo "✓ 编译成功！"
    else
        echo ""
        echo "✗ 编译失败，错误信息见上"
        echo ""
        echo "建议: 使用修复功能迭代改进"
    fi
    
    rm -rf "$TEMP_DIR"
else
    echo "✗ 未能提取 Rust 代码"
    echo "LLM 响应:"
    echo "$RESPONSE" | head -50
fi

echo ""
echo "翻译完成！"
