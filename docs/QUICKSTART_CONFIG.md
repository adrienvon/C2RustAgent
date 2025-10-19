# 快速开始指南 - LLM 配置

本指南帮助您快速配置 C2RustAgent 的 LLM 功能。

## 选择配置方法

根据您的使用场景选择合适的配置方法：

| 场景 | 推荐方法 | 说明 |
|------|---------|------|
| 个人开发 | 用户配置文件 | 一次配置，全局生效 |
| 团队协作 | 项目配置文件 | 团队共享设置，个人 Key 用环境变量 |
| CI/CD | 环境变量 | 无需配置文件，安全性高 |
| 测试开发 | Mock 模式 | 无需 API Key |

## 方法一：用户配置文件（推荐）

**适用场景**: 个人开发，希望一次配置后在所有项目中使用。

### 步骤

1. **初始化配置文件**
   ```bash
   cargo run --bin c2rust-agent-config -- init
   ```

2. **编辑配置文件**
   
   配置文件位置：
   - **Windows**: `%APPDATA%\c2rust-agent\config.toml`
   - **Linux**: `~/.config/c2rust-agent/config.toml`
   - **macOS**: `~/Library/Application Support/c2rust-agent/config.toml`
   
   使用您喜欢的编辑器打开：
   ```bash
   # Windows
   notepad %APPDATA%\c2rust-agent\config.toml
   
   # Linux/macOS
   nano ~/.config/c2rust-agent/config.toml
   ```

3. **设置 API Key**
   
   修改 `api_key` 行：
   ```toml
   api_key = "sk-your-actual-api-key-here"
   ```
   
   > 💡 获取 API Key：访问 https://platform.openai.com/api-keys

4. **（可选）调整其他参数**
   ```toml
   model = "gpt-4o-mini"    # 使用的模型
   temperature = 0.3         # 输出随机性（0.0-2.0）
   max_tokens = 1000         # 最大生成长度
   ```

5. **验证配置**
   ```bash
   cargo run --bin c2rust-agent-config -- validate
   ```
   
   应该看到 "✅ 配置验证通过！"

6. **开始使用**
   ```bash
   cargo run  # 运行主程序
   ```

## 方法二：环境变量

**适用场景**: CI/CD、临时使用、不想创建配置文件。

### 设置环境变量

```bash
# Linux/macOS
export OPENAI_API_KEY=sk-your-api-key-here

# Windows PowerShell
$env:OPENAI_API_KEY="sk-your-api-key-here"

# Windows CMD
set OPENAI_API_KEY=sk-your-api-key-here
```

### 持久化（可选）

**Linux/macOS** - 添加到 `~/.bashrc` 或 `~/.zshrc`：
```bash
echo 'export OPENAI_API_KEY=sk-your-api-key' >> ~/.bashrc
source ~/.bashrc
```

**Windows PowerShell** - 添加到用户环境变量：
```powershell
[System.Environment]::SetEnvironmentVariable('OPENAI_API_KEY', 'sk-your-key', 'User')
```

### 验证
```bash
cargo run --bin c2rust-agent-config -- show
```

## 方法三：项目配置文件

**适用场景**: 团队协作，不同项目使用不同配置。

### 步骤

1. **创建项目配置**
   ```bash
   cargo run --bin c2rust-agent-config -- init-project
   ```
   
   这会在当前目录创建 `c2rust-agent.toml`。

2. **配置团队共享参数**
   
   编辑 `c2rust-agent.toml`，设置团队共享的参数（不包含 API Key）：
   ```toml
   provider = "openai"
   model = "gpt-4o-mini"
   temperature = 0.3
   max_tokens = 1000
   # 不要在这里设置 api_key！
   ```

3. **添加到 .gitignore**
   
   如果配置文件包含敏感信息（如 API Key），添加到 `.gitignore`：
   ```bash
   echo 'c2rust-agent.toml' >> .gitignore
   ```

4. **个人 API Key 设置**
   
   每个团队成员使用环境变量或用户配置文件设置个人 API Key：
   ```bash
   export OPENAI_API_KEY=sk-personal-key
   ```

## 方法四：Mock 模式（测试）

**适用场景**: 开发测试，不想消耗 API 配额。

### 临时 Mock 模式

```bash
# Linux/macOS
export USE_MOCK_LLM=true

# Windows PowerShell
$env:USE_MOCK_LLM="true"
```

### 配置文件 Mock 模式

在任何配置文件中添加：
```toml
use_mock = true
```

### 运行测试
```bash
cargo test --lib llm_assists
```

## 高级配置

### 使用代理

如果需要通过代理访问 OpenAI API：

```toml
api_url = "https://your-proxy-domain.com/v1"
api_key = "sk-your-key"
```

### Azure OpenAI

```toml
provider = "openai"  # 或 "azure"
api_url = "https://your-endpoint.openai.azure.com"
api_key = "your-azure-key"
model = "your-deployment-name"
```

### 本地模型（LocalAI, Ollama）

```toml
api_url = "http://localhost:8080/v1"
model = "llama3"
use_mock = false  # 使用实际 API
# api_key 可能不需要，取决于本地服务配置
```

## 配置优先级

配置按以下优先级合并（高优先级覆盖低优先级）：

```
1. 环境变量（最高优先级）
   ↓
2. 用户配置文件 (~/.config/c2rust-agent/config.toml)
   ↓
3. 项目配置文件 (./c2rust-agent.toml)
   ↓
4. 默认值（最低优先级）
```

### 查看当前配置

```bash
# 查看生效的配置
cargo run --bin c2rust-agent-config -- show

# 查看详细信息（包括配置来源）
cargo run --bin c2rust-agent-config -- show --verbose
```

## 常见问题

### Q: 如何获取 OpenAI API Key？

访问 https://platform.openai.com/api-keys 创建新的 API Key。

### Q: 忘记了配置文件位置？

运行：
```bash
cargo run --bin c2rust-agent-config -- path
```

### Q: 配置验证失败？

常见原因：
1. API Key 未设置或格式错误
2. 网络问题（无法连接 OpenAI）
3. API Key 额度不足

解决方案：
```bash
# 查看详细错误信息
cargo run --bin c2rust-agent-config -- validate

# 检查当前配置
cargo run --bin c2rust-agent-config -- show --verbose
```

### Q: 如何在 CI/CD 中使用？

在 CI/CD 环境中设置环境变量：

**GitHub Actions**:
```yaml
env:
  OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
```

**GitLab CI**:
```yaml
variables:
  OPENAI_API_KEY: $OPENAI_API_KEY  # 在 Settings > CI/CD > Variables 中设置
```

### Q: 多个项目如何共享配置？

使用用户配置文件（方法一），所有项目自动共享。

需要项目特定配置时，使用项目配置文件覆盖部分参数。

### Q: 不小心提交了 API Key 到 Git 怎么办？

1. 立即在 OpenAI 控制台吊销该 Key
2. 生成新的 API Key
3. 从 Git 历史中移除敏感信息：
   ```bash
   git filter-branch --force --index-filter \
     'git rm --cached --ignore-unmatch c2rust-agent.toml' \
     --prune-empty --tag-name-filter cat -- --all
   ```

## 成本估算

使用 GPT-4o-mini 的典型成本（2024年1月价格）：

| 操作 | Token 消耗 | 成本 |
|------|-----------|------|
| 函数语义推断 | ~500 tokens | $0.0001 |
| 模块文档生成 | ~1000 tokens | $0.0002 |
| Unsafe 块说明 | ~800 tokens | $0.00016 |

**估算**: 转换 1000 行 C 代码约消耗 $0.1 - $0.5。

详细成本分析见 [docs/openai_api_integration.md](../docs/openai_api_integration.md)。

## 下一步

- 📖 阅读完整文档：[docs/openai_api_integration.md](../docs/openai_api_integration.md)
- 🧪 运行测试：`cargo test`
- 🚀 开始转换：`cargo run`

---

**需要帮助？** 查看 [GitHub Issues](https://github.com/your-repo/issues) 或阅读完整文档。
