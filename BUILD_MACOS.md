# macOS Build Instructions

## Prerequisites
1. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Install Node.js: Download from https://nodejs.org/
3. Install Xcode Command Line Tools: `xcode-select --install`

## Build Steps
```bash
cd script-runner
npm install
chmod +x build.sh
./build.sh
```

## Output
The macOS app bundle will be created at:
- `src-tauri/target/release/bundle/dmg/ScriptRunner_0.1.0_x64.dmg`
- `src-tauri/target/release/bundle/macos/ScriptRunner.app`

## Running
Double-click the `.dmg` file to mount it, then drag ScriptRunner to Applications folder.

## Admin Key Setup
Place `sr-admin.key` file on your Desktop to enable admin features.
