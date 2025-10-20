//! 工具函数
//!
//! 包含 UTF-8 处理、文件操作等通用功能

use console::{Term, style};
use std::io::{self, Write};

/// 安全地输出 UTF-8 文本到控制台
///
/// 解决 Windows 控制台乱码问题
pub fn print_utf8(text: &str) {
    let term = Term::stdout();
    let _ = term.write_str(text);
    let _ = io::stdout().flush();
}

/// 输出彩色文本
pub fn print_colored(text: &str, color: &str) {
    let styled_text = match color {
        "green" => style(text).green(),
        "red" => style(text).red(),
        "yellow" => style(text).yellow(),
        "blue" => style(text).blue(),
        "cyan" => style(text).cyan(),
        "magenta" => style(text).magenta(),
        _ => style(text).white(),
    };

    println!("{}", styled_text);
}

/// 输出成功消息
pub fn print_success(message: &str) {
    print_colored(&format!("✓ {}", message), "green");
}

/// 输出错误消息
pub fn print_error(message: &str) {
    print_colored(&format!("✗ {}", message), "red");
}

/// 输出警告消息
pub fn print_warning(message: &str) {
    print_colored(&format!("⚠ {}", message), "yellow");
}

/// 输出信息消息
pub fn print_info(message: &str) {
    print_colored(&format!("ℹ {}", message), "blue");
}

/// 提取代码块内容
///
/// 从 Markdown 格式的响应中提取代码块
pub fn extract_code_block(text: &str, language: &str) -> Option<String> {
    let fence_start = format!("```{}", language);
    let fence_end = "```";

    if let Some(start_pos) = text.find(&fence_start) {
        let content_start = start_pos + fence_start.len();
        if let Some(end_pos) = text[content_start..].find(fence_end) {
            let code = &text[content_start..content_start + end_pos];
            return Some(code.trim().to_string());
        }
    }

    // 尝试不带语言标识的代码块
    if let Some(start_pos) = text.find("```") {
        let content_start = start_pos + 3;
        // 跳过第一行（可能是语言标识）
        if let Some(first_newline) = text[content_start..].find('\n') {
            let code_start = content_start + first_newline + 1;
            if let Some(end_pos) = text[code_start..].find("```") {
                let code = &text[code_start..code_start + end_pos];
                return Some(code.trim().to_string());
            }
        }
    }

    None
}

/// 计算 unsafe 代码占比
pub fn calculate_unsafe_ratio(rust_code: &str) -> f32 {
    let total_lines: usize = rust_code.lines().count();
    if total_lines == 0 {
        return 0.0;
    }

    let mut unsafe_lines = 0;
    let mut in_unsafe_block = false;
    let mut brace_depth = 0;

    for line in rust_code.lines() {
        let trimmed = line.trim();

        // 检测 unsafe 块开始
        if trimmed.contains("unsafe") && trimmed.contains("{") {
            in_unsafe_block = true;
            brace_depth = 1;
            unsafe_lines += 1;
            continue;
        }

        if in_unsafe_block {
            unsafe_lines += 1;

            // 计算花括号深度
            brace_depth += trimmed.chars().filter(|&c| c == '{').count();
            brace_depth -= trimmed.chars().filter(|&c| c == '}').count();

            if brace_depth == 0 {
                in_unsafe_block = false;
            }
        }
    }

    (unsafe_lines as f32) / (total_lines as f32)
}

/// 格式化文件大小
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_code_block() {
        let text = r#"
Some text before

```rust
fn main() {
    println!("Hello");
}
```

Some text after
"#;

        let code = extract_code_block(text, "rust").unwrap();
        assert!(code.contains("fn main()"));
    }

    #[test]
    fn test_calculate_unsafe_ratio() {
        let code = r#"
fn safe_function() {
    let x = 42;
}

unsafe fn unsafe_function() {
    let ptr = std::ptr::null_mut();
}

fn another_safe() {
    unsafe {
        let y = *ptr;
    }
}
"#;

        let ratio = calculate_unsafe_ratio(code);
        assert!(ratio > 0.0 && ratio < 1.0);
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(2048), "2.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
    }
}
