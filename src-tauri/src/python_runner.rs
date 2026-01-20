use std::path::PathBuf;
use std::process::Command;
use std::fs::File;
use std::io::Write;

pub async fn execute_script(script_path: &PathBuf, python_exec: &PathBuf) -> Result<String, String> {
    let output = Command::new(python_exec)
        .arg(script_path)
        .output()
        .map_err(|e| format!("Failed to execute script: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Save logs
    if let Some(script_name) = script_path.file_stem() {
        let log_path = script_path.parent().unwrap().join(format!("{}.log", script_name.to_string_lossy()));
        if let Ok(mut file) = File::create(&log_path) {
            let _ = writeln!(file, "=== Script Output ===\n{}\n", stdout);
            let _ = writeln!(file, "=== Errors ===\n{}\n", stderr);
            let _ = writeln!(file, "=== Status ===\n{}\n", if output.status.success() { "Success" } else { "Failed" });
        }
    }

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(format!("Script failed: {}", stderr))
    }
}
