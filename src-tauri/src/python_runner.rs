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

// Map of Windows-specific imports to their descriptions
fn get_windows_specific_imports() -> Vec<(&'static str, &'static str)> {
    vec![
        ("win32com", "Windows COM library"),
        ("win32con", "Windows constants"),
        ("win32api", "Windows API access"),
        ("win32file", "Windows file operations"),
        ("win32event", "Windows event handling"),
        ("win32gui", "Windows GUI operations"),
        ("win32inet", "Windows internet operations"),
        ("win32net", "Windows network operations"),
        ("win32netcon", "Windows network constants"),
        ("win32pipe", "Windows pipe operations"),
        ("win32print", "Windows print operations"),
        ("win32process", "Windows process operations"),
        ("win32security", "Windows security operations"),
        ("win32service", "Windows service operations"),
        ("win32wnet", "Windows network functions"),
        ("pywintypes", "Python Windows types"),
        ("pywin32", "Python Windows extensions"),
        ("ctypes.windll", "Windows DLL access"),
        ("winreg", "Windows registry access"),
        ("winsound", "Windows sound operations"),
        ("msvcrt", "Microsoft C runtime library"),
        ("_winreg", "Windows registry (legacy)"),
        ("_ctypes", "C types library (Windows-specific usage)"),
        ("comtypes", "COM types for Windows"),
        ("pywinauto", "Windows GUI automation"),
        ("win_inet_pton", "Windows inet functions"),
    ]
}

pub fn check_platform_compatibility(script_content: &str) -> Result<Vec<String>, String> {
    let os = std::env::consts::OS;

    // Only check non-Windows platforms
    if os == "windows" {
        return Ok(vec![]);
    }

    let windows_imports = get_windows_specific_imports();
    let mut found_issues = vec![];

    for line in script_content.lines() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("#") {
            continue;
        }

        for (import, description) in &windows_imports {
            // Check for: import X, from X import Y, from X.Y import Z
            if trimmed.contains(&format!("import {}", import))
                || trimmed.contains(&format!("from {}", import))
                || trimmed.contains(&format!(
                    "from {}",
                    import.split('.').next().unwrap_or(import)
                ))
            {
                found_issues.push(format!("  • {} ({})", import, description));
            }
        }
    }

    Ok(found_issues)
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
    let (temp_file, script_to_execute) = if script_path.extension().and_then(|s| s.to_str())
        == Some("enc")
    {
        use std::io::Write;
        let temp_path = std::env::temp_dir().join(format!("sr_temp_{}.py", uuid::Uuid::new_v4()));
        let mut file = std::fs::File::create(&temp_path)
            .map_err(|e| format!("Failed to create temp file: {}", e))?;
        file.write_all(script_content.as_bytes())
            .map_err(|e| format!("Failed to write temp file: {}", e))?;
        (Some(temp_path.clone()), temp_path)
    } else {
        (None, script_path.clone())
    };

    let mut cmd = Command::new(python_exec);
    cmd.arg(&script_to_execute);
    apply_no_console_window(&mut cmd);
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
