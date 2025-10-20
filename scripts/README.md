# 📁 Scripts 目录说明

## 🎯 保留的脚本

### 核心运行脚本

#### `docker_run.ps1` (Windows)
- **用途**: Windows 环境下启动 Docker 容器
- **功能**:
  - 启动 c2rust-translate Docker 环境
  - 自动运行测试
  - 支持完整翻译模式
- **使用**: `.\scripts\docker_run.ps1`
- **状态**: ✅ 活跃使用

#### `docker_run.sh` (Linux/Mac)
- **用途**: Linux/Mac 环境下启动 Docker 容器
- **功能**: 与 docker_run.ps1 功能相同
- **使用**: `./scripts/docker_run.sh`
- **状态**: ✅ 活跃使用

#### `setup_permissions.sh`
- **用途**: 设置脚本执行权限
- **功能**: 批量添加可执行权限
- **使用**: `./scripts/setup_permissions.sh`
- **状态**: ✅ 工具脚本

---

### 可选转换工具

#### `c_to_rust.py` (184 行)
- **用途**: Python 实现的 C→Rust 转换器
- **功能**:
  - 使用正则表达式解析 C 代码
  - 生成基础 Rust 代码框架
  - 规则驱动的转换（无需 LLM）
- **使用**: `python3 scripts/c_to_rust.py <input.c> <output.rs>`
- **状态**: ⚠️ 实验性工具，备用方案

#### `translate_intelligent.sh` (485 行)
- **用途**: 智能 C→Rust 转换脚本
- **功能**:
  - 读取 C 代码结构
  - 生成合理的 Rust 框架
  - 支持增量转换
- **使用**: `./scripts/translate_intelligent.sh`
- **状态**: ⚠️ 高级转换工具，可选

#### `translate_single_file.sh` (150 行)
- **用途**: 单文件翻译脚本
- **功能**:
  - 利用 LLM 大上下文 (1049K)
  - 一次性翻译单个 C 文件
- **使用**: `./scripts/translate_single_file.sh <input.c> [output.rs]`
- **状态**: ⚠️ LLM 增强工具，可选

---

## 🗑️ 已清理的临时脚本

以下脚本为一次性任务或已完成的工作，已删除：

- ❌ `add_hashmap.sh` - 添加 hashmap 模块（已完成）
- ❌ `add_strings.sh` - 添加 strings 模块（已完成）
- ❌ `generate_types.sh` - 生成 types.rs（已完成）
- ❌ `generate_final_report.sh` - 生成最终报告（一次性）
- ❌ `test_simple_translation.sh` - 简单翻译测试（临时）
- ❌ `test_translation.sh` - 翻译测试（临时）
- ❌ `translate_chibicc_full.sh` - chibicc 完整翻译（已完成）
- ❌ `translate_complete.sh` - 完整翻译脚本（旧版）
- ❌ `translate_with_python.sh` - Python 辅助翻译（实验性）

**清理效果**: 从 15 个脚本减少到 6 个 (-60%)

---

## 📋 使用指南

### 快速开始

```bash
# Windows 用户
.\scripts\docker_run.ps1

# Linux/Mac 用户
./scripts/setup_permissions.sh  # 首次运行
./scripts/docker_run.sh
```

### 高级用途

```bash
# 使用 Python 转换工具
python3 scripts/c_to_rust.py input.c output.rs

# 智能转换（Docker 内）
docker exec -it c2rust-translate /workspace/scripts/translate_intelligent.sh

# 单文件 LLM 翻译（需配置 API Key）
./scripts/translate_single_file.sh tokenize.c tokenize.rs
```

---

## 🔧 维护建议

### 何时删除可选脚本

如果满足以下条件，可以删除可选脚本：
1. ✅ 已完成所有 chibicc 模块转换
2. ✅ 不再需要实验性转换工具
3. ✅ 只使用主程序 C2RustAgent

删除命令：
```bash
cd scripts
rm c_to_rust.py translate_intelligent.sh translate_single_file.sh
```

### 何时保留可选脚本

建议保留的情况：
1. 🔄 未来可能转换其他 C 项目
2. 🧪 需要对比不同转换方法的效果
3. 🛠️ 作为 C2RustAgent 的补充工具

---

## 📊 脚本统计

| 类别 | 数量 | 总代码行数 |
|------|------|------------|
| 核心脚本 | 3 | ~350 行 |
| 可选工具 | 3 | ~819 行 |
| **总计** | **6** | **~1169 行** |

**已删除**: 9 个临时脚本

---

## 💡 推荐配置

**最小化配置** (仅核心功能):
```
scripts/
├── docker_run.ps1          ✅ 必需
├── docker_run.sh           ✅ 必需
└── setup_permissions.sh    ✅ 必需
```

**完整配置** (包含可选工具):
```
scripts/
├── docker_run.ps1          ✅ 必需
├── docker_run.sh           ✅ 必需
├── setup_permissions.sh    ✅ 必需
├── c_to_rust.py            ⚠️ 可选
├── translate_intelligent.sh ⚠️ 可选
└── translate_single_file.sh ⚠️ 可选
```

---

**更新时间**: 2025-01-20  
**维护者**: C2RustAgent 项目组
