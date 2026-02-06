# Build Configuration Summary

## Standalone/Portable Windows Build

ScriptRunner now supports building a **portable Windows version** that doesn't require installation.

### Build Output Formats

When you build ScriptRunner for Windows, you get **3 formats**:

1. **MSI Installer** - Traditional Windows installer
   - Location: `src-tauri/target/release/bundle/msi/`
   
2. **NSIS Installer** - Modern setup wizard
   - Location: `src-tauri/target/release/bundle/nsis/`
   
3. **Portable EXE** ✨ - Standalone executable (no installation)
   - Location: `portable/ScriptRunner-Portable.exe`
   - Just download and run - no admin rights needed
   - Perfect for USB drives or restricted environments

### Local Build

Use the provided batch script:

```cmd
cd p:\python_runner_github\script-runner
build.bat
```

This will:
1. Clean previous builds
2. Build the application
3. Create portable version
4. Display all output locations

### GitHub Actions Build

When you push a tag (e.g., `v1.0.0`), GitHub Actions will:
1. Build for Windows (MSI + NSIS + Portable)
2. Build for macOS (DMG)
3. Create a draft release with all artifacts

### Portable Version Features

- **No installation required** - just run the .exe
- **Fully functional** - same features as installed version
- **Self-contained** - includes all dependencies
- **Portable settings** - stores config in local directory

### Manual Portable Build

If you want to create a portable version manually:

```cmd
cd p:\python_runner_github\script-runner
npm run tauri build
mkdir portable
copy src-tauri\target\release\script-runner.exe portable\ScriptRunner-Portable.exe
```

---

**Note**: The portable version is automatically created with every build, both locally and on GitHub Actions.
