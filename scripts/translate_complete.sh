#!/bin/bash
# 完整的 chibicc C 到 Rust 转换流程

set -e

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo_info() { echo -e "${BLUE}ℹ${NC} $1"; }
echo_success() { echo -e "${GREEN}✓${NC} $1"; }
echo_error() { echo -e "${RED}✗${NC} $1"; }

# 配置
C_DIR="/workspace/translate_chibicc/src"
OUT_DIR="/workspace/rust_output_final"
REPORT="/workspace/final_report.txt"

mkdir -p "$OUT_DIR"

echo "========================================="
echo "chibicc 完整转换流程"
echo "========================================="
echo ""

# 步骤 1: 生成公共类型
echo_info "步骤 1/4: 生成公共类型定义..."
bash /workspace/scripts/generate_types.sh
echo_success "types.rs 已生成"
echo ""

# 测试types.rs能否编译
echo_info "测试 types.rs 编译..."
if rustc --crate-type lib --edition 2021 "$OUT_DIR/types.rs" -o "/tmp/types.rlib" 2> "$OUT_DIR/types.rs.errors"; then
    echo_success "types.rs 编译通过"
    rm -f "$OUT_DIR/types.rs.errors"
else
    echo_error "types.rs 编译失败"
    cat "$OUT_DIR/types.rs.errors"
    exit 1
fi
echo ""

# 步骤 2: 转换 unicode.c（最简单）
echo_info "步骤 2/4: 转换 unicode.c..."
cat > "$OUT_DIR/unicode.rs" << 'UNICODE_EOF'
//! Unicode 处理模块
//! 从 unicode.c 转换而来

#![allow(dead_code)]
#![allow(unused_variables)]

use std::os::raw::{c_char, c_int};
use crate::types::uint32_t;

/// UTF-8 编码
/// 将 Unicode 码点编码为 UTF-8
pub unsafe fn encode_utf8(buf: *mut c_char, c: uint32_t) -> c_int {
    let buf = buf as *mut u8;
    
    if c <= 0x7F {
        *buf = c as u8;
        return 1;
    }
    
    if c <= 0x7FF {
        *buf.offset(0) = (0b11000000 | (c >> 6)) as u8;
        *buf.offset(1) = (0b10000000 | (c & 0b00111111)) as u8;
        return 2;
    }
    
    if c <= 0xFFFF {
        *buf.offset(0) = (0b11100000 | (c >> 12)) as u8;
        *buf.offset(1) = (0b10000000 | ((c >> 6) & 0b00111111)) as u8;
        *buf.offset(2) = (0b10000000 | (c & 0b00111111)) as u8;
        return 3;
    }
    
    *buf.offset(0) = (0b11110000 | (c >> 18)) as u8;
    *buf.offset(1) = (0b10000000 | ((c >> 12) & 0b00111111)) as u8;
    *buf.offset(2) = (0b10000000 | ((c >> 6) & 0b00111111)) as u8;
    *buf.offset(3) = (0b10000000 | (c & 0b00111111)) as u8;
    4
}

/// UTF-8 解码
/// 从 UTF-8 序列解码 Unicode 码点
pub unsafe fn decode_utf8(new_pos: *mut *mut c_char, p: *mut c_char) -> uint32_t {
    let p_u8 = p as *const u8;
    
    if *p_u8 < 128 {
        *new_pos = p.offset(1);
        return *p_u8 as uint32_t;
    }
    
    let start = p;
    let len: c_int;
    let mut c: uint32_t;
    
    if *p_u8 >= 0b11110000 {
        len = 4;
        c = (*p_u8 & 0b111) as uint32_t;
    } else if *p_u8 >= 0b11100000 {
        len = 3;
        c = (*p_u8 & 0b1111) as uint32_t;
    } else if *p_u8 >= 0b11000000 {
        len = 2;
        c = (*p_u8 & 0b11111) as uint32_t;
    } else {
        // 错误：无效的 UTF-8 序列
        // error_at(start, "invalid UTF-8 sequence");
        panic!("invalid UTF-8 sequence");
    }
    
    for i in 1..len {
        let byte = *p_u8.offset(i as isize);
        if (byte >> 6) != 0b10 {
            panic!("invalid UTF-8 sequence");
        }
        c = (c << 6) | ((byte & 0b111111) as uint32_t);
    }
    
    *new_pos = p.offset(len as isize);
    c
}

/// 检查字符是否在指定范围内
unsafe fn in_range(range: *const uint32_t, c: uint32_t) -> bool {
    let mut i = 0;
    loop {
        let val = *range.offset(i);
        if val == u32::MAX {
            break;
        }
        let start = val;
        let end = *range.offset(i + 1);
        if start <= c && c <= end {
            return true;
        }
        i += 2;
    }
    false
}

/// 检查字符是否可以作为标识符的第一个字符
pub fn is_ident1(c: uint32_t) -> bool {
    static RANGE: [uint32_t; 113] = [
        b'_' as u32, b'_' as u32, b'a' as u32, b'z' as u32, 
        b'A' as u32, b'Z' as u32, b'$' as u32, b'$' as u32,
        0x00A8, 0x00A8, 0x00AA, 0x00AA, 0x00AD, 0x00AD, 0x00AF, 0x00AF,
        0x00B2, 0x00B5, 0x00B7, 0x00BA, 0x00BC, 0x00BE, 0x00C0, 0x00D6,
        0x00D8, 0x00F6, 0x00F8, 0x00FF, 0x0100, 0x02FF, 0x0370, 0x167F,
        0x1681, 0x180D, 0x180F, 0x1DBF, 0x1E00, 0x1FFF, 0x200B, 0x200D,
        0x202A, 0x202E, 0x203F, 0x2040, 0x2054, 0x2054, 0x2060, 0x206F,
        0x2070, 0x20CF, 0x2100, 0x218F, 0x2460, 0x24FF, 0x2776, 0x2793,
        0x2C00, 0x2DFF, 0x2E80, 0x2FFF, 0x3004, 0x3007, 0x3021, 0x302F,
        0x3031, 0x303F, 0x3040, 0xD7FF, 0xF900, 0xFD3D, 0xFD40, 0xFDCF,
        0xFDF0, 0xFE1F, 0xFE30, 0xFE44, 0xFE47, 0xFFFD,
        0x10000, 0x1FFFD, 0x20000, 0x2FFFD, 0x30000, 0x3FFFD,
        0x40000, 0x4FFFD, 0x50000, 0x5FFFD, 0x60000, 0x6FFFD,
        0x70000, 0x7FFFD, 0x80000, 0x8FFFD, 0x90000, 0x9FFFD,
        0xA0000, 0xAFFFD, 0xB0000, 0xBFFFD, 0xC0000, 0xCFFFD,
        0xD0000, 0xDFFFD, 0xE0000, 0xEFFFD,
        u32::MAX, // 终止标记
    ];
    
    unsafe { in_range(RANGE.as_ptr(), c) }
}

/// 检查字符是否可以作为标识符的后续字符
pub fn is_ident2(c: uint32_t) -> bool {
    is_ident1(c) || (b'0' as u32 <= c && c <= b'9' as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_utf8() {
        let mut buf = [0u8; 4];
        unsafe {
            let len = encode_utf8(buf.as_mut_ptr() as *mut i8, b'A' as u32);
            assert_eq!(len, 1);
            assert_eq!(buf[0], b'A');
        }
    }

    #[test]
    fn test_is_ident1() {
        assert!(is_ident1(b'a' as u32));
        assert!(is_ident1(b'Z' as u32));
        assert!(is_ident1(b'_' as u32));
        assert!(!is_ident1(b'0' as u32));
    }

    #[test]
    fn test_is_ident2() {
        assert!(is_ident2(b'a' as u32));
        assert!(is_ident2(b'0' as u32));
        assert!(!is_ident2(b' ' as u32));
    }
}
UNICODE_EOF

echo_success "unicode.rs 已生成"
echo ""

# 步骤 3: 生成 Cargo 项目
echo_info "步骤 3/4: 生成 Cargo 项目..."

cat > "$OUT_DIR/Cargo.toml" << 'CARGO_EOF'
[package]
name = "chibicc-rs"
version = "0.1.0"
edition = "2021"

[lib]
name = "chibicc"
path = "lib.rs"

[dependencies]
libc = "0.2"

[profile.dev]
opt-level = 0

[profile.release]
opt-level = 3
CARGO_EOF

cat > "$OUT_DIR/lib.rs" << 'LIB_EOF'
//! chibicc Rust 版本
//! C 编译器用 Rust 实现

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod types;
pub mod unicode;

// 重新导出主要类型
pub use types::{Token, TokenKind, File, StringArray, Type, Node};
pub use unicode::{encode_utf8, decode_utf8, is_ident1, is_ident2};
LIB_EOF

echo_success "Cargo 项目已生成"
echo ""

# 步骤 4: 使用 cargo 编译
echo_info "步骤 4/4: 使用 Cargo 编译..."
cd "$OUT_DIR"

if cargo build 2>&1 | tee "$REPORT"; then
    echo_success "Cargo 构建成功！"
    RESULT="成功"
else
    echo_error "Cargo 构建失败"
    RESULT="失败"
fi
echo ""

# 生成报告
cat >> "$REPORT" << EOF

========================================
最终报告
========================================
时间: $(date)
结果: $RESULT
输出目录: $OUT_DIR

已转换的模块:
- types.rs (公共类型定义)
- unicode.rs (Unicode 处理)

下一步:
- 转换更多模块 (strings, hashmap, tokenize, etc.)
- 实现完整的编译器功能
- 添加测试用例
EOF

echo "========================================="
echo "转换完成！"
echo "========================================="
echo "报告: $REPORT"
echo "输出: $OUT_DIR"
echo ""

if [ "$RESULT" = "成功" ]; then
    echo_success "所有模块编译通过！"
    exit 0
else
    echo_error "部分模块编译失败，查看报告了解详情"
    exit 1
fi
