#!/bin/bash
# 添加 strings 模块

OUT_DIR="/workspace/rust_output_final"

cat > "$OUT_DIR/strings.rs" << 'EOF'
//! 字符串工具模块
//! 从 strings.c 转换而来

#![allow(dead_code)]
#![allow(unused_variables)]

use std::os::raw::{c_char, c_int};
use std::ptr;
use std::alloc::{alloc, realloc, Layout};
use std::ffi::CString;
use crate::types::StringArray;

/// 向 StringArray 添加字符串
pub unsafe fn strarray_push(arr: *mut StringArray, s: *mut c_char) {
    let arr_ref = &mut *arr;
    
    // 首次初始化
    if arr_ref.data.is_null() {
        let layout = Layout::array::<*mut c_char>(8).unwrap();
        arr_ref.data = alloc(layout) as *mut *mut c_char;
        arr_ref.capacity = 8;
        // 初始化为 NULL
        for i in 0..8 {
            *arr_ref.data.offset(i) = ptr::null_mut();
        }
    }
    
    // 需要扩容
    if arr_ref.capacity == arr_ref.len {
        let old_capacity = arr_ref.capacity as usize;
        let new_capacity = old_capacity * 2;
        
        let old_layout = Layout::array::<*mut c_char>(old_capacity).unwrap();
        let new_layout = Layout::array::<*mut c_char>(new_capacity).unwrap();
        
        arr_ref.data = realloc(
            arr_ref.data as *mut u8,
            old_layout,
            new_layout.size()
        ) as *mut *mut c_char;
        
        arr_ref.capacity = new_capacity as c_int;
        
        // 新分配的空间初始化为 NULL
        for i in arr_ref.len..arr_ref.capacity {
            *arr_ref.data.offset(i as isize) = ptr::null_mut();
        }
    }
    
    // 添加元素
    *arr_ref.data.offset(arr_ref.len as isize) = s;
    arr_ref.len += 1;
}

/// 创建新的 StringArray
pub unsafe fn strarray_new() -> StringArray {
    StringArray {
        data: ptr::null_mut(),
        capacity: 0,
        len: 0,
    }
}

/// 释放 StringArray
pub unsafe fn strarray_free(arr: *mut StringArray) {
    if arr.is_null() {
        return;
    }
    
    let arr_ref = &mut *arr;
    if !arr_ref.data.is_null() && arr_ref.capacity > 0 {
        let layout = Layout::array::<*mut c_char>(arr_ref.capacity as usize).unwrap();
        std::alloc::dealloc(arr_ref.data as *mut u8, layout);
        arr_ref.data = ptr::null_mut();
    }
    arr_ref.capacity = 0;
    arr_ref.len = 0;
}

/// 格式化字符串（简化版）
/// 注意：这是一个简化实现，不支持完整的 printf 格式
pub unsafe fn format(fmt: *const c_char) -> *mut c_char {
    if fmt.is_null() {
        return ptr::null_mut();
    }
    
    // 简单地复制输入字符串
    use std::ffi::CStr;
    let c_str = CStr::from_ptr(fmt);
    let rust_str = c_str.to_string_lossy();
    
    // 创建新的 C 字符串
    match CString::new(rust_str.as_ref()) {
        Ok(cstring) => cstring.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Rust 风格的字符串格式化
pub fn format_rust(fmt: &str, args: &[&str]) -> String {
    let mut result = fmt.to_string();
    for (i, arg) in args.iter().enumerate() {
        let placeholder = format!("{{{}}}", i);
        result = result.replace(&placeholder, arg);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::StringArray;

    #[test]
    fn test_strarray_new() {
        unsafe {
            let arr = strarray_new();
            assert!(arr.data.is_null());
            assert_eq!(arr.capacity, 0);
            assert_eq!(arr.len, 0);
        }
    }

    #[test]
    fn test_strarray_push() {
        unsafe {
            let mut arr = strarray_new();
            let s1 = CString::new("hello").unwrap().into_raw();
            let s2 = CString::new("world").unwrap().into_raw();
            
            strarray_push(&mut arr, s1);
            assert_eq!(arr.len, 1);
            assert_eq!(arr.capacity, 8);
            
            strarray_push(&mut arr, s2);
            assert_eq!(arr.len, 2);
            
            // 清理
            strarray_free(&mut arr);
            
            // 重新获取所有权以防止内存泄漏
            let _ = CString::from_raw(s1);
            let _ = CString::from_raw(s2);
        }
    }

    #[test]
    fn test_format_rust() {
        let result = format_rust("Hello {0}, welcome to {1}!", &["Alice", "Rust"]);
        assert_eq!(result, "Hello Alice, welcome to Rust!");
    }

    #[test]
    fn test_strarray_expansion() {
        unsafe {
            let mut arr = strarray_new();
            
            // 添加超过初始容量的元素
            for i in 0..10 {
                let s = CString::new(format!("item{}", i)).unwrap().into_raw();
                strarray_push(&mut arr, s);
            }
            
            assert_eq!(arr.len, 10);
            assert!(arr.capacity >= 10);
            
            // 清理
            for i in 0..arr.len {
                let s = *arr.data.offset(i as isize);
                if !s.is_null() {
                    let _ = CString::from_raw(s);
                }
            }
            strarray_free(&mut arr);
        }
    }
}
EOF

echo "✓ 生成 strings.rs"

# 更新 lib.rs
cat > "$OUT_DIR/lib.rs" << 'EOF'
//! chibicc Rust 版本
//! C 编译器用 Rust 实现

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod types;
pub mod unicode;
pub mod strings;

// 重新导出主要类型
pub use types::{Token, TokenKind, File, StringArray, Type, Node};
pub use unicode::{encode_utf8, decode_utf8, is_ident1, is_ident2};
pub use strings::{strarray_push, strarray_new, strarray_free, format, format_rust};
EOF

echo "✓ 更新 lib.rs"
