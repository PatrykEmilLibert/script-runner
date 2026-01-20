@echo off
cd /d P:\python_runner_github\script-runner\src-tauri
cargo clean
cd ..
npm run tauri build
pause
