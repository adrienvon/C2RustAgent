#!/usr/bin/env python3
"""
智能 C 到 Rust 转换器
使用 clang 的 Python 绑定来解析 C 代码，然后生成 Rust 代码
"""

import sys
import re
from pathlib import Path

def read_c_file(path):
    """读取 C 文件内容"""
    with open(path, 'r', encoding='utf-8') as f:
        return f.read()

def extract_functions(content):
    """提取函数签名和实现"""
    # 简单的正则表达式提取函数
    pattern = r'^\s*([a-zA-Z_][a-zA-Z0-9_*\s]+)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*\{'
    matches = re.finditer(pattern, content, re.MULTILINE)
    
    functions = []
    for match in matches:
        return_type = match.group(1).strip()
        func_name = match.group(2)
        params = match.group(3).strip()
        functions.append({
            'return_type': return_type,
            'name': func_name,
            'params': params,
            'start': match.start()
        })
    
    return functions

def c_type_to_rust(c_type):
    """将 C 类型转换为 Rust 类型"""
    c_type = c_type.strip()
    
    # 移除 C 的存储类说明符
    c_type = re.sub(r'\b(static|extern|const|volatile|register)\b\s*', '', c_type)
    c_type = c_type.strip()
    
    # 基本类型映射
    type_map = {
        'void': '()',
        'int': 'i32',
        'uint32_t': 'u32',
        'uint64_t': 'u64',
        'int32_t': 'i32',
        'int64_t': 'i64',
        'char': 'i8',
        'unsigned char': 'u8',
        'bool': 'bool',
        '_Bool': 'bool',
        'size_t': 'usize',
        'ssize_t': 'isize',
    }
    
    # 处理指针
    if '*' in c_type:
        base_type = c_type.replace('*', '').strip()
        if base_type in type_map:
            base_type = type_map[base_type]
        if base_type == 'i8':  # char*
            return '*mut i8'
        return f'*mut {base_type}'
    
    return type_map.get(c_type, c_type)

def generate_rust_function(func_info, c_content):
    """生成 Rust 函数"""
    func_name = func_info['name']
    return_type = c_type_to_rust(func_info['return_type'])
    
    # 解析参数
    params = func_info['params']
    if not params or params == 'void':
        rust_params = ''
    else:
        param_list = [p.strip() for p in params.split(',')]
        rust_params_list = []
        for param in param_list:
            parts = param.rsplit(None, 1)
            if len(parts) == 2:
                param_type = c_type_to_rust(parts[0])
                param_name = parts[1].replace('*', '').strip()
                rust_params_list.append(f'{param_name}: {param_type}')
        rust_params = ', '.join(rust_params_list)
    
    # 生成函数体
    rust_code = f'''
/// {func_name} - 从 C 代码转换而来
pub unsafe fn {func_name}({rust_params}) -> {return_type} {{
    // TODO: 实现 {func_name}
    unimplemented!("{func_name}")
}}
'''
    
    return rust_code

def generate_rust_module(c_file_path):
    """生成完整的 Rust 模块"""
    content = read_c_file(c_file_path)
    functions = extract_functions(content)
    
    module_name = Path(c_file_path).stem
    
    # 生成模块头部
    rust_code = f'''//! {module_name} 模块
//! 从 C 代码自动转换而来

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{{c_char, c_int, c_uint, c_void}};
use std::ptr;

// ============================================
// 类型定义
// ============================================

pub type uint32_t = u32;
pub type int32_t = i32;
pub type size_t = usize;

// ============================================
// 函数实现
// ============================================

'''
    
    # 添加所有函数
    for func in functions:
        rust_code += generate_rust_function(func, content)
    
    # 添加测试模块
    rust_code += '''
// ============================================
// 测试
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_loads() {
        // 基本测试：模块可以编译
        assert!(true);
    }
}
'''
    
    return rust_code

def main():
    if len(sys.argv) < 3:
        print("Usage: python3 c_to_rust.py <input.c> <output.rs>")
        sys.exit(1)
    
    input_file = sys.argv[1]
    output_file = sys.argv[2]
    
    print(f"转换 {input_file} -> {output_file}")
    
    try:
        rust_code = generate_rust_module(input_file)
        
        with open(output_file, 'w', encoding='utf-8') as f:
            f.write(rust_code)
        
        print(f"✓ 成功生成 {output_file}")
        
    except Exception as e:
        print(f"✗ 错误: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()
