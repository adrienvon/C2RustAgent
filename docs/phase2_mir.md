# 阶段二：MIR（中级中间表示）设计

## 概述

本阶段成功设计并实现了 C2Rust-LLM 混合智能体的核心数据结构：**MIR (Middle-level Intermediate Representation)**。这个中间表示是连接 Clang AST 和最终 Rust 代码生成的关键桥梁。

## 核心数据结构

### 1. **Function（函数）**
表示一个完整的函数，包含：
- `name`: 函数名称
- `parameters`: 参数列表
- `return_type`: 返回类型
- `basic_blocks`: 基本块集合
- **`annotations`**: LLM 生成的语义注释（关键集成点）

### 2. **BasicBlock（基本块）**
控制流图中的节点，包含：
- `id`: 唯一标识符
- `statements`: 语句序列
- `terminator`: 终结符（控制流转移）

### 3. **Statement（语句）**
基本块内的操作，支持：
- `Assign(LValue, RValue)`: 赋值语句
- `Annotated`: 带 LLM 注释的语句包装器

### 4. **Terminator（终结符）**
基本块的结束方式：
- `Goto(BasicBlockId)`: 无条件跳转
- `Return(Option<RValue>)`: 函数返回
- `If { condition, then_block, else_block }`: 条件分支

### 5. **LValue（左值）**
可被赋值的内存位置：
- `Variable(VarId)`: 变量引用
- `Deref(Box<RValue>)`: 解引用操作

### 6. **RValue（右值）**
计算出的值：
- `Use(Box<LValue>)`: 使用左值
- `Constant(i32)`: 整数常量
- `BinaryOp(BinOp, Box<RValue>, Box<RValue>)`: 二元运算
- `UnaryOp(UnOp, Box<RValue>)`: 一元运算
- `AddressOf(Box<LValue>)`: 取地址

### 7. **Type（类型系统）**
支持的类型：
- `Int`: 整数
- `Float`: 浮点数
- `Pointer(Box<Type>)`: 指针
- `Void`: 空类型
- `Unknown`: 未知类型

### 8. **运算符**
- **BinOp**: Add, Sub, Mul, Div, Mod, Eq, Ne, Lt, Le, Gt, Ge, And, Or
- **UnOp**: Not, Neg

## LLM 集成点

### 关键设计特性

1. **Function.annotations**
   - 存储函数级别的语义注释
   - 例如：所有权契约、函数意图、安全前置条件等

2. **Statement::Annotated**
   - 为特定语句添加上下文注释
   - 支持嵌套结构，可以包装任何语句

这些注释字段为后续的 LLM 语义分析预留了接口，使得 LLM 可以注入：
- 所有权推断信息
- 变量用途识别
- 命名规范分析
- 编程模式识别

## 示例

### C 代码
```c
int add(int a, int b) { return a + b; }
```

### MIR 表示（JSON 格式）
```json
{
  "name": "add",
  "parameters": [
    {
      "name": "a",
      "param_type": "Int",
      "var_id": 0
    },
    {
      "name": "b",
      "param_type": "Int",
      "var_id": 1
    }
  ],
  "return_type": "Int",
  "basic_blocks": [
    {
      "id": 0,
      "statements": [],
      "terminator": {
        "Return": {
          "BinaryOp": [
            "Add",
            {
              "Use": {
                "Variable": 0
              }
            },
            {
              "Use": {
                "Variable": 1
              }
            }
          ]
        }
      }
    }
  ],
  "annotations": [
    "Function takes ownership of parameters",
    "Returns sum of two integers"
  ]
}
```

## 技术实现细节

### 1. 递归类型处理
- 使用 `Box<T>` 打破 LValue 和 RValue 之间的递归循环
- 确保类型有限大小，满足 Rust 的类型系统要求

### 2. 序列化支持
- 派生 `Serialize` 和 `Deserialize` trait
- 支持将 MIR 导出为 JSON 格式
- 便于调试、可视化和与其他工具集成

### 3. 测试覆盖
- `test_create_simple_function`: 验证基本函数创建和 MIR 构建
- `test_statement_with_annotations`: 验证注释功能

## 下一步计划

### 阶段三：AST 到 MIR 的转换
1. 实现 Clang AST 遍历器
2. 建立 AST 节点到 MIR 节点的映射
3. 处理复杂的控制流结构（循环、条件语句）
4. 构建变量符号表

### 阶段四：LLM 语义注释器
1. 集成 LLM API（如 OpenAI、Claude）
2. 实现语义分析提示词工程
3. 将 LLM 推断的所有权信息注入 MIR

## 架构优势

### 分离关注点
- **形式化分析**：由静态分析器处理（借用检查、数据流）
- **语义理解**：由 LLM 处理（意图推断、模式识别）

### 可扩展性
- 模块化设计，易于添加新的语句类型和终结符
- 注释系统开放式，支持任意文本注释

### 正确性基础
- 强类型系统确保 MIR 结构的一致性
- Rust 的所有权系统防止内存安全问题

## 总结

阶段二成功实现了一个设计良好、结构清晰的 MIR，为 C2Rust-LLM 混合智能体的核心管道奠定了坚实基础。通过在关键结构中预留 LLM 注释字段，我们为后续的智能语义分析铺平了道路，同时保持了形式化方法的严谨性。
