@echo off
REM Script Runner - Windows Setup Script
REM This script installs all dependencies for ScriptRunner development

echo.
echo ========================================
echo   ScriptRunner - Windows Setup
echo ========================================
echo.

REM Check if Node.js is installed
echo Checking Node.js...
node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [!] Node.js not found. Please install from https://nodejs.org
    pause
    exit /b 1
)
echo [OK] Node.js installed
node --version

REM Check if Rust is installed
echo.
echo Checking Rust...
cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [!] Rust not found. Installing...
    powershell -Command "[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://win.rustup.rs/x86_64'))"
    if %errorlevel% neq 0 (
        echo [!] Failed to install Rust
        echo Please install manually from https://rustup.rs
        pause
        exit /b 1
    )
)
echo [OK] Rust installed
cargo --version

REM Install npm dependencies
echo.
echo Installing npm dependencies...
call npm install
if %errorlevel% neq 0 (
    echo [!] Failed to install npm dependencies
    pause
    exit /b 1
)
echo [OK] npm dependencies installed

REM Create .env file if not exists
echo.
if not exist .env (
    echo Creating .env file...
    (
        echo SCRIPTS_REPO_URL=https://github.com/PatrykEmilLibert/python-scripts
        echo KILL_SWITCH_REPO=https://github.com/PatrykEmilLibert/script-runner-config
    ) > .env
    echo [OK] .env created
)

echo.
echo ========================================
echo   Setup Complete!
echo ========================================
echo.
echo To start development:
echo   npm run tauri dev
echo.
echo To build for production:
echo   npm run tauri build
echo.
pause
