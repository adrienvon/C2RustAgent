# 监控翻译进度

Write-Host "🔍 监控翻译进度..." -ForegroundColor Cyan
Write-Host ""

$outputDir = "..\rust_output_chibicc"

while ($true) {
    Clear-Host
    Write-Host "🚀 Chibicc 项目翻译进度监控" -ForegroundColor Cyan
    Write-Host "=" * 60
    Write-Host ""
    
    # 统计已生成的 .rs 文件
    if (Test-Path $outputDir) {
        $rsFiles = Get-ChildItem -Path $outputDir -Filter "*.rs" -ErrorAction SilentlyContinue
        $totalFiles = 9  # chibicc 有 9 个 C 文件
        
        Write-Host "📊 已完成: $($rsFiles.Count) / $totalFiles 个文件" -ForegroundColor Green
        Write-Host ""
        
        if ($rsFiles) {
            Write-Host "✅ 已生成的文件:" -ForegroundColor Green
            foreach ($file in $rsFiles) {
                $lines = (Get-Content $file.FullName | Measure-Object -Line).Lines
                $size = [math]::Round($file.Length / 1KB, 2)
                Write-Host "  • $($file.Name) - $lines 行 - $size KB"
            }
        }
        
        Write-Host ""
        Write-Host "⏳ 等待中..." -ForegroundColor Yellow
        
        if ($rsFiles.Count -eq $totalFiles) {
            Write-Host ""
            Write-Host "🎉 翻译完成！" -ForegroundColor Green
            break
        }
    } else {
        Write-Host "⏳ 等待翻译开始..." -ForegroundColor Yellow
    }
    
    Write-Host ""
    Write-Host "按 Ctrl+C 退出监控"
    Start-Sleep -Seconds 10
}

Write-Host ""
Write-Host "💡 查看完整结果:" -ForegroundColor Cyan
Write-Host "  cd $outputDir"
Write-Host "  ls *.rs"
