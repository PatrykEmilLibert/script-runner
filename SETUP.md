# ScriptRunner Setup Guide

## Installation for Contributors

### Prerequisites
- **Windows 10/11** or **Mac OS 11+**
- **Node.js 18+** (download from [nodejs.org](https://nodejs.org))
- **Rust 1.60+** (install from [rustup.rs](https://rustup.rs))
- **Python 3.12+** (for testing, development only)

### Setup Steps

1. **Clone the repository**
```bash
git clone https://github.com/PatrykEmilLibert/script-runner.git
cd script-runner
```

2. **Install Node dependencies**
```bash
npm install
```

3. **Install Tauri CLI** (optional, auto-installed by npm scripts)
```bash
npm install --save-dev @tauri-apps/cli
```

4. **Run in development mode**
```bash
npm run tauri dev
```

This starts:
- Vite dev server on `http://localhost:1420`
- Rust backend with auto-reload
- Hot module reloading (HMR)

### Building for Production

**Windows:**
```bash
npm run tauri build
# Output: src-tauri/target/release/script-runner.exe
```

**Mac:**
```bash
npm run tauri build
# Output: src-tauri/target/release/bundle/dmg/ScriptRunner.dmg
```

### Environment Configuration

Create `.env` file in project root:
```
SCRIPTS_REPO_URL=https://github.com/YOUR_USERNAME/python-scripts
```

## For End Users

### Windows Installation

1. Download `ScriptRunner-installer.exe` from [Releases](../../releases)
2. Run the installer
3. App will auto-update scripts and dependencies on first run

### Mac Installation

1. Download `ScriptRunner.dmg` from [Releases](../../releases)
2. Open the DMG file
3. Drag `ScriptRunner.app` to Applications folder
4. Launch from Applications

### Updating

The app checks for updates automatically on startup. Updates are applied in the background without interruption.

## Troubleshooting

### npm install fails
- Ensure Node.js is in PATH: `node --version`
- Try: `npm cache clean --force` then `npm install`

### Rust compilation errors
- Update Rust: `rustup update`
- Clear cache: `cargo clean`

### Python dependencies not installing
- Check internet connection (needs to reach PyPI)
- Verify Python 3.12.9 path in `tauri.conf.json`
- Check logs in `~/.scriptrunner/logs/`

## Development Workflow

1. Make changes to React components (`src/`)
2. Make changes to Rust backend (`src-tauri/src/`)
3. Changes auto-reload in dev mode
4. Test functionality
5. Build and package: `npm run tauri build`

## Creating Releases

1. Create GitHub release with tag format: `v1.0.0`
2. GitHub Actions automatically builds for Windows & Mac
3. Executables uploaded to release assets
4. Users can download and install

## Project Structure

```
script-runner/
├── src/                          # React frontend
│   ├── App.tsx                  # Main component
│   ├── components/              # React components
│   ├── App.css                  # Styling
│   └── index.css
├── src-tauri/                    # Rust backend
│   ├── src/                     # Rust source
│   ├── assets/                  # Embedded assets
│   ├── Cargo.toml               # Dependencies
│   └── tauri.conf.json          # Config
├── .github/workflows/           # GitHub Actions
├── package.json                 # Node packages
├── vite.config.ts               # Build config
└── README.md
```

## Support

For issues or questions:
- Check [Issues](../../issues)
- Create new issue with details
- Include logs from `~/.scriptrunner/logs/`
