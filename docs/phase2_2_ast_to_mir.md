# 阶段 2.2：AST 到 MIR 转换器框架

## 概述

本阶段成功实现了 AST 到 MIR 的转换器框架，建立了从 Clang AST 到我们自定义 MIR 的转换管道。这是 C2Rust-LLM 混合智能体的关键组件，负责将 C 代码的语法树转换为适合静态分析和代码生成的中间表示。

## 新增文件

### `src/ast_to_mir.rs`

完整的 AST 到 MIR 转换器模块，包含：

## 核心组件

### 1. **Converter 结构体**

转换器的核心状态管理结构：

```rust
pub struct Converter {
    /// 当前正在构建的 MIR 函数
    function: Function,
    
    /// 变量名到变量 ID 的映射表
    var_map: HashMap<String, VarId>,
    
    /// 下一个可用的变量 ID
    next_var_id: VarId,
    
    /// 下一个可用的基本块 ID
    next_block_id: BasicBlockId,
    
    /// 当前正在构建的基本块
    current_block: Option<BasicBlock>,
}
```

**设计亮点**：
- 维护转换过程中的所有状态信息
- 使用 HashMap 快速查找变量映射
- 自动分配唯一的变量和基本块 ID
- 支持增量构建基本块

### 2. **公共 API**

#### `convert(c_file: &str) -> Result<Function>`

独立的转换入口点，适用于独立使用：
- 初始化 Clang 实例
- 解析 C 文件
- 查找函数声明
- 转换为 MIR

#### `convert_from_entity(root_entity: &Entity) -> Result<Function>`

从已存在的 AST 节点转换，适用于：
- 已有 TranslationUnit 的情况
- 避免多次初始化 Clang（Clang 单例限制）
- 与其他工具链集成

### 3. **核心转换方法**

#### `visit_function_decl(&mut self, entity: &Entity)`

函数声明的转换核心：
1. ✅ 提取函数参数
2. ✅ 注册参数到变量映射表
3. ✅ 创建入口基本块
4. 🚧 遍历函数体（占位实现）

#### `visit_parameter(&mut self, entity: &Entity)`

参数处理：
- 提取参数名称和类型
- 分配唯一的变量 ID
- 注册到变量映射表
- 添加到 MIR 函数的参数列表

#### `visit_compound_stmt(&mut self, entity: &Entity)`

函数体遍历（当前为占位实现）：
- 遍历所有子语句
- 打印语句类型（调试信息）
- **TODO**: 完整的语句转换逻辑

### 4. **辅助功能**

#### `convert_clang_type(clang_type: &Type) -> Type`

类型转换映射：
- `Int`, `Long`, `Short`, `CharS`, `CharU` → `Type::Int`
- `Float`, `Double` → `Type::Float`
- `Void` → `Type::Void`
- `Pointer` → `Type::Pointer(Box<Type>)`（递归处理）
- 其他 → `Type::Unknown`

#### ID 分配器

- `allocate_var_id()`: 分配新的变量 ID
- `allocate_block_id()`: 分配新的基本块 ID
- `lookup_var(name)`: 查找变量 ID（预留接口）

## 测试覆盖

### 1. `test_convert_simple_function`

测试带参数和返回值的函数：

```c
int add(int a, int b) { return a + b; }
```

验证点：
- ✅ 函数名提取正确
- ✅ 参数数量和名称正确
- ✅ 参数类型映射正确

### 2. `test_convert_void_function`

测试无参数 void 函数：

```c
void hello() { }
```

验证点：
- ✅ void 返回类型处理正确
- ✅ 空参数列表处理正确
- ✅ 空函数体处理正确

## 当前实现状态

### ✅ 已完成

1. **框架搭建**
   - Converter 结构体定义
   - 状态管理系统
   - 变量和基本块 ID 分配

2. **函数签名转换**
   - 函数名提取
   - 参数列表处理
   - 返回类型映射

3. **类型系统映射**
   - 基本类型转换
   - 指针类型递归处理
   - 未知类型标记

4. **测试覆盖**
   - 基本函数转换测试
   - Void 函数测试
   - 所有测试通过 ✅

### 🚧 待实现（占位）

1. **函数体转换**
   ```rust
   // TODO: Traverse function body's CFG and convert statements.
   ```

   需要实现：
   - 语句类型识别（return, if, while, for, etc.）
   - 表达式转换为 RValue/LValue
   - 控制流构建（创建新基本块）
   - 终结符生成

2. **表达式处理**
   - 二元运算符
   - 一元运算符
   - 函数调用
   - 数组访问
   - 成员访问

3. **控制流构建**
   - if-else 分支
   - while/for 循环
   - switch 语句
   - break/continue 处理

## 示例输出

### 输入（C 代码）

```c
int add(int a, int b) { return a + b; }
```

### 转换过程输出

```
正在转换函数: Some("add")
  访问复合语句（函数体）
    函数体包含 1 个语句
    语句 0: ReturnStmt
```

### 输出（MIR JSON）

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
        "Return": null
      }
    }
  ],
  "annotations": []
}
```

**注意**：当前终结符为 `Return: null` 是因为函数体转换尚未完全实现。

## 架构优势

### 1. 模块化设计

- 每个转换步骤都有独立的方法
- 职责清晰，易于扩展
- 便于添加新的语句类型处理

### 2. 状态封装

- 所有转换状态都在 Converter 中管理
- 避免全局变量
- 支持并发转换（未来扩展）

### 3. 类型安全

- 利用 Rust 类型系统保证正确性
- VarId 和 BasicBlockId 类型别名防止混淆
- Result 类型统一错误处理

### 4. 可测试性

- 每个方法都可独立测试
- 模拟数据结构简单
- 测试覆盖关键路径

## 与 LLM 集成点

虽然当前实现还未集成 LLM，但框架已预留了扩展点：

1. **变量映射表**
   - 将来可存储 LLM 推断的变量语义
   - 例如："buffer" → 推断为"字符串缓冲区"

2. **类型转换**
   - `convert_clang_type` 可接受 LLM 提示
   - 例如：`char*` 根据上下文推断为 `&str` 还是 `*mut u8`

3. **函数注释**
   - 在 `visit_function_decl` 中可调用 LLM
   - 生成所有权契约注释

## 下一步计划

### 阶段 2.3：实现表达式转换

需要实现：
1. `visit_return_stmt` - 转换 return 语句
2. `visit_binary_operator` - 转换二元运算
3. `visit_decl_ref_expr` - 转换变量引用
4. `convert_expr_to_rvalue` - 表达式转 RValue

### 阶段 2.4：实现控制流构建

需要实现：
1. `visit_if_stmt` - if-else 分支
2. `visit_while_stmt` - while 循环
3. 基本块分裂和跳转逻辑

## 总结

阶段 2.2 成功搭建了 AST 到 MIR 转换器的完整框架。虽然具体的语句转换逻辑尚未实现（占位），但核心架构、状态管理、类型映射和函数签名提取都已就绪。这为后续的表达式转换和控制流构建奠定了坚实的基础。

**关键成就**：
- ✅ 转换器框架完成
- ✅ 函数签名提取正常工作
- ✅ 参数映射系统运行良好
- ✅ 测试覆盖关键功能
- ✅ 代码结构清晰，易于扩展

这个框架设计使得我们可以逐步填充具体的转换逻辑，而不需要大规模重构！🎉
