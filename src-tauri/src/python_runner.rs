use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::{Mutex, OnceLock};

use std::collections::HashMap;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

static RUNNING_SCRIPT_PIDS: OnceLock<Mutex<HashMap<String, u32>>> = OnceLock::new();

fn running_script_pids() -> &'static Mutex<HashMap<String, u32>> {
    RUNNING_SCRIPT_PIDS.get_or_init(|| Mutex::new(HashMap::new()))
}

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
    cmd.env("PYTHONFAULTHANDLER", "1");
    cmd.env("PYTHONUNBUFFERED", "1");
    // Force UTF-8 for the child's stdout/stderr. Otherwise Python on a non-UTF-8
    // Windows locale (e.g. Polish cp1250) encodes piped output in the ANSI code
    // page, so a traceback containing non-ASCII bytes is not valid UTF-8 and the
    // output reader would drop it. This only affects the std streams, not
    // open()/filesystem encoding, so script file I/O behaviour is unchanged.
    cmd.env("PYTHONIOENCODING", "utf-8");

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
const MACOS_VERSION_COMPAT_LAUNCHER: &str = r#"import os, platform, plistlib, runpy, subprocess, sys
os.environ['SYSTEM_VERSION_COMPAT'] = '0'

def _real_macos_version():
    # Priority 1: read SystemVersion.plist directly — completely immune to
    # SYSTEM_VERSION_COMPAT, always returns the real marketing version (e.g. 26.x
    # for macOS 26 Tahoe) even when the Tauri parent binary was compiled with an
    # old MACOSX_DEPLOYMENT_TARGET that would otherwise trigger compat mode.
    try:
        with open('/System/Library/CoreServices/SystemVersion.plist', 'rb') as _plist_file:
            _pdata = plistlib.load(_plist_file)
        _pver = _pdata.get('ProductVersion', '')
        if _pver:
            return _pver
    except Exception:
        pass
    # Fallback: sw_vers subprocess (inherits SYSTEM_VERSION_COMPAT=0 from env)
    try:
        return subprocess.check_output(
            ['/usr/bin/sw_vers', '-productVersion'], text=True
        ).strip()
    except Exception:
        return None

_version = _real_macos_version()
if _version:
    _machine = platform.machine() or 'x86_64'
    _parts = [int(p) for p in _version.split('.') if p.isdigit()]
    _major = _parts[0] if _parts else 0
    _minor = _parts[1] if len(_parts) > 1 else 0

    def _patched_mac_ver(release='', versioninfo=('', '', ''), machine=''):
        return (_version, ('', '', ''), _machine)

    platform.mac_ver = _patched_mac_ver

    # Patch sysconfig.get_platform so that pip platform tags and any code using
    # sysconfig also see the correct macOS version (e.g. macosx-26.0-arm64).
    try:
        import re as _re
        import sysconfig as _sysconfig
        _orig_get_platform = _sysconfig.get_platform

        def _patched_get_platform():
            return _re.sub(
                r'(macosx-)[\d]+\.[\d]+',
                f'\\g<1>{_major}.{_minor}',
                _orig_get_platform()
            )

        _sysconfig.get_platform = _patched_get_platform
    except Exception:
        pass

script_path = sys.argv[1]
sys.argv = [script_path] + sys.argv[2:]
runpy.run_path(script_path, run_name='__main__')"#;

pub fn check_platform_compatibility(script_content: &str) -> Result<Vec<String>, String> {
    let _ = script_content;
    Ok(vec![])
}

/// Payload streamed to the UI for each line a running script prints.
#[derive(Clone, serde::Serialize)]
struct ScriptOutput {
    script: String,
    stream: String,
    line: String,
}

/// Reads a child pipe line-by-line, emitting each line to the UI as it arrives
/// and accumulating the full text (returned on join) for logs and the result.
fn spawn_output_reader<R: std::io::Read + Send + 'static>(
    app: tauri::AppHandle,
    script_name: String,
    pipe: R,
    is_stderr: bool,
) -> std::thread::JoinHandle<String> {
    use std::io::BufRead;
    use tauri::Emitter;

    std::thread::spawn(move || {
        let mut collected = String::new();
        let mut reader = std::io::BufReader::new(pipe);
        let mut raw: Vec<u8> = Vec::new();
        // Read raw bytes and decode lossily instead of using `lines()`, which
        // yields Err on invalid UTF-8. A strict reader would stop at the first
        // non-UTF-8 byte and silently discard the rest of the output — e.g. a
        // Python traceback written in a non-UTF-8 Windows code page collapses to
        // just "Traceback (most recent call last):". Lossy decoding keeps every
        // line (undecodable bytes become U+FFFD).
        loop {
            raw.clear();
            match reader.read_until(b'\n', &mut raw) {
                Ok(0) => break,
                Ok(_) => {
                    if raw.last() == Some(&b'\n') {
                        raw.pop();
                        if raw.last() == Some(&b'\r') {
                            raw.pop();
                        }
                    }
                    let line = String::from_utf8_lossy(&raw).into_owned();
                    let _ = app.emit(
                        "script-output",
                        ScriptOutput {
                            script: script_name.clone(),
                            stream: if is_stderr { "stderr" } else { "stdout" }.to_string(),
                            line: line.clone(),
                        },
                    );
                    collected.push_str(&line);
                    collected.push('\n');
                }
                Err(_) => break,
            }
        }
        collected
    })
}

pub async fn execute_script(
    app: &tauri::AppHandle,
    script_name: &str,
    script_path: &PathBuf,
    python_exec: &PathBuf,
    args: Option<Vec<String>>,
    timeout_secs: Option<u64>,
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

    // Backward compatibility: older path rewrite could emit rstr(...), which should be str(...).
    let needs_rstr_compat = script_content.contains("rstr(")
        && !script_content.contains("def rstr")
        && !script_content.contains("rstr =");

    let execution_content = if needs_rstr_compat {
        format!("rstr = str\n{}", script_content)
    } else {
        script_content.clone()
    };

    // For encrypted scripts (and compatibility-shimmed scripts): write to temp file, execute, then delete
    let (temp_file, script_to_execute) =
        if script_path.extension().and_then(|s| s.to_str()) == Some("enc") || needs_rstr_compat {
            use std::io::Write;
            let temp_path = script_path
                .parent()
                .map(|dir| dir.join(format!(".sr_runtime_{}.py", uuid::Uuid::new_v4())))
                .unwrap_or_else(|| {
                    std::env::temp_dir().join(format!("sr_temp_{}.py", uuid::Uuid::new_v4()))
                });
            let mut file = std::fs::File::create(&temp_path)
                .map_err(|e| format!("Failed to create temp file: {}", e))?;
            file.write_all(execution_content.as_bytes())
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
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(script_dir) = script_path.parent() {
        cmd.current_dir(script_dir);
    }

    // Add script arguments if provided
    if let Some(script_args) = args {
        cmd.args(script_args);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to execute script: {}", e))?;

    let pid = child.id();
    {
        let mut processes = running_script_pids()
            .lock()
            .map_err(|_| "Failed to lock running script process map".to_string())?;
        processes.insert(script_name.to_string(), pid);
    }

    // Stream stdout/stderr to the UI line-by-line while capturing the full text.
    let stdout_reader = child
        .stdout
        .take()
        .map(|pipe| spawn_output_reader(app.clone(), script_name.to_string(), pipe, false));
    let stderr_reader = child
        .stderr
        .take()
        .map(|pipe| spawn_output_reader(app.clone(), script_name.to_string(), pipe, true));

    // Wait for completion, terminating the process tree if it exceeds the
    // optional timeout (0 or None means no limit — long scripts run freely).
    let start = std::time::Instant::now();
    let mut timed_out = false;
    let exit_status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if let Some(limit) = timeout_secs {
                    if limit > 0 && start.elapsed().as_secs() >= limit {
                        let _ = kill_process_tree(pid);
                        let _ = child.wait();
                        timed_out = true;
                        break None;
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => {
                let _ = child.wait();
                if let Ok(mut processes) = running_script_pids().lock() {
                    processes.remove(script_name);
                }
                return Err(format!("Failed to execute script: {}", e));
            }
        }
    };

    {
        if let Ok(mut processes) = running_script_pids().lock() {
            processes.remove(script_name);
        }
    }

    let stdout = stdout_reader
        .map(|h| h.join().unwrap_or_default())
        .unwrap_or_default();
    let stderr = stderr_reader
        .map(|h| h.join().unwrap_or_default())
        .unwrap_or_default();

    // Clean up temp file if it was created
    if let Some(temp) = temp_file {
        let _ = std::fs::remove_file(temp); // Ignore errors on cleanup
    }

    // Save logs
    if let Some(script_name) = script_path.file_stem() {
        let log_path = script_path
            .parent()
            .unwrap()
            .join(format!("{}.log", script_name.to_string_lossy()));
        if let Ok(mut file) = File::create(&log_path) {
            let cwd = script_path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<unknown>".to_string());
            let _ = writeln!(file, "=== Script Output ===\n{}\n", stdout);
            let _ = writeln!(file, "=== Errors ===\n{}\n", stderr);
            let _ = writeln!(
                file,
                "=== Execution Context ===\npython_exec: {}\nscript_path: {}\ncwd: {}\nos: {}\n",
                python_exec.display(),
                script_to_execute.display(),
                cwd,
                std::env::consts::OS
            );
            let status_label = if timed_out {
                "Timed out"
            } else if exit_status.map(|s| s.success()).unwrap_or(false) {
                "Success"
            } else {
                "Failed"
            };
            let _ = writeln!(file, "=== Status ===\n{}\n", status_label);
        }
    }

    // Combine stdout and stderr for display: many scripts print errors to stdout
    // (not stderr), so discarding stdout on failure would hide all error context.
    let combined = {
        let s = stdout.trim_end();
        let e = stderr.trim_end();
        match (s.is_empty(), e.is_empty()) {
            (false, false) => format!("{}\n{}", s, e),
            (false, true) => stdout.clone(),
            (true, false) => stderr.clone(),
            (true, true) => String::new(),
        }
    };

    if timed_out {
        let base = format!(
            "Script timed out after {}s and was terminated (python: {}).",
            timeout_secs.unwrap_or(0),
            python_exec.display()
        );
        return Err(if combined.is_empty() {
            base
        } else {
            format!("{}\n{}", base, combined)
        });
    }

    let status = exit_status.expect("exit status is present when the script did not time out");

    if status.success() {
        // On success also return combined so stderr warnings are visible
        if combined.is_empty() {
            Ok(stdout)
        } else {
            Ok(combined)
        }
    } else {
        let exit_detail = if let Some(code) = status.code() {
            format!("exit code {}", code)
        } else {
            #[cfg(unix)]
            {
                if let Some(signal) = status.signal() {
                    format!("terminated by signal {}", signal)
                } else {
                    "unknown process termination".to_string()
                }
            }

            #[cfg(not(unix))]
            {
                "unknown process termination".to_string()
            }
        };

        if combined.is_empty() {
            let message = {
                let base = format!("Script failed ({}) with no output captured", exit_detail);

                #[cfg(unix)]
                {
                    if status.signal() == Some(6) {
                        format!(
                            "{}. Process aborted (SIGABRT) before writing stdout/stderr; this often indicates a Python runtime/bootstrap crash on macOS.",
                            base
                        )
                    } else {
                        base
                    }
                }

                #[cfg(not(unix))]
                {
                    base
                }
            };

            Err(message)
        } else {
            let mut message = format!(
                "Script failed (python: {}):\n{}",
                python_exec.display(),
                combined
            );

            if combined
                .contains("confidence keyword argument is only available if OpenCV is installed")
            {
                message.push_str(
                    "\n\nHint: This runtime is missing OpenCV. Install `opencv-python` in the same Python used by Script Runner.",
                );
            }

            if combined.contains("pyautogui")
                && (combined.contains("screenshot")
                    || combined.contains("Screen")
                    || combined.contains("capture"))
            {
                message.push_str(
                    "\n\nHint: Image search/screenshot can fail when app-level screen permissions are missing. Verify Screen Recording/Accessibility permissions for Script Runner (and for its Python runtime).",
                );
            }

            Err(message)
        }
    }
}

fn kill_process_tree(pid: u32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let status = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .status()
            .map_err(|e| format!("Failed to invoke taskkill: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("taskkill failed with status {}", status))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let parent_status = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()
            .map_err(|e| format!("Failed to invoke kill: {}", e))?;

        if parent_status.success() {
            Ok(())
        } else {
            Err(format!("kill failed with status {}", parent_status))
        }
    }
}

pub fn stop_script_execution(script_name: &str) -> Result<String, String> {
    let pid = {
        let processes = running_script_pids()
            .lock()
            .map_err(|_| "Failed to lock running script process map".to_string())?;
        processes.get(script_name).copied()
    };

    let Some(pid) = pid else {
        return Err(format!("Script '{}' is not running", script_name));
    };

    kill_process_tree(pid)?;

    if let Ok(mut processes) = running_script_pids().lock() {
        processes.remove(script_name);
    }

    Ok(format!("Stopped script '{}'", script_name))
}
