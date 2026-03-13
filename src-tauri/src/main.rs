#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use chrono::TimeZone;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::State;
use tauri_plugin_updater::UpdaterExt;

#[cfg(target_os = "windows")]
use walkdir::WalkDir;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

mod analytics;
mod dependency_manager;
mod git_manager;
mod github_auth;
mod kill_switch;
mod kill_switch_manager;
mod python_runner;
mod run_history;
mod script_encryption;
mod script_manager;
mod settings;

#[derive(Clone)]
pub struct AppState {
    scripts_dir: PathBuf,
    python_exec: PathBuf,
}

const DEFAULT_RELEASES_REPO: &str = "PatrykEmilLibert/script-runner";
const DEFAULT_UPDATER_ENDPOINT: &str =
    "https://github.com/PatrykEmilLibert/script-runner/releases/latest/download/latest.json";
const COMPILED_RELEASES_REPO: Option<&str> = option_env!("SR_RELEASES_REPO");
const COMPILED_UPDATER_ENDPOINT: Option<&str> = option_env!("SR_UPDATER_ENDPOINT");
const COMPILED_UPDATER_PUBKEY: Option<&str> = option_env!("SR_UPDATER_PUBKEY");

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, serde::Serialize)]
struct AvailableUpdateInfo {
    version: String,
    current_version: String,
    notes: Option<String>,
    pub_date: Option<String>,
    download_url: String,
}

fn read_non_empty_env(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(target_os = "windows")]
fn apply_no_console_window(cmd: &mut Command) {
    cmd.creation_flags(CREATE_NO_WINDOW);
}

fn resolve_script_directory(
    scripts_root: &Path,
    script_name: &str,
    subdir: Option<&str>,
) -> Result<PathBuf, String> {
    let user_dir = script_manager::get_user_script_dir(scripts_root, script_name);
    let official_dir = scripts_root.join("official").join(script_name);

    match subdir {
        Some("scripts") => {
            if user_dir.exists() {
                Ok(user_dir)
            } else {
                Err(format!("User script not found: {}", script_name))
            }
        }
        Some("official") => {
            if official_dir.exists() {
                Ok(official_dir)
            } else {
                Err(format!("Official script not found: {}", script_name))
            }
        }
        _ => {
            if user_dir.exists() {
                Ok(user_dir)
            } else if official_dir.exists() {
                Ok(official_dir)
            } else {
                Err(format!("Script not found: {}", script_name))
            }
        }
    }
}

fn releases_repo() -> String {
    if let Some(runtime) = read_non_empty_env("SR_RELEASES_REPO") {
        return runtime;
    }

    if let Some(compiled) = COMPILED_RELEASES_REPO {
        let trimmed = compiled.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    DEFAULT_RELEASES_REPO.to_string()
}

fn releases_latest_api_url() -> String {
    format!(
        "https://api.github.com/repos/{}/releases/latest",
        releases_repo()
    )
}

fn releases_page_url() -> String {
    format!("https://github.com/{}/releases", releases_repo())
}

fn parse_version_segments(version: &str) -> Vec<u32> {
    version
        .trim()
        .trim_start_matches('v')
        .split('.')
        .map(|part| {
            let digits: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse::<u32>().unwrap_or(0)
        })
        .collect()
}

fn is_newer_version(latest: &str, current: &str) -> bool {
    let latest_parts = parse_version_segments(latest);
    let current_parts = parse_version_segments(current);
    let max_len = latest_parts.len().max(current_parts.len());

    for index in 0..max_len {
        let latest_part = latest_parts.get(index).copied().unwrap_or(0);
        let current_part = current_parts.get(index).copied().unwrap_or(0);

        if latest_part > current_part {
            return true;
        }
        if latest_part < current_part {
            return false;
        }
    }

    false
}

fn updater_endpoint() -> String {
    if let Some(runtime) = read_non_empty_env("SR_UPDATER_ENDPOINT") {
        return runtime;
    }

    if let Some(compiled) = COMPILED_UPDATER_ENDPOINT {
        let trimmed = compiled.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    DEFAULT_UPDATER_ENDPOINT.to_string()
}

fn updater_pubkey() -> Result<String, String> {
    if let Some(runtime) = read_non_empty_env("SR_UPDATER_PUBKEY") {
        return Ok(runtime);
    }

    if let Some(compiled) = COMPILED_UPDATER_PUBKEY {
        let trimmed = compiled.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    Err(
        "Updater public key is not configured. Set SR_UPDATER_PUBKEY for build/runtime."
            .to_string(),
    )
}

async fn check_runtime_update(
    app: tauri::AppHandle,
) -> Result<Option<tauri_plugin_updater::Update>, String> {
    let endpoint = updater_endpoint();
    let endpoint_url = url::Url::parse(&endpoint)
        .map_err(|e| format!("Invalid updater endpoint '{}': {}", endpoint, e))?;
    let pubkey = updater_pubkey()?;

    let updater = app
        .updater_builder()
        .pubkey(pubkey)
        .endpoints(vec![endpoint_url])
        .map_err(|e| format!("Failed to configure updater endpoint: {}", e))?
        .build()
        .map_err(|e| format!("Failed to initialize updater: {}", e))?;

    updater
        .check()
        .await
        .map_err(|e| format!("Failed to check for updates: {}", e))
}

#[tauri::command]
async fn read_file_content(file_path: String) -> Result<String, String> {
    use std::fs;
    fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file {}: {}", file_path, e))
}

#[tauri::command]
async fn send_desktop_notification(
    title: String,
    body: String,
    _notification_type: String,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let ps_script = format!(
            r#"
            [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
            [Windows.UI.Notifications.ToastNotification, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
            [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null

            $template = @"
            <toast>
                <visual>
                    <binding template=\"ToastGeneric\">
                        <text>{}</text>
                        <text>{}</text>
                    </binding>
                </visual>
            </toast>
"@

            $xml = New-Object Windows.Data.Xml.Dom.XmlDocument
            $xml.LoadXml($template)
            $toast = New-Object Windows.UI.Notifications.ToastNotification $xml
            [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier(\"ScriptRunner\").Show($toast)
            \"#,
            title.replace('"', "\\\""),
            body.replace('"', "\\\"")
        );

        let mut cmd = Command::new("powershell");
        cmd.args(["-NoProfile", "-Command", &ps_script]);
        apply_no_console_window(&mut cmd);

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;

        if !output.status.success() {
            log::warn!("PowerShell notification failed, output: {:?}", output);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("osascript")
            .args([
                "-e",
                &format!(
                    r#"display notification "{}" with title "{}""#,
                    body.replace('"', r#"\""#),
                    title.replace('"', r#"\""#)
                ),
            ])
            .output()
            .map_err(|e| format!("Failed to send macOS notification: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("notify-send")
            .args([&title, &body])
            .output()
            .map_err(|e| format!("Failed to send Linux notification: {}", e))?;
    }

    Ok(())
}

fn resolve_scripts_dir() -> PathBuf {
    if let Ok(custom) = env::var("SR_SCRIPTS_DIR") {
        let custom_path = PathBuf::from(custom);
        if is_directory_writable(&custom_path) {
            log::info!("Using SR_SCRIPTS_DIR for scripts: {:?}", custom_path);
            return custom_path;
        }

        log::warn!(
            "SR_SCRIPTS_DIR is not writable, ignoring custom path: {:?}",
            custom_path
        );
    }

    if let Some(install_adjacent) = resolve_install_adjacent_scripts_dir() {
        if is_directory_writable(&install_adjacent) {
            log::info!(
                "Using install-adjacent directory for scripts: {:?}",
                install_adjacent
            );
            return install_adjacent;
        }

        log::warn!(
            "Install-adjacent scripts path is not writable: {:?}",
            install_adjacent
        );
    }

    // Use user data directory (persistent across updates)
    if let Some(data_dir) = dirs::data_dir() {
        let app_data = data_dir.join("ScriptRunner").join("scripts");
        if is_directory_writable(&app_data) {
            log::info!("Using user data directory for scripts: {:?}", app_data);
            return app_data;
        }

        log::warn!(
            "User data scripts directory is not writable, skipping: {:?}",
            app_data
        );
    }

    // Final fallback (should rarely happen)
    let local_fallback = PathBuf::from("./scripts");
    log::warn!(
        "Falling back to local scripts path (writability not guaranteed): {:?}",
        local_fallback
    );
    local_fallback
}

fn resolve_install_adjacent_scripts_dir() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|p| p.to_path_buf()))?;

    Some(exe_dir.join("ScriptRunnerData").join("scripts"))
}

fn is_directory_writable(path: &PathBuf) -> bool {
    if let Err(e) = fs::create_dir_all(path) {
        log::warn!("Failed to create scripts directory {:?}: {}", path, e);
        return false;
    }

    let probe = path.join(".sr_write_test");
    match fs::write(&probe, b"ok") {
        Ok(_) => {
            let _ = fs::remove_file(&probe);
            true
        }
        Err(e) => {
            log::warn!("Scripts directory is not writable {:?}: {}", path, e);
            false
        }
    }
}

#[cfg(target_os = "windows")]
fn is_path_writable(path: &Path) -> bool {
    if let Err(e) = fs::create_dir_all(path) {
        log::warn!("Failed to create directory {:?}: {}", path, e);
        return false;
    }

    let probe = path.join(".sr_write_test");
    match fs::write(&probe, b"ok") {
        Ok(_) => {
            let _ = fs::remove_file(&probe);
            true
        }
        Err(e) => {
            log::warn!("Directory is not writable {:?}: {}", path, e);
            false
        }
    }
}

#[cfg(target_os = "windows")]
fn infer_python_runtime_root(python_exec: &Path) -> Option<PathBuf> {
    let parent = python_exec.parent()?;
    if parent
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.eq_ignore_ascii_case("Scripts"))
        .unwrap_or(false)
    {
        return parent.parent().map(|p| p.to_path_buf());
    }

    Some(parent.to_path_buf())
}

#[cfg(target_os = "windows")]
fn copy_directory_recursive(source: &Path, target: &Path) -> Result<(), String> {
    for entry in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative = path
            .strip_prefix(source)
            .map_err(|e| format!("Failed to resolve relative path during copy: {}", e))?;
        let destination = target.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&destination)
                .map_err(|e| format!("Failed to create directory {:?}: {}", destination, e))?;
        } else {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create parent directory {:?}: {}", parent, e)
                })?;
            }
            fs::copy(path, &destination).map_err(|e| {
                format!(
                    "Failed to copy file from {:?} to {:?}: {}",
                    path, destination, e
                )
            })?;
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn resolve_writable_python_exec(candidate: &PathBuf) -> PathBuf {
    let Some(runtime_root) = infer_python_runtime_root(candidate) else {
        return candidate.clone();
    };

    if is_path_writable(&runtime_root) {
        return candidate.clone();
    }

    let data_root = dirs::data_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("ScriptRunner")
        .join("python-runtime")
        .join(env!("CARGO_PKG_VERSION"));

    let relative_exec = match candidate.strip_prefix(&runtime_root) {
        Ok(relative) => relative.to_path_buf(),
        Err(_) => PathBuf::from("python.exe"),
    };

    let writable_exec = data_root.join(&relative_exec);

    if writable_exec.exists() && is_python_exec_usable(&writable_exec) {
        log::info!(
            "Using writable per-user Python runtime: {}",
            writable_exec.display()
        );
        return writable_exec;
    }

    log::warn!(
        "Bundled Python runtime is read-only at {}. Copying runtime to {}",
        runtime_root.display(),
        data_root.display()
    );

    if data_root.exists() {
        let _ = fs::remove_dir_all(&data_root);
    }

    match copy_directory_recursive(&runtime_root, &data_root) {
        Ok(_) => {
            if is_python_exec_usable(&writable_exec) {
                log::info!(
                    "Using copied writable Python runtime: {}",
                    writable_exec.display()
                );
                writable_exec
            } else {
                log::warn!(
                    "Copied Python runtime is not usable, falling back to original runtime: {}",
                    candidate.display()
                );
                candidate.clone()
            }
        }
        Err(e) => {
            log::warn!(
                "Failed to prepare writable Python runtime copy: {}. Falling back to {}",
                e,
                candidate.display()
            );
            candidate.clone()
        }
    }
}

fn is_python_exec_usable(candidate: &PathBuf) -> bool {
    let mut cmd = Command::new(candidate);
    cmd.arg("--version");

    #[cfg(target_os = "windows")]
    {
        apply_no_console_window(&mut cmd);
    }

    match cmd.output() {
        Ok(output) if output.status.success() => true,
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!(
                "Ignoring unusable Python executable {}: {}",
                candidate.display(),
                stderr.trim()
            );
            false
        }
        Err(e) => {
            log::warn!(
                "Ignoring unusable Python executable {}: {}",
                candidate.display(),
                e
            );
            false
        }
    }
}

fn is_python_runtime_healthy(candidate: &PathBuf) -> bool {
    let modules = if cfg!(target_os = "windows") || cfg!(target_os = "macos") {
        "ssl,sqlite3,venv,ensurepip,pip,tkinter"
    } else {
        "ssl,sqlite3,venv,ensurepip,pip"
    };

    let healthcheck_code = format!(
        "import importlib.util, sys; mods='{modules}'.split(','); missing=[m for m in mods if importlib.util.find_spec(m) is None]; sys.exit(0 if not missing else (print('missing:' + ','.join(missing)) or 1))"
    );

    let mut cmd = Command::new(candidate);
    cmd.args(["-c", &healthcheck_code]);

    #[cfg(target_os = "windows")]
    {
        apply_no_console_window(&mut cmd);
    }

    match cmd.output() {
        Ok(output) if output.status.success() => true,
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            log::warn!(
                "Python runtime healthcheck failed for {}: {} {}",
                candidate.display(),
                stdout.trim(),
                stderr.trim()
            );
            false
        }
        Err(e) => {
            log::warn!(
                "Failed to run Python runtime healthcheck for {}: {}",
                candidate.display(),
                e
            );
            false
        }
    }
}

fn isolated_python_exec_path(venv_dir: &Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        return venv_dir.join("Scripts").join("python.exe");
    }

    #[cfg(not(target_os = "windows"))]
    {
        let py3 = venv_dir.join("bin").join("python3");
        if py3.exists() {
            return py3;
        }
        venv_dir.join("bin").join("python")
    }
}

fn ensure_isolated_python_exec(base_python: &PathBuf) -> PathBuf {
    let venv_dir = dirs::data_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("ScriptRunner")
        .join("python-venv")
        .join(env!("CARGO_PKG_VERSION"));

    let venv_python = isolated_python_exec_path(&venv_dir);
    if venv_python.exists() && is_python_exec_usable(&venv_python) {
        return venv_python;
    }

    if let Some(parent) = venv_dir.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if venv_dir.exists() {
        let _ = fs::remove_dir_all(&venv_dir);
    }

    let mut cmd = Command::new(base_python);
    cmd.args(["-m", "venv", &venv_dir.to_string_lossy()]);

    #[cfg(target_os = "windows")]
    {
        apply_no_console_window(&mut cmd);
    }

    match cmd.output() {
        Ok(output) if output.status.success() => {
            if venv_python.exists() && is_python_exec_usable(&venv_python) {
                log::info!(
                    "Using isolated app Python environment: {}",
                    venv_python.display()
                );
                venv_python
            } else {
                log::warn!(
                    "Isolated app Python environment creation completed but interpreter not usable, falling back to {}",
                    base_python.display()
                );
                base_python.clone()
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!(
                "Failed to create isolated app Python environment (exit: {:?}): {}. Falling back to {}",
                output.status.code(),
                stderr.trim(),
                base_python.display()
            );
            base_python.clone()
        }
        Err(e) => {
            log::warn!(
                "Failed to launch venv creation using {}: {}. Falling back to base interpreter",
                base_python.display(),
                e
            );
            base_python.clone()
        }
    }
}

fn resolve_python_exec() -> PathBuf {
    if let Ok(custom) = env::var("PYTHON_EXEC") {
        return PathBuf::from(custom);
    }

    let mut candidates: Vec<PathBuf> = Vec::new();

    #[cfg(target_os = "windows")]
    {
        candidates.push(PathBuf::from("./python/python.exe"));
        candidates.push(PathBuf::from("./python/Scripts/python.exe"));

        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                candidates.push(exe_dir.join("python").join("python.exe"));
                candidates.push(exe_dir.join("python").join("Scripts").join("python.exe"));
                candidates.push(exe_dir.join("resources").join("python").join("python.exe"));
                candidates.push(
                    exe_dir
                        .join("resources")
                        .join("python")
                        .join("Scripts")
                        .join("python.exe"),
                );
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        candidates.push(PathBuf::from("./python/bin/python3.12"));
        candidates.push(PathBuf::from("./python/bin/python"));
        candidates.push(PathBuf::from("./python/bin/python3"));

        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                candidates.push(exe_dir.join("python").join("bin").join("python3.12"));
                candidates.push(exe_dir.join("python").join("bin").join("python"));
                candidates.push(exe_dir.join("python").join("bin").join("python3"));

                if let Some(contents_dir) = exe_dir.parent() {
                    let resources_dir = contents_dir.join("Resources");
                    candidates.push(resources_dir.join("python").join("bin").join("python3.12"));
                    candidates.push(resources_dir.join("python").join("bin").join("python"));
                    candidates.push(resources_dir.join("python").join("bin").join("python3"));
                }
            }
        }
    }

    for candidate in candidates {
        if candidate.exists() && is_python_exec_usable(&candidate) {
            #[cfg(target_os = "windows")]
            {
                let selected = resolve_writable_python_exec(&candidate);
                if is_python_runtime_healthy(&selected) {
                    let isolated = ensure_isolated_python_exec(&selected);
                    log::info!("Using bundled Python runtime: {}", selected.display());
                    return isolated;
                }

                log::warn!(
                    "Bundled Python runtime failed healthcheck: {}",
                    selected.display()
                );
                continue;
            }

            #[cfg(not(target_os = "windows"))]
            {
                if is_python_runtime_healthy(&candidate) {
                    let isolated = ensure_isolated_python_exec(&candidate);
                    log::info!("Using bundled Python runtime: {}", candidate.display());
                    return isolated;
                }

                log::warn!(
                    "Bundled Python runtime failed healthcheck: {}",
                    candidate.display()
                );
                continue;
            }
        }
    }

    let fallback_candidates = if cfg!(target_os = "windows") {
        vec![PathBuf::from("python"), PathBuf::from("py")]
    } else {
        vec![PathBuf::from("python3"), PathBuf::from("python")]
    };

    for fallback in &fallback_candidates {
        if is_python_exec_usable(fallback) && is_python_runtime_healthy(fallback) {
            log::warn!(
                "Bundled Python runtime not found/usable, falling back to system executable: {}",
                fallback.display()
            );
            return ensure_isolated_python_exec(fallback);
        }
    }

    let fallback = fallback_candidates
        .into_iter()
        .next()
        .unwrap_or_else(|| PathBuf::from("python"));

    log::warn!(
        "No usable Python runtime found, returning default executable name: {}",
        fallback.display()
    );
    fallback
}

fn extract_missing_module_name(python_error: &str) -> Option<String> {
    for line in python_error.lines() {
        let marker = "No module named ";
        if let Some(pos) = line.find(marker) {
            let tail = line[pos + marker.len()..].trim();

            if let Some(stripped) = tail.strip_prefix('\'') {
                if let Some(end) = stripped.find('\'') {
                    let module = stripped[..end].trim();
                    if !module.is_empty() {
                        return Some(module.to_string());
                    }
                }
            }

            if let Some(stripped) = tail.strip_prefix('"') {
                if let Some(end) = stripped.find('"') {
                    let module = stripped[..end].trim();
                    if !module.is_empty() {
                        return Some(module.to_string());
                    }
                }
            }

            let module = tail.trim_matches(['\'', '"', '.']);
            if !module.is_empty() {
                return Some(module.to_string());
            }
        }
    }

    None
}

fn extract_recoverable_dependency(python_error: &str) -> Option<String> {
    if let Some(module) = extract_missing_module_name(python_error) {
        return Some(module);
    }

    let lower = python_error.to_lowercase();

    // pyautogui/pyscreeze requires OpenCV for confidence-based locate APIs.
    if lower.contains("confidence keyword argument is only available if opencv is installed")
        || (lower.contains("pyautogui") && lower.contains("confidence") && lower.contains("opencv"))
    {
        return Some("opencv-python".to_string());
    }

    None
}

#[tauri::command]
async fn check_kill_switch() -> Result<bool, String> {
    // Check local override first
    if let Some(allow) = kill_switch_manager::check_local_override() {
        log::warn!("Kill switch local override active: allow={}", allow);
        return Ok(!allow); // Invert because function returns "is_blocked"
    }

    kill_switch::check_remote_status().await
}

#[tauri::command]
async fn check_kill_switch_status() -> Result<kill_switch::KillSwitchConfig, String> {
    kill_switch::check_remote_status_advanced().await
}

#[tauri::command]
async fn get_kill_switch_status() -> Result<kill_switch::KillSwitchConfig, String> {
    // Alias for backward compatibility
    check_kill_switch_status().await
}

#[tauri::command]
async fn require_github_admin() -> Result<(), String> {
    match github_auth::check_admin_status().await {
        Ok(true) => Ok(()),
        Ok(false) => Err("Operation requires GitHub admin login".to_string()),
        Err(e) => Err(format!("Failed to verify GitHub admin status: {}", e)),
    }
}

#[tauri::command]
async fn toggle_kill_switch(enabled: bool, reason: String) -> Result<String, String> {
    require_github_admin().await?;
    kill_switch_manager::toggle_kill_switch(enabled, reason).await
}

#[tauri::command]
async fn schedule_kill_switch(scheduled_for: String, reason: String) -> Result<String, String> {
    require_github_admin().await?;

    let until = match chrono::DateTime::parse_from_rfc3339(&scheduled_for) {
        Ok(dt) => dt.to_rfc3339(),
        Err(_) => {
            let naive = chrono::NaiveDateTime::parse_from_str(&scheduled_for, "%Y-%m-%dT%H:%M")
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(&scheduled_for, "%Y-%m-%dT%H:%M:%S")
                })
                .map_err(|e| format!("Invalid scheduled date format: {}", e))?;

            let local_dt = chrono::Local
                .from_local_datetime(&naive)
                .single()
                .ok_or("Invalid local scheduled datetime")?;

            local_dt.with_timezone(&chrono::Utc).to_rfc3339()
        }
    };

    kill_switch_manager::schedule_block(until, reason).await
}

#[tauri::command]
async fn add_to_whitelist(item: String) -> Result<String, String> {
    require_github_admin().await?;
    kill_switch_manager::add_to_whitelist(item).await
}

#[tauri::command]
async fn remove_from_whitelist(item: String) -> Result<String, String> {
    require_github_admin().await?;
    kill_switch_manager::remove_from_whitelist(item).await
}

#[tauri::command]
fn get_current_machine_id() -> Result<String, String> {
    Ok(kill_switch::get_machine_id())
}

#[tauri::command]
async fn set_kill_switch_message(
    message: String,
    redirect_url: Option<String>,
) -> Result<String, String> {
    require_github_admin().await?;
    kill_switch_manager::set_custom_message(message, redirect_url).await
}

#[tauri::command]
async fn create_kill_switch_override(allow: bool) -> Result<String, String> {
    require_github_admin().await?;
    kill_switch_manager::create_local_override(allow)
}

#[tauri::command]
async fn sync_scripts(state: State<'_, AppState>) -> Result<String, String> {
    let res = git_manager::sync_scripts(&state.scripts_dir)?;

    let encrypt_enabled = env::var("SR_ENCRYPT_OFFICIAL_LOCAL")
        .map(|v| v != "0")
        .unwrap_or(true);

    if encrypt_enabled {
        match script_encryption::encrypt_official_scripts(&state.scripts_dir) {
            Ok(count) if count > 0 => {
                log::info!("Encrypted {} official scripts for local protection", count)
            }
            Ok(_) => {}
            Err(e) => log::warn!("Failed to encrypt official scripts locally: {}", e),
        }
    } else {
        log::info!("Local official script encryption disabled by SR_ENCRYPT_OFFICIAL_LOCAL");
    }

    dependency_manager::ensure_all_scripts_requirements(&state.scripts_dir, &state.python_exec)
        .await?;
    Ok(res)
}

#[tauri::command]
async fn run_script(
    script_name: String,
    args: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let start_time = chrono::Utc::now();
    let script_dir = resolve_script_directory(&state.scripts_dir, &script_name, None)?;
    let official_dir_path = state.scripts_dir.join("official").join(&script_name);

    // Check for encrypted or plain script
    let main_enc = script_dir.join("main.py.enc");
    let main_py = script_dir.join("main.py");

    let script_path = if main_enc.exists() {
        main_enc.clone()
    } else if main_py.exists() {
        main_py.clone()
    } else {
        return Err(format!("No script file found in {}", script_name));
    };

    // Install deps from requirements.txt if present (cached by hash)
    dependency_manager::ensure_requirements(&script_dir, &state.python_exec).await?;

    // Fallback: auto-detect imports when no requirements.txt exists
    // For encrypted scripts, decrypt to analyze dependencies
    if !script_dir.join("requirements.txt").exists() {
        let content = if main_enc.exists() {
            script_encryption::decrypt_script(&main_enc)?
        } else {
            std::fs::read_to_string(&main_py)
                .map_err(|e| format!("Failed to read script: {}", e))?
        };

        // Write temp file for analysis
        let temp_analysis =
            std::env::temp_dir().join(format!("sr_analyze_{}.py", uuid::Uuid::new_v4()));
        std::fs::write(&temp_analysis, content)
            .map_err(|e| format!("Failed to write temp analysis file: {}", e))?;

        let deps = dependency_manager::detect_dependencies(&temp_analysis).await?;
        dependency_manager::install_dependencies(&deps, &state.python_exec).await?;

        let _ = std::fs::remove_file(temp_analysis); // Cleanup
    }

    // Run script and iteratively self-heal missing dependency errors.
    let mut result =
        python_runner::execute_script(&script_name, &script_path, &state.python_exec, args.clone())
            .await;
    let mut attempted_modules: HashSet<String> = HashSet::new();
    const MAX_AUTO_INSTALL_ATTEMPTS: usize = 20;

    for _ in 0..MAX_AUTO_INSTALL_ATTEMPTS {
        let missing_module = match &result {
            Ok(_) => None,
            Err(error_text) => extract_recoverable_dependency(error_text),
        };

        let Some(module_name) = missing_module else {
            break;
        };

        if !attempted_modules.insert(module_name.clone()) {
            log::warn!(
                "Detected repeated missing module '{}' while running '{}'. Stopping auto-install loop.",
                module_name,
                script_name
            );
            break;
        }

        log::warn!(
            "Detected missing module '{}' while running '{}'. Installing and retrying.",
            module_name,
            script_name
        );

        if dependency_manager::install_dependencies(&[module_name], &state.python_exec)
            .await
            .is_err()
        {
            break;
        }

        result = python_runner::execute_script(
            &script_name,
            &script_path,
            &state.python_exec,
            args.clone(),
        )
        .await;
    }
    let end_time = chrono::Utc::now();
    let duration = end_time.signed_duration_since(start_time);

    let (status, error) = match &result {
        Ok(_) => ("success".to_string(), None),
        Err(e) => ("error".to_string(), Some(e.clone())),
    };

    let record = run_history::RunRecord {
        id: uuid::Uuid::new_v4().to_string(),
        script_name: script_name.clone(),
        start_time: start_time.to_rfc3339(),
        end_time: end_time.to_rfc3339(),
        duration_ms: duration.num_milliseconds() as u64,
        status,
        output: result.as_ref().unwrap_or(&String::new()).clone(),
        error,
    };

    let _ = run_history::add_record(record);

    // Track analytics
    let is_official = official_dir_path.exists() && script_dir == official_dir_path;
    let category = script_name
        .split('_')
        .next()
        .unwrap_or("uncategorized")
        .to_string();

    let _ = analytics::track_execution(
        script_name.clone(),
        duration.num_milliseconds() as u64,
        result.is_ok(),
        Some(category),
        Some(is_official),
    );

    result
}

#[tauri::command]
fn stop_script(script_name: String) -> Result<String, String> {
    python_runner::stop_script_execution(&script_name)
}

#[tauri::command]
async fn update_all_dependencies(state: State<'_, AppState>) -> Result<(), String> {
    dependency_manager::ensure_all_scripts_requirements(&state.scripts_dir, &state.python_exec)
        .await
}

#[tauri::command]
async fn encrypt_official_script(
    script_name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let official_dir = state.scripts_dir.join("official").join(&script_name);
    let main_py = official_dir.join("main.py");

    if !main_py.exists() {
        return Err("Script not found".to_string());
    }

    script_encryption::encrypt_script(&main_py)?;
    Ok(format!("Script {} encrypted successfully", script_name))
}

#[tauri::command]
fn list_scripts(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let scripts_dir = state.scripts_dir.to_string_lossy().to_string();
    let mut scripts = Vec::new();

    scripts.extend(
        script_manager::get_local_scripts(scripts_dir.clone(), Some("official".to_string()))?
            .into_iter()
            .map(|s| s.name),
    );

    scripts.extend(
        script_manager::get_local_scripts(scripts_dir, Some("scripts".to_string()))?
            .into_iter()
            .map(|s| s.name),
    );

    scripts.sort();
    scripts.dedup();
    Ok(scripts)
}

#[tauri::command]
fn check_script_compatibility(
    script_name: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let user_dir = script_manager::get_user_script_dir(&state.scripts_dir, &script_name);
    let official_dir = state.scripts_dir.join("official").join(&script_name);

    let script_dir = if user_dir.exists() {
        user_dir
    } else {
        official_dir
    };

    // Check for encrypted or plain script
    let main_py = script_dir.join("main.py");
    let main_enc = script_dir.join("main.py.enc");

    let content = if main_enc.exists() {
        script_encryption::decrypt_script(&main_enc)?
    } else if main_py.exists() {
        std::fs::read_to_string(&main_py).map_err(|e| format!("Failed to read script: {}", e))?
    } else {
        return Err("Script not found".to_string());
    };

    python_runner::check_platform_compatibility(&content)
}

#[tauri::command]
async fn get_script_logs(
    script_name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let user_log =
        script_manager::get_user_script_dir(&state.scripts_dir, &script_name).join("main.log");

    let official_log = state
        .scripts_dir
        .join("official")
        .join(&script_name)
        .join("main.log");

    let log_path = if user_log.exists() {
        user_log
    } else {
        official_log
    };

    std::fs::read_to_string(log_path).map_err(|e| format!("Failed to read logs: {}", e))
}

// GitHub Authentication Commands

#[tauri::command]
async fn github_login(token: String) -> Result<github_auth::AuthSession, String> {
    github_auth::github_login(token).await
}

#[tauri::command]
fn github_logout() -> Result<(), String> {
    github_auth::github_logout()
}

#[tauri::command]
fn get_github_user() -> Result<Option<github_auth::GitHubUser>, String> {
    github_auth::get_current_user()
}

#[tauri::command]
async fn check_admin_status() -> Result<bool, String> {
    github_auth::check_admin_status().await
}

#[tauri::command]
async fn refresh_github_admin_status() -> Result<bool, String> {
    github_auth::refresh_admin_status().await
}

#[tauri::command]
async fn check_github_admin_status() -> Result<bool, String> {
    github_auth::check_admin_status().await
}

#[tauri::command]
fn get_run_history(limit: usize) -> Result<Vec<run_history::RunRecord>, String> {
    run_history::get_records(limit)
}

#[tauri::command]
fn export_history_as_csv(limit: usize) -> Result<String, String> {
    let records = run_history::get_records(limit)?;
    Ok(run_history::export_as_csv(&records))
}

#[tauri::command]
fn toggle_dark_mode() -> Result<bool, String> {
    settings::toggle_dark_mode()
}

#[tauri::command]
fn get_settings() -> Result<settings::AppSettings, String> {
    settings::load_settings()
}

#[tauri::command]
fn set_auto_update(enabled: bool) -> Result<bool, String> {
    settings::set_auto_update(enabled)
}

#[tauri::command]
fn get_scripts_dir(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.scripts_dir.to_string_lossy().to_string())
}

#[tauri::command]
fn open_scripts_root_folder(state: State<'_, AppState>) -> Result<String, String> {
    let path = state.scripts_dir.clone();
    if !path.exists() {
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create scripts root directory: {}", e))?;
    }

    open_path_in_file_manager(&path)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn open_script_folder(
    script_name: String,
    subdir: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let script_dir = resolve_script_directory(&state.scripts_dir, &script_name, subdir.as_deref())?;
    open_path_in_file_manager(&script_dir)?;
    Ok(script_dir.to_string_lossy().to_string())
}

fn open_path_in_file_manager(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open folder in Explorer: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open folder in Finder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open folder in file manager: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
fn get_app_instance_id() -> Result<String, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to resolve current executable path: {}", e))?;
    let normalized = exe_path.to_string_lossy().to_lowercase();

    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in normalized.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }

    Ok(format!("{:016x}", hash))
}

#[tauri::command]
async fn check_for_updates() -> Result<bool, String> {
    const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
    let latest_url = releases_latest_api_url();

    let client = reqwest::Client::new();

    match client
        .get(&latest_url)
        .header("User-Agent", "ScriptRunner")
        .send()
        .await
    {
        Ok(response) => {
            if !response.status().is_success() {
                log::warn!("GitHub API returned status: {}", response.status());
                return Ok(false);
            }

            match response.json::<serde_json::Value>().await {
                Ok(json) => {
                    if let Some(tag) = json.get("tag_name").and_then(|v| v.as_str()) {
                        let latest = tag.trim_start_matches('v');
                        log::info!(
                            "Current version: {}, Latest version: {}",
                            CURRENT_VERSION,
                            latest
                        );
                        Ok(is_newer_version(latest, CURRENT_VERSION))
                    } else {
                        Ok(false)
                    }
                }
                Err(e) => {
                    log::warn!("Failed to parse GitHub response: {}", e);
                    Ok(false)
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to fetch updates from GitHub: {}", e);
            Ok(false)
        }
    }
}

#[tauri::command]
async fn get_download_url() -> Result<String, String> {
    let latest_url = releases_latest_api_url();
    let fallback_url = releases_page_url();
    let client = reqwest::Client::new();

    match client
        .get(&latest_url)
        .header("User-Agent", "ScriptRunner")
        .send()
        .await
    {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(json) => {
                if let Some(url) = json.get("html_url").and_then(|v| v.as_str()) {
                    Ok(url.to_string())
                } else {
                    Ok(fallback_url)
                }
            }
            Err(e) => {
                log::warn!("Failed to parse GitHub response: {}", e);
                Ok(fallback_url)
            }
        },
        Err(e) => {
            log::warn!("Failed to fetch releases from GitHub: {}", e);
            Ok(fallback_url)
        }
    }
}

#[tauri::command]
async fn check_for_updates_with_details(
    app: tauri::AppHandle,
) -> Result<Option<AvailableUpdateInfo>, String> {
    let Some(update) = check_runtime_update(app).await? else {
        return Ok(None);
    };

    Ok(Some(AvailableUpdateInfo {
        version: update.version,
        current_version: update.current_version,
        notes: update.body,
        pub_date: update.date.map(|date| date.to_string()),
        download_url: update.download_url.to_string(),
    }))
}

#[tauri::command]
async fn install_update(app: tauri::AppHandle) -> Result<bool, String> {
    let Some(update) = check_runtime_update(app).await? else {
        return Ok(false);
    };

    update
        .download_and_install(|_chunk_length, _content_length| {}, || {})
        .await
        .map_err(|e| format!("Failed to download and install update: {}", e))?;

    Ok(true)
}

// Analytics commands
#[tauri::command]
fn get_analytics_data(days: Option<u32>) -> Result<analytics::AnalyticsData, String> {
    analytics::get_analytics_data(days)
}

#[tauri::command]
fn export_analytics(format: String, days: Option<u32>) -> Result<String, String> {
    analytics::export_analytics(format, days)
}

#[tauri::command]
fn clear_analytics_data() -> Result<(), String> {
    analytics::clear_analytics_data()
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            send_desktop_notification,
            check_kill_switch,
            check_kill_switch_status,
            get_kill_switch_status,
            toggle_kill_switch,
            schedule_kill_switch,
            add_to_whitelist,
            remove_from_whitelist,
            get_current_machine_id,
            set_kill_switch_message,
            create_kill_switch_override,
            sync_scripts,
            run_script,
            stop_script,
            list_scripts,
            check_script_compatibility,
            get_script_logs,
            script_manager::add_script,
            script_manager::add_official_script,
            script_manager::add_official_script_from_path,
            script_manager::delete_script,
            script_manager::get_local_scripts,
            script_manager::list_official_scripts,
            script_manager::get_all_scripts_info,
            script_manager::get_script_source,
            script_manager::replace_official_script_content,
            script_manager::update_official_script_full,
            script_manager::bulk_delete_official_scripts,
            script_manager::bulk_encrypt_official_scripts,
            script_manager::bulk_update_official_metadata,
            update_all_dependencies,
            check_admin_status,
            github_login,
            github_logout,
            get_github_user,
            refresh_github_admin_status,
            check_github_admin_status,
            get_run_history,
            export_history_as_csv,
            toggle_dark_mode,
            get_settings,
            set_auto_update,
            get_scripts_dir,
            open_scripts_root_folder,
            open_script_folder,
            get_app_instance_id,
            encrypt_official_script,
            check_for_updates,
            get_download_url,
            check_for_updates_with_details,
            install_update,
            get_analytics_data,
            export_analytics,
            clear_analytics_data,
            read_file_content
        ])
        .setup(|_app| {
            #[cfg(debug_assertions)]
            {
                env_logger::builder()
                    .filter_level(log::LevelFilter::Debug)
                    .try_init()
                    .ok();
            }
            #[cfg(not(debug_assertions))]
            {
                env_logger::builder()
                    .filter_level(log::LevelFilter::Info)
                    .try_init()
                    .ok();
            }
            Ok(())
        })
        .manage(AppState {
            scripts_dir: resolve_scripts_dir(),
            python_exec: resolve_python_exec(),
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
