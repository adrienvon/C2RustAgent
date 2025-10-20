# C2RustAgent - 混合智能 C 到 Rust 转译器

> 基于形式化静态分析与 LLM 增强的 C→Rust 自动迁移系统，从"能用"到"好用"的智能代码转换助理

---

## 项目简介

**C2RustAgent** 是一个创新的混合智能体系统，旨在将 C 代码**安全、高效、地道**地转换为 Rust 代码。不同于传统的纯机械翻译，本项目采用"形式化方法 + LLM 协处理"的架构，在保证正确性的同时注入语义理解，生成高质量、可维护的 Rust 代码。

### 核心设计理念

系统的骨架（Clang → MIR → 静态分析 → 代码生成）提供**正确性的基石**，LLM 在管道的关键节点注入**人类程序员的上下文与意图理解**：

```
+----------------+      +-------------------+      +-----------------+
| C Source Code  | ---> | Clang Frontend    | ---> | Clang AST & CFG |
+----------------+      +-------------------+      +-----------------+
       |                      ^
       | (Phase 1)            | (LLM Semantic Analysis on raw source)
       v                      |
+----------------+      +---------------------+      +---------------------+ <---> +------------------+
| MIR Conversion | ---> |     Our MIR         | ---> |  Analysis Pipeline  |       | LLM Co-pilot     |
| (翻译为MIR)    |      | (中级中间表示)      |      |  (分析管道)         |       | (语言模型协处理器) |
+----------------+      +---------------------+      +---------------------+ <---> +------------------+
       |                      |                                ^
       | (Phase 2)            | (Annotated by LLM)             | (Phase 3: Gets heuristic help)
       v                      v
+-----------------+      +---------------------+      +-------------------+
| Annotated MIR   | ---> | Rust Code Generator | ---> | Rust Cargo Project|
| (注解后的MIR)   |      | (代码生成器)        |      | (最终产物)        |
+-----------------+      +---------------------+      +-------------------+
       |                               ^
       | (Phase 4: Gets refinement help) |
       v
```

**设计优势**：
- 🔒 **形式化方法保证正确性** - 可信的静态分析基础
- 🧠 **LLM 注入语义理解** - 理解代码意图、命名规范和编程模式
- ✨ **从"能用"到"好用"** - 生成地道、自解释、易维护的 Rust 代码

📖 **详细设计文档**：查看 [核心设计理念](docs/CORE_DESIGN.md)

---

## 运行条件

### 子项目快速实现版本（translate_hybrid/）

**translate_hybrid** 是一个快速实现的简化转译子项目，展示了 LLM 辅助翻译的完整流程：

**必需环境**：
* Rust 工具链 1.70+ (推荐使用 rustup)
* LLM API 访问（支持 OpenAI 兼容接口）
* Docker (推荐) 或 LLVM/Clang 本地安装
* Windows/Linux/macOS 操作系统

**推荐配置**：
* 8GB+ 可用内存
* 网络连接（用于 API 调用）
* VS Code + Rust Analyzer (开发调试)

---

## 运行说明

### 方式一：使用 Docker (推荐)

Docker 环境已预配置所有依赖，无需手动安装 LLVM/Clang：

```powershell
# 1. Windows 用户 - 基础测试
.\scripts\docker_run.ps1

# 2. 完整翻译（翻译 chibicc 的 9 个 C 文件）
.\scripts\docker_run.ps1 -FullTranslation

# 3. Linux/Mac 用户
bash scripts/docker_run.sh
```

**Docker 环境提供**：
- ✅ 完整的 Rust + Clang + LLVM 工具链
- ✅ 自动生成编译数据库 (compile_commands.json)
- ✅ 单文件翻译测试（利用 1049K 上下文）
- ✅ 编译验证和 unsafe 代码分析
- ✅ 迭代修复机制
- ✅ 详细的翻译报告

### 方式二：本地运行（需手动配置）

```bash
# 1. 安装 LLVM/Clang（Windows 用户从官网下载安装器）
#    https://github.com/llvm/llvm-project/releases
#    设置环境变量 LIBCLANG_PATH

# 2. 配置 LLM API
cargo run --bin c2rust-agent-config -- init
# 编辑 %APPDATA%\c2rust-agent\config.toml 填写 API Key

# 3. 转换示例项目
cd translate_chibicc
cargo run -- ./src

# 4. 查看生成的 Rust 代码
cd rust_output_final
cargo build --release
cargo test
```

**子项目快速开始**：查看 [`translate_hybrid/QUICKSTART.md`](translate_hybrid/QUICKSTART.md)

---

## 测试说明

### 已验证的测试场景

**chibicc C 编译器项目**（translate_chibicc/）：
- 📦 项目规模：~8,150 行 C 代码，9 个源文件
- ✅ 已转换：4 个核心模块 (unicode, strings, hashmap, types)
- ✅ Rust 输出：945 行代码，12/12 单元测试通过
- ✅ 编译状态：cargo build --release 成功
- ⏱️ 预计全项目翻译时间：15-30 分钟（使用 LLM）
- 🎯 目标：编译通过率 >90%，unsafe 代码 <5%

**测试命令**：
```bash
# 运行子项目测试
cd rust_output_final
cargo test

# 查看翻译报告
cat ../docs/reports/TRANSLATION_SUCCESS_SUMMARY.md
```

**测试结果文档**：
- [转换成功总结](docs/reports/TRANSLATION_SUCCESS_SUMMARY.md) ⭐ 推荐
- [详细技术报告](docs/reports/FINAL_TRANSLATION_REPORT.md)
- [chibicc 转换指南](docs/translation/CHIBICC_TRANSLATION.md)

---

## 技术架构

### 混合智能体系统 (Hybrid Intelligence Agent)

本项目采用**"形式化分析 + LLM 协处理"**的创新架构，在四个关键阶段集成 LLM：

#### 🎯 核心技术栈

**主系统（核心设计 - 进行中）**：
* **Rust 2024 Edition** - 系统实现语言
* **libclang/clang-sys** - C 代码前端解析
* **自定义 MIR** - 中级中间表示（支持 LLM 注释）
* **静态分析框架** - 数据流分析、借用检查模拟
* **异步 LLM 集成** - tokio + async-openai

**子项目（快速实现版）**：
* **直接 LLM 翻译** - 利用大上下文 (1049K) 端到端转换
* **迭代修复机制** - 自动 cargo check + 错误反馈
* **流式响应处理** - 实时进度显示
* **unsafe 优化分析** - 智能减少 unsafe 代码占比

#### 📐 LLM 增强的四个阶段

**阶段一：语义标注器 (Semantic Annotator)**
- 任务：从 C 源码注释和命名中推断所有权契约
- 输入：函数签名、变量声明、注释
- 输出：`[Ownership::Takes]`, `[ReturnsNewResource(free)]` 等标注
- 集成：标注附加到 MIR 节点作为元数据

**阶段二：启发式顾问 (Heuristic Advisor)**
- 任务：在静态分析模糊时提供置信度建议
- 场景：指针类型决策 (&T vs Box<T>)、别名分析冲突
- 输出：带置信度的建议 + 修复方案
- 集成：作为分析器的启发式输入

**阶段三：代码润色器 (Code Refiner)**
- 任务：将机械翻译转换为地道 Rust
- 优化：C 风格循环 → 迭代器、指针链 → 组合子
- 输出：可读性更强的 Rust 代码
- 集成：后处理生成的代码

**阶段四：安全文档生成器 (Safety Documenter)**
- 任务：为 unsafe 块生成人类可读的安全注释
- 输入：原始 C 代码 + unsafe Rust + 静态分析理由
- 输出：`// SAFETY: ...` 注释 + 前置条件说明
- 集成：插入到 unsafe 块上方

#### � 当前实现状态

**✅ 已完成（主系统基础）**：
- Clang AST 解析
- MIR 数据结构设计
- LLM 异步调用框架
- 分析管理器架构

**✅ 已完成（子项目）**：
- 端到端 LLM 翻译
- Docker 测试环境
- 4 个模块的成功转换
- 完整的测试和文档

**🚧 进行中**：
- AST → MIR 完整转换
- 静态分析算法实现
- 借用检查模拟
- 地道 Rust 代码生成

**� 规划中**：
- 指针来源分析
- 生命周期推断
- unsafe 块最小化
- 完整项目级翻译

**详细设计文档**：
- [完整架构图](docs/CORE_DESIGN.md)
- [LLM 集成策略](docs/P4提示词.md)
- [文档导航](docs/INDEX.md)

## 技术栈

- **Rust**: 2024 Edition
- **Clang**: libclang 绑定 (clang 2.0, clang-sys 1.8)
- **序列化**: serde 1.0 + serde_json 1.0
- **错误处理**: anyhow 1.0, thiserror 1.0
- **LLM 集成**: async-openai 0.24, tokio 1.x
---

## 协作者

### 项目团队

**核心开发者**：
- [@adrienvon](https://github.com/adrienvon) - 项目架构设计与核心开发

**特别感谢**：
- chibicc 项目 - 提供优秀的测试用例
- Rust 社区 - 技术支持和最佳实践
- OpenAI - LLM API 支持

### 贡献指南

欢迎贡献！请查看我们的贡献指南：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

**贡献方向**：
- 🔧 完善静态分析算法实现
- 📝 改进 LLM 提示词设计
- 🧪 添加更多测试用例
- 📚 完善文档和教程
- 🐛 Bug 修复和性能优化

---

## 文档导航

### 快速入口
- 📖 [完整文档索引](docs/INDEX.md) - 所有文档的中心导航
- 🚀 [Docker 快速参考](docs/docker/DOCKER_QUICKREF.md) - 常用命令速查
- 📊 [转换成功总结](docs/reports/TRANSLATION_SUCCESS_SUMMARY.md) - 最新成果展示

### 技术文档
- 🏗️ [核心设计理念](docs/CORE_DESIGN.md) - 混合智能体架构详解
- 🔬 [LLM 集成策略](docs/P4提示词.md) - 语义标注、启发式建议
- 🛠️ [chibicc 转换指南](docs/translation/CHIBICC_TRANSLATION.md) - 完整转换流程
- 🐳 [Docker 使用指南](docs/docker/DOCKER_GUIDE.md) - 环境配置详解
- 📋 [项目清理报告](docs/CLEANUP_REPORT.md) - 代码整理记录
- 📝 [Scripts 使用说明](scripts/README.md) - 脚本工具文档

### 子项目文档
- ⚡ [Translate Hybrid 快速开始](translate_hybrid/QUICKSTART.md) - 简化版转译器
- 📂 [Translate Chibicc 信息](translate_chibicc/INFO) - chibicc 测试项目

---

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

---

## 项目状态

- **开发阶段**：早期开发 (Alpha)
- **主系统**：核心架构设计完成，静态分析算法实现中
- **子项目**：功能完整，已成功转换 4 个模块并通过测试
- **最后更新**：2025-01-20

**Star ⭐ 本项目以支持我们的工作！**

---

<p align="center">
  <strong>从"能用"到"好用"的 C→Rust 智能迁移助理</strong><br>
  <em>Hybrid Intelligence for Safe Code Migration</em>
</p>

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
