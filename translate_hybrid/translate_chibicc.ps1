# 使用 translate_hybrid 翻译 chibicc 项目的 PowerShell 脚本

param(
    [switch]$SingleFile,
    [string]$File = "",
    [switch]$SkipCheck
)

$ErrorActionPreference = "Stop"

Write-Host "================================" -ForegroundColor Cyan
Write-Host "Chibicc C 到 Rust 翻译器" -ForegroundColor Cyan
Write-Host "使用 translate_hybrid 子项目" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# 检查当前目录
if (-not (Test-Path "config/hybrid_config.toml")) {
    Write-Host "❌ 错误: 请在 translate_hybrid 目录下运行此脚本" -ForegroundColor Red
    Write-Host "   cd translate_hybrid" -ForegroundColor Yellow
    exit 1
}

# 检查配置文件
if (-not (Test-Path "config/hybrid_config.toml")) {
    Write-Host "❌ 错误: 配置文件不存在" -ForegroundColor Red
    Write-Host "   请先运行: cargo run -- init" -ForegroundColor Yellow
    exit 1
}

# 源文件和输出目录
$SrcDir = "..\translate_chibicc\src"
$OutputDir = ".\rust_output_chibicc"

# 创建输出目录
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir | Out-Null
}

# 要翻译的文件列表
$Files = @(
    "unicode.c",
    "strings.c", 
    "hashmap.c",
    "type.c",
    "tokenize.c",
    "parse.c",
    "codegen.c",
    "preprocess.c",
    "main.c"
)

Write-Host "📁 源文件目录: $SrcDir" -ForegroundColor Gray
Write-Host "📁 输出目录: $OutputDir" -ForegroundColor Gray
Write-Host ""

# 单文件模式
if ($SingleFile -and $File) {
    if (-not $Files.Contains($File)) {
        Write-Host "⚠️  警告: $File 不在标准文件列表中" -ForegroundColor Yellow
    }
    $Files = @($File)
    Write-Host "📝 单文件模式: $File" -ForegroundColor Cyan
}
else {
    Write-Host "📝 批量翻译模式: $($Files.Count) 个文件" -ForegroundColor Cyan
}

Write-Host ""

# 翻译统计
$SuccessCount = 0
$FailCount = 0
$StartTime = Get-Date

# 逐个翻译文件
foreach ($CFile in $Files) {
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
    Write-Host "📄 正在翻译: $CFile" -ForegroundColor Cyan
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
    
    $InputFile = Join-Path $SrcDir $CFile
    $OutputFile = Join-Path $OutputDir ($CFile -replace '\.c$', '.rs')
    
    if (-not (Test-Path $InputFile)) {
        Write-Host "⚠️  跳过: 文件不存在 $InputFile" -ForegroundColor Yellow
        $FailCount++
        continue
    }
    
    # 显示文件信息
    $FileInfo = Get-Item $InputFile
    Write-Host "   大小: $([math]::Round($FileInfo.Length / 1KB, 2)) KB" -ForegroundColor Gray
    
    # 调用翻译命令
    Write-Host "🔄 调用 LLM 翻译..." -ForegroundColor Yellow
    
    try {
        cargo run --quiet -- translate `
            --input $InputFile `
            --output $OutputFile `
            2>&1 | ForEach-Object {
            Write-Host $_ -ForegroundColor Gray
        }
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ 翻译完成: $OutputFile" -ForegroundColor Green
            $SuccessCount++
            
            # 显示输出文件信息
            if (Test-Path $OutputFile) {
                $OutInfo = Get-Item $OutputFile
                Write-Host "   输出大小: $([math]::Round($OutInfo.Length / 1KB, 2)) KB" -ForegroundColor Gray
            }
        }
        else {
            Write-Host "❌ 翻译失败: $CFile (错误码: $LASTEXITCODE)" -ForegroundColor Red
            $FailCount++
        }
    }
    catch {
        Write-Host "❌ 翻译失败: $CFile" -ForegroundColor Red
        Write-Host "   错误: $_" -ForegroundColor Red
        $FailCount++
    }
    
    Write-Host ""
}

$EndTime = Get-Date
$Duration = $EndTime - $StartTime

Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Green
Write-Host "📊 翻译统计" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Green
Write-Host "✅ 成功: $SuccessCount" -ForegroundColor Green
Write-Host "❌ 失败: $FailCount" -ForegroundColor $(if ($FailCount -gt 0) { "Red" } else { "Gray" })
Write-Host "⏱️  耗时: $($Duration.ToString('mm\:ss'))" -ForegroundColor Gray
Write-Host ""

if ($SuccessCount -gt 0) {
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "📦 下一步操作" -ForegroundColor Cyan
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "1. 查看生成的代码:" -ForegroundColor Yellow
    Write-Host "   cd $OutputDir" -ForegroundColor Gray
    Write-Host ""
    
    if (-not $SkipCheck) {
        Write-Host "2. 创建 Cargo 项目并验证:" -ForegroundColor Yellow
        Write-Host "   cd $OutputDir" -ForegroundColor Gray
        Write-Host "   cargo init --lib" -ForegroundColor Gray
        Write-Host "   cargo build" -ForegroundColor Gray
        Write-Host ""
        
        Write-Host "3. 如果有编译错误，使用修复命令:" -ForegroundColor Yellow
        Write-Host "   cargo check 2> errors.txt" -ForegroundColor Gray
        Write-Host "   cargo run -- fix --file <rust_file> --errors errors.txt" -ForegroundColor Gray
        Write-Host ""
        
        Write-Host "4. 优化 unsafe 代码:" -ForegroundColor Yellow
        Write-Host "   cargo run -- optimize-unsafe --file <rust_file>" -ForegroundColor Gray
        Write-Host ""
    }
}

Write-Host "💡 提示:" -ForegroundColor Cyan
Write-Host "   - 单文件翻译: .\translate_chibicc.ps1 -SingleFile -File unicode.c" -ForegroundColor Gray
Write-Host "   - 查看帮助: cargo run -- --help" -ForegroundColor Gray
Write-Host ""
