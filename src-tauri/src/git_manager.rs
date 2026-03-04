use git2::Repository;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::script_manager;

fn count_scripts(scripts_dir: &PathBuf) -> usize {
    if !scripts_dir.exists() {
        return 0;
    }

    let official_root = scripts_dir.join("official");
    let user_root = script_manager::get_user_scripts_root(scripts_dir.as_path());

    WalkDir::new(scripts_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .filter(|e| {
            let path = e.path();
            let is_official = path.starts_with(&official_root);
            let is_current_user_script = path.starts_with(&user_root);

            (is_official || is_current_user_script)
                && (path.join("main.py").exists() || path.join("main.py.enc").exists())
        })
        .count()
}

pub fn sync_scripts(scripts_dir: &PathBuf) -> Result<String, String> {
    log::info!("Syncing scripts to: {:?}", scripts_dir);

    // Count before sync
    let count_before = count_scripts(scripts_dir);

    if !scripts_dir.exists() {
        log::info!("Creating scripts directory: {:?}", scripts_dir);
        std::fs::create_dir_all(scripts_dir)
            .map_err(|e| format!("Failed to create scripts dir: {}", e))?;
    }

    // Try to open repo at scripts_dir; if missing, clone from env or default URL
    let result = match Repository::open(scripts_dir) {
        Ok(_) => {
            log::info!("Existing repository found, syncing...");
            sync_from_git(scripts_dir)
        }
        Err(e) => {
            log::info!("No repository found ({}), will clone", e);
            let remote_url = std::env::var("SCRIPTS_REPO_URL").unwrap_or_else(|_| {
                "https://github.com/PatrykEmilLibert/script-runner-scripts.git".to_string()
            });
            log::info!("Cloning scripts repo from: {}", remote_url);
            log::info!("Target directory: {:?}", scripts_dir);

            match Repository::clone(&remote_url, scripts_dir) {
                Ok(_) => {
                    log::info!("Repository cloned successfully");
                    sync_from_git(scripts_dir)
                }
                Err(e) => {
                    let error_msg = format!(
                        "Failed to clone scripts repository.\n\
                        Error: {}\n\
                        Repository: {}\n\
                        Target: {:?}\n\
                        Tip: Check internet connection and SCRIPTS_REPO_URL.",
                        e, remote_url, scripts_dir
                    );
                    log::error!("{}", error_msg);
                    Err(error_msg)
                }
            }
        }
    };

    // Count after sync and append info if successful
    if let Ok(msg) = result {
        let count_after = count_scripts(scripts_dir);
        let new_count = count_after.saturating_sub(count_before);

        if new_count > 0 {
            Ok(format!("{}|new_scripts:{}", msg, new_count))
        } else {
            Ok(msg)
        }
    } else {
        result
    }
}

fn sync_from_git(repo_path: &Path) -> Result<String, String> {
    match Repository::open(repo_path) {
        Ok(repo) => {
            log::info!("Repository found, attempting sync");

            let mut remote = repo
                .find_remote("origin")
                .map_err(|e| format!("Failed to find remote: {}", e))?;

            // Fetch from remote - requires internet connection
            if let Err(e) = remote.fetch(&["main"], None, None) {
                log::warn!(
                    "Failed to fetch from remote, using cached local scripts: {}",
                    e
                );
                return Ok("Using cached scripts (offline mode)".to_string());
            }

            let oid = repo
                .refname_to_id("refs/remotes/origin/main")
                .map_err(|e| format!("Failed to find remote main: {}", e))?;

            let object = repo
                .find_object(oid, None)
                .map_err(|e| format!("Failed to find object: {}", e))?;

            repo.reset(&object, git2::ResetType::Hard, None)
                .map_err(|e| format!("Failed to reset: {}", e))?;

            Ok("Scripts synced successfully".to_string())
        }
        Err(_) => {
            log::info!("No Git repo found at {:?}, skipping sync", repo_path);
            Err("No repository".to_string())
        }
    }
}

#[allow(dead_code)]
pub fn list_available_scripts(scripts_dir: &PathBuf) -> Result<Vec<String>, String> {
    let mut scripts = Vec::new();

    if !scripts_dir.exists() {
        log::warn!("Scripts directory does not exist: {:?}", scripts_dir);
        return Ok(scripts);
    }

    let official_root = scripts_dir.join("official");
    let user_root = script_manager::get_user_scripts_root(scripts_dir.as_path());

    for entry in WalkDir::new(scripts_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let path = entry.path();
        let is_official = path.starts_with(&official_root);
        let is_current_user_script = path.starts_with(&user_root);

        if (is_official || is_current_user_script)
            && (path.join("main.py").exists() || path.join("main.py.enc").exists())
        {
            if let Some(name) = entry.file_name().to_str() {
                scripts.push(name.to_string());
            }
        }
    }

    scripts.sort();
    scripts.dedup();

    log::info!("Found {} scripts", scripts.len());
    Ok(scripts)
}
