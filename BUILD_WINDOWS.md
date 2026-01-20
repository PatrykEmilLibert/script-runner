# Windows Build Instructions

## Prerequisites
1. Install Rust: Download from https://rustup.rs/
2. Install Node.js: Download from https://nodejs.org/
3. Install Visual Studio Build Tools or Visual Studio with C++ tools

## Build Steps
```cmd
cd script-runner
npm install
build.bat
```

## Output Files
The Windows installers are created at:
- **MSI Installer**: `src-tauri/target/release/bundle/msi/ScriptRunner_0.1.0_x64_en-US.msi`
- **NSIS Installer**: `src-tauri/target/release/bundle/nsis/ScriptRunner_0.1.0_x64-setup.exe`

## Running
Double-click either installer to install the application.

## Admin Key Setup
Place `sr-admin.key` file on your Desktop (or in `C:\Users\Public\Desktop\`) to enable admin features.

## Current Build
✅ **Windows installers ready:**
- MSI: 4.74 MB
- NSIS: 3.28 MB
