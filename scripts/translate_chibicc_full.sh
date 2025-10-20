#!/bin/bash

# chibicc 完整项目翻译脚本
# 使用 translate_hybrid 子项目和大模型（1049K 上下文）

set -e

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

print_header() {
    echo ""
    echo -e "${CYAN}================================${NC}"
    echo -e "${CYAN}$1${NC}"
    echo -e "${CYAN}================================${NC}"
    echo ""
}

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# 检查配置
check_config() {
    print_header "检查 LLM 配置"
    
    CONFIG_FILE="/workspace/translate_hybrid/config/hybrid_config.toml"
    
    if [ ! -f "$CONFIG_FILE" ]; then
        print_error "配置文件不存在"
        print_info "请先运行: cp config/hybrid_config.toml.example $CONFIG_FILE"
        return 1
    fi
    
    if grep -q "your-api-key-here" "$CONFIG_FILE"; then
        print_error "API Key 未设置"
        print_info "请编辑配置文件: nano $CONFIG_FILE"
        return 1
    fi
    
    # 显示配置信息
    API_KEY=$(grep "api_key" "$CONFIG_FILE" | cut -d'"' -f2 | head -c 20)
    MODEL=$(grep "model" "$CONFIG_FILE" | cut -d'"' -f2)
    BASE_URL=$(grep "base_url" "$CONFIG_FILE" | cut -d'"' -f2)
    
    print_success "配置已就绪"
    print_info "API Endpoint: $BASE_URL"
    print_info "Model: $MODEL"
    print_info "API Key: ${API_KEY}..."
    echo ""
    
    return 0
}

# 构建项目
build_projects() {
    print_header "构建项目"
    
    # 构建 translate_hybrid
    print_info "构建 translate_hybrid 子项目..."
    cd /workspace/translate_hybrid
    
    if cargo build --release 2>&1 | tail -20; then
        print_success "translate_hybrid 构建成功"
    else
        print_error "translate_hybrid 构建失败"
        return 1
    fi
    
    echo ""
    return 0
}

# 测试 LLM 连接
test_llm() {
    print_header "测试 LLM 连接"
    
    cd /workspace/translate_hybrid
    
    print_info "发送测试请求..."
    
    if timeout 30 cargo run --release -- test-llm --prompt "Hello, test connection" 2>&1 | tail -20; then
        print_success "LLM 连接正常"
        return 0
    else
        print_error "LLM 连接失败"
        return 1
    fi
}

# 翻译单个 C 文件
translate_file() {
    local C_FILE=$1
    local OUTPUT_FILE=$2
    local FILE_NAME=$(basename "$C_FILE")
    
    print_info "翻译 $FILE_NAME..."
    
    # 读取 C 代码
    if [ ! -f "$C_FILE" ]; then
        print_error "文件不存在: $C_FILE"
        return 1
    fi
    
    local LINES=$(wc -l < "$C_FILE")
    local SIZE=$(du -h "$C_FILE" | cut -f1)
    print_info "  文件大小: $SIZE ($LINES 行)"
    
    # 读取头文件
    local HEADER_FILE="/workspace/translate_chibicc/src/chibicc.h"
    local CONTEXT=""
    
    if [ -f "$HEADER_FILE" ]; then
        CONTEXT="相关头文件定义 (chibicc.h - 前200行):\n\`\`\`c\n$(head -200 "$HEADER_FILE")\n\`\`\`\n\n"
    fi
    
    # 构建 Prompt
    local PROMPT_FILE="/tmp/translate_prompt_${FILE_NAME}.txt"
    cat > "$PROMPT_FILE" << EOF
你是一个精通 C 和 Rust 的系统编程专家。请将以下 C 代码翻译成地道的 Rust 代码。

要求:
1. 保持功能完全等价
2. 使用 Rust 的惯用法（idiomatic Rust）
3. 最小化 unsafe 使用（目标 <5%）
4. 为复杂逻辑添加注释
5. 使用合适的 Rust 类型（Option, Result, Vec, Box, &str 等）
6. 错误处理使用 Result 而不是返回错误码
7. 内存管理使用 Rust 的所有权系统

${CONTEXT}

C 代码文件 ($FILE_NAME):
\`\`\`c
$(cat "$C_FILE")
\`\`\`

请直接返回完整的 Rust 代码，用 \`\`\`rust 和 \`\`\` 包裹。
EOF
    
    # 调用 LLM
    cd /workspace/translate_hybrid
    
    print_info "  调用 LLM API（流式输出）..."
    local START_TIME=$(date +%s)
    
    local RESPONSE_FILE="/tmp/response_${FILE_NAME}.txt"
    if cargo run --release -- test-llm --prompt "$(cat "$PROMPT_FILE")" > "$RESPONSE_FILE" 2>&1; then
        local END_TIME=$(date +%s)
        local DURATION=$((END_TIME - START_TIME))
        print_info "  翻译耗时: ${DURATION}秒"
    else
        print_error "  LLM 调用失败"
        return 1
    fi
    
    # 提取 Rust 代码
    print_info "  提取 Rust 代码..."
    
    # 尝试提取 ```rust 代码块
    if grep -q '```rust' "$RESPONSE_FILE"; then
        sed -n '/```rust/,/```/p' "$RESPONSE_FILE" | sed '1d;$d' > "$OUTPUT_FILE"
    # 尝试提取 ``` 代码块
    elif grep -q '```' "$RESPONSE_FILE"; then
        sed -n '/```/,/```/p' "$RESPONSE_FILE" | sed '1d;$d' > "$OUTPUT_FILE"
    else
        print_warning "  未找到代码块标记，使用完整响应"
        cat "$RESPONSE_FILE" > "$OUTPUT_FILE"
    fi
    
    if [ ! -s "$OUTPUT_FILE" ]; then
        print_error "  未能提取 Rust 代码"
        print_info "  响应内容:"
        head -50 "$RESPONSE_FILE"
        return 1
    fi
    
    local RUST_LINES=$(wc -l < "$OUTPUT_FILE")
    print_success "  生成 Rust 代码: $RUST_LINES 行"
    
    # 统计 unsafe
    local UNSAFE_COUNT=$(grep -c "unsafe" "$OUTPUT_FILE" || echo 0)
    local UNSAFE_RATIO=$(echo "scale=2; $UNSAFE_COUNT * 100 / $RUST_LINES" | bc -l 2>/dev/null || echo "0")
    print_info "  unsafe 关键字出现: ${UNSAFE_COUNT} 次 (${UNSAFE_RATIO}%)"
    
    return 0
}

# 编译测试
compile_test() {
    local RUST_FILE=$1
    local FILE_NAME=$(basename "$RUST_FILE")
    
    print_info "编译测试 $FILE_NAME..."
    
    # 创建临时测试文件
    local TEST_DIR="/tmp/rust_test_$(date +%s)"
    mkdir -p "$TEST_DIR"
    
    cat > "$TEST_DIR/lib.rs" << 'RUST_EOF'
#![allow(unused)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

RUST_EOF
    
    cat "$RUST_FILE" >> "$TEST_DIR/lib.rs"
    
    # 尝试编译
    local COMPILE_OUTPUT="/tmp/compile_${FILE_NAME}.log"
    if rustc --crate-type lib "$TEST_DIR/lib.rs" -o "$TEST_DIR/lib.rlib" 2>&1 | tee "$COMPILE_OUTPUT" | head -50; then
        print_success "  ✓ 编译成功"
        rm -rf "$TEST_DIR"
        return 0
    else
        print_error "  ✗ 编译失败"
        
        # 显示错误统计
        local ERROR_COUNT=$(grep -c "error\[" "$COMPILE_OUTPUT" || echo 0)
        local WARNING_COUNT=$(grep -c "warning:" "$COMPILE_OUTPUT" || echo 0)
        print_info "  错误数: $ERROR_COUNT"
        print_info "  警告数: $WARNING_COUNT"
        
        # 保存错误信息
        local ERROR_FILE="${RUST_FILE}.errors"
        cp "$COMPILE_OUTPUT" "$ERROR_FILE"
        print_info "  错误信息已保存至: $ERROR_FILE"
        
        rm -rf "$TEST_DIR"
        return 1
    fi
}

# 主流程
main() {
    print_header "chibicc 项目完整翻译测试"
    
    # 步骤 1: 检查配置
    if ! check_config; then
        print_error "配置检查失败，退出"
        exit 1
    fi
    
    # 步骤 2: 构建项目
    if ! build_projects; then
        print_error "项目构建失败，退出"
        exit 1
    fi
    
    # 步骤 3: 测试 LLM
    if ! test_llm; then
        print_error "LLM 测试失败，退出"
        exit 1
    fi
    
    # 步骤 4: 准备输出目录
    print_header "准备翻译"
    
    OUTPUT_DIR="/workspace/rust_output"
    mkdir -p "$OUTPUT_DIR"
    print_info "输出目录: $OUTPUT_DIR"
    
    # chibicc 源文件列表（按复杂度排序）
    declare -a C_FILES=(
        "unicode.c"      # 最简单
        "strings.c"      # 字符串处理
        "hashmap.c"      # 数据结构
        "tokenize.c"     # 词法分析
        "type.c"         # 类型系统
        "preprocess.c"   # 预处理器
        "parse.c"        # 语法分析
        "codegen.c"      # 代码生成
        "main.c"         # 主程序
    )
    
    print_info "计划翻译文件数: ${#C_FILES[@]}"
    echo ""
    
    # 步骤 5: 翻译每个文件
    print_header "开始翻译"
    
    local SUCCESS_COUNT=0
    local FAIL_COUNT=0
    local COMPILE_SUCCESS=0
    local COMPILE_FAIL=0
    
    for C_FILE in "${C_FILES[@]}"; do
        local C_PATH="/workspace/translate_chibicc/src/$C_FILE"
        local RS_FILE="${C_FILE%.c}.rs"
        local RS_PATH="$OUTPUT_DIR/$RS_FILE"
        
        echo ""
        print_info "===== 处理 $C_FILE ====="
        
        # 翻译
        if translate_file "$C_PATH" "$RS_PATH"; then
            SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
            
            # 编译测试
            if compile_test "$RS_PATH"; then
                COMPILE_SUCCESS=$((COMPILE_SUCCESS + 1))
            else
                COMPILE_FAIL=$((COMPILE_FAIL + 1))
            fi
        else
            FAIL_COUNT=$((FAIL_COUNT + 1))
            print_error "跳过编译测试"
        fi
        
        # 等待避免 API 限流
        if [ $SUCCESS_COUNT -lt ${#C_FILES[@]} ]; then
            print_info "等待 3 秒..."
            sleep 3
        fi
    done
    
    # 步骤 6: 生成报告
    print_header "翻译完成 - 生成报告"
    
    local REPORT_FILE="/workspace/chibicc_translation_report.txt"
    
    cat > "$REPORT_FILE" << EOF
================================
chibicc 翻译测试报告
================================
测试时间: $(date)

项目信息:
- 源文件数: ${#C_FILES[@]}
- 翻译成功: $SUCCESS_COUNT
- 翻译失败: $FAIL_COUNT
- 编译通过: $COMPILE_SUCCESS
- 编译失败: $COMPILE_FAIL

成功率:
- 翻译成功率: $(echo "scale=2; $SUCCESS_COUNT * 100 / ${#C_FILES[@]}" | bc)%
- 编译通过率: $(echo "scale=2; $COMPILE_SUCCESS * 100 / $SUCCESS_COUNT" | bc 2>/dev/null || echo 0)%

输出目录: $OUTPUT_DIR

翻译的文件:
EOF
    
    for C_FILE in "${C_FILES[@]}"; do
        local RS_FILE="${C_FILE%.c}.rs"
        local RS_PATH="$OUTPUT_DIR/$RS_FILE"
        
        if [ -f "$RS_PATH" ]; then
            local LINES=$(wc -l < "$RS_PATH")
            local SIZE=$(du -h "$RS_PATH" | cut -f1)
            local STATUS="✓"
            
            if [ -f "${RS_PATH}.errors" ]; then
                STATUS="✗ (有编译错误)"
            fi
            
            echo "  $STATUS $RS_FILE ($LINES 行, $SIZE)" >> "$REPORT_FILE"
        else
            echo "  ✗ $RS_FILE (翻译失败)" >> "$REPORT_FILE"
        fi
    done
    
    cat >> "$REPORT_FILE" << EOF

详细信息:
- 翻译后的 Rust 代码: $OUTPUT_DIR/*.rs
- 编译错误日志: $OUTPUT_DIR/*.rs.errors (如有)

下一步建议:
1. 查看编译错误: cat $OUTPUT_DIR/*.rs.errors
2. 手动修复或使用 LLM 迭代修复
3. 创建完整的 Cargo 项目
4. 添加测试用例
EOF
    
    # 显示报告
    cat "$REPORT_FILE"
    
    print_success "报告已保存至: $REPORT_FILE"
    echo ""
    
    # 总结
    print_header "测试总结"
    
    if [ $COMPILE_SUCCESS -eq ${#C_FILES[@]} ]; then
        print_success "🎉 所有文件翻译并编译成功！"
    elif [ $COMPILE_SUCCESS -gt 0 ]; then
        print_warning "部分文件编译成功 ($COMPILE_SUCCESS/${#C_FILES[@]})"
        print_info "查看错误: cat $OUTPUT_DIR/*.rs.errors"
    else
        print_error "所有文件编译失败"
        print_info "需要手动调试或迭代修复"
    fi
    
    echo ""
    print_info "查看生成的 Rust 代码:"
    print_info "  cd $OUTPUT_DIR && ls -lh"
    echo ""
}

# 运行主流程
main
