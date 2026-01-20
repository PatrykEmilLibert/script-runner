#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use dirs;
use std::env;
use std::path::{Path, PathBuf};
use tauri::State;

mod admin_key;
mod dependency_manager;
mod git_manager;
mod kill_switch;
mod python_runner;
mod script_manager;

#[derive(Clone)]
pub struct AppState {
    scripts_dir: PathBuf,
    python_exec: PathBuf,
}

#[tauri::command]
async fn check_kill_switch() -> Result<bool, String> {
    kill_switch::check_remote_status().await
}

#[tauri::command]
fn sync_scripts(state: State<'_, AppState>) -> Result<String, String> {
    git_manager::sync_scripts(&state.scripts_dir)
}

#[tauri::command]
async fn run_script(script_name: String, state: State<'_, AppState>) -> Result<String, String> {
    let script_path = state.scripts_dir.join(&script_name);

    // Auto-detect dependencies
    let deps = dependency_manager::detect_dependencies(&script_path).await?;
    dependency_manager::install_dependencies(&deps, &state.python_exec).await?;

    // Run script
    python_runner::execute_script(&script_path, &state.python_exec).await
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
fn check_admin_key() -> bool {
    // Resolve admin key path with env override and cross-platform Desktop detection
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(custom) = env::var("ADMIN_KEY_PATH") {
        candidates.push(PathBuf::from(custom));
    }

    if let Some(desktop) = dirs::desktop_dir() {
        candidates.push(desktop.join("sr-admin.key"));
    }

    #[cfg(target_os = "windows")]
    {
        candidates.push(PathBuf::from("C:/Users/Public/Desktop/sr-admin.key"));
    }

    #[cfg(not(target_os = "windows"))]
    {
        candidates.push(PathBuf::from("/tmp/sr-admin.key"));
    }

    candidates
        .into_iter()
        .any(|p| admin_key::validate_key_file(&p))
}

#[tauri::command]
fn generate_admin_key() -> Result<String, String> {
    let path = admin_key::desktop_key_path();
    let payload = admin_key::write_key_file(&path)?;
    Ok(format!("{}", path.to_string_lossy()))
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
            check_admin_key,
            generate_admin_key
        ])
        .manage(AppState {
            scripts_dir: PathBuf::from("./scripts"),
            python_exec: PathBuf::from("./python/bin/python"),
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
