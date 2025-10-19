# C2RustAgent - C 到 Rust 的智能转译器

基于 LLM 增强的 C 到 Rust 代码转换工具，结合形式化静态分析与大语言模型的语义理解能力。

## 项目概述

C2RustAgent 是一个混合智能体系统，旨在将 C 代码安全、高效地转换为地道的 Rust 代码。系统采用多阶段管道设计：

```
C 源码 → Clang AST → MIR → 静态分析 → Rust 代码生成
              ↓           ↓         ↓            ↓
            LLM 语义分析和注释注入（贯穿全流程）
```

## 核心特性

### ✅ 已实现

#### 阶段一：Clang 前端解析
- ✅ 使用 `clang` crate 解析 C 代码
- ✅ AST 遍历和结构提取
- ✅ 支持标准 C11

#### 阶段二：MIR（中级中间表示）
- ✅ 完整的 MIR 数据结构设计
- ✅ 基本块（Basic Block）和控制流表示
- ✅ 左值/右值区分
- ✅ LLM 注释集成点预留
- ✅ JSON 序列化支持

#### 阶段三：静态分析管道与 LLM 集成
- ✅ 分析管理器（AnalysisManager）架构
- ✅ 活跃变量分析接口
- ✅ LLM 外部 API 语义推断
  - 资源管理语义（如 malloc/free）
  - 所有权转移标注
  - 副作用识别
  - 参数前置条件
- ✅ 异步 LLM 调用框架
- ✅ 完整测试套件

### 🚧 规划中

#### 阶段四：AST 到 MIR 转换（部分完成）
- 🚧 函数声明转换
- 🚧 表达式降级
- 🚧 控制流构建（循环、条件）
- 🚧 变量符号表管理

#### 阶段五：静态分析算法实现
- 🚧 活跃变量分析算法（数据流分析）
- 🚧 指针来源分析
- 🚧 借用检查模拟
- 🚧 生命周期推断
- 🚧 可变性分析

#### 阶段六：Rust 代码生成
- 🚧 地道 Rust 代码生成
- 🚧 unsafe 块最小化
- 🚧 安全注释生成
- 🚧 代码格式化（rustfmt）

## 技术栈

- **Rust**: 2024 Edition
- **Clang**: libclang 绑定 (clang 2.0, clang-sys 1.8)
- **序列化**: serde 1.0 + serde_json 1.0
- **错误处理**: anyhow 1.0, thiserror 1.0

## 快速开始

### 环境要求

- Rust 工具链（推荐使用 rustup）
- LLVM/Clang 开发库（需要安装 libclang）

### 安装依赖

```bash
# Windows (LLVM 官网下载安装器)
# https://github.com/llvm/llvm-project/releases

# Linux (Ubuntu/Debian)
sudo apt-get install llvm-dev libclang-dev

# macOS
brew install llvm
```

### 构建项目

```bash
git clone <repository-url>
cd C2RustAgent
cargo build
```

### 运行示例

```bash
cargo run
```

当前示例将展示：
1. C 代码的 Clang AST 解析
2. MIR 数据结构的构建
3. JSON 序列化输出

### 运行测试

```bash
cargo test
```

## 项目结构

```
C2RustAgent/
├── Cargo.toml              # 项目配置和依赖
├── src/
│   ├── main.rs            # 主程序入口
│   └── mir.rs             # MIR 数据结构定义
├── docs/
│   └── phase2_mir.md      # 阶段二文档
└── target/                # 构建输出
```

## 核心设计理念

### 混合智能体架构

本项目采用 **C2Rust-LLM 混合智能体** 设计：

1. **形式化骨架**：使用传统编译器技术（Clang → MIR → 静态分析）确保正确性
2. **LLM 增强**：在关键节点注入语义理解，提升代码质量和可读性

### LLM 集成策略

LLM 不会替代静态分析，而是作为 **语义协处理器**：

- **阶段二（MIR 转换）**：作为"语义标注器"，推断所有权契约
- **阶段三（静态分析）**：作为"启发式顾问"，辅助决策
- **阶段四（代码生成）**：作为"代码润色器"，生成地道 Rust 和安全文档

### 分离关注点

- ✅ **正确性**：由形式化方法保证（静态分析、类型检查）
- ✅ **可读性**：由 LLM 提升（注释、命名、模式识别）
- ✅ **安全性**：两者协同（unsafe 块最小化、前置条件生成）

## 示例输出

### 输入（C 代码）

```c
int add(int a, int b) { 
    return a + b; 
}
```

### 输出（MIR JSON）

```json
{
  "name": "add",
  "parameters": [
    {"name": "a", "param_type": "Int", "var_id": 0},
    {"name": "b", "param_type": "Int", "var_id": 1}
  ],
  "return_type": "Int",
  "basic_blocks": [
    {
      "id": 0,
      "statements": [],
      "terminator": {
        "Return": {
          "BinaryOp": ["Add", 
            {"Use": {"Variable": 0}},
            {"Use": {"Variable": 1}}
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

## 贡献指南

欢迎贡献！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

### 开发规范

- 使用 `rustfmt` 格式化代码
- 运行 `cargo clippy` 检查代码质量
- 为新功能添加测试
- 更新相关文档

## 许可证

待定

## 致谢

本项目灵感来源于：
- [C2Rust](https://github.com/immunant/c2rust) - 自动化 C 到 Rust 转换
- [rust-clippy](https://github.com/rust-lang/rust-clippy) - Rust linting 工具
- 混合智能体研究（结合传统编译技术与 LLM）

## 联系方式

- 项目主页: [待添加]
- 问题反馈: [GitHub Issues]
- 文档: [docs/](./docs/)

---

**注意**: 本项目当前处于早期开发阶段，API 和架构可能会有重大变更。
