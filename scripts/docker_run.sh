#!/bin/bash

# Docker 快速启动脚本
# Windows PowerShell 使用: pwsh docker_run.sh

echo "================================"
echo "C2RustAgent Docker 测试环境"
echo "================================"
echo ""

# 检查 Docker 是否安装
if ! command -v docker &> /dev/null; then
    echo "错误: Docker 未安装或未运行"
    echo "请先安装 Docker Desktop: https://www.docker.com/products/docker-desktop"
    exit 1
fi

# 设置变量
IMAGE_NAME="c2rust-agent-translate"
CONTAINER_NAME="c2rust-test"

# 构建 Docker 镜像
echo "步骤 1: 构建 Docker 镜像..."
echo "这可能需要几分钟时间（仅首次运行）..."
echo ""

docker build -t $IMAGE_NAME -f Dockerfile.translate . || {
    echo "错误: Docker 镜像构建失败"
    exit 1
}

echo ""
echo "✓ Docker 镜像构建成功"
echo ""

# 停止并删除已存在的容器
docker rm -f $CONTAINER_NAME 2>/dev/null || true

# 运行容器
echo "步骤 2: 启动 Docker 容器..."
echo ""

docker run -it --name $CONTAINER_NAME \
    -v "$(pwd)":/workspace \
    -w /workspace \
    $IMAGE_NAME \
    /bin/bash -c "chmod +x /workspace/scripts/*.sh && /workspace/scripts/test_translation.sh && /bin/bash"

echo ""
echo "容器已退出"
