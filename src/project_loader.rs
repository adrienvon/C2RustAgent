//! C 项目加载器：基于 compile_commands.json 解析项目并按需构建 TranslationUnits

use anyhow::{Context, Result, anyhow};
use clang::{Clang, Index, TranslationUnit};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// compile_commands.json 的条目格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileCommandEntry {
    /// 编译命令所在的工作目录
    pub directory: String,
    /// 源文件路径（相对于 directory 或绝对路径）
    pub file: String,
    /// 可选的完整命令（某些生成器提供）
    #[serde(default)]
    pub command: String,
    /// 参数数组形式（cmake 新版本常见）
    #[serde(default)]
    pub arguments: Vec<String>,
}

/// 一个编译单元规范（源文件 + 编译参数）
#[derive(Debug, Clone)]
pub struct UnitSpec {
    pub source: PathBuf,
    pub args: Vec<String>,
}

/// C 项目：包含所有编译单元
#[derive(Debug, Clone)]
pub struct CProject {
    pub root: PathBuf,
    pub units: Vec<UnitSpec>,
}

impl CProject {
    /// 加载一个 C 项目：读取 compile_commands.json 并解析所有 .c 文件
    pub fn load(root: &Path) -> Result<Self> {
        let cc_path = root.join("compile_commands.json");
        if !cc_path.exists() {
            return Err(anyhow!(
                "未找到 compile_commands.json 于: {}\n提示: 使用 'bear -- make' 或 'cmake -DCMAKE_EXPORT_COMPILE_COMMANDS=ON .' 生成该文件",
                cc_path.display()
            ));
        }

        let json = fs::read_to_string(&cc_path)
            .with_context(|| format!("读取 compile_commands.json 失败: {}", cc_path.display()))?;
        let entries: Vec<CompileCommandEntry> = serde_json::from_str(&json)
            .with_context(|| "解析 compile_commands.json 失败".to_string())?;

        let mut units = Vec::new();
        for entry in entries {
            // 规范化源文件路径
            let dir = PathBuf::from(&entry.directory);
            let src = if Path::new(&entry.file).is_absolute() {
                PathBuf::from(&entry.file)
            } else {
                dir.join(&entry.file)
            };

            // 构造参数：优先使用 arguments 字段，否则从 command 拆分
            let args = if !entry.arguments.is_empty() {
                entry.arguments.clone()
            } else if !entry.command.is_empty() {
                // 粗略拆分 command（不处理复杂引号场景，但通常足够）
                shlex_split(&entry.command)
            } else {
                Vec::new()
            };

            // 过滤出传给编译器的参数，剔除源文件本身与输出选项
            let filtered_args = filter_compiler_args(&args);

            units.push(UnitSpec {
                source: src,
                args: filtered_args,
            });
        }

        Ok(CProject {
            root: root.to_path_buf(),
            units,
        })
    }

    /// 遍历并处理所有编译单元：按需创建 TranslationUnit 并通过回调处理
    pub fn process_units<F>(&self, mut f: F) -> Result<()>
    where
        F: FnMut(&UnitSpec, &TranslationUnit) -> Result<()>,
    {
        let clang = Clang::new().map_err(|e| anyhow!("初始化 Clang 失败: {}", e))?;
        let index = Index::new(&clang, false, false);
        for spec in &self.units {
            let parser_path = spec.source.to_string_lossy().to_string();
            let tu = index
                .parser(&parser_path)
                .arguments(&spec.args)
                .parse()
                .with_context(|| format!("解析源文件失败: {}", spec.source.display()))?;
            f(spec, &tu)?;
        }
        Ok(())
    }
}

/// 朴素的 command 拆分（尽量处理简单引号）
fn shlex_split(cmd: &str) -> Vec<String> {
    // 简化实现：遇到复杂边界情况可替换为更完整的解析器
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    for ch in cmd.chars() {
        match ch {
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            ' ' | '\t' if !in_single && !in_double => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

/// 过滤掉不需要传给 libclang 的参数
fn filter_compiler_args(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip_next = false;
    for (i, a) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        // 排除源文件与输出选项、编译器名等
        if a == "-c" || a == "-o" {
            skip_next = true;
            continue;
        }
        if a.ends_with(".c") || a.ends_with(".o") {
            continue;
        }
        // 常见需要保留的参数：宏定义、包含目录、语言标准等
        // 保留 -I, -D, -U, -include, -isystem, -std=...
        if a.starts_with("-I")
            || a.starts_with("-D")
            || a.starts_with("-U")
            || a.starts_with("-include")
            || a.starts_with("-isystem")
            || a.starts_with("-std=")
            || a == "-nostdinc"
            || a == "-nostdlib"
        {
            out.push(a.clone());
            // 某些参数如 -include path 需要跟参数
            if a == "-include" || a == "-isystem" {
                if let Some(nxt) = args.get(i + 1) {
                    out.push(nxt.clone());
                    skip_next = true;
                }
            }
            continue;
        }
        // Windows/MSVC 等其他可能参数可在此扩展
        out.push(a.clone());
    }
    out
}
