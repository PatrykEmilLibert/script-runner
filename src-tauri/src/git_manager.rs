use git2::Repository;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn count_scripts(scripts_dir: &PathBuf) -> usize {
    if !scripts_dir.exists() {
        return 0;
    }
    WalkDir::new(scripts_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.file_name().map_or(false, |name| name == "main.py")
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
        },
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
                },
                Err(e) => {
                    log::error!("Clone failed: {}. Trying local fallback.", e);
                    let mut candidates: Vec<PathBuf> = Vec::new();
                    if let Ok(env_path) = std::env::var("SCRIPTS_LOCAL_PATH") {
                        candidates.push(PathBuf::from(env_path));
                    }
                    candidates.push(PathBuf::from("../script-runner-scripts"));
                    candidates.push(PathBuf::from("../../script-runner-scripts"));
                    
                    log::info!("Searching for local fallback in: {:?}", candidates);
                    
                    if let Some(found) = candidates.into_iter().find(|p| p.exists()) {
                        log::info!("Using local fallback at {:?}", found);
                        copy_dir_all(&found, scripts_dir)
                            .map_err(|err| format!("Fallback copy failed: {}", err))?;
                        Ok("Scripts synced from local fallback".to_string())
                    } else {
                        let error_msg = format!(
                            "Failed to clone repository and no local fallback found.\n\
                            Error: {}\n\
                            Repository: {}\n\
                            Target: {:?}\n\
                            Tip: Check your internet connection or set SCRIPTS_LOCAL_PATH environment variable.",
                            e, remote_url, scripts_dir
                        );
                        log::error!("{}", error_msg);
                        Err(error_msg)
                    }
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

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn sync_from_git(repo_path: &Path) -> Result<String, String> {
    match Repository::open(repo_path) {
        Ok(repo) => {
            log::info!("Repository found, attempting sync");

            let mut remote = repo
                .find_remote("origin")
                .map_err(|e| format!("Failed to find remote: {}", e))?;

            // Fetch from remote - requires internet connection
            remote
                .fetch(&["main"], None, None)
                .map_err(|e| format!("Failed to fetch (no internet connection?): {}", e))?;

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

pub fn list_available_scripts(scripts_dir: &PathBuf) -> Result<Vec<String>, String> {
    let mut scripts = Vec::new();

    if !scripts_dir.exists() {
        log::warn!("Scripts directory does not exist: {:?}", scripts_dir);
        return Ok(scripts);
    }

    for entry in WalkDir::new(scripts_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "py"))
    {
        if let Some(name) = entry.file_name().to_str() {
            scripts.push(name.to_string());
        }
    }

    log::info!("Found {} scripts", scripts.len());
    Ok(scripts)
}
