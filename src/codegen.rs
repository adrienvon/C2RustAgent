//! Rust 代码生成器
//!
//! 从 MIR 和静态分析结果生成模块化的 Rust Cargo 项目

use crate::analysis::ProjectAnalysisResults;
use crate::llm_assists::{generate_module_documentation, generate_unsafe_explanation};
use crate::mir::{
    BinOp, Function, GlobalVar, LValue, ProjectMIR, RValue, Statement, Terminator, Type, UnOp,
};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Rust 代码生成器
pub struct CodeGenerator {
    /// 输出目录
    output_dir: PathBuf,
    /// 项目名称
    project_name: String,
    /// 源文件到模块名的映射
    source_to_module: HashMap<String, String>,
}

impl CodeGenerator {
    /// 创建新的代码生成器
    ///
    /// # 参数
    /// - `output_dir`: 输出目录路径
    /// - `project_name`: Rust 项目名称（C 项目名 + "_rs" 后缀）
    pub fn new(output_dir: impl AsRef<Path>, project_name: String) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
            project_name,
            source_to_module: HashMap::new(),
        }
    }

    /// 生成完整的 Rust Cargo 项目（不使用 LLM，向后兼容）
    ///
    /// # 参数
    /// - `project_mir`: 项目 MIR
    /// - `analysis_results`: 静态分析结果
    pub fn generate(
        &mut self,
        project_mir: &ProjectMIR,
        _analysis_results: &ProjectAnalysisResults,
    ) -> Result<()> {
        // 1. 创建项目目录结构
        self.create_project_structure()?;

        // 2. 生成 Cargo.toml
        self.generate_cargo_toml()?;

        // 3. 构建源文件到模块的映射
        self.build_source_module_mapping(project_mir)?;

        // 4. 生成全局变量模块（如果有）
        if !project_mir.globals.is_empty() {
            self.generate_globals_module(project_mir)?;
        }

        // 5. 按源文件分组生成模块
        self.generate_modules(project_mir)?;

        // 6. 生成 lib.rs
        self.generate_lib_rs(project_mir)?;

        Ok(())
    }

    /// 生成完整的 Rust Cargo 项目（带 LLM 增强）
    ///
    /// # 参数
    /// - `project_mir`: 项目 MIR
    /// - `analysis_results`: 静态分析结果
    ///
    /// # LLM 集成点
    /// - 模块级文档生成
    /// - unsafe 代码块的详细安全注释
    pub async fn generate_with_llm(
        &mut self,
        project_mir: &ProjectMIR,
        _analysis_results: &ProjectAnalysisResults,
    ) -> Result<()> {
        // 1. 创建项目目录结构
        self.create_project_structure()?;

        // 2. 生成 Cargo.toml
        self.generate_cargo_toml()?;

        // 3. 构建源文件到模块的映射
        self.build_source_module_mapping(project_mir)?;

        // 4. 生成全局变量模块（如果有）
        if !project_mir.globals.is_empty() {
            self.generate_globals_module(project_mir)?;
        }

        // 5. 按源文件分组生成模块（使用 LLM）
        self.generate_modules_with_llm(project_mir).await?;

        // 6. 生成 lib.rs
        self.generate_lib_rs(project_mir)?;

        Ok(())
    }

    /// 创建项目目录结构
    fn create_project_structure(&self) -> Result<()> {
        let src_dir = self.output_dir.join("src");
        fs::create_dir_all(&src_dir)
            .with_context(|| format!("创建目录失败: {}", src_dir.display()))?;
        Ok(())
    }

    /// 生成 Cargo.toml
    fn generate_cargo_toml(&self) -> Result<()> {
        let cargo_toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
# C 标准库功能映射到 Rust
libc = "0.2"

[lib]
name = "{}"
path = "src/lib.rs"
"#,
            self.project_name,
            self.project_name.replace('-', "_")
        );

        let path = self.output_dir.join("Cargo.toml");
        fs::write(&path, cargo_toml)
            .with_context(|| format!("写入 Cargo.toml 失败: {}", path.display()))?;
        Ok(())
    }

    /// 构建源文件到模块名的映射
    fn build_source_module_mapping(&mut self, project_mir: &ProjectMIR) -> Result<()> {
        // 从函数名推断源文件（简化实现：假设函数名包含源文件信息）
        // 实际项目中应从 AST 转换时记录源文件信息
        for (func_name, _func) in &project_mir.functions {
            // 这里简化处理：所有函数放在对应的模块中
            // 实际应该从 Function 结构中添加 source_file 字段
            let module_name = self.infer_module_name(func_name);
            self.source_to_module.insert(func_name.clone(), module_name);
        }

        // 为全局变量也建立映射
        for (var_name, _) in &project_mir.globals {
            self.source_to_module
                .insert(var_name.clone(), "globals".to_string());
        }

        Ok(())
    }

    /// 推断模块名（从函数名或其他元数据）
    fn infer_module_name(&self, _func_name: &str) -> String {
        // 简化实现：将所有内容放在 generated 模块
        // 实际应该从源文件路径推断
        "generated".to_string()
    }

    /// 生成全局变量模块
    fn generate_globals_module(&self, project_mir: &ProjectMIR) -> Result<()> {
        let mut code = String::new();
        code.push_str("//! 全局变量模块\n\n");
        code.push_str("use std::sync::Mutex;\n\n");

        for (name, global_var) in &project_mir.globals {
            code.push_str(&self.generate_global_var(name, global_var)?);
            code.push_str("\n\n");
        }

        let path = self.output_dir.join("src/globals.rs");
        fs::write(&path, code)
            .with_context(|| format!("写入 globals.rs 失败: {}", path.display()))?;
        Ok(())
    }

    /// 生成单个全局变量
    fn generate_global_var(&self, name: &str, global_var: &GlobalVar) -> Result<String> {
        let mut code = String::new();

        // 添加文档注释
        code.push_str(&format!("/// 全局变量: {}\n", name));

        // 生成 Rust 类型
        let rust_type = self.type_to_rust(&global_var.var_type)?;

        // 根据可见性和可变性生成
        let visibility = if global_var.is_public { "pub " } else { "" };

        if global_var.is_static {
            // static 变量使用 Mutex 包装以支持内部可变性
            code.push_str(&format!(
                "{}static {}: Mutex<{}> = Mutex::new(Default::default());",
                visibility,
                name.to_uppercase(),
                rust_type
            ));
        } else {
            // 公共全局变量
            code.push_str(&format!(
                "{}static mut {}: {} = Default::default();",
                visibility,
                name.to_uppercase(),
                rust_type
            ));
        }

        Ok(code)
    }

    /// 按模块生成代码
    fn generate_modules(&self, project_mir: &ProjectMIR) -> Result<()> {
        // 按模块分组函数
        let mut modules: HashMap<String, Vec<&Function>> = HashMap::new();

        for func in project_mir.functions.values() {
            let module_name = self.infer_module_name(&func.name);
            modules.entry(module_name).or_default().push(func);
        }

        // 为每个模块生成文件
        for (module_name, functions) in modules {
            self.generate_module_file(&module_name, &functions)?;
        }

        Ok(())
    }

    /// 按模块生成代码（带 LLM 增强）
    async fn generate_modules_with_llm(&self, project_mir: &ProjectMIR) -> Result<()> {
        // 按模块分组函数
        let mut modules: HashMap<String, Vec<&Function>> = HashMap::new();

        for func in project_mir.functions.values() {
            let module_name = self.infer_module_name(&func.name);
            modules.entry(module_name).or_default().push(func);
        }

        // 为每个模块生成文件（使用 LLM）
        for (module_name, functions) in modules {
            self.generate_module_file_async(&module_name, &functions)
                .await?;
        }

        Ok(())
    }

    /// 生成单个模块文件（带 LLM 增强的文档）
    async fn generate_module_file_async(
        &self,
        module_name: &str,
        functions: &[&Function],
    ) -> Result<()> {
        let mut code = String::new();

        // 【LLM 集成点 1】使用 LLM 生成模块级文档
        // 推断源文件名（基于模块名）
        let file_name = format!("{}.c", module_name);

        // 调用 LLM 生成文档
        let llm_doc = generate_module_documentation(
            module_name,
            &file_name,
            &self.project_name,
            None, // 可以传入从 README 提取的项目摘要
        )
        .await
        .unwrap_or_else(|_| {
            // LLM 调用失败时使用默认文档
            format!("//! 模块: {}\n//! 从 C 代码自动生成\n", module_name)
        });

        code.push_str(&llm_doc);
        code.push_str("\n");

        // 导入必要的库
        code.push_str("#![allow(unused)]\n");
        code.push_str("use libc::*;\n\n");

        // 生成所有函数（带 LLM 增强的 unsafe 注释）
        for func in functions {
            let func_code = self.generate_function_with_llm(func, &file_name).await?;
            code.push_str(&func_code);
            code.push_str("\n\n");
        }

        let path = self.output_dir.join(format!("src/{}.rs", module_name));
        fs::write(&path, code)
            .with_context(|| format!("写入 {}.rs 失败: {}", module_name, path.display()))?;
        Ok(())
    }

    /// 生成单个模块文件（不使用 LLM，用于向后兼容和同步调用）
    fn generate_module_file(&self, module_name: &str, functions: &[&Function]) -> Result<()> {
        let mut code = String::new();

        // 模块文档（不使用 LLM）
        code.push_str(&format!("//! 模块: {}\n", module_name));
        code.push_str("//! 从 C 代码自动生成\n\n");

        // 导入必要的库
        code.push_str("#![allow(unused)]\n");
        code.push_str("use libc::*;\n\n");

        // 生成所有函数
        for func in functions {
            code.push_str(&self.generate_function(func)?);
            code.push_str("\n\n");
        }

        let path = self.output_dir.join(format!("src/{}.rs", module_name));
        fs::write(&path, code)
            .with_context(|| format!("写入 {}.rs 失败: {}", module_name, path.display()))?;
        Ok(())
    }

    /// 生成 lib.rs
    fn generate_lib_rs(&self, project_mir: &ProjectMIR) -> Result<()> {
        let mut code = String::new();

        // 库文档
        code.push_str(&format!(
            "//! {} - 从 C 项目转译的 Rust 库\n",
            self.project_name
        ));
        code.push_str("//! 使用 C2RustAgent 自动生成\n\n");

        code.push_str("#![allow(non_snake_case)]\n");
        code.push_str("#![allow(non_camel_case_types)]\n");
        code.push_str("#![allow(dead_code)]\n\n");

        // 声明全局变量模块
        if !project_mir.globals.is_empty() {
            code.push_str("pub mod globals;\n");
        }

        // 声明所有生成的模块
        let mut module_names: Vec<String> = self
            .source_to_module
            .values()
            .cloned()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        module_names.sort();

        for module_name in module_names {
            if module_name != "globals" {
                code.push_str(&format!("pub mod {};\n", module_name));
            }
        }

        let path = self.output_dir.join("src/lib.rs");
        fs::write(&path, code).with_context(|| format!("写入 lib.rs 失败: {}", path.display()))?;
        Ok(())
    }

    /// 生成单个函数
    fn generate_function(&self, func: &Function) -> Result<String> {
        let mut code = String::new();

        // 函数文档注释
        code.push_str(&format!("/// 函数: {}\n", func.name));
        if !func.annotations.is_empty() {
            code.push_str("///\n");
            code.push_str("/// # LLM 语义注释\n");
            for annotation in &func.annotations {
                code.push_str(&format!("/// - {}\n", annotation));
            }
        }

        // 函数签名
        let visibility = if func.is_static { "" } else { "pub " };
        let return_type = if let Some(ref ty) = func.return_type {
            format!(" -> {}", self.type_to_rust(ty)?)
        } else {
            String::new()
        };

        code.push_str(&format!("{}fn {}(", visibility, func.name));

        // 参数
        let params: Vec<String> = func
            .parameters
            .iter()
            .map(|p| {
                let rust_type = self
                    .type_to_rust(&p.param_type)
                    .unwrap_or_else(|_| "()".to_string());
                format!("{}: {}", p.name, rust_type)
            })
            .collect();
        code.push_str(&params.join(", "));
        code.push_str(&format!("){} {{\n", return_type));

        // 函数体
        if func.basic_blocks.is_empty() {
            code.push_str("    // TODO: 函数体待实现\n");
            if func.return_type.is_some() {
                code.push_str("    unimplemented!()\n");
            }
        } else {
            code.push_str(&self.generate_function_body(func)?);
        }

        code.push_str("}");

        Ok(code)
    }

    /// 生成带 LLM 增强的函数（包含详细的 unsafe 注释）
    async fn generate_function_with_llm(&self, func: &Function, file_name: &str) -> Result<String> {
        let mut code = String::new();

        // 函数文档注释
        code.push_str(&format!("/// 函数: {}\n", func.name));
        if !func.annotations.is_empty() {
            code.push_str("///\n");
            code.push_str("/// # LLM 语义注释\n");
            for annotation in &func.annotations {
                code.push_str(&format!("/// - {}\n", annotation));
            }
        }

        // 函数签名
        let visibility = if func.is_static { "" } else { "pub " };
        let return_type = if let Some(ref ty) = func.return_type {
            format!(" -> {}", self.type_to_rust(ty)?)
        } else {
            String::new()
        };

        code.push_str(&format!("{}fn {}(", visibility, func.name));

        // 参数
        let params: Vec<String> = func
            .parameters
            .iter()
            .map(|p| {
                let rust_type = self
                    .type_to_rust(&p.param_type)
                    .unwrap_or_else(|_| "()".to_string());
                format!("{}: {}", p.name, rust_type)
            })
            .collect();
        code.push_str(&params.join(", "));
        code.push_str(&format!("){} {{\n", return_type));

        // 函数体（带 LLM 增强的 unsafe 注释）
        if func.basic_blocks.is_empty() {
            code.push_str("    // TODO: 函数体待实现\n");
            if func.return_type.is_some() {
                code.push_str("    unimplemented!()\n");
            }
        } else {
            code.push_str(
                &self
                    .generate_function_body_with_llm(func, file_name)
                    .await?,
            );
        }

        code.push_str("}");

        Ok(code)
    }

    /// 生成函数体
    fn generate_function_body(&self, func: &Function) -> Result<String> {
        let mut code = String::new();

        // 生成基本块
        for (idx, bb) in func.basic_blocks.iter().enumerate() {
            // 基本块标签
            if idx > 0 {
                code.push_str(&format!("    // 基本块 {}\n", bb.id));
            }

            // 生成语句
            for stmt in &bb.statements {
                code.push_str(&self.generate_statement(stmt, 1)?);
            }

            // 生成终结符
            code.push_str(&self.generate_terminator(&bb.terminator, 1)?);
        }

        Ok(code)
    }

    /// 生成函数体（带 LLM 增强的 unsafe 注释）
    async fn generate_function_body_with_llm(
        &self,
        func: &Function,
        file_name: &str,
    ) -> Result<String> {
        let mut code = String::new();

        // 生成基本块
        for (idx, bb) in func.basic_blocks.iter().enumerate() {
            // 基本块标签
            if idx > 0 {
                code.push_str(&format!("    // 基本块 {}\n", bb.id));
            }

            // 生成语句（带 LLM 增强的 unsafe 注释）
            for stmt in &bb.statements {
                code.push_str(
                    &self
                        .generate_statement_with_llm(stmt, 1, func, file_name)
                        .await?,
                );
            }

            // 生成终结符
            code.push_str(&self.generate_terminator(&bb.terminator, 1)?);
        }

        Ok(code)
    }

    /// 生成语句（带 LLM 增强的 unsafe 注释）
    async fn generate_statement_with_llm(
        &self,
        stmt: &Statement,
        indent_level: usize,
        func: &Function,
        file_name: &str,
    ) -> Result<String> {
        let indent = "    ".repeat(indent_level);
        let mut code = String::new();

        match stmt {
            Statement::Assign(lvalue, rvalue) => {
                let lval = self.generate_lvalue(lvalue)?;
                let rval = self.generate_rvalue(rvalue)?;

                // 检查是否需要 unsafe（例如涉及原始指针、FFI 调用等）
                let needs_unsafe = self.statement_needs_unsafe(stmt);

                if needs_unsafe {
                    // 【LLM 集成点 2】生成详细的 unsafe 注释
                    let c_code = format!("{} = {};", lval, rval); // 简化的 C 代码近似
                    let rust_code = format!("let {} = {};", lval, rval);
                    let reason = self.infer_unsafe_reason(stmt);

                    let safety_comment = generate_unsafe_explanation(
                        &self.project_name,
                        file_name,
                        &func.name,
                        &c_code,
                        &rust_code,
                        &reason,
                    )
                    .await
                    .unwrap_or_else(|_| format!("{}// SAFETY: {}\n", indent, reason));

                    code.push_str(&safety_comment);
                    code.push_str(&format!("{}unsafe {{\n", indent));
                    code.push_str(&format!("{}    let {} = {};\n", indent, lval, rval));
                    code.push_str(&format!("{}}}\n", indent));
                } else {
                    code.push_str(&format!("{}let {} = {};\n", indent, lval, rval));
                }
            }
            Statement::Call(target, func_name, args) => {
                let lval = self.generate_lvalue(target)?;
                let arg_strs: Vec<String> = args
                    .iter()
                    .map(|a| self.generate_rvalue(a))
                    .collect::<Result<_>>()?;

                let needs_unsafe = self.statement_needs_unsafe(stmt);

                if needs_unsafe {
                    let c_code = format!("{} = {}({});", lval, func_name, arg_strs.join(", "));
                    let rust_code =
                        format!("let {} = {}({});", lval, func_name, arg_strs.join(", "));
                    let reason = self.infer_unsafe_reason(stmt);

                    let safety_comment = generate_unsafe_explanation(
                        &self.project_name,
                        file_name,
                        &func.name,
                        &c_code,
                        &rust_code,
                        &reason,
                    )
                    .await
                    .unwrap_or_else(|_| format!("{}// SAFETY: {}\n", indent, reason));

                    code.push_str(&safety_comment);
                    code.push_str(&format!("{}unsafe {{\n", indent));
                    code.push_str(&format!(
                        "{}    let {} = {}({});\n",
                        indent,
                        lval,
                        func_name,
                        arg_strs.join(", ")
                    ));
                    code.push_str(&format!("{}}}\n", indent));
                } else {
                    code.push_str(&format!(
                        "{}let {} = {}({});\n",
                        indent,
                        lval,
                        func_name,
                        arg_strs.join(", ")
                    ));
                }
            }
            Statement::Annotated { stmt, annotations } => {
                // 添加注释
                for annotation in annotations {
                    code.push_str(&format!("{}// {}\n", indent, annotation));
                }
                // 使用 Box::pin 避免无限大小的 future
                let inner_code =
                    Box::pin(self.generate_statement_with_llm(stmt, indent_level, func, file_name))
                        .await?;
                code.push_str(&inner_code);
            }
        }

        Ok(code)
    }

    /// 检查语句是否需要 unsafe
    fn statement_needs_unsafe(&self, stmt: &Statement) -> bool {
        match stmt {
            Statement::Call(_, func_name, _) => {
                // FFI 调用或涉及指针操作的函数需要 unsafe
                func_name.starts_with("libc::")
                    || func_name.contains("malloc")
                    || func_name.contains("free")
                    || func_name.contains("ptr")
                    || func_name.contains("ffi")
            }
            Statement::Assign(_, rvalue) => {
                // 涉及指针操作（通过 AddressOf 或 Use 推断）
                self.rvalue_contains_pointer(rvalue)
            }
            Statement::Annotated { stmt, .. } => self.statement_needs_unsafe(stmt),
        }
    }

    /// 检查 RValue 是否涉及指针操作
    fn rvalue_contains_pointer(&self, rvalue: &RValue) -> bool {
        match rvalue {
            RValue::AddressOf(_) => true,
            RValue::BinaryOp(_, left, right) => {
                self.rvalue_contains_pointer(left) || self.rvalue_contains_pointer(right)
            }
            RValue::UnaryOp(_, operand) => self.rvalue_contains_pointer(operand),
            _ => false,
        }
    }

    /// 推断需要 unsafe 的原因
    fn infer_unsafe_reason(&self, stmt: &Statement) -> String {
        match stmt {
            Statement::Call(_, func_name, _) => {
                if func_name.starts_with("libc::") {
                    format!("调用外部 C 函数 {}", func_name)
                } else if func_name.contains("malloc") {
                    "分配原始内存".to_string()
                } else if func_name.contains("free") {
                    "释放原始内存".to_string()
                } else {
                    "调用可能不安全的函数".to_string()
                }
            }
            Statement::Assign(_, rvalue) => {
                if self.rvalue_contains_pointer(rvalue) {
                    "操作原始指针".to_string()
                } else {
                    "潜在的不安全操作".to_string()
                }
            }
            Statement::Annotated { stmt, .. } => self.infer_unsafe_reason(stmt),
        }
    }

    /// 生成语句
    fn generate_statement(&self, stmt: &Statement, indent_level: usize) -> Result<String> {
        let indent = "    ".repeat(indent_level);
        let mut code = String::new();

        match stmt {
            Statement::Assign(lvalue, rvalue) => {
                let lval = self.generate_lvalue(lvalue)?;
                let rval = self.generate_rvalue(rvalue)?;
                code.push_str(&format!("{}let {} = {};\n", indent, lval, rval));
            }
            Statement::Call(target, func_name, args) => {
                let lval = self.generate_lvalue(target)?;
                let arg_strs: Vec<String> = args
                    .iter()
                    .map(|a| self.generate_rvalue(a))
                    .collect::<Result<_>>()?;
                code.push_str(&format!(
                    "{}let {} = {}({});\n",
                    indent,
                    lval,
                    func_name,
                    arg_strs.join(", ")
                ));
            }
            Statement::Annotated { stmt, annotations } => {
                // 添加注释
                for annotation in annotations {
                    code.push_str(&format!("{}// {}\n", indent, annotation));
                }
                code.push_str(&self.generate_statement(stmt, indent_level)?);
            }
        }

        Ok(code)
    }

    /// 生成终结符
    fn generate_terminator(&self, term: &Terminator, indent_level: usize) -> Result<String> {
        let indent = "    ".repeat(indent_level);
        let mut code = String::new();

        match term {
            Terminator::Return(None) => {
                code.push_str(&format!("{}return;\n", indent));
            }
            Terminator::Return(Some(rvalue)) => {
                let rval = self.generate_rvalue(rvalue)?;
                code.push_str(&format!("{}return {};\n", indent, rval));
            }
            Terminator::Goto(bb_id) => {
                code.push_str(&format!("{}// goto block {}\n", indent, bb_id));
            }
            Terminator::If {
                condition,
                then_block,
                else_block,
            } => {
                let cond = self.generate_rvalue(condition)?;
                code.push_str(&format!("{}if {} {{\n", indent, cond));
                code.push_str(&format!("{}    // goto block {}\n", indent, then_block));
                code.push_str(&format!("{}}} else {{\n", indent));
                code.push_str(&format!("{}    // goto block {}\n", indent, else_block));
                code.push_str(&format!("{}}}\n", indent));
            }
        }

        Ok(code)
    }

    /// 生成左值
    fn generate_lvalue(&self, lvalue: &LValue) -> Result<String> {
        match lvalue {
            LValue::Variable(var_id) => Ok(format!("var_{}", var_id)),
            LValue::Deref(rvalue) => {
                let rval = self.generate_rvalue(rvalue)?;
                Ok(format!("(*{})", rval))
            }
        }
    }

    /// 生成右值
    fn generate_rvalue(&self, rvalue: &RValue) -> Result<String> {
        match rvalue {
            RValue::Use(lvalue) => self.generate_lvalue(lvalue),
            RValue::Constant(val) => Ok(val.to_string()),
            RValue::BinaryOp(op, left, right) => {
                let left_str = self.generate_rvalue(left)?;
                let right_str = self.generate_rvalue(right)?;
                let op_str = self.binop_to_rust(op);
                Ok(format!("({} {} {})", left_str, op_str, right_str))
            }
            RValue::UnaryOp(op, operand) => {
                let operand_str = self.generate_rvalue(operand)?;
                let op_str = self.unop_to_rust(op);
                Ok(format!("({}{})", op_str, operand_str))
            }
            RValue::AddressOf(lvalue) => {
                let lval = self.generate_lvalue(lvalue)?;
                Ok(format!("(&{})", lval))
            }
        }
    }

    /// 将 MIR 类型转换为 Rust 类型字符串
    fn type_to_rust(&self, ty: &Type) -> Result<String> {
        match ty {
            Type::Int => Ok("i32".to_string()),
            Type::Float => Ok("f64".to_string()),
            Type::Pointer(inner) => {
                let inner_type = self.type_to_rust(inner)?;
                Ok(format!("*mut {}", inner_type))
            }
            Type::Void => Ok("()".to_string()),
            Type::Unknown => Ok("()".to_string()),
        }
    }

    /// 二元运算符转 Rust
    fn binop_to_rust(&self, op: &BinOp) -> &'static str {
        match op {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        }
    }

    /// 一元运算符转 Rust
    fn unop_to_rust(&self, op: &UnOp) -> &'static str {
        match op {
            UnOp::Not => "!",
            UnOp::Neg => "-",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{BasicBlock, Parameter};
    use tempfile::TempDir;

    #[test]
    fn test_generate_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        let mut generator = CodeGenerator::new(temp_dir.path(), "test_project_rs".to_string());

        let project_mir = ProjectMIR::new();
        let analysis_results = ProjectAnalysisResults {
            results: HashMap::new(),
        };

        let result = generator.generate(&project_mir, &analysis_results);
        assert!(result.is_ok());

        // 验证生成的文件
        assert!(temp_dir.path().join("Cargo.toml").exists());
        assert!(temp_dir.path().join("src/lib.rs").exists());
    }

    #[test]
    fn test_generate_simple_function() {
        let temp_dir = TempDir::new().unwrap();
        let mut generator = CodeGenerator::new(temp_dir.path(), "test_project_rs".to_string());

        let mut project_mir = ProjectMIR::new();
        let func = Function {
            name: "add".to_string(),
            parameters: vec![
                Parameter {
                    name: "a".to_string(),
                    param_type: Type::Int,
                    var_id: 0,
                },
                Parameter {
                    name: "b".to_string(),
                    param_type: Type::Int,
                    var_id: 1,
                },
            ],
            return_type: Some(Type::Int),
            basic_blocks: vec![BasicBlock {
                id: 0,
                statements: vec![],
                terminator: Terminator::Return(Some(RValue::BinaryOp(
                    BinOp::Add,
                    Box::new(RValue::Use(Box::new(LValue::Variable(0)))),
                    Box::new(RValue::Use(Box::new(LValue::Variable(1)))),
                ))),
            }],
            annotations: vec![],
            is_static: false,
            is_public: true,
        };

        project_mir.functions.insert("add".to_string(), func);

        let analysis_results = ProjectAnalysisResults {
            results: HashMap::new(),
        };

        let result = generator.generate(&project_mir, &analysis_results);
        assert!(result.is_ok());

        // 验证生成的文件存在
        assert!(temp_dir.path().join("src/generated.rs").exists());

        // 读取并验证函数代码
        let generated_code = fs::read_to_string(temp_dir.path().join("src/generated.rs")).unwrap();
        assert!(generated_code.contains("pub fn add"));
        assert!(generated_code.contains("a: i32"));
        assert!(generated_code.contains("b: i32"));
    }

    #[test]
    fn test_type_conversion() {
        let temp_dir = TempDir::new().unwrap();
        let generator = CodeGenerator::new(temp_dir.path(), "test".to_string());

        assert_eq!(generator.type_to_rust(&Type::Int).unwrap(), "i32");
        assert_eq!(generator.type_to_rust(&Type::Float).unwrap(), "f64");
        assert_eq!(generator.type_to_rust(&Type::Void).unwrap(), "()");
        assert_eq!(
            generator
                .type_to_rust(&Type::Pointer(Box::new(Type::Int)))
                .unwrap(),
            "*mut i32"
        );
    }
}
