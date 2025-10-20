# 📚 C2RustAgent 项目文档索引

> C 到 Rust 自动转换工具 - 基于静态分析和 LLM 增强

---

## 🚀 快速开始

### 新用户
1. 阅读 [README.md](../README.md) - 项目概述和快速上手
2. 阅读 [QUICKSTART.md](quickstart/QUICKSTART.md) - 5分钟快速教程

### 使用 Docker
1. [Docker 快速参考](docker/DOCKER_QUICKREF.md) - 常用命令速查
2. [Docker 完整指南](docker/DOCKER_GUIDE.md) - 详细使用说明
3. [Docker 技术总结](docker/DOCKER_SUMMARY.md) - 架构和技术细节

### chibicc 项目转换
- [chibicc 转换指南](translation/CHIBICC_TRANSLATION.md) - 完整的转换命令参考
- [转换系统说明](translation/TRANSLATION_SYSTEM.md) - 自动化转换系统

---

## 📊 项目报告

### 最新成果（2025-10-20）
- **[转换成功总结](reports/TRANSLATION_SUCCESS_SUMMARY.md)** ⭐ 推荐阅读
  - ✅ 4个模块已转换（unicode, strings, hashmap, types）
  - ✅ 945 行 Rust 代码
  - ✅ 12/12 测试全部通过
  - ✅ 编译成功

- **[详细技术报告](reports/FINAL_TRANSLATION_REPORT.md)**
  - 各模块实现细节
  - 性能对比
  - 技术决策
  - 未来计划

---

## 📖 技术文档

### 核心概念
- [架构设计](../docs/phase*.md) - 项目各阶段设计文档
- [P4提示词](../docs/P4提示词.md) - LLM 提示词设计

### 配置指南
- [配置快速开始](../QUICKSTART_CONFIG.md) - API 配置指南
- [配置文件示例](../c2rust-agent.toml.example) - 配置模板

---

## 🛠️ 开发指南

### 项目结构
```
C2RustAgent/
├── src/                    # 核心代码
│   ├── main.rs            # CLI 入口
│   ├── mir.rs             # MIR 数据结构
│   ├── ast_to_mir.rs      # AST 转换
│   ├── codegen.rs         # 代码生成
│   ├── llm_assists.rs     # LLM 集成
│   └── analysis/          # 静态分析
│
├── translate_hybrid/       # 混合转换子项目
│   ├── src/
│   │   ├── llm_client.rs  # LLM 客户端
│   │   └── utils.rs       # 工具函数
│   └── config/            # 配置文件
│
├── rust_output_final/      # ✅ 转换后的 Rust 代码
│   ├── types.rs           # 公共类型
│   ├── unicode.rs         # Unicode 处理
│   ├── strings.rs         # 字符串工具
│   └── hashmap.rs         # 哈希表实现
│
├── scripts/                # 自动化脚本
│   ├── translate_complete.sh
│   ├── add_strings.sh
│   └── add_hashmap.sh
│
└── docs/                   # 📚 文档目录（你在这里）
    ├── docker/            # Docker 相关文档
    ├── translation/       # 转换指南
    ├── reports/           # 项目报告
    └── INDEX.md           # 本文件
```

---

## 🎯 使用场景

### 场景 1: 快速测试转换
```bash
# 使用 Docker 测试单个文件
.\scripts\docker_run.ps1
```

### 场景 2: 完整项目转换
```bash
# 转换整个 chibicc 项目
.\scripts\docker_run.ps1 -FullTranslation
```

### 场景 3: 开发和调试
```powershell
# 本地开发
cargo build
cargo test
cargo run -- ./translate_chibicc
```

---

## 📈 项目进度

### 已完成 ✅
- [x] 核心类型系统（types.rs）
- [x] Unicode 处理（unicode.rs）
- [x] 字符串工具（strings.rs）
- [x] 哈希表实现（hashmap.rs）
- [x] Docker 测试环境
- [x] 自动化转换脚本
- [x] 完整测试覆盖（12个测试）

### 进行中 🚧
- [ ] tokenize.c 转换（词法分析器）
- [ ] type.c 转换（类型系统）

### 计划中 📋
- [ ] preprocess.c（预处理器）
- [ ] parse.c（语法分析器）
- [ ] codegen.c（代码生成器）
- [ ] main.c（主程序）
- [ ] LLM 增强转换

---

## 🤝 贡献指南

### 报告问题
- 在 GitHub Issues 中提交 bug 报告
- 提供详细的复现步骤

### 提交代码
- Fork 项目
- 创建 feature 分支
- 提交 Pull Request

---

## 📞 联系方式

- **项目**: C2RustAgent
- **仓库**: adrienvon/C2RustAgent
- **分支**: master

---

## 📝 更新日志

### 2025-10-20
- ✅ 完成 4 个核心模块的转换
- ✅ 所有测试通过（12/12）
- ✅ 整理项目文档结构
- ✅ 创建文档索引

---

**最后更新**: 2025-10-20  
**文档版本**: v1.0
