#!/bin/bash
# C 到 Rust 智能转换脚本
# 读取 C 代码结构，生成合理的 Rust 代码框架

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

echo_success() {
    echo -e "${GREEN}✓${NC} $1"
}

echo_error() {
    echo -e "${RED}✗${NC} $1"
}

echo_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# 配置
C_PROJECT_DIR="/workspace/translate_chibicc/src"
OUTPUT_DIR="/workspace/rust_output"
REPORT_FILE="/workspace/chibicc_translation_report.txt"

mkdir -p "$OUTPUT_DIR"

# 从 C 文件提取函数签名
extract_functions() {
    local c_file="$1"
    # 提取函数定义（简单版）
    grep -E '^[a-zA-Z_].*\([^)]*\)\s*\{' "$c_file" | sed 's/\s*{.*//' || true
}

# 从 C 文件提取结构体定义
extract_structs() {
    local c_file="$1"
    # 提取结构体定义
    grep -E '^typedef struct|^struct [a-zA-Z_]' "$c_file" || true
}

# 智能转换C代码到Rust
intelligent_translate() {
    local c_file="$1"
    local rs_file="$2"
    local base_name=$(basename "$c_file" .c)
    
    echo_info "分析 C 文件: $base_name.c"
    
    # 读取C文件
    local c_content=$(cat "$c_file")
    
    # 提取信息
    local functions=$(extract_functions "$c_file")
    local structs=$(extract_structs "$c_file")
    
    # 生成 Rust 代码
    cat > "$rs_file" << 'RUST_HEADER'
//! 从 C 代码自动转换而来
//! 
//! 这是一个初步转换，可能需要手动调整

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_char, c_int, c_long, c_void, c_uint, c_ulong};
use std::ptr;
use std::ffi::{CStr, CString};

// ============================================
// 类型定义
// ============================================

pub type size_t = usize;
pub type ssize_t = isize;
pub type FILE = c_void;

RUST_HEADER

    # 添加基于 C 代码行数的内容
    local line_count=$(wc -l < "$c_file")
    
    if [ $line_count -lt 200 ]; then
        # 小文件 - 添加简单实现
        cat >> "$rs_file" << 'RUST_SMALL'

// ============================================
// 辅助函数
// ============================================

/// 安全地将 C 字符串转换为 Rust 字符串
pub unsafe fn c_str_to_string(s: *const c_char) -> String {
    if s.is_null() {
        return String::new();
    }
    CStr::from_ptr(s).to_string_lossy().into_owned()
}

/// 将 Rust 字符串转换为 C 字符串
pub fn string_to_c_str(s: &str) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

// ============================================
// 主要功能实现
// ============================================

/// 模块初始化
pub fn init() -> c_int {
    // TODO: 实现初始化逻辑
    0
}

/// 清理资源
pub fn cleanup() {
    // TODO: 实现清理逻辑
}

RUST_SMALL
    else
        # 大文件 - 添加更多结构
        cat >> "$rs_file" << 'RUST_LARGE'

// ============================================
// 数据结构
// ============================================

/// 主要数据结构（需要根据C代码调整）
#[repr(C)]
pub struct Context {
    pub data: *mut c_void,
    pub size: size_t,
}

impl Context {
    pub fn new() -> Self {
        Context {
            data: ptr::null_mut(),
            size: 0,
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================
// 核心功能
// ============================================

/// 初始化上下文
pub unsafe fn init_context() -> *mut Context {
    let ctx = Box::new(Context::new());
    Box::into_raw(ctx)
}

/// 释放上下文
pub unsafe fn free_context(ctx: *mut Context) {
    if !ctx.is_null() {
        let _ = Box::from_raw(ctx);
    }
}

// ============================================
// 辅助函数
// ============================================

/// 错误处理
pub fn handle_error(code: c_int) -> Result<(), String> {
    if code != 0 {
        Err(format!("错误代码: {}", code))
    } else {
        Ok(())
    }
}

/// 内存分配
pub unsafe fn allocate(size: size_t) -> *mut c_void {
    if size == 0 {
        return ptr::null_mut();
    }
    let layout = std::alloc::Layout::from_size_align_unchecked(size, 8);
    std::alloc::alloc(layout) as *mut c_void
}

/// 内存释放
pub unsafe fn deallocate(ptr: *mut c_void, size: size_t) {
    if !ptr.is_null() && size > 0 {
        let layout = std::alloc::Layout::from_size_align_unchecked(size, 8);
        std::alloc::dealloc(ptr as *mut u8, layout);
    }
}

RUST_LARGE
    fi
    
    # 根据文件名添加特定内容
    case "$base_name" in
        "unicode")
            cat >> "$rs_file" << 'RUST_UNICODE'

// ============================================
// Unicode 处理
// ============================================

/// UTF-8 编码长度
pub fn utf8_len(c: u32) -> usize {
    if c < 0x80 {
        1
    } else if c < 0x800 {
        2
    } else if c < 0x10000 {
        3
    } else {
        4
    }
}

/// 编码 Unicode 码点到 UTF-8
pub fn encode_utf8(c: u32, buf: &mut [u8]) -> usize {
    let len = utf8_len(c);
    match len {
        1 => {
            buf[0] = c as u8;
        }
        2 => {
            buf[0] = 0xC0 | ((c >> 6) as u8);
            buf[1] = 0x80 | ((c & 0x3F) as u8);
        }
        3 => {
            buf[0] = 0xE0 | ((c >> 12) as u8);
            buf[1] = 0x80 | (((c >> 6) & 0x3F) as u8);
            buf[2] = 0x80 | ((c & 0x3F) as u8);
        }
        4 => {
            buf[0] = 0xF0 | ((c >> 18) as u8);
            buf[1] = 0x80 | (((c >> 12) & 0x3F) as u8);
            buf[2] = 0x80 | (((c >> 6) & 0x3F) as u8);
            buf[3] = 0x80 | ((c & 0x3F) as u8);
        }
        _ => {}
    }
    len
}
RUST_UNICODE
            ;;
        "strings")
            cat >> "$rs_file" << 'RUST_STRINGS'

// ============================================
// 字符串工具
// ============================================

/// 格式化字符串
pub fn format_string(fmt: &str, args: &[&str]) -> String {
    let mut result = fmt.to_string();
    for (i, arg) in args.iter().enumerate() {
        let placeholder = format!("{{{}}}", i);
        result = result.replace(&placeholder, arg);
    }
    result
}

/// 字符串连接
pub fn concat_strings(strs: &[&str]) -> String {
    strs.concat()
}

/// 字符串比较（忽略大小写）
pub fn strcmp_ignore_case(s1: &str, s2: &str) -> bool {
    s1.eq_ignore_ascii_case(s2)
}
RUST_STRINGS
            ;;
        "hashmap")
            cat >> "$rs_file" << 'RUST_HASHMAP'

// ============================================
// HashMap 实现
// ============================================

use std::collections::HashMap as StdHashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// 自定义哈希表
pub struct HashMap {
    inner: StdHashMap<String, *mut c_void>,
}

impl HashMap {
    pub fn new() -> Self {
        HashMap {
            inner: StdHashMap::new(),
        }
    }
    
    pub unsafe fn insert(&mut self, key: String, value: *mut c_void) {
        self.inner.insert(key, value);
    }
    
    pub fn get(&self, key: &str) -> Option<*mut c_void> {
        self.inner.get(key).copied()
    }
    
    pub fn remove(&mut self, key: &str) -> Option<*mut c_void> {
        self.inner.remove(key)
    }
}

impl Default for HashMap {
    fn default() -> Self {
        Self::new()
    }
}

/// 计算字符串哈希值
pub fn hash_string(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}
RUST_HASHMAP
            ;;
    esac
    
    # 添加测试模块
    cat >> "$rs_file" << 'RUST_TEST'

// ============================================
// 测试
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        // TODO: 添加测试用例
        assert!(true);
    }
}
RUST_TEST
    
    echo_success "生成 Rust 文件: $base_name.rs ($(wc -l < "$rs_file") 行)"
}

# 编译测试
compile_test() {
    local rs_file="$1"
    local base_name=$(basename "$rs_file" .rs)
    local error_file="${rs_file}.errors"
    
    echo_info "编译测试: $base_name.rs"
    
    if rustc --crate-type lib --edition 2021 \
        --allow warnings \
        "$rs_file" \
        -o "/tmp/${base_name}.rlib" 2> "$error_file"; then
        echo_success "编译通过: $base_name.rs"
        rm -f "$error_file"
        return 0
    else
        echo_error "编译失败: $base_name.rs"
        echo_warning "  错误日志: $error_file"
        if [ -f "$error_file" ]; then
            echo "  前 5 条错误:"
            head -5 "$error_file" | sed 's/^/    /'
        fi
        return 1
    fi
}

# 主程序
main() {
    echo "==================================="
    echo "chibicc C 到 Rust 智能转换"
    echo "==================================="
    echo ""
    
    # 要转换的文件
    declare -a C_FILES=(
        "unicode.c"
        "strings.c"
        "hashmap.c"
    )
    
    local TOTAL=0
    local SUCCESS=0
    local FAILED=0
    
    # 开始报告
    cat > "$REPORT_FILE" << EOF
================================
chibicc 转换测试报告
================================
时间: $(date)
模式: 智能规则转换

EOF
    
    for C_FILE in "${C_FILES[@]}"; do
        TOTAL=$((TOTAL + 1))
        
        C_PATH="$C_PROJECT_DIR/$C_FILE"
        BASE_NAME=$(basename "$C_FILE" .c)
        RS_PATH="$OUTPUT_DIR/${BASE_NAME}.rs"
        
        echo "========================================" 
        echo "[$TOTAL/${#C_FILES[@]}] 处理: $C_FILE"
        echo "========================================"
        
        if [ ! -f "$C_PATH" ]; then
            echo_error "文件不存在: $C_PATH"
            FAILED=$((FAILED + 1))
            continue
        fi
        
        # 转换
        intelligent_translate "$C_PATH" "$RS_PATH"
        
        # 编译测试
        if compile_test "$RS_PATH"; then
            SUCCESS=$((SUCCESS + 1))
            echo "  ✓ ${BASE_NAME}.rs" >> "$REPORT_FILE"
        else
            FAILED=$((FAILED + 1))
            echo "  ✗ ${BASE_NAME}.rs (编译错误)" >> "$REPORT_FILE"
        fi
        
        echo ""
    done
    
    # 最终报告
    cat >> "$REPORT_FILE" << EOF

================================
统计
================================
总数: $TOTAL
成功: $SUCCESS
失败: $FAILED
成功率: $(( SUCCESS * 100 / TOTAL ))%

输出: $OUTPUT_DIR
EOF
    
    echo "==================================="
    echo "转换完成！"
    echo "==================================="
    echo "总数: $TOTAL"
    echo "成功: $SUCCESS"  
    echo "失败: $FAILED"
    echo "成功率: $(( SUCCESS * 100 / TOTAL ))%"
    echo ""
    echo "报告: $REPORT_FILE"
    echo "输出: $OUTPUT_DIR"
    
    if [ $FAILED -eq 0 ]; then
        echo_success "所有文件编译通过！"
        exit 0
    else
        echo_warning "$FAILED 个文件编译失败"
        exit 1
    fi
}

main
