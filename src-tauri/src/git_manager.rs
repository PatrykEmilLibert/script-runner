use git2::Repository;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn sync_scripts(scripts_dir: &PathBuf) -> Result<String, String> {
    if !scripts_dir.exists() {
        std::fs::create_dir_all(scripts_dir)
            .map_err(|e| format!("Failed to create scripts dir: {}", e))?;
    }

    let default_path = PathBuf::from(".");
    let repo_path = scripts_dir.parent().unwrap_or(&default_path);

    // Require internet connection - no offline mode
    sync_from_git(repo_path)
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
            log::info!("No Git repo found, skipping sync");
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
