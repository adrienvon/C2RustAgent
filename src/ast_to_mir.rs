//! AST 到 MIR 的转换器
//!
//! 这个模块负责将 Clang AST 转换为我们的中级中间表示 (MIR)。
//! 转换过程包括：
//! - 提取函数签名和参数
//! - 构建控制流图（CFG）
//! - 转换表达式和语句
//! - 管理变量和基本块的映射

use crate::mir::{self, BasicBlock, BasicBlockId, Function, ProjectMIR, Terminator, Type, VarId};
use crate::project_loader::CProject;
use anyhow::{Result, anyhow};
use clang::{Clang, Entity, EntityKind, Index};
use std::collections::HashMap;
use std::path::Path;

/// AST 到 MIR 的转换器
///
/// 维护转换过程中的状态信息，包括：
/// - 当前正在构建的 MIR 函数
/// - 变量名到变量 ID 的映射
/// - 基本块管理
pub struct Converter {
    /// 当前正在构建的 MIR 函数
    function: Function,

    /// 变量名到变量 ID 的映射表
    /// 用于在转换过程中查找变量
    var_map: HashMap<String, VarId>,

    /// 下一个可用的变量 ID
    next_var_id: VarId,

    /// 下一个可用的基本块 ID
    next_block_id: BasicBlockId,

    /// 当前正在构建的基本块
    /// 转换语句时会向这个基本块添加语句
    current_block: Option<BasicBlock>,
}

impl Converter {
    /// 项目级转换入口：两阶段 (发现符号 + 转换函数体)
    pub fn convert_project(project: &CProject) -> Result<ProjectMIR> {
        let mut proj_mir = ProjectMIR::new();
        // Pass 1: 发现符号
        Self::discover_symbols(project, &mut proj_mir)?;
        // Pass 2: 转换函数体
        Self::convert_bodies(project, &mut proj_mir)?;
        Ok(proj_mir)
    }

    /// 第一阶段：在不展开函数体的前提下，收集所有函数与全局符号
    fn discover_symbols(project: &CProject, out: &mut ProjectMIR) -> Result<()> {
        use clang::EntityKind;
        project.process_units(|_spec, tu| {
            let root = tu.get_entity();
            for child in root.get_children() {
                match child.get_kind() {
                    EntityKind::FunctionDecl => {
                        if let Some(name) = child.get_name() {
                            let ret_ty = child
                                .get_result_type()
                                .map(|t| Self::convert_clang_type(&t));
                            let mut f = Function::new(name.clone(), ret_ty);
                            // 可见性：占位，后续通过存储类/修饰符精确判定
                            f.is_static = false;
                            f.is_public = true;
                            out.functions.entry(name).or_insert(f);
                        }
                    }
                    EntityKind::VarDecl => {
                        if let Some(name) = child.get_name() {
                            let ty = child
                                .get_type()
                                .map(|t| Self::convert_clang_type(&t))
                                .unwrap_or(Type::Unknown);
                            let gv = mir::GlobalVar {
                                name: name.clone(),
                                var_type: ty,
                                is_static: false,
                                is_public: true,
                            };
                            out.globals.entry(name).or_insert(gv);
                        }
                    }
                    _ => {}
                }
            }
            Ok(())
        })
    }

    /// 第二阶段：转换每个函数的函数体，填充基本块与语句
    fn convert_bodies(project: &CProject, out: &mut ProjectMIR) -> Result<()> {
        use clang::EntityKind;
        project.process_units(|_spec, tu| {
            let root = tu.get_entity();
            for child in root.get_children() {
                if child.get_kind() == EntityKind::FunctionDecl {
                    // 仅处理有函数体的定义（不是仅声明）
                    if !child.is_definition() {
                        continue;
                    }
                    if let Some(name) = child.get_name() {
                        // 找到占位函数
                        if let Some(func) = out.functions.get_mut(&name) {
                            // 基本策略：创建入口 BB，终结符保持 Return(None) 占位
                            let entry_id = 0usize; // 简化：每个函数只有一个入口 BB
                            let entry_block = BasicBlock::new(entry_id, Terminator::Return(None));
                            func.basic_blocks.clear();
                            func.basic_blocks.push(entry_block);
                            // TODO: 遍历 child 的函数体，建立完整的 CFG 与语句
                        }
                    }
                }
            }
            Ok(())
        })
    }
    /// 创建新的转换器实例
    fn new(function_name: String, return_type: Option<Type>) -> Self {
        Converter {
            function: Function::new(function_name, return_type),
            var_map: HashMap::new(),
            next_var_id: 0,
            next_block_id: 0,
            current_block: None,
        }
    }

    /// 主入口点：从 C 文件转换为 MIR 函数
    ///
    /// # 参数
    /// - `c_file`: C 源代码文件路径
    ///
    /// # 返回
    /// - 成功时返回转换后的 MIR `Function`
    /// - 失败时返回错误信息
    pub fn convert(c_file: &str) -> Result<mir::Function> {
        // 初始化 Clang
        let clang = Clang::new().map_err(|e| anyhow!("无法初始化 Clang: {}", e))?;
        let index = Index::new(&clang, false, false);

        // 解析 C 文件
        let path = Path::new(c_file);
        if !path.exists() {
            return Err(anyhow!("文件不存在: {}", c_file));
        }

        let translation_unit = index
            .parser(c_file)
            .arguments(&["-std=c11"])
            .parse()
            .map_err(|e| anyhow!("无法解析 C 代码: {:?}", e))?;

        // 获取根实体
        let root_entity = translation_unit.get_entity();

        // 查找第一个函数声明
        let children = root_entity.get_children();
        for child in children {
            if child.get_kind() == EntityKind::FunctionDecl {
                // 找到函数声明，开始转换
                return Self::convert_function(&child);
            }
        }

        Err(anyhow!("在文件中未找到函数声明"))
    }

    /// 从已存在的 Clang Entity 转换函数（用于已有 TranslationUnit 的情况）
    ///
    /// # 参数
    /// - `root_entity`: Clang AST 根节点
    ///
    /// # 返回
    /// - 成功时返回转换后的 MIR `Function`
    /// - 失败时返回错误信息
    pub fn convert_from_entity(root_entity: &Entity) -> Result<mir::Function> {
        // 查找第一个函数声明
        let children = root_entity.get_children();
        for child in children {
            if child.get_kind() == EntityKind::FunctionDecl {
                // 找到函数声明，开始转换
                return Self::convert_function(&child);
            }
        }

        Err(anyhow!("在 AST 中未找到函数声明"))
    }

    /// 从 Clang Entity 转换单个函数
    fn convert_function(entity: &Entity) -> Result<mir::Function> {
        // 提取函数名
        let function_name = entity.get_name().ok_or_else(|| anyhow!("函数没有名称"))?;

        // 提取返回类型
        let return_type = entity
            .get_result_type()
            .map(|t| Self::convert_clang_type(&t));

        // 创建转换器实例
        let mut converter = Converter::new(function_name, return_type);

        // 访问并转换函数声明
        converter.visit_function_decl(entity)?;

        // 返回构建好的 MIR 函数
        Ok(converter.function)
    }

    /// 访问并转换函数声明
    ///
    /// 这是转换一个 C 函数的核心方法，负责：
    /// 1. 提取函数参数
    /// 2. 创建入口基本块
    /// 3. 遍历函数体并转换语句
    fn visit_function_decl(&mut self, entity: &Entity) -> Result<()> {
        println!("正在转换函数: {:?}", entity.get_name());

        // 1. 提取并注册函数参数
        let children = entity.get_children();
        for child in &children {
            if child.get_kind() == EntityKind::ParmDecl {
                self.visit_parameter(&child)?;
            }
        }

        // 2. 创建入口基本块
        let entry_block_id = self.allocate_block_id();
        let entry_block = BasicBlock::new(
            entry_block_id,
            Terminator::Return(None), // 临时终结符，会被后续处理替换
        );
        self.current_block = Some(entry_block);

        // 3. 遍历函数体
        for child in &children {
            if child.get_kind() == EntityKind::CompoundStmt {
                // 找到函数体（复合语句）
                self.visit_compound_stmt(&child)?;
                break;
            }
        }

        // 4. 完成当前基本块并添加到函数中
        if let Some(block) = self.current_block.take() {
            self.function.add_basic_block(block);
        }

        Ok(())
    }

    /// 访问并注册函数参数
    fn visit_parameter(&mut self, entity: &Entity) -> Result<()> {
        // 获取参数名称
        let param_name = entity.get_name().ok_or_else(|| anyhow!("参数没有名称"))?;

        // 获取参数类型
        let param_type = entity.get_type().ok_or_else(|| anyhow!("参数没有类型"))?;
        let mir_type = Self::convert_clang_type(&param_type);

        // 分配变量 ID
        let var_id = self.allocate_var_id();

        // 注册变量映射
        self.var_map.insert(param_name.clone(), var_id);

        // 添加参数到函数
        self.function.add_parameter(param_name, mir_type, var_id);

        Ok(())
    }

    /// 访问复合语句（函数体）
    fn visit_compound_stmt(&mut self, entity: &Entity) -> Result<()> {
        println!("  访问复合语句（函数体）");

        // TODO: Traverse function body's CFG and convert statements.
        //
        // 这里需要实现：
        // 1. 遍历复合语句中的所有子语句
        // 2. 根据语句类型（return、if、while 等）转换为 MIR Statement 或 Terminator
        // 3. 处理控制流，创建新的基本块
        // 4. 转换表达式为 RValue 和 LValue
        //
        // 当前为占位实现，后续会填充具体逻辑。

        let children = entity.get_children();
        println!("    函数体包含 {} 个语句", children.len());

        // 临时实现：仅打印语句类型
        for (i, child) in children.iter().enumerate() {
            println!("    语句 {}: {:?}", i, child.get_kind());
        }

        Ok(())
    }

    /// 将 Clang 类型转换为 MIR 类型
    fn convert_clang_type(clang_type: &clang::Type) -> Type {
        use clang::TypeKind;

        match clang_type.get_kind() {
            TypeKind::Int
            | TypeKind::Long
            | TypeKind::Short
            | TypeKind::CharS
            | TypeKind::CharU => Type::Int,
            TypeKind::Float | TypeKind::Double => Type::Float,
            TypeKind::Void => Type::Void,
            TypeKind::Pointer => {
                // 递归处理指针指向的类型
                if let Some(pointee) = clang_type.get_pointee_type() {
                    Type::Pointer(Box::new(Self::convert_clang_type(&pointee)))
                } else {
                    Type::Pointer(Box::new(Type::Unknown))
                }
            }
            _ => {
                // 其他类型暂时标记为未知
                println!("    警告: 遇到未处理的类型: {:?}", clang_type.get_kind());
                Type::Unknown
            }
        }
    }

    /// 分配新的变量 ID
    fn allocate_var_id(&mut self) -> VarId {
        let id = self.next_var_id;
        self.next_var_id += 1;
        id
    }

    /// 分配新的基本块 ID
    fn allocate_block_id(&mut self) -> BasicBlockId {
        let id = self.next_block_id;
        self.next_block_id += 1;
        id
    }

    /// 查找变量 ID
    fn lookup_var(&self, name: &str) -> Option<VarId> {
        self.var_map.get(name).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_simple_function() {
        // 创建临时测试文件
        let test_file = "test_simple.c";
        let c_code = "int add(int a, int b) { return a + b; }";
        std::fs::write(test_file, c_code).unwrap();

        // 转换
        let result = Converter::convert(test_file);

        // 清理
        let _ = std::fs::remove_file(test_file);

        // 验证
        assert!(result.is_ok(), "转换应该成功");
        let func = result.unwrap();
        assert_eq!(func.name, "add");
        assert_eq!(func.parameters.len(), 2);
        assert_eq!(func.parameters[0].name, "a");
        assert_eq!(func.parameters[1].name, "b");
    }

    #[test]
    fn test_convert_void_function() {
        // 创建临时测试文件
        let test_file = "test_void.c";
        let c_code = "void hello() { }";
        std::fs::write(test_file, c_code).unwrap();

        // 转换
        let result = Converter::convert(test_file);

        // 清理
        let _ = std::fs::remove_file(test_file);

        // 验证
        assert!(result.is_ok(), "转换应该成功: {:?}", result.err());
        let func = result.unwrap();
        assert_eq!(func.name, "hello");
        assert_eq!(func.parameters.len(), 0);
        assert!(matches!(func.return_type, Some(Type::Void)));
    }
}
