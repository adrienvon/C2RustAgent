# 翻译整个 chibicc 项目的 PowerShell 脚本

Write-Host "🚀 开始翻译 chibicc 项目..." -ForegroundColor Cyan
Write-Host ""

# 设置路径
$PROJECT_DIR = "..\translate_chibicc\src"
$OUTPUT_DIR = "..\rust_output_chibicc"

# 检查配置文件
if (-not (Test-Path "config\hybrid_config.toml")) {
    Write-Host "❌ 配置文件不存在，请先运行: cargo run -- init" -ForegroundColor Red
    exit 1
}

# 检查项目目录
if (-not (Test-Path $PROJECT_DIR)) {
    Write-Host "❌ 找不到 chibicc 项目目录: $PROJECT_DIR" -ForegroundColor Red
    exit 1
}

Write-Host "📁 项目目录: $PROJECT_DIR" -ForegroundColor Green
Write-Host "📁 输出目录: $OUTPUT_DIR" -ForegroundColor Green
Write-Host ""

# 执行翻译
cargo run --release -- translate-project `
    --project-dir $PROJECT_DIR `
    --output-dir $OUTPUT_DIR `
    --pattern "*.c" `
    --jobs 1

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "✅ 翻译完成！" -ForegroundColor Green
    Write-Host ""
    Write-Host "下一步:" -ForegroundColor Yellow
    Write-Host "  cd $OUTPUT_DIR"
    Write-Host "  cargo check"
    Write-Host "  cargo build"
} else {
    Write-Host ""
    Write-Host "❌ 翻译失败，请检查错误信息" -ForegroundColor Red
}
