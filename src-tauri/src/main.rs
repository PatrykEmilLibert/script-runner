#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::env;
use std::path::PathBuf;
use tauri::State;
use serde_json;

mod admin_key;
mod analytics;
mod dependency_manager;
mod git_manager;
mod kill_switch;
mod kill_switch_manager;
mod python_runner;
mod script_manager;
mod run_history;
mod settings;
mod script_encryption;

#[derive(Clone)]
pub struct AppState {
    scripts_dir: PathBuf,
    python_exec: PathBuf,
}

#[tauri::command]
async fn read_file_content(file_path: String) -> Result<String, String> {
    use std::fs;
    fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path, e))
}

#[tauri::command]
async fn send_desktop_notification(
    title: String,
    body: String,
    _notification_type: String,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        // Use Windows Toast Notifications via PowerShell
        let ps_script = format!(
            r#"
            [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
            [Windows.UI.Notifications.ToastNotification, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
            [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null

            $template = @"
            <toast>
                <visual>
                    <binding template="ToastGeneric">
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

        let output = Command::new("powershell")
            .args(&["-NoProfile", "-Command", &ps_script])
            .output()
            .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;

        if !output.status.success() {
            log::warn!("PowerShell notification failed, output: {:?}", output);
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        let _ = Command::new("osascript")
            .args(&[
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
        use std::process::Command;
        
        let _ = Command::new("notify-send")
            .args(&[&title, &body])
            .output()
            .map_err(|e| format!("Failed to send Linux notification: {}", e))?;
    }

    Ok(())
}

fn resolve_scripts_dir() -> PathBuf {
    if let Ok(custom) = env::var("SR_SCRIPTS_DIR") {
        let p = PathBuf::from(custom);
        if p.exists() { return p; }
    }

    // In development: prefer workspace-level repo outside src-tauri to avoid rebuild loops
    #[cfg(debug_assertions)]
    {
        let dev_candidates = [
            PathBuf::from("../../script-runner-scripts"),
            PathBuf::from("../script-runner-scripts"),
            PathBuf::from("./script-runner-scripts"),
        ];

        for c in dev_candidates.iter() {
            if c.exists() { return c.clone(); }
        }
    }

    // In production: use user data directory (persistent across updates)
    if let Some(data_dir) = dirs::data_dir() {
        let app_data = data_dir.join("ScriptRunner").join("scripts");
        log::info!("Using user data directory for scripts: {:?}", app_data);
        return app_data;
    }

    // Final fallback (should rarely happen)
    log::warn!("Could not determine data directory, using current directory");
    PathBuf::from("./script-runner-scripts")
}

fn resolve_python_exec() -> PathBuf {
    if let Ok(custom) = env::var("PYTHON_EXEC") {
        return PathBuf::from(custom);
    }
    #[cfg(target_os = "windows")]
    {
        let bundled = PathBuf::from("./python/Scripts/python.exe");
        if bundled.exists() {
            return bundled;
        }
        // Fallback to system python on PATH
        PathBuf::from("python")
    }

    #[cfg(not(target_os = "windows"))]
    {
        let bundled = PathBuf::from("./python/bin/python");
        if bundled.exists() {
            return bundled;
        }
        // Fallback to system python on PATH
        PathBuf::from("python3")
    }
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
async fn toggle_kill_switch_cmd(
    blocked: bool,
    reason: String,
    admin_key: String,
) -> Result<String, String> {
    admin_key::verify_admin_key(&admin_key)?;
    kill_switch_manager::toggle_kill_switch(blocked, reason).await
}

#[tauri::command]
async fn schedule_kill_switch(
    until: String,
    reason: String,
    admin_key: String,
) -> Result<String, String> {
    admin_key::verify_admin_key(&admin_key)?;
    kill_switch_manager::schedule_block(until, reason).await
}

#[tauri::command]
async fn add_machine_to_whitelist(
    machine_id: String,
    admin_key: String,
) -> Result<String, String> {
    admin_key::verify_admin_key(&admin_key)?;
    kill_switch_manager::add_to_whitelist(machine_id).await
}

#[tauri::command]
async fn remove_machine_from_whitelist(
    machine_id: String,
    admin_key: String,
) -> Result<String, String> {
    admin_key::verify_admin_key(&admin_key)?;
    kill_switch_manager::remove_from_whitelist(machine_id).await
}

#[tauri::command]
fn get_current_machine_id() -> Result<String, String> {
    Ok(kill_switch::get_machine_id())
}

#[tauri::command]
async fn set_kill_switch_message(
    message: String,
    redirect_url: Option<String>,
    admin_key: String,
) -> Result<String, String> {
    admin_key::verify_admin_key(&admin_key)?;
    kill_switch_manager::set_custom_message(message, redirect_url).await
}

#[tauri::command]
fn create_kill_switch_override(
    allow: bool,
    admin_key: String,
) -> Result<String, String> {
    admin_key::verify_admin_key(&admin_key)?;
    kill_switch_manager::create_local_override(allow)
}

#[tauri::command]
async fn sync_scripts(state: State<'_, AppState>) -> Result<String, String> {
    let res = git_manager::sync_scripts(&state.scripts_dir)?;
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
    // Prefer user script; fall back to official if user copy not found
    let user_dir = state.scripts_dir.join("scripts").join(&script_name);
    let official_dir_path = state.scripts_dir.join("official").join(&script_name);

    let script_dir = if user_dir.exists() {
        log::info!("Using user script: {:?}", user_dir);
        user_dir
    } else if official_dir_path.exists() {
        log::info!("Using official script: {:?}", official_dir_path);
        official_dir_path.clone()
    } else {
        let err = format!("Script not found: {}", script_name);
        log::error!("{}", err);
        return Err(err);
    };

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
        let temp_analysis = std::env::temp_dir().join(format!("sr_analyze_{}.py", uuid::Uuid::new_v4()));
        std::fs::write(&temp_analysis, content)
            .map_err(|e| format!("Failed to write temp analysis file: {}", e))?;
        
        let deps = dependency_manager::detect_dependencies(&temp_analysis).await?;
        dependency_manager::install_dependencies(&deps, &state.python_exec).await?;
        
        let _ = std::fs::remove_file(temp_analysis); // Cleanup
    }

    // Run script and capture history
    let result = python_runner::execute_script(&script_path, &state.python_exec, args).await;
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
    git_manager::list_available_scripts(&state.scripts_dir)
}

#[tauri::command]
fn check_script_compatibility(script_name: String, state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let user_dir = state.scripts_dir.join("scripts").join(&script_name);
    let official_dir = state.scripts_dir.join("official").join(&script_name);
    
    let script_dir = if user_dir.exists() { user_dir } else { official_dir };
    
    // Check for encrypted or plain script
    let main_py = script_dir.join("main.py");
    let main_enc = script_dir.join("main.py.enc");
    
    let content = if main_enc.exists() {
        script_encryption::decrypt_script(&main_enc)?
    } else if main_py.exists() {
        std::fs::read_to_string(&main_py)
            .map_err(|e| format!("Failed to read script: {}", e))?
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
    let user_log = state
        .scripts_dir
        .join("scripts")
        .join(&script_name)
        .join("main.log");

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

#[tauri::command]
fn check_admin_key() -> Result<bool, String> {
    // Resolve admin key path with env override and cross-platform Desktop detection
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(custom) = env::var("SR_ADMIN_KEY_PATH") {
        candidates.push(PathBuf::from(custom));
    }

    if let Some(desktop) = dirs::desktop_dir() {
        candidates.push(desktop.join("sr-admin.key"));
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(userprofile) = env::var("USERPROFILE") {
            candidates.push(
                PathBuf::from(userprofile)
                    .join("Desktop")
                    .join("sr-admin.key"),
            );
        }
        candidates.push(PathBuf::from("C:/Users/Public/Desktop/sr-admin.key"));
        if let Ok(appdata) = env::var("APPDATA") {
            candidates.push(
                PathBuf::from(appdata)
                    .join("script-runner")
                    .join("sr-admin.key"),
            );
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = env::var("HOME") {
            candidates.push(PathBuf::from(home).join("Desktop").join("sr-admin.key"));
        }
        candidates.push(PathBuf::from("/tmp/sr-admin.key"));
    }

    let is_valid = candidates.iter().any(|p| admin_key::validate_key_file(p));

    log::info!("Admin key check: {}", is_valid);
    Ok(is_valid)
}

#[tauri::command]
fn get_admin_key_info() -> Result<serde_json::Value, String> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(custom) = env::var("SR_ADMIN_KEY_PATH") {
        candidates.push(PathBuf::from(custom));
    }

    if let Some(desktop) = dirs::desktop_dir() {
        candidates.push(desktop.join("sr-admin.key"));
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(userprofile) = env::var("USERPROFILE") {
            candidates.push(
                PathBuf::from(userprofile)
                    .join("Desktop")
                    .join("sr-admin.key"),
            );
        }
        candidates.push(PathBuf::from("C:/Users/Public/Desktop/sr-admin.key"));
        if let Ok(appdata) = env::var("APPDATA") {
            candidates.push(
                PathBuf::from(appdata)
                    .join("script-runner")
                    .join("sr-admin.key"),
            );
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = env::var("HOME") {
            candidates.push(PathBuf::from(home).join("Desktop").join("sr-admin.key"));
        }
        candidates.push(PathBuf::from("/tmp/sr-admin.key"));
    }

    let info: Vec<serde_json::Value> = candidates
        .iter()
        .map(|p| {
            serde_json::json!({
                "path": p.to_string_lossy().to_string(),
                "exists": p.exists(),
                "valid": admin_key::validate_key_file(&p),
            })
        })
        .collect();

    Ok(serde_json::json!({ "candidates": info }))
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
fn get_scripts_dir(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.scripts_dir.to_string_lossy().to_string())
}

#[tauri::command]
async fn check_for_updates() -> Result<bool, String> {
    const CURRENT_VERSION: &str = "0.5.1";
    
    let client = reqwest::Client::new();
    
    match client
        .get("https://api.github.com/repos/YOUR_GITHUB_USERNAME/python_runner_github/releases/latest")
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
                        log::info!("Current version: {}, Latest version: {}", CURRENT_VERSION, latest);
                        Ok(latest != CURRENT_VERSION && latest > CURRENT_VERSION)
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
    let client = reqwest::Client::new();
    
    match client
        .get("https://api.github.com/repos/YOUR_GITHUB_USERNAME/python_runner_github/releases/latest")
        .header("User-Agent", "ScriptRunner")
        .send()
        .await
    {
        Ok(response) => {
            match response.json::<serde_json::Value>().await {
                Ok(json) => {
                    if let Some(url) = json.get("html_url").and_then(|v| v.as_str()) {
                        Ok(url.to_string())
                    } else {
                        Ok("https://github.com/YOUR_GITHUB_USERNAME/python_runner_github/releases".to_string())
                    }
                }
                Err(e) => {
                    log::warn!("Failed to parse GitHub response: {}", e);
                    Ok("https://github.com/YOUR_GITHUB_USERNAME/python_runner_github/releases".to_string())
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to fetch releases from GitHub: {}", e);
            Ok("https://github.com/YOUR_GITHUB_USERNAME/python_runner_github/releases".to_string())
        }
    }
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
        .invoke_handler(tauri::generate_handler![
            send_desktop_notification,
            check_kill_switch,
            check_kill_switch_status,
            get_kill_switch_status,
            toggle_kill_switch_cmd,
            schedule_kill_switch,
            add_machine_to_whitelist,
            remove_machine_from_whitelist,
            get_current_machine_id,
            set_kill_switch_message,
            create_kill_switch_override,
            sync_scripts,
            run_script,
            list_scripts,
            check_script_compatibility,
            get_script_logs,
            script_manager::add_script,
            script_manager::add_official_script,
            script_manager::add_official_script_from_path,
            script_manager::delete_script,
            script_manager::get_local_scripts,
            update_all_dependencies,
            check_admin_key,
            get_admin_key_info,
            get_run_history,
            export_history_as_csv,
            toggle_dark_mode,
            get_settings,
            get_scripts_dir,
            encrypt_official_script,
            check_for_updates,
            get_download_url,
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
