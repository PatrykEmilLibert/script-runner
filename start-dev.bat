@echo off
REM ScriptRunner - Start Development
REM This script starts the development server with all necessary checks

setlocal enabledelayedexpansion

cls
color 0A
echo.
echo ========================================
echo   ScriptRunner - Development Server
echo ========================================
echo.

REM Check if we're in the right directory
if not exist "src-tauri" (
    color 0C
    echo [ERROR] Not in script-runner directory!
    echo Please run from: p:\python_runner_github\script-runner
    echo.
    pause
    exit /b 1
)

REM Refresh PATH
set PATH=%PATH%;C:\Program Files\nodejs;C:\Users\%USERNAME%\.cargo\bin

REM Check Node.js
echo Checking Node.js...
node --version >nul 2>&1
if %errorlevel% neq 0 (
    color 0C
    echo [ERROR] Node.js not found!
    echo Please install from https://nodejs.org
    pause
    exit /b 1
)
for /f "tokens=*" %%i in ('node --version') do set NODE_VERSION=%%i
echo [OK] %NODE_VERSION%

REM Check Rust
echo Checking Rust...
cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    color 0C
    echo [ERROR] Rust not found!
    echo Please install from https://rustup.rs
    pause
    exit /b 1
)
for /f "tokens=*" %%i in ('cargo --version') do set CARGO_VERSION=%%i
echo [OK] %CARGO_VERSION%

REM Check dependencies
echo.
echo Checking npm dependencies...
if not exist "node_modules" (
    echo [*] Installing dependencies...
    call npm install
    if %errorlevel% neq 0 (
        color 0C
        echo [ERROR] Failed to install dependencies
        pause
        exit /b 1
    )
)
echo [OK] Dependencies ready

REM Start dev server
echo.
echo ========================================
echo   Starting ScriptRunner Dev Server
echo ========================================
echo.
echo Available endpoints:
echo   Frontend: http://localhost:1420
echo   Backend:  Rust (auto-compile)
echo.
echo Press Ctrl+C to stop
echo.

call npm run tauri dev

pause
