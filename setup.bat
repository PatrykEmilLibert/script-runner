@echo off
REM Script Runner - Windows Setup Script
REM This script installs all dependencies for ScriptRunner development

echo.
echo ========================================
echo   ScriptRunner - Windows Setup
echo   Pink Theme Edition
echo ========================================
echo.

REM Check Node.js version
echo Checking Node.js...
node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [!] Node.js not found. Please install from https://nodejs.org
    pause
    exit /b 1
)

REM Extract version and check if it's 18+
for /f "tokens=1 delims=." %%a in ('node --version') do set NODE_MAJOR=%%a
set NODE_MAJOR=%NODE_MAJOR:v=%
if %NODE_MAJOR% lss 18 (
    echo [!] Node.js version must be 18 or higher
    echo     Current version:
    node --version
    echo     Please upgrade from https://nodejs.org
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
echo This may take a few minutes...
call npm install
if %errorlevel% neq 0 (
    echo [!] Failed to install npm dependencies
    pause
    exit /b 1
)
echo [OK] npm dependencies installed

REM Check Rust dependencies
echo.
echo Checking Rust dependencies...
cd src-tauri
cargo check --quiet
if %errorlevel% neq 0 (
    echo [!] Rust dependency check failed
    echo Please check Cargo.toml for errors
    cd ..
    pause
    exit /b 1
)
cd ..
echo [OK] Rust dependencies verified

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
echo Theme: Pink (#EC4899) with Dark/Light mode
echo.
echo To start development:
echo   npm run tauri dev
echo.
echo To build for production:
echo   npm run tauri build
echo.
echo For more info, see:
echo   - README.md (overview)
echo   - UPGRADE_GUIDE.md (new features)
echo   - ADMIN_GUIDE.md (admin panel)
echo   - TESTING.md (test checklist)
echo.
pause
