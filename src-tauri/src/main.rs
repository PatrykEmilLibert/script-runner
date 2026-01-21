#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::env;
use std::path::PathBuf;
use tauri::State;

mod admin_key;
mod dependency_manager;
mod git_manager;
mod kill_switch;
mod python_runner;
mod script_manager;
mod run_history;
mod settings;

#[derive(Clone)]
pub struct AppState {
    scripts_dir: PathBuf,
    python_exec: PathBuf,
}

fn resolve_scripts_dir() -> PathBuf {
    if let Ok(custom) = env::var("SR_SCRIPTS_DIR") {
        let p = PathBuf::from(custom);
        if p.exists() { return p; }
    }

    // Prefer workspace-level repo outside src-tauri to avoid rebuild loops
    let candidates = [
        PathBuf::from("../../script-runner-scripts"),
        PathBuf::from("../script-runner-scripts"),
        PathBuf::from("./script-runner-scripts"),
    ];

    for c in candidates.iter() {
        if c.exists() { return c.clone(); }
    }

    // Fallback to local folder (will be created if syncing/cloning happens)
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
    kill_switch::check_remote_status().await
}

#[tauri::command]
async fn sync_scripts(state: State<'_, AppState>) -> Result<String, String> {
    let res = git_manager::sync_scripts(&state.scripts_dir)?;
    dependency_manager::ensure_all_scripts_requirements(&state.scripts_dir, &state.python_exec)
        .await?;
    Ok(res)
}

#[tauri::command]
async fn run_script(script_name: String, state: State<'_, AppState>) -> Result<String, String> {
    let start_time = chrono::Utc::now();
    let script_dir = state.scripts_dir.join(&script_name);
    let main_py = script_dir.join("main.py");

    // Install deps from requirements.txt if present (cached by hash)
    dependency_manager::ensure_requirements(&script_dir, &state.python_exec).await?;

    // Fallback: auto-detect imports when no requirements.txt exists
    if !script_dir.join("requirements.txt").exists() {
        let deps = dependency_manager::detect_dependencies(&main_py).await?;
        dependency_manager::install_dependencies(&deps, &state.python_exec).await?;
    }

    // Run script and capture history
    let result = python_runner::execute_script(&main_py, &state.python_exec).await;
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

    result
}

#[tauri::command]
async fn update_all_dependencies(state: State<'_, AppState>) -> Result<(), String> {
    dependency_manager::ensure_all_scripts_requirements(&state.scripts_dir, &state.python_exec)
        .await
}

#[tauri::command]
fn list_scripts(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    git_manager::list_available_scripts(&state.scripts_dir)
}

#[tauri::command]
async fn get_script_logs(
    script_name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let log_path = state.scripts_dir.join(format!("{}.log", script_name));

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
fn generate_admin_key() -> Result<String, String> {
    let path = admin_key::desktop_key_path();
    let _payload = admin_key::write_key_file(&path)?;
    Ok(format!("{}", path.to_string_lossy()))
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            check_kill_switch,
            sync_scripts,
            run_script,
            list_scripts,
            get_script_logs,
            script_manager::add_script,
            script_manager::add_official_script,
            script_manager::get_local_scripts,
            update_all_dependencies,
            check_admin_key,
            generate_admin_key,
            get_admin_key_info,
            get_run_history,
            export_history_as_csv,
            toggle_dark_mode,
            get_settings,
            get_scripts_dir
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
