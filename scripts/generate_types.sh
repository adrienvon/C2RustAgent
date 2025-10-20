#!/bin/bash
# 生成 chibicc 的 common types 模块

OUTPUT_DIR="/workspace/rust_output_final"
mkdir -p "$OUTPUT_DIR"

cat > "$OUTPUT_DIR/types.rs" << 'EOF'
//! chibicc 公共类型定义
//! 从 chibicc.h 转换而来

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_char, c_int, c_long, c_void};
use std::ptr;

// ============================================
// 基本类型别名
// ============================================

pub type int64_t = i64;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type size_t = usize;
pub type ssize_t = isize;

// ============================================
// 前向声明（使用不透明类型）
// ============================================

#[repr(C)]
pub struct Type {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Node {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Member {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Relocation {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Hideset {
    _private: [u8; 0],
}

// ============================================
// TokenKind 枚举
// ============================================

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenKind {
    TK_IDENT,   // Identifiers
    TK_PUNCT,   // Punctuators
    TK_KEYWORD, // Keywords
    TK_STR,     // String literals
    TK_NUM,     // Numeric literals
    TK_PP_NUM,  // Preprocessing numbers
    TK_EOF,     // End-of-file markers
}

// ============================================
// File 结构体
// ============================================

#[repr(C)]
pub struct File {
    pub name: *mut c_char,
    pub file_no: c_int,
    pub contents: *mut c_char,
    pub display_name: *mut c_char,
    pub line_delta: c_int,
}

impl File {
    pub fn new() -> Self {
        File {
            name: ptr::null_mut(),
            file_no: 0,
            contents: ptr::null_mut(),
            display_name: ptr::null_mut(),
            line_delta: 0,
        }
    }
}

// ============================================
// Token 结构体
// ============================================

#[repr(C)]
pub struct Token {
    pub kind: TokenKind,
    pub next: *mut Token,
    pub val: int64_t,
    pub fval: f64,
    pub loc: *mut c_char,
    pub len: c_int,
    pub ty: *mut Type,
    pub str: *mut c_char,
    pub file: *mut File,
    pub filename: *mut c_char,
    pub line_no: c_int,
    pub line_delta: c_int,
    pub at_bol: bool,
    pub has_space: bool,
    pub hideset: *mut Hideset,
    pub origin: *mut Token,
}

impl Token {
    pub fn new() -> Self {
        Token {
            kind: TokenKind::TK_EOF,
            next: ptr::null_mut(),
            val: 0,
            fval: 0.0,
            loc: ptr::null_mut(),
            len: 0,
            ty: ptr::null_mut(),
            str: ptr::null_mut(),
            file: ptr::null_mut(),
            filename: ptr::null_mut(),
            line_no: 0,
            line_delta: 0,
            at_bol: false,
            has_space: false,
            hideset: ptr::null_mut(),
            origin: ptr::null_mut(),
        }
    }
}

// ============================================
// StringArray 结构体
// ============================================

#[repr(C)]
pub struct StringArray {
    pub data: *mut *mut c_char,
    pub capacity: c_int,
    pub len: c_int,
}

impl StringArray {
    pub fn new() -> Self {
        StringArray {
            data: ptr::null_mut(),
            capacity: 0,
            len: 0,
        }
    }
}

// ============================================
// 错误处理函数（外部链接）
// ============================================

extern "C" {
    pub fn error(fmt: *const c_char, ...);
    pub fn error_at(loc: *mut c_char, fmt: *const c_char, ...);
    pub fn error_tok(tok: *mut Token, fmt: *const c_char, ...);
    pub fn warn_tok(tok: *mut Token, fmt: *const c_char, ...);
}

// ============================================
// 辅助函数
// ============================================

/// 安全地创建 C 字符串
pub fn make_c_string(s: &str) -> *mut c_char {
    use std::ffi::CString;
    CString::new(s).unwrap().into_raw()
}

/// 从 C 字符串读取到 Rust String
pub unsafe fn c_str_to_string(s: *const c_char) -> String {
    if s.is_null() {
        return String::new();
    }
    use std::ffi::CStr;
    CStr::from_ptr(s).to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_kind() {
        let kind = TokenKind::TK_EOF;
        assert_eq!(kind, TokenKind::TK_EOF);
    }

    #[test]
    fn test_token_new() {
        let token = Token::new();
        assert_eq!(token.kind, TokenKind::TK_EOF);
        assert!(token.next.is_null());
    }
}
EOF

echo "✓ 生成 types.rs"
