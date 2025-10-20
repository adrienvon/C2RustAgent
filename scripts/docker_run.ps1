# Docker 快速启动脚本 (PowerShell 版本)
# 用于 Windows 环境

Write-Host "================================" -ForegroundColor Cyan
Write-Host "C2RustAgent Docker 测试环境" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# 检查 Docker 是否运行
$dockerRunning = $false
try {
    docker ps | Out-Null
    $dockerRunning = $true
} catch {
    $dockerRunning = $false
}

if (-not $dockerRunning) {
    Write-Host "错误: Docker 未运行" -ForegroundColor Red
    Write-Host "请先启动 Docker Desktop" -ForegroundColor Yellow
    exit 1
}

# 设置变量
$ImageName = "c2rust-agent-translate"
$ContainerName = "c2rust-test"
$WorkspaceDir = Get-Location

Write-Host "当前目录: $WorkspaceDir" -ForegroundColor Gray
Write-Host ""

# 构建 Docker 镜像
Write-Host "步骤 1: 构建 Docker 镜像..." -ForegroundColor Green
Write-Host "这可能需要几分钟时间（仅首次运行）..." -ForegroundColor Yellow
Write-Host ""

docker build -t $ImageName -f Dockerfile.translate .

if ($LASTEXITCODE -ne 0) {
    Write-Host "错误: Docker 镜像构建失败" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✓ Docker 镜像构建成功" -ForegroundColor Green
Write-Host ""

# 停止并删除已存在的容器
docker rm -f $ContainerName 2>$null

# 运行容器
Write-Host "步骤 2: 启动 Docker 容器..." -ForegroundColor Green
Write-Host ""

# 在 Windows 上需要转换路径格式
$UnixPath = $WorkspaceDir.Path -replace '\\', '/' -replace 'C:', '/c'

docker run -it --name $ContainerName `
    -v "${WorkspaceDir}:/workspace" `
    -w /workspace `
    $ImageName `
    /bin/bash -c "chmod +x /workspace/scripts/*.sh && /workspace/scripts/test_translation.sh && /bin/bash"

Write-Host ""
Write-Host "容器已退出" -ForegroundColor Gray

# 清理选项
Write-Host ""
$cleanup = Read-Host "是否删除容器? (y/n)"
if ($cleanup -eq 'y') {
    docker rm $ContainerName
    Write-Host "容器已删除" -ForegroundColor Green
}
