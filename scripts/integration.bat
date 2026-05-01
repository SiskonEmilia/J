@echo off
setlocal enabledelayedexpansion
set "EXEPATH=%~dp0..\target\release\j.exe"
if not exist "%EXEPATH%" (
    echo Building release...
    pushd "%~dp0.."
    cargo build --release || exit /b 1
    popd
)

rem 准备工作区
set "WORKSPACE=%TEMP%\j_itest_%RANDOM%%RANDOM%"
mkdir "%WORKSPACE%\d3\Data" 2>nul

set "CFG=%WORKSPACE%\config.jsonc"
> "%CFG%" (
    echo {
    echo   "commands": {},
    echo   "templates": { "u": { "children": { "d": { "path": "Data" } } } },
    echo   "roots": {
    echo     "d3": { "path": "%WORKSPACE:\=\\%\\d3", "templates": ["u"] }
    echo   }
    echo }
)
set "J_CONFIG=%CFG%"

rem 生成 shim bat 到临时目录并调用
set "BINDIR=%WORKSPACE%\bin"
mkdir "%BINDIR%" 2>nul
"%EXEPATH%" :init cmd > "%BINDIR%\j.bat"
set "PATH=%BINDIR%;%PATH%"

rem 测 1: jump 到 root
pushd "%TEMP%"
call j d3
call :assert_cwd "%WORKSPACE%\d3" "jump root" || goto :fail
popd

rem 测 2: jump 到模板符号
pushd "%TEMP%"
call j d3 d
call :assert_cwd "%WORKSPACE%\d3\Data" "jump template sym" || goto :fail
popd

echo ALL PASSED.
rmdir /s /q "%WORKSPACE%" >nul 2>&1
exit /b 0

:fail
popd >nul 2>&1
rmdir /s /q "%WORKSPACE%" >nul 2>&1
exit /b 1

:assert_cwd
set "expected=%~1"
set "label=%~2"
for /f "delims=" %%p in ('cd') do set "got=%%p"
if /i "%got%"=="%expected%" (
    echo [OK] %label%: %got%
) else (
    echo [FAIL] %label%: got "%got%" expected "%expected%"
    exit /b 1
)
goto :eof
