# 测试单个文件翻译

Write-Host "🧪 测试单个文件翻译功能..." -ForegroundColor Cyan
Write-Host ""

# 创建测试 C 文件
$testC = @"
#include <stdio.h>
#include <stdlib.h>

// 简单的加法函数
int add(int a, int b) {
    return a + b;
}

// 字符串长度计算
size_t my_strlen(const char* str) {
    size_t len = 0;
    while (str[len] != '\0') {
        len++;
    }
    return len;
}

// 主函数
int main() {
    int result = add(10, 20);
    printf("10 + 20 = %d\n", result);
    
    const char* hello = "Hello, Rust!";
    printf("Length: %zu\n", my_strlen(hello));
    
    return 0;
}
"@

# 写入测试文件
$testC | Out-File -FilePath "test_input.c" -Encoding UTF8

Write-Host "✅ 已创建测试文件: test_input.c" -ForegroundColor Green
Write-Host ""

# 检查配置
if (-not (Test-Path "config\hybrid_config.toml")) {
    Write-Host "⚠️  配置文件不存在，使用示例配置..." -ForegroundColor Yellow
    if (Test-Path "config\hybrid_config.toml.example") {
        Copy-Item "config\hybrid_config.toml.example" "config\hybrid_config.toml"
        Write-Host "❌ 请编辑 config\hybrid_config.toml 并设置你的 API Key" -ForegroundColor Red
        exit 1
    } else {
        Write-Host "❌ 找不到示例配置文件" -ForegroundColor Red
        exit 1
    }
}

# 执行翻译
Write-Host "🚀 开始翻译..." -ForegroundColor Cyan
cargo run -- translate -i test_input.c -o test_output.rs

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "✅ 翻译成功！" -ForegroundColor Green
    Write-Host ""
    
    if (Test-Path "test_output.rs") {
        Write-Host "📄 翻译结果预览:" -ForegroundColor Yellow
        Write-Host "----------------------------------------"
        Get-Content "test_output.rs" | Select-Object -First 30
        Write-Host "----------------------------------------"
        Write-Host ""
        Write-Host "💡 完整内容请查看: test_output.rs" -ForegroundColor Cyan
    }
} else {
    Write-Host ""
    Write-Host "❌ 翻译失败" -ForegroundColor Red
}
