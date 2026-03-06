# 📂 Where Are My Scripts Stored?

Scripts are stored in the user data directory and synchronized from remote GitHub repository.

## Script Storage Location

When running the app (`dev` and built), scripts are stored in this order:

1. `SR_SCRIPTS_DIR` (if set and writable)
2. Install-adjacent folder: `<app_folder>/ScriptRunnerData/scripts` (if writable)
3. User data fallback (`AppData/Roaming` on Windows)

Default fallback locations:

### Windows
```
C:\Users\<YourUsername>\AppData\Roaming\ScriptRunner\scripts\
```

### macOS
```
~/Library/Application Support/ScriptRunner/scripts/
```

### Linux
```
~/.local/share/ScriptRunner/scripts/
```

## How It Works

1. **First Launch**: The app clones the repository from GitHub:
   ```
   https://github.com/PatrykEmilLibert/script-runner-scripts.git
   ```

2. **Subsequent Launches**: The app syncs with GitHub to get updates

3. **Offline Mode**: If GitHub is unavailable (after first successful clone), app uses cached local scripts

## Custom Location

You can override the default location by setting an environment variable:

### Windows (PowerShell)
```powershell
$env:SR_SCRIPTS_DIR = "C:\MyCustomPath\scripts"
```

### Windows (Persistent)
```powershell
[System.Environment]::SetEnvironmentVariable('SR_SCRIPTS_DIR', 'C:\MyCustomPath\scripts', 'User')
```

### macOS/Linux
```bash
export SR_SCRIPTS_DIR="/path/to/your/scripts"
```

## Troubleshooting

### Scripts Not Showing Up?

1. **Check the logs**: Look in the console for messages about script directory
2. **Verify internet connection**: First launch requires internet to clone
3. **Check permissions**: Ensure the app can write to AppData/Application Support
4. **Manual location**: Set `SR_SCRIPTS_DIR` to a known working directory

### Network Issues?

If you can't connect to GitHub, you can:
1. Clone the repository manually
2. Set `SR_SCRIPTS_DIR` to point to your local clone

Example:
```powershell
git clone https://github.com/PatrykEmilLibert/script-runner-scripts.git C:\ScriptRunnerScripts
$env:SR_SCRIPTS_DIR = "C:\ScriptRunnerScripts"
```

## Directory Structure

```
ScriptRunner/
└── scripts/
    ├── official/          # Official scripts (read-only)
    │   ├── 1_system_info/
    │   │   ├── main.py
    │   │   ├── metadata.json
    │   │   └── requirements.txt
    │   └── ...
    └── scripts/           # User scripts (editable)
        └── ...
```
