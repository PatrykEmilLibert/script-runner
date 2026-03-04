# ScriptRunner - Quick Start Guide

## 🚀 For Users

### Installation

⚠️ **Prerequisites:** Internet connection required for all features

**Windows:**
1. Download latest `.exe` from [Releases](../../releases)
2. Run installer
3. Done! App will auto-update

**Mac:**
1. Download latest `.dmg` from [Releases](../../releases)
2. Open → Drag to Applications
3. Launch from Applications → Done!

### Usage

1. **First Run**: App syncs scripts from your GitHub repo (requires internet)
2. **Select Script**: Click on script in left panel
3. **Run**: Click green "▶ Run Script" button
4. **View Output**: Logs display in real-time
5. **Auto-Update**: Dependencies install automatically

## 👨‍💻 For Developers

### 1. One-Time Setup

```bash
cd script-runner
./setup.bat  # Windows
# or
./setup.sh   # Mac (coming soon)
```

### 2. Start Development

```bash
npm run tauri dev
```

This opens the desktop app with:
- Hot module reloading
- Rust auto-compilation
- Live logs

### 3. Make Changes

**Frontend** (React):
- Edit files in `src/`
- Changes auto-reload

**Backend** (Rust):
- Edit files in `src-tauri/src/`
- Changes auto-rebuild

### 4. Build for Production

```bash
npm run tauri build
```

Output:
- Windows: `src-tauri/target/release/script-runner.exe`
- Mac: `src-tauri/target/release/bundle/dmg/ScriptRunner.dmg`

## 📝 How to Create Scripts

### Simple Example

Create `my_script.py`:

```python
#!/usr/bin/env python3
import json
from datetime import datetime

print("Hello from ScriptRunner!")
print(json.dumps({"timestamp": datetime.now().isoformat()}))
```

ScriptRunner **auto-detects** dependencies:
- ✅ `json` (stdlib - auto)
- ✅ `datetime` (stdlib - auto)
- ❌ Nothing to install!

### With External Package

Create `analyze.py`:

```python
import pandas as pd
import numpy as np

data = pd.read_csv("data.csv")
print(f"Mean: {np.mean(data['values'])}")
```

ScriptRunner automatically:
1. Detects `pandas` and `numpy` imports
2. Installs packages via pip
3. Runs script
4. Captures output

No need for `requirements.txt` - but you can add one if you want specific versions.

## 🐛 Troubleshooting

### Scripts not showing?
- Check internet connection (needs GitHub)
- Verify `SCRIPTS_REPO_URL` in `.env`
- Check app logs: `~/.scriptrunner/logs/`

### Dependencies not installing?
- Check Python version: `python --version`
- Verify internet (needs PyPI)
- Check package name spelling

### App won't start?
- Check OS compatibility (Windows 10+ or Mac 11+)
- Try reinstalling
- Check logs

## 📚 Learn More

- [Setup Guide](SETUP.md) - Development setup
- [README](README.md) - Full documentation
- [Contributing](CONTRIBUTING.md) - Contributing guidelines
- [Scripts Location](SCRIPTS_LOCATION.md) - Where synchronized scripts are stored

## 🎯 Next Steps

1. **Create Python scripts** → Add to your GitHub repo
2. **Set up scripts repo** → Point app to it via `.env`
3. **Test locally** → `npm run tauri dev`
4. **Build & release** → `npm run tauri build` → Upload to GitHub Releases
5. **Share with team** → Send download link!

---

**Questions?** Open an issue on GitHub or check existing issues!
