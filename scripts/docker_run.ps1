# Docker 快速启动脚本 (PowerShell 版本)
# 用于 Windows 环境

param(
    [switch]$FullTranslation,  # 运行完整翻译测试
    [switch]$Help
)

if ($Help) {
    Write-Host @"
用法: .\scripts\docker_run.ps1 [选项]

选项:
  -FullTranslation    运行完整的 chibicc 翻译测试
  -Help              显示此帮助信息

示例:
  .\scripts\docker_run.ps1                  # 基础测试（自动运行）
  .\scripts\docker_run.ps1 -FullTranslation # 完整翻译测试

"@ -ForegroundColor Cyan
    exit 0
}

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

# 选择运行的脚本
$RunScript = if ($FullTranslation) {
    Write-Host "模式: 完整翻译测试" -ForegroundColor Cyan
    Write-Host "警告: 这将翻译所有 9 个 C 文件，可能需要 15-30 分钟" -ForegroundColor Yellow
    Write-Host ""
    "/workspace/scripts/translate_chibicc_full.sh"
} else {
    Write-Host "模式: 基础测试" -ForegroundColor Cyan
    Write-Host "提示: 使用 -FullTranslation 参数运行完整翻译" -ForegroundColor Gray
    Write-Host ""
    "/workspace/scripts/test_translation.sh"
}

docker run -it --name $ContainerName `
    -v "${WorkspaceDir}:/workspace" `
    -w /workspace `
    $ImageName `
    /bin/bash -c "chmod +x /workspace/scripts/*.sh && $RunScript && /bin/bash"

Write-Host ""
Write-Host "容器已退出" -ForegroundColor Gray

# 清理选项
Write-Host ""
$cleanup = Read-Host "是否删除容器? (y/n)"
if ($cleanup -eq 'y') {
    docker rm $ContainerName
    Write-Host "容器已删除" -ForegroundColor Green
}
