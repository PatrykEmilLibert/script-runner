use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn apply_no_console_window(cmd: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = cmd;
    }
}

fn apply_macos_runtime_env(cmd: &mut Command) {
    #[cfg(target_os = "macos")]
    {
        // Ensure Python sees the actual macOS version (not legacy compatibility 10.16).
        cmd.env("SYSTEM_VERSION_COMPAT", "0");
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = cmd;
    }
}

#[cfg(target_os = "macos")]
const MACOS_VERSION_COMPAT_LAUNCHER: &str = r#"import os, platform, runpy, subprocess, sys
os.environ['SYSTEM_VERSION_COMPAT'] = '0'

def _real_macos_version():
    try:
        return subprocess.check_output(['/usr/bin/sw_vers', '-productVersion'], text=True).strip()
    except Exception:
        return None

_version = _real_macos_version()
if _version:
    _machine = platform.machine() or 'x86_64'
    _parts = [int(p) for p in _version.split('.') if p.isdigit()]
    _major = _parts[0] if _parts else 0
    _minor = _parts[1] if len(_parts) > 1 else 0
    _micro = _parts[2] if len(_parts) > 2 else 0

    def _patched_mac_ver(release='', versioninfo=('', '', ''), machine=''):
        return (_version, ('', '', ''), _machine)

    def _patched_release():
        return f"{_major}.{_minor}.{_micro}"

    def _patched_version():
        return _patched_release()

    def _patched_platform(*args, **kwargs):
        return f"macOS-{_version}-{_machine}"

    platform.mac_ver = _patched_mac_ver
    platform.release = _patched_release
    platform.version = _patched_version
    platform.platform = _patched_platform

    try:
        _orig_uname = os.uname

        def _patched_uname():
            u = _orig_uname()
            return os.uname_result((u.sysname, u.nodename, _patched_release(), _patched_version(), u.machine))

        os.uname = _patched_uname
    except Exception:
        pass

script_path = sys.argv[1]
sys.argv = [script_path] + sys.argv[2:]
runpy.run_path(script_path, run_name='__main__')"#;

pub fn check_platform_compatibility(script_content: &str) -> Result<Vec<String>, String> {
    let _ = script_content;
    Ok(vec![])
}

pub async fn execute_script(
    script_path: &PathBuf,
    python_exec: &PathBuf,
    args: Option<Vec<String>>,
) -> Result<String, String> {
    use crate::script_encryption;

    // Check if script is encrypted
    let script_content = if script_path.extension().and_then(|s| s.to_str()) == Some("enc") {
        // Decrypt in-memory
        script_encryption::decrypt_script(script_path)?
    } else {
        // Read plain file
        std::fs::read_to_string(script_path).map_err(|e| format!("Failed to read script: {}", e))?
    };

    // Check platform compatibility
    let compatibility_issues = check_platform_compatibility(&script_content)?;
    if !compatibility_issues.is_empty() {
        let warning = format!(
            "⚠️ WARNING: This script contains Windows-specific libraries:\n{}\n\nThis may not work correctly on {}. Consider creating a cross-platform version.",
            compatibility_issues.join("\n"),
            std::env::consts::OS
        );
        log::warn!("{}", warning);
    }

    // For encrypted scripts: write to temp file, execute, then delete
    let (temp_file, script_to_execute) =
        if script_path.extension().and_then(|s| s.to_str()) == Some("enc") {
            use std::io::Write;
            let temp_path = script_path
                .parent()
                .map(|dir| dir.join(format!(".sr_runtime_{}.py", uuid::Uuid::new_v4())))
                .unwrap_or_else(|| {
                    std::env::temp_dir().join(format!("sr_temp_{}.py", uuid::Uuid::new_v4()))
                });
            let mut file = std::fs::File::create(&temp_path)
                .map_err(|e| format!("Failed to create temp file: {}", e))?;
            file.write_all(script_content.as_bytes())
                .map_err(|e| format!("Failed to write temp file: {}", e))?;
            (Some(temp_path.clone()), temp_path)
        } else {
            (None, script_path.clone())
        };

    let mut cmd = Command::new(python_exec);
    #[cfg(target_os = "macos")]
    {
        cmd.args([
            "-c",
            MACOS_VERSION_COMPAT_LAUNCHER,
            &script_to_execute.to_string_lossy(),
        ]);
    }

    #[cfg(not(target_os = "macos"))]
    {
        cmd.arg(&script_to_execute);
    }

    apply_no_console_window(&mut cmd);
    apply_macos_runtime_env(&mut cmd);
    if let Some(script_dir) = script_path.parent() {
        cmd.current_dir(script_dir);
    }

    // Add script arguments if provided
    if let Some(script_args) = args {
        cmd.args(script_args);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute script: {}", e))?;

    // Clean up temp file if it was created
    if let Some(temp) = temp_file {
        let _ = std::fs::remove_file(temp); // Ignore errors on cleanup
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Save logs
    if let Some(script_name) = script_path.file_stem() {
        let log_path = script_path
            .parent()
            .unwrap()
            .join(format!("{}.log", script_name.to_string_lossy()));
        if let Ok(mut file) = File::create(&log_path) {
            let _ = writeln!(file, "=== Script Output ===\n{}\n", stdout);
            let _ = writeln!(file, "=== Errors ===\n{}\n", stderr);
            let _ = writeln!(
                file,
                "=== Status ===\n{}\n",
                if output.status.success() {
                    "Success"
                } else {
                    "Failed"
                }
            );
        }
    }

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(format!("Script failed: {}", stderr))
    }
}
