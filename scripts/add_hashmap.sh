#!/bin/bash
# 添加 hashmap 模块

OUT_DIR="/workspace/rust_output_final"

# 首先更新 types.rs 添加 HashMap 定义
cat >> "$OUT_DIR/types.rs" << 'EOF'

// ============================================
// HashMap 相关类型
// ============================================

#[repr(C)]
pub struct HashEntry {
    pub key: *mut c_char,
    pub keylen: c_int,
    pub val: *mut c_void,
}

#[repr(C)]
pub struct HashMap {
    pub buckets: *mut HashEntry,
    pub capacity: c_int,
    pub used: c_int,
}

impl HashMap {
    pub fn new() -> Self {
        HashMap {
            buckets: ptr::null_mut(),
            capacity: 0,
            used: 0,
        }
    }
}
EOF

# 创建 hashmap.rs
cat > "$OUT_DIR/hashmap.rs" << 'EOF'
//! HashMap 实现
//! 开放寻址哈希表
//! 从 hashmap.c 转换而来

#![allow(dead_code)]
#![allow(unused_variables)]

use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::alloc::{alloc, realloc, Layout};
use crate::types::{HashMap, HashEntry};

// 常量
const INIT_SIZE: usize = 16;
const HIGH_WATERMARK: c_int = 70;
const LOW_WATERMARK: c_int = 50;
const TOMBSTONE: *mut c_char = (-1isize) as *mut c_char;

/// FNV-1a 哈希算法
unsafe fn fnv_hash(s: *const c_char, len: c_int) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    let s_u8 = s as *const u8;
    
    for i in 0..len {
        hash = hash.wrapping_mul(0x100000001b3);
        hash ^= *s_u8.offset(i as isize) as u64;
    }
    
    hash
}

/// 检查条目是否匹配
unsafe fn match_entry(ent: *const HashEntry, key: *const c_char, keylen: c_int) -> bool {
    let ent_ref = &*ent;
    
    if ent_ref.key.is_null() || ent_ref.key == TOMBSTONE {
        return false;
    }
    
    if ent_ref.keylen != keylen {
        return false;
    }
    
    // 比较内存
    std::ptr::eq(
        std::slice::from_raw_parts(ent_ref.key as *const u8, keylen as usize),
        std::slice::from_raw_parts(key as *const u8, keylen as usize)
    ) || std::slice::from_raw_parts(ent_ref.key as *const u8, keylen as usize)
        == std::slice::from_raw_parts(key as *const u8, keylen as usize)
}

/// 获取条目
unsafe fn get_entry(map: *mut HashMap, key: *const c_char, keylen: c_int) -> *mut HashEntry {
    let map_ref = &*map;
    
    if map_ref.buckets.is_null() {
        return ptr::null_mut();
    }
    
    let hash = fnv_hash(key, keylen);
    
    for i in 0..map_ref.capacity {
        let idx = ((hash as usize + i as usize) % map_ref.capacity as usize) as isize;
        let ent = map_ref.buckets.offset(idx);
        
        if match_entry(ent, key, keylen) {
            return ent;
        }
        
        if (*ent).key.is_null() {
            return ptr::null_mut();
        }
    }
    
    ptr::null_mut()
}

/// rehash - 重新哈希
unsafe fn rehash(map: *mut HashMap) {
    let map_ref = &mut *map;
    
    // 计算实际的键数量
    let mut nkeys = 0;
    for i in 0..map_ref.capacity {
        let ent = &*map_ref.buckets.offset(i as isize);
        if !ent.key.is_null() && ent.key != TOMBSTONE {
            nkeys += 1;
        }
    }
    
    // 计算新容量
    let mut cap = map_ref.capacity;
    while (nkeys * 100) / cap >= LOW_WATERMARK {
        cap *= 2;
    }
    
    // 创建新的 bucket 数组
    let layout = Layout::array::<HashEntry>(cap as usize).unwrap();
    let new_buckets = alloc(layout) as *mut HashEntry;
    
    // 初始化为零
    for i in 0..cap {
        let ent = new_buckets.offset(i as isize);
        ptr::write(ent, HashEntry {
            key: ptr::null_mut(),
            keylen: 0,
            val: ptr::null_mut(),
        });
    }
    
    // 创建临时 map
    let mut map2 = HashMap {
        buckets: new_buckets,
        capacity: cap,
        used: 0,
    };
    
    // 复制所有键值对
    for i in 0..map_ref.capacity {
        let ent = &*map_ref.buckets.offset(i as isize);
        if !ent.key.is_null() && ent.key != TOMBSTONE {
            hashmap_put2(&mut map2, ent.key, ent.keylen, ent.val);
        }
    }
    
    // 释放旧的 buckets
    if !map_ref.buckets.is_null() && map_ref.capacity > 0 {
        let old_layout = Layout::array::<HashEntry>(map_ref.capacity as usize).unwrap();
        std::alloc::dealloc(map_ref.buckets as *mut u8, old_layout);
    }
    
    // 更新 map
    *map_ref = map2;
}

/// 获取或插入条目
unsafe fn get_or_insert_entry(
    map: *mut HashMap,
    key: *mut c_char,
    keylen: c_int
) -> *mut HashEntry {
    let map_ref = &mut *map;
    
    // 初始化 buckets
    if map_ref.buckets.is_null() {
        let layout = Layout::array::<HashEntry>(INIT_SIZE).unwrap();
        map_ref.buckets = alloc(layout) as *mut HashEntry;
        map_ref.capacity = INIT_SIZE as c_int;
        
        // 初始化为零
        for i in 0..INIT_SIZE {
            let ent = map_ref.buckets.offset(i as isize);
            ptr::write(ent, HashEntry {
                key: ptr::null_mut(),
                keylen: 0,
                val: ptr::null_mut(),
            });
        }
    } else if (map_ref.used * 100) / map_ref.capacity >= HIGH_WATERMARK {
        rehash(map);
        // 重新获取 map_ref（因为 rehash 可能改变了指针）
        let map_ref = &mut *map;
    }
    
    let map_ref = &mut *map;
    let hash = fnv_hash(key, keylen);
    
    for i in 0..map_ref.capacity {
        let idx = ((hash as usize + i as usize) % map_ref.capacity as usize) as isize;
        let ent = map_ref.buckets.offset(idx);
        let ent_ref = &mut *ent;
        
        if match_entry(ent, key, keylen) {
            return ent;
        }
        
        if ent_ref.key == TOMBSTONE {
            ent_ref.key = key;
            ent_ref.keylen = keylen;
            return ent;
        }
        
        if ent_ref.key.is_null() {
            ent_ref.key = key;
            ent_ref.keylen = keylen;
            map_ref.used += 1;
            return ent;
        }
    }
    
    panic!("HashMap full - should not happen");
}

/// 获取值（使用 strlen）
pub unsafe fn hashmap_get(map: *mut HashMap, key: *mut c_char) -> *mut c_void {
    let keylen = libc::strlen(key) as c_int;
    hashmap_get2(map, key, keylen)
}

/// 获取值（指定长度）
pub unsafe fn hashmap_get2(
    map: *mut HashMap,
    key: *mut c_char,
    keylen: c_int
) -> *mut c_void {
    let ent = get_entry(map, key, keylen);
    if ent.is_null() {
        ptr::null_mut()
    } else {
        (*ent).val
    }
}

/// 设置值（使用 strlen）
pub unsafe fn hashmap_put(map: *mut HashMap, key: *mut c_char, val: *mut c_void) {
    let keylen = libc::strlen(key) as c_int;
    hashmap_put2(map, key, keylen, val);
}

/// 设置值（指定长度）
pub unsafe fn hashmap_put2(
    map: *mut HashMap,
    key: *mut c_char,
    keylen: c_int,
    val: *mut c_void
) {
    let ent = get_or_insert_entry(map, key, keylen);
    (*ent).val = val;
}

/// 删除键（使用 strlen）
pub unsafe fn hashmap_delete(map: *mut HashMap, key: *mut c_char) {
    let keylen = libc::strlen(key) as c_int;
    hashmap_delete2(map, key, keylen);
}

/// 删除键（指定长度）
pub unsafe fn hashmap_delete2(map: *mut HashMap, key: *mut c_char, keylen: c_int) {
    let ent = get_entry(map, key, keylen);
    if !ent.is_null() {
        (*ent).key = TOMBSTONE;
    }
}

/// 创建新的 HashMap
pub unsafe fn hashmap_new() -> *mut HashMap {
    let layout = Layout::new::<HashMap>();
    let map = alloc(layout) as *mut HashMap;
    ptr::write(map, HashMap::new());
    map
}

/// 释放 HashMap
pub unsafe fn hashmap_free(map: *mut HashMap) {
    if map.is_null() {
        return;
    }
    
    let map_ref = &mut *map;
    if !map_ref.buckets.is_null() && map_ref.capacity > 0 {
        let layout = Layout::array::<HashEntry>(map_ref.capacity as usize).unwrap();
        std::alloc::dealloc(map_ref.buckets as *mut u8, layout);
    }
    
    let layout = Layout::new::<HashMap>();
    std::alloc::dealloc(map as *mut u8, layout);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_hashmap_basic() {
        unsafe {
            let map = hashmap_new();
            
            let key1 = CString::new("hello").unwrap().into_raw();
            let key2 = CString::new("world").unwrap().into_raw();
            
            hashmap_put(map, key1, 42 as *mut c_void);
            hashmap_put(map, key2, 100 as *mut c_void);
            
            let val1 = hashmap_get(map, key1) as usize;
            let val2 = hashmap_get(map, key2) as usize;
            
            assert_eq!(val1, 42);
            assert_eq!(val2, 100);
            
            // 清理
            hashmap_free(map);
            let _ = CString::from_raw(key1);
            let _ = CString::from_raw(key2);
        }
    }

    #[test]
    fn test_hashmap_delete() {
        unsafe {
            let map = hashmap_new();
            
            let key = CString::new("test").unwrap().into_raw();
            hashmap_put(map, key, 123 as *mut c_void);
            
            let val = hashmap_get(map, key) as usize;
            assert_eq!(val, 123);
            
            hashmap_delete(map, key);
            let val2 = hashmap_get(map, key);
            assert!(val2.is_null());
            
            // 清理
            hashmap_free(map);
            let _ = CString::from_raw(key);
        }
    }

    #[test]
    fn test_hashmap_expansion() {
        unsafe {
            let map = hashmap_new();
            
            // 添加多个元素触发扩容
            for i in 0..50 {
                let key = CString::new(format!("key{}", i)).unwrap().into_raw();
                hashmap_put(map, key, i as *mut c_void);
            }
            
            // 验证所有值
            for i in 0..50 {
                let key = CString::new(format!("key{}", i)).unwrap().into_raw();
                let val = hashmap_get(map, key) as usize;
                assert_eq!(val, i);
                let _ = CString::from_raw(key);
            }
            
            hashmap_free(map);
        }
    }
}
EOF

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
pub mod hashmap;

// 重新导出主要类型
pub use types::{Token, TokenKind, File, StringArray, Type, Node, HashMap, HashEntry};
pub use unicode::{encode_utf8, decode_utf8, is_ident1, is_ident2};
pub use strings::{strarray_push, strarray_new, strarray_free, format, format_rust};
pub use hashmap::{hashmap_get, hashmap_get2, hashmap_put, hashmap_put2, 
                  hashmap_delete, hashmap_delete2, hashmap_new, hashmap_free};
EOF

echo "✓ 添加 hashmap 模块"
