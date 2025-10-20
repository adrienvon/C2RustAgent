#!/bin/bash
# 简单的 C 到 Rust 转换测试脚本（基于规则，不使用 LLM）
# 用于测试编译流程和发现问题

set -e

echo "==================================="
echo "C to Rust 简单转换测试（chibicc）"
echo "==================================="

# 配置
C_PROJECT_DIR="/workspace/translate_chibicc/src"
OUTPUT_DIR="/workspace/rust_output_simple"
REPORT_FILE="/workspace/simple_translation_report.txt"

# 创建输出目录
mkdir -p "$OUTPUT_DIR"

# 统计信息
TOTAL_FILES=0
COMPILED_FILES=0
FAILED_FILES=0

# 开始报告
cat > "$REPORT_FILE" << EOF
================================
chibicc 简单转换测试报告
================================
测试时间: $(date)
模式: 基于规则的简单转换（不使用 LLM）

EOF

# 简单的C到Rust转换函数
simple_translate() {
    local c_file="$1"
    local rs_file="$2"
    local base_name=$(basename "$c_file" .c)
    
    echo "📝 转换: $base_name.c -> $base_name.rs"
    
    # 创建一个最简单的 Rust 文件框架
    cat > "$rs_file" << 'RUST_EOF'
// 自动从 C 代码转换而来（简单规则版本）
// 这是一个占位实现，用于测试编译流程

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::ffi::CStr;

// C 字符串工具
pub fn c_str_to_str(c_str: *const c_char) -> &'static str {
    if c_str.is_null() {
        return "";
    }
    unsafe {
        CStr::from_ptr(c_str).to_str().unwrap_or("")
    }
}

// 基本类型别名
pub type size_t = usize;
pub type FILE = c_void;
pub type va_list = *mut c_void;

// 占位结构体和函数
// TODO: 需要根据实际 C 代码生成

pub fn placeholder_function() -> c_int {
    0
}
RUST_EOF
    
    echo "✓ 生成基础 Rust 文件: $rs_file"
}

# 编译测试
compile_test() {
    local rs_file="$1"
    local base_name=$(basename "$rs_file" .rs)
    local error_file="${rs_file}.errors"
    
    echo "🔨 编译测试: $base_name.rs"
    
    if rustc --crate-type lib --edition 2021 "$rs_file" -o "/tmp/${base_name}.rlib" 2> "$error_file"; then
        echo "✓ 编译成功: $base_name.rs"
        rm -f "$error_file"
        return 0
    else
        echo "✗ 编译失败: $base_name.rs"
        echo "  错误日志: $error_file"
        return 1
    fi
}

# 要转换的文件（按复杂度排序）
declare -a C_FILES=(
    "unicode.c"
    "strings.c"
    "hashmap.c"
)

echo ""
echo "开始转换 ${#C_FILES[@]} 个文件..."
echo ""

# 转换每个文件
for C_FILE in "${C_FILES[@]}"; do
    TOTAL_FILES=$((TOTAL_FILES + 1))
    
    C_PATH="$C_PROJECT_DIR/$C_FILE"
    BASE_NAME=$(basename "$C_FILE" .c)
    RS_PATH="$OUTPUT_DIR/${BASE_NAME}.rs"
    
    echo "----------------------------------------"
    echo "[$TOTAL_FILES/${#C_FILES[@]}] 处理: $C_FILE"
    echo "----------------------------------------"
    
    if [ ! -f "$C_PATH" ]; then
        echo "✗ 文件不存在: $C_PATH"
        FAILED_FILES=$((FAILED_FILES + 1))
        continue
    fi
    
    # 转换
    simple_translate "$C_PATH" "$RS_PATH"
    
    # 编译测试
    if compile_test "$RS_PATH"; then
        COMPILED_FILES=$((COMPILED_FILES + 1))
        echo "✓ [$BASE_NAME] 转换并编译成功"
        echo "  ✓ ${BASE_NAME}.rs" >> "$REPORT_FILE"
    else
        FAILED_FILES=$((FAILED_FILES + 1))
        echo "✗ [$BASE_NAME] 编译失败"
        echo "  ✗ ${BASE_NAME}.rs (编译错误)" >> "$REPORT_FILE"
    fi
    
    echo ""
done

# 生成最终报告
cat >> "$REPORT_FILE" << EOF

================================
统计信息
================================
总文件数: $TOTAL_FILES
编译成功: $COMPILED_FILES
编译失败: $FAILED_FILES
成功率: $(( COMPILED_FILES * 100 / TOTAL_FILES ))%

输出目录: $OUTPUT_DIR
EOF

echo "==================================="
echo "转换测试完成！"
echo "==================================="
echo "总文件数: $TOTAL_FILES"
echo "编译成功: $COMPILED_FILES"
echo "编译失败: $FAILED_FILES"
echo "成功率: $(( COMPILED_FILES * 100 / TOTAL_FILES ))%"
echo ""
echo "报告文件: $REPORT_FILE"
echo "输出目录: $OUTPUT_DIR"
echo ""

if [ $FAILED_FILES -gt 0 ]; then
    echo "⚠️  有 $FAILED_FILES 个文件编译失败"
    echo "查看错误: cat $OUTPUT_DIR/*.errors"
    exit 1
else
    echo "✅ 所有文件编译成功！"
    exit 0
fi
