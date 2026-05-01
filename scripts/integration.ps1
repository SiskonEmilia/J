#requires -Version 5.0
# PowerShell shim smoke-test for `j`. Runs without Pester (Pester 5 的作用域对 `&` 调用运算符会抛 RuntimeException，不使用)。
# 退出码 0 = all pass；非 0 = 至少一个断言失败。

$ErrorActionPreference = 'Stop'

function Assert-Equal {
    param([string]$Label, $Expected, $Actual)
    if ($Expected -eq $Actual) {
        Write-Host ("[OK] {0}: {1}" -f $Label, $Actual)
    } else {
        Write-Host ("[FAIL] {0}: expected '{1}', got '{2}'" -f $Label, $Expected, $Actual)
        $script:Failures++
    }
}

function Assert-Match {
    param([string]$Label, [string]$Pattern, [string]$Actual)
    if ($Actual -match $Pattern) {
        Write-Host ("[OK] {0}: matches /{1}/" -f $Label, $Pattern)
    } else {
        Write-Host ("[FAIL] {0}: /{1}/ not in '{2}'" -f $Label, $Pattern, $Actual)
        $script:Failures++
    }
}

$script:Failures = 0

$ExePath = [string](Resolve-Path "$PSScriptRoot\..\target\release\j.exe")
if (-not (Test-Path $ExePath)) {
    Write-Host "Building release..."
    Push-Location "$PSScriptRoot\.."
    cargo build --release
    Pop-Location
    $ExePath = [string](Resolve-Path "$PSScriptRoot\..\target\release\j.exe")
}

$Workspace = (New-Item -ItemType Directory -Path "$env:TEMP\j_itest_$((Get-Random))" -Force).FullName
try {
    New-Item -ItemType Directory -Path "$Workspace\d3\Data" -Force | Out-Null
    New-Item -ItemType Directory -Path "$Workspace\d3\Shared\Data" -Force | Out-Null

    $ConfigPath = Join-Path $Workspace "config.jsonc"
    $WsEscaped = $Workspace -replace '\\', '\\'
    @"
{
  "commands": { "c": "echo" },
  "templates": { "u": { "children": {
    "d":  { "path": "Data" },
    "sd": { "path": "Shared/Data" }
  }}},
  "roots": {
    "d3": { "path": "$WsEscaped\\d3", "templates": ["u"] }
  }
}
"@ | Set-Content -Path $ConfigPath -Encoding ASCII

    $env:J_CONFIG = $ConfigPath

    # 定义 j 函数（不带 Register-ArgumentCompleter；只做跳转集成测试）
    $fnTemplate = @'
function j {
    $out = (& '__EXE__' --shell=powershell @args) -join [Environment]::NewLine
    if ($LASTEXITCODE -eq 0 -and $out) {
        Invoke-Expression $out
    }
}
'@
    $fn = $fnTemplate.Replace('__EXE__', $ExePath.Replace("'", "''"))
    Invoke-Expression $fn

    # 测 1：jump 到 root
    Push-Location $env:TEMP
    j d3
    Assert-Equal 'jump root' (Join-Path $Workspace 'd3') (Get-Location).Path
    Pop-Location

    # 测 2：jump 到模板符号
    Push-Location $env:TEMP
    j d3 d
    Assert-Equal 'jump template sym' (Join-Path (Join-Path $Workspace 'd3') 'Data') (Get-Location).Path
    Pop-Location

    # 测 3：alias 透传参数
    Push-Location $env:TEMP
    $captured = (j d3 d -c --new-window 2>&1 | Out-String)
    Assert-Match 'alias passthrough contains "."' '\.' $captured
    Assert-Match 'alias passthrough contains --new-window' '--new-window' $captured
    Pop-Location

    if ($script:Failures -gt 0) {
        Write-Host ("{0} assertion(s) failed" -f $script:Failures)
        exit 1
    }
    Write-Host 'ALL PASSED.'
    exit 0
}
finally {
    Remove-Item -Recurse -Force $Workspace -ErrorAction SilentlyContinue
    Remove-Item Env:\J_CONFIG -ErrorAction SilentlyContinue
}
