#!/bin/bash
# 一键设置脚本执行权限

echo "设置脚本文件执行权限..."

chmod +x scripts/docker_run.sh
chmod +x scripts/test_translation.sh
chmod +x scripts/translate_single_file.sh

echo "✓ 权限设置完成"
echo ""
echo "现在可以运行："
echo "  bash scripts/docker_run.sh"
