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
mod script_encryption;

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
async fn run_script(
    script_name: String,
    args: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let start_time = chrono::Utc::now();
    // Prefer user script; fall back to official if user copy not found
    let user_dir = state.scripts_dir.join("scripts").join(&script_name);
    let official_dir = state.scripts_dir.join("official").join(&script_name);

    let script_dir = if user_dir.exists() {
        log::info!("Using user script: {:?}", user_dir);
        user_dir
    } else if official_dir.exists() {
        log::info!("Using official script: {:?}", official_dir);
        official_dir
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
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            check_kill_switch,
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
            generate_admin_key,
            get_admin_key_info,
            get_run_history,
            export_history_as_csv,
            toggle_dark_mode,
            get_settings,
            get_scripts_dir,
            encrypt_official_script
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
