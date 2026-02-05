# ScriptRunner - Python Script Executor

A modern desktop application built with **Tauri + React** for executing Python scripts with:
- 🌸 **Elegant Pink Theme** with dark/light mode support
- 🔄 **Auto-update** from GitHub (both app & scripts)
- 📦 **Automatic dependency detection** using AST parsing
- 🛡️ **Remote kill switch** for security
- 📊 **Analytics Dashboard** with usage statistics
- 🔔 **Smart Notifications** system
- 🎛️ **Admin Panel** for advanced management
- 🚀 **Standalone executables** for Windows & Mac (no dependencies required)

## Features

### 🌸 Pink Theme
- Modern pink color palette (#EC4899 primary)
- Smooth dark/light mode transitions
- Glassmorphism effects
- Responsive design with Tailwind CSS
- Framer Motion animations

### 📊 Analytics Dashboard
- Real-time execution statistics
- Script usage charts (Recharts)
- Success/failure rate tracking
- Performance metrics
- Visual data representation

### 🛡️ Admin Panel
- Secure admin key authentication
- Kill Switch management
- Script upload/management via drag & drop
- System diagnostics
- Configuration override

### 🔔 Notification System
- Success/Error/Warning/Info notifications
- Auto-dismiss with configurable timing
- Position control (top-right by default)
- Mantine Notifications integration
- Custom styling per notification type

### 1. **Automatic Dependency Resolution**
- Scans Python imports using AST parsing
- Filters out stdlib modules automatically
- Installs missing packages with one click
- Supports `requirements.txt` and `pyproject.toml`

### 2. **Script Management**
- List all Python scripts from GitHub repository
- One-click script execution
- Real-time output streaming
- Log history per script

### 3. **Remote Kill Switch**
- Check GitHub for `kill_switch.json` at startup
- Blocks app if `"blocked": true`
- Requires internet connection
- No way to bypass without modifying binary

### 4. **Auto-Update**
- Updates app from GitHub releases
- Syncs Python scripts from repository
- Automatic dependency installation

## Requirements

⚠️ **Internet connection required on first launch** - Application needs network access to:
- Clone scripts repository from GitHub (first launch only)
- Check kill switch status
- Install Python dependencies
- Auto-update the application

ℹ️ **After first launch**: Scripts are cached locally and app works offline

📂 **Script storage location**: See [SCRIPTS_LOCATION.md](SCRIPTS_LOCATION.md) for details

## Project Structure

```
script-runner/
├── src/                    # React frontend
│   ├── App.tsx            # Main app component
│   ├── components/
│   │   ├── Dashboard.tsx   # Stats & overview
│   │   ├── ScriptList.tsx  # List of scripts
│   │   ├── ScriptExecutor.tsx # Run & output
│   │   └── LogViewer.tsx   # Log history
│   ├── App.css            # Dark mode styling
│   └── index.css
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Tauri app entry
│   │   ├── kill_switch.rs # Remote block logic
│   │   ├── git_manager.rs # Git sync
│   │   ├── python_runner.rs # Execute scripts
│   │   └── dependency_manager.rs # Pip + AST
│   └── Cargo.toml
├── package.json           # Node dependencies
├── vite.config.ts         # Vite config
└── tauri.conf.json        # Tauri config
```

## Setup & Development

### Prerequisites
- **Node.js** 18+ (with npm)
- **Rust** 1.60+ (for building Tauri)
- **Python** 3.12.9 (embedded in final build)

### Install Dependencies

```bash
cd script-runner
npm install
```

### Development Mode

```bash
npm run tauri dev
```

### Build for Production

```bash
npm run tauri build
```

This creates:
- **Windows**: `src-tauri/target/release/script-runner.exe`
- **Mac**: `src-tauri/target/release/bundle/dmg/ScriptRunner.dmg`

## Configuration

### Environment Variables

Create `.env` in project root:

```
SCRIPTS_REPO_URL=https://github.com/PatrykEmilLibert/python-scripts
KILL_SWITCH_REPO=https://github.com/PatrykEmilLibert/script-runner-config
```

### Kill Switch Setup

1. Create `kill_switch.json` in your config repo:

```json
{
  "blocked": false,
  "timestamp": "2026-01-20T12:00:00"
}
```

2. Set `KILL_SWITCH_REPO` to your config repository URL
3. To block: set `"blocked": true` and push to GitHub
4. App checks on startup (requires internet connection)

## How It Works

### Script Execution Flow

```
User selects script
↓
App reads Python file
↓
AST parser extracts imports
↓
Filters out stdlib (sys, os, json, etc.)
↓
Compares with installed packages
↓
Pip installs missing packages
↓
Executes script with embedded Python
↓
Captures output & saves logs
```

### Kill Switch Flow

```
App starts
↓
Fetches kill_switch.json from GitHub
↓
If no internet → Show error & exit
↓
If blocked=true → Show blocking screen & exit
↓
If clear → Proceed normally
```

## API Commands (Rust → React)

```rust
// Check if app is blocked
#[tauri::command]
async fn check_kill_switch() -> Result<bool, String>

// Sync scripts from GitHub
#[tauri::command]
async fn sync_scripts(state: State<AppState>) -> Result<String, String>

// Run a script with auto-dependencies
#[tauri::command]
async fn run_script(script_name: String) -> Result<String, String>

// List available scripts
#[tauri::command]
async fn list_scripts() -> Result<Vec<String>, String>

// Get script logs
#[tauri::command]
async fn get_script_logs(script_name: String) -> Result<String, String>
```

## Deployment

### Create GitHub Releases

1. Tag version: `git tag v0.1.0`
2. Push tag: `git push origin v0.1.0`
3. Build: `npm run tauri build`
4. Upload `.exe` and `.dmg` to GitHub Releases

### Distribute to Team

- **Windows**: Download `.exe` and run installer
- **Mac**: Download `.dmg`, drag app to Applications
- **Auto-update**: App checks releases on startup

## Security Notes

⚠️ **Important**: This app runs on user machines. For enterprise use:
- Code-sign executables
- Use VPN for script repositories
- Implement user authentication
- Monitor script execution logs
- Use kill switch for emergency lockdown

## Next Steps

1. ✅ Set up GitHub repositories
2. ✅ Add your Python scripts
3. ✅ Create kill_switch.json config
4. ✅ Build standalone executables
5. ✅ Distribute to team

## Tech Stack

- **Frontend**: React 18 + TypeScript + TailwindCSS + Framer Motion
- **UI Components**: Mantine UI + Lucide Icons
- **Charts**: Recharts (responsive, interactive)
- **Desktop**: Tauri 2.x (lightweight Electron alternative)
- **Backend**: Rust + reqwest + git2 + rusqlite
- **Python Execution**: Subprocess + AST parsing for dependencies
- **Database**: SQLite (analytics storage)
- **Styling**: Custom pink theme with glassmorphism

## Screenshots

### 🌸 Pink Theme (Dark Mode)
```
[Placeholder: Screenshot showing dark mode interface]
- Charcoal background with pink accents
- Smooth transitions and gradients
- Modern, professional design
```

### 📊 Analytics Dashboard
```
[Placeholder: Analytics tab with charts]
- Execution timeline (bar chart)
- Success rate pie chart
- Top scripts ranking
- Performance metrics
```

### 🛡️ Admin Panel
```
[Placeholder: Admin panel interface]
- Admin key authentication
- Kill Switch status indicator
- Drag & drop script upload
- System diagnostics panel
```

### 🔔 Notification System
```
[Placeholder: Notification examples]
- Success (green with checkmark)
- Error (red with X)
- Warning (yellow with exclamation)
- Info (blue with info icon)
```

## Documentation

- 📖 **[README.md](README.md)** - This file (overview & quick start)
- 🔄 **[UPGRADE_GUIDE.md](UPGRADE_GUIDE.md)** - What's new & migration steps
- 🛡️ **[ADMIN_GUIDE.md](ADMIN_GUIDE.md)** - Admin panel & kill switch management
- ✅ **[TESTING.md](TESTING.md)** - Complete testing checklist
- 🚀 **[QUICK_START.md](QUICK_START.md)** - Getting started quickly
- 📋 **[PROJECT_SUMMARY.md](PROJECT_SUMMARY.md)** - Technical overview
- 📦 **[DEPLOYMENT.md](DEPLOYMENT.md)** - Build & release process

## License

MIT
