#!/bin/bash
# 使用 Python 脚本进行更智能的 C 到 Rust 转换

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo_info() { echo -e "${BLUE}ℹ${NC} $1"; }
echo_success() { echo -e "${GREEN}✓${NC} $1"; }
echo_error() { echo -e "${RED}✗${NC} $1"; }
echo_warning() { echo -e "${YELLOW}⚠${NC} $1"; }

# 配置
C_PROJECT_DIR="/workspace/translate_chibicc/src"
OUTPUT_DIR="/workspace/rust_output_v2"
REPORT_FILE="/workspace/translation_report_v2.txt"
PYTHON_SCRIPT="/workspace/scripts/c_to_rust.py"

mkdir -p "$OUTPUT_DIR"

echo "==================================="
echo "chibicc C 到 Rust 转换（Python版）"
echo "==================================="
echo ""

# 检查 Python
if ! command -v python3 &> /dev/null; then
    echo_error "Python3 未安装"
    exit 1
fi

# 要转换的文件
declare -a C_FILES=(
    "unicode.c"
    "strings.c"
    "hashmap.c"
    "tokenize.c"
    "type.c"
)

TOTAL=0
SUCCESS=0
FAILED=0

# 开始报告
cat > "$REPORT_FILE" << EOF
================================
chibicc 转换报告（Python版）
================================
时间: $(date)

EOF

for C_FILE in "${C_FILES[@]}"; do
    TOTAL=$((TOTAL + 1))
    
    C_PATH="$C_PROJECT_DIR/$C_FILE"
    BASE_NAME=$(basename "$C_FILE" .c)
    RS_PATH="$OUTPUT_DIR/${BASE_NAME}.rs"
    
    echo "========================================" 
    echo "[$TOTAL/${#C_FILES[@]}] $C_FILE"
    echo "========================================"
    
    if [ ! -f "$C_PATH" ]; then
        echo_error "文件不存在: $C_PATH"
        FAILED=$((FAILED + 1))
        continue
    fi
    
    # Python 转换
    echo_info "使用 Python 转换..."
    if python3 "$PYTHON_SCRIPT" "$C_PATH" "$RS_PATH"; then
        echo_success "生成 Rust 文件"
    else
        echo_error "转换失败"
        FAILED=$((FAILED + 1))
        echo "  ✗ ${BASE_NAME}.rs (转换失败)" >> "$REPORT_FILE"
        continue
    fi
    
    # 编译测试
    echo_info "编译测试..."
    ERROR_FILE="${RS_PATH}.errors"
    
    if rustc --crate-type lib --edition 2021 \
        --allow warnings \
        "$RS_PATH" \
        -o "/tmp/${BASE_NAME}.rlib" 2> "$ERROR_FILE"; then
        echo_success "编译通过"
        SUCCESS=$((SUCCESS + 1))
        rm -f "$ERROR_FILE"
        echo "  ✓ ${BASE_NAME}.rs" >> "$REPORT_FILE"
    else
        echo_error "编译失败"
        FAILED=$((FAILED + 1))
        echo_warning "  错误日志: $ERROR_FILE"
        echo "  ✗ ${BASE_NAME}.rs (编译错误)" >> "$REPORT_FILE"
        
        if [ -f "$ERROR_FILE" ]; then
            echo "  前 3 条错误:"
            head -3 "$ERROR_FILE" | sed 's/^/    /'
        fi
    fi
    
    echo ""
done

# 最终报告
cat >> "$REPORT_FILE" << EOF

================================
统计
================================
总数: $TOTAL
成功: $SUCCESS
失败: $FAILED
成功率: $(( SUCCESS * 100 / TOTAL ))%

输出: $OUTPUT_DIR
EOF

echo "==================================="
echo "转换完成！"
echo "==================================="
echo "总数: $TOTAL"
echo "成功: $SUCCESS"
echo "失败: $FAILED"
echo "成功率: $(( SUCCESS * 100 / TOTAL ))%"
echo ""
echo "报告: $REPORT_FILE"
echo "输出: $OUTPUT_DIR"

if [ $FAILED -eq 0 ]; then
    echo_success "所有文件编译通过！"
    exit 0
else
    echo_warning "$FAILED 个文件需要修复"
    exit 1
fi
