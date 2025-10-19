//! 中级中间表示 (MIR - Middle-level Intermediate Representation)
//!
//! 这个模块定义了从 Clang AST 转换而来的中间表示，用于后续的静态分析和 Rust 代码生成。

use serde::{Deserialize, Serialize};

/// 基本块 ID 类型别名
pub type BasicBlockId = usize;

/// 变量 ID 类型别名
pub type VarId = usize;

/// 函数的 MIR 表示
///
/// 表示一个完整的函数，包含其签名、参数、基本块等信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    /// 函数名称
    pub name: String,

    /// 函数参数列表
    pub parameters: Vec<Parameter>,

    /// 返回类型
    pub return_type: Option<Type>,

    /// 函数体的基本块集合
    pub basic_blocks: Vec<BasicBlock>,

    /// LLM 生成的语义注释
    /// 例如：所有权契约、函数意图等
    pub annotations: Vec<String>,
}

/// 函数参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// 参数名称
    pub name: String,

    /// 参数类型
    pub param_type: Type,

    /// 参数对应的变量 ID
    pub var_id: VarId,
}

/// 类型表示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Type {
    /// 整数类型
    Int,

    /// 浮点数类型
    Float,

    /// 指针类型
    Pointer(Box<Type>),

    /// Void 类型
    Void,

    /// 未知或未实现的类型
    Unknown,
}

/// 基本块
///
/// 控制流图中的一个节点，包含一系列语句和一个终结符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    /// 基本块的唯一标识符
    pub id: BasicBlockId,

    /// 基本块中的语句序列
    pub statements: Vec<Statement>,

    /// 基本块的终结符（控制流转移）
    pub terminator: Terminator,
}

/// 语句
///
/// 表示基本块内的操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    /// 赋值语句: LValue = RValue
    Assign(LValue, RValue),

    /// LLM 生成的语义注释
    /// 用于存储对特定语句的高层语义理解
    Annotated {
        /// 实际的语句
        stmt: Box<Statement>,
        /// 注释内容
        annotations: Vec<String>,
    },
}

/// 终结符
///
/// 表示基本块的结束方式和控制流转移
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Terminator {
    /// 无条件跳转到指定基本块
    Goto(BasicBlockId),

    /// 函数返回
    /// None 表示 void 返回，Some 表示返回一个值
    Return(Option<RValue>),

    /// 条件分支: if (condition) then_block else else_block
    If {
        /// 条件表达式
        condition: RValue,
        /// 条件为真时跳转的基本块
        then_block: BasicBlockId,
        /// 条件为假时跳转的基本块
        else_block: BasicBlockId,
    },
}

/// 左值 (LValue)
///
/// 表示可以被赋值的内存位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LValue {
    /// 变量引用
    Variable(VarId),

    /// 解引用操作: *rvalue
    Deref(Box<RValue>),
}

/// 右值 (RValue)
///
/// 表示计算出的值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RValue {
    /// 使用一个左值的值
    Use(Box<LValue>),

    /// 整数常量
    Constant(i32),

    /// 二元运算: left op right
    BinaryOp(BinOp, Box<RValue>, Box<RValue>),

    /// 一元运算: op operand
    UnaryOp(UnOp, Box<RValue>),

    /// 取地址: &lvalue
    AddressOf(Box<LValue>),
}

/// 二元运算符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinOp {
    /// 加法 +
    Add,

    /// 减法 -
    Sub,

    /// 乘法 *
    Mul,

    /// 除法 /
    Div,

    /// 取模 %
    Mod,

    /// 等于 ==
    Eq,

    /// 不等于 !=
    Ne,

    /// 小于 <
    Lt,

    /// 小于等于 <=
    Le,

    /// 大于 >
    Gt,

    /// 大于等于 >=
    Ge,

    /// 逻辑与 &&
    And,

    /// 逻辑或 ||
    Or,
}

/// 一元运算符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnOp {
    /// 逻辑非 !
    Not,

    /// 取负 -
    Neg,
}

impl Function {
    /// 创建一个新的函数
    pub fn new(name: String, return_type: Option<Type>) -> Self {
        Function {
            name,
            parameters: Vec::new(),
            return_type,
            basic_blocks: Vec::new(),
            annotations: Vec::new(),
        }
    }

    /// 添加参数
    pub fn add_parameter(&mut self, name: String, param_type: Type, var_id: VarId) {
        self.parameters.push(Parameter {
            name,
            param_type,
            var_id,
        });
    }

    /// 添加基本块
    pub fn add_basic_block(&mut self, block: BasicBlock) {
        self.basic_blocks.push(block);
    }

    /// 添加 LLM 注释
    pub fn add_annotation(&mut self, annotation: String) {
        self.annotations.push(annotation);
    }
}

impl BasicBlock {
    /// 创建一个新的基本块
    pub fn new(id: BasicBlockId, terminator: Terminator) -> Self {
        BasicBlock {
            id,
            statements: Vec::new(),
            terminator,
        }
    }

    /// 添加语句到基本块
    pub fn add_statement(&mut self, stmt: Statement) {
        self.statements.push(stmt);
    }
}

impl Statement {
    /// 为语句添加注释
    pub fn with_annotations(self, annotations: Vec<String>) -> Self {
        Statement::Annotated {
            stmt: Box::new(self),
            annotations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_simple_function() {
        // 创建一个简单的函数: int add(int a, int b) { return a + b; }
        let mut func = Function::new("add".to_string(), Some(Type::Int));

        // 添加参数
        func.add_parameter("a".to_string(), Type::Int, 0);
        func.add_parameter("b".to_string(), Type::Int, 1);

        // 创建基本块
        let bb = BasicBlock::new(
            0,
            Terminator::Return(Some(RValue::BinaryOp(
                BinOp::Add,
                Box::new(RValue::Use(Box::new(LValue::Variable(0)))),
                Box::new(RValue::Use(Box::new(LValue::Variable(1)))),
            ))),
        );

        func.add_basic_block(bb);

        assert_eq!(func.name, "add");
        assert_eq!(func.parameters.len(), 2);
        assert_eq!(func.basic_blocks.len(), 1);
    }

    #[test]
    fn test_statement_with_annotations() {
        let stmt = Statement::Assign(LValue::Variable(0), RValue::Constant(42));

        let annotated =
            stmt.with_annotations(vec!["This is an LLM-generated annotation".to_string()]);

        match annotated {
            Statement::Annotated { annotations, .. } => {
                assert_eq!(annotations.len(), 1);
            }
            _ => panic!("Expected annotated statement"),
        }
    }
}
