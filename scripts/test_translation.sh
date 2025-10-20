#!/bin/bash

# chibicc 项目翻译测试脚本
# 使用大模型（1049K 上下文）进行 C 到 Rust 的完整翻译

set -e  # 遇到错误立即退出

echo "================================"
echo "chibicc 翻译测试脚本"
echo "================================"
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# 步骤 1: 检查环境
print_info "检查环境..."
rustc --version || { print_error "Rust 未安装"; exit 1; }
cargo --version || { print_error "Cargo 未安装"; exit 1; }
clang --version || { print_error "Clang 未安装"; exit 1; }
print_success "环境检查通过"
echo ""

# 步骤 2: 进入 chibicc 目录
print_info "进入 translate_chibicc 目录..."
cd /workspace/translate_chibicc
print_success "当前目录: $(pwd)"
echo ""

# 步骤 3: 生成 compile_commands.json
print_info "生成 compile_commands.json..."
if [ ! -f "compile_commands.json" ]; then
    print_info "运行 make clean..."
    make clean 2>/dev/null || true
    
    print_info "使用 bear 生成编译数据库..."
    bear -- make 2>&1 | head -20
    
    if [ -f "compile_commands.json" ]; then
        print_success "compile_commands.json 生成成功"
        print_info "编译单元数量: $(cat compile_commands.json | grep -o '"file"' | wc -l)"
    else
        print_error "compile_commands.json 生成失败"
        exit 1
    fi
else
    print_success "compile_commands.json 已存在"
fi
echo ""

# 步骤 4: 分析 C 源代码
print_info "分析 C 源代码..."
C_FILES=$(find src -name "*.c" | wc -l)
H_FILES=$(find src -name "*.h" | wc -l)
TOTAL_LINES=$(find src -name "*.c" -o -name "*.h" | xargs wc -l | tail -1 | awk '{print $1}')

print_info "C 源文件数量: $C_FILES"
print_info "头文件数量: $H_FILES"
print_info "总代码行数: $TOTAL_LINES"
echo ""

# 步骤 5: 检查配置文件
print_info "检查 LLM 配置..."
CONFIG_FILE="/workspace/translate_hybrid/config/hybrid_config.toml"

if [ ! -f "$CONFIG_FILE" ]; then
    print_warning "配置文件不存在，创建默认配置..."
    mkdir -p /workspace/translate_hybrid/config
    cp /workspace/translate_hybrid/config/hybrid_config.toml.example "$CONFIG_FILE" 2>/dev/null || {
        print_error "无法创建配置文件"
        exit 1
    }
fi

# 检查 API Key 是否设置
if grep -q "your-api-key-here" "$CONFIG_FILE" 2>/dev/null; then
    print_warning "API Key 未设置，请先配置 $CONFIG_FILE"
    print_info "跳过 LLM 翻译，仅测试 AST 解析..."
    USE_LLM=false
else
    print_success "LLM 配置已设置"
    USE_LLM=true
fi
echo ""

# 步骤 6: 构建主项目
print_info "构建 C2RustAgent 主项目..."
cd /workspace
cargo build --release 2>&1 | tail -20

if [ $? -eq 0 ]; then
    print_success "主项目构建成功"
else
    print_error "主项目构建失败"
    exit 1
fi
echo ""

# 步骤 7: 运行 AST 到 MIR 转换（不使用 LLM）
print_info "运行 AST 到 MIR 转换..."
cd /workspace

print_info "开始转换 chibicc 项目..."
OUTPUT=$(timeout 300 cargo run --release -- /workspace/translate_chibicc 2>&1)
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    print_success "AST 到 MIR 转换成功"
    echo "$OUTPUT" | grep -E "已解析|函数数|全局变量数" || true
elif [ $EXIT_CODE -eq 124 ]; then
    print_error "转换超时（5分钟限制）"
else
    print_error "转换失败"
    echo "$OUTPUT" | tail -30
fi
echo ""

# 步骤 8: 使用 translate_hybrid 进行 LLM 辅助翻译（如果配置可用）
if [ "$USE_LLM" = true ]; then
    print_info "构建 translate_hybrid 子项目..."
    cd /workspace/translate_hybrid
    
    cargo build --release 2>&1 | tail -10
    
    if [ $? -eq 0 ]; then
        print_success "translate_hybrid 构建成功"
        
        print_info "测试 LLM 连接..."
        timeout 30 cargo run --release -- test-llm --prompt "Hello" 2>&1 | tail -20
        
        if [ $? -eq 0 ]; then
            print_success "LLM 连接测试通过"
            
            print_info "准备进行完整翻译..."
            print_warning "注意: 翻译大型项目可能需要较长时间和大量 API 调用"
            print_info "项目规模: $TOTAL_LINES 行代码，约 $C_FILES 个函数"
            print_info "建议: 先翻译单个文件进行测试"
        else
            print_error "LLM 连接测试失败"
        fi
    else
        print_error "translate_hybrid 构建失败"
    fi
else
    print_info "跳过 LLM 翻译（未配置 API Key）"
fi
echo ""

# 步骤 9: 生成测试报告
print_info "生成测试报告..."
REPORT_FILE="/workspace/translation_report.txt"

cat > "$REPORT_FILE" << EOF
================================
chibicc 翻译测试报告
================================
测试时间: $(date)

项目信息:
- C 源文件: $C_FILES
- 头文件: $H_FILES  
- 总代码行数: $TOTAL_LINES

环境信息:
- Rust 版本: $(rustc --version)
- Clang 版本: $(clang --version | head -1)

测试结果:
- compile_commands.json: $([ -f /workspace/translate_chibicc/compile_commands.json ] && echo "✓ 已生成" || echo "✗ 未生成")
- AST 解析: $([ $EXIT_CODE -eq 0 ] && echo "✓ 成功" || echo "✗ 失败")
- LLM 配置: $([ "$USE_LLM" = true ] && echo "✓ 已配置" || echo "✗ 未配置")

建议下一步:
1. 配置 LLM API Key (编辑 /workspace/translate_hybrid/config/hybrid_config.toml)
2. 先翻译单个 C 文件进行测试
3. 逐步翻译整个项目
4. 运行编译测试验证生成的 Rust 代码

详细日志请查看上述输出。
EOF

cat "$REPORT_FILE"
print_success "报告已保存至: $REPORT_FILE"
echo ""

# 步骤 10: 提供交互式 shell
print_info "测试完成！"
print_info "你现在可以:"
echo "  1. 编辑配置: vi /workspace/translate_hybrid/config/hybrid_config.toml"
echo "  2. 测试翻译单个文件: cd /workspace/translate_hybrid && cargo run --release -- test-llm"
echo "  3. 查看源代码: cd /workspace/translate_chibicc/src && ls -lh"
echo "  4. 手动运行转换: cd /workspace && cargo run --release -- /workspace/translate_chibicc"
echo ""
print_success "环境已就绪！输入 'exit' 退出容器"
