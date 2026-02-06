@echo off
echo ========================================
echo ScriptRunner - Build Script
echo ========================================
echo.

cd /d P:\python_runner_github\script-runner

echo [1/4] Cleaning previous build...
cd src-tauri
cargo clean
cd ..

echo.
echo [2/4] Building application...
npm run tauri build

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo Build failed!
    pause
    exit /b 1
)

echo.
echo [3/4] Creating portable version...
if not exist "portable" mkdir portable
copy "src-tauri\target\release\script-runner.exe" "portable\ScriptRunner-Portable.exe"
echo Standalone portable version - no installation required > portable\README.txt

echo.
echo [4/4] Build complete!
echo.
echo Output files:
echo - Installer (MSI): src-tauri\target\release\bundle\msi\
echo - Installer (NSIS): src-tauri\target\release\bundle\nsis\
echo - Portable: portable\ScriptRunner-Portable.exe
echo.
pause
