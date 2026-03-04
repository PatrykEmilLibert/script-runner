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

## CI/Release Build (signed updater artifacts)
For production auto-update, create a tagged release and let GitHub Actions build signed assets.

Required repository secrets:
- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (only if your key is password-protected)
- `TAURI_UPDATER_PUBLIC_KEY`

Release flow:
1. Bump version in `src-tauri/tauri.conf.json` and `src-tauri/tauri.release.conf.json`.
2. Push commit to `main`.
3. Create and push tag, e.g. `v0.5.2`.
4. Workflow `.github/workflows/build.yml` will publish:
	- installers (`.msi`, NSIS `.exe`)
	- updater signatures (`.sig`)
	- `latest.json` manifest used by in-app updater.

## Output Files
The Windows installers are created at:
- **MSI Installer**: `src-tauri/target/release/bundle/msi/ScriptRunner_*.msi`
- **NSIS Installer**: `src-tauri/target/release/bundle/nsis/ScriptRunner_*_x64-setup.exe`

Updater artifacts (release config only):
- `src-tauri/target/release/bundle/nsis/*.exe.sig`

## Running
Double-click either installer to install the application.

## Admin Access Setup
Admin features now require GitHub admin login (PAT + `admins.json` in `script-runner-config`).
See `GITHUB_AUTH_GUIDE.md` for setup details.

## Current Build
✅ **Windows installers ready:**
- MSI: 4.74 MB
- NSIS: 3.28 MB
