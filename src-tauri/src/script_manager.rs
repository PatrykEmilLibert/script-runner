use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Serialize, Deserialize)]
pub struct ScriptMetadata {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub created_at: String,
    pub last_modified: String,
}

#[tauri::command]
pub async fn add_script(
    script_name: String,
    script_content: String,
    description: String,
    author: String,
    scripts_dir: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<String, String> {
    let result = add_script_internal(
        script_name,
        script_content,
        description,
        author,
        scripts_dir.clone(),
        "scripts",
    )?;
    // Ensure all scripts' requirements are installed
    crate::dependency_manager::ensure_all_scripts_requirements(&std::path::PathBuf::from(&scripts_dir), &state.python_exec).await?;
    Ok(result)
}

#[tauri::command]
pub async fn add_official_script(
    file_name: String,
    file_content: String,
    description: String,
    author: String,
    scripts_dir: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<String, String> {
    let script_name = file_name.trim_end_matches(".py").to_string();
    let result = add_script_internal(
        script_name,
        file_content,
        description,
        author,
        scripts_dir.clone(),
        "official",
    )?;
    // Ensure all scripts' requirements are installed
    crate::dependency_manager::ensure_all_scripts_requirements(&std::path::PathBuf::from(&scripts_dir), &state.python_exec).await?;
    Ok(result)
}

// Fallback dla drop z systemu plików (Tauri file-drop) – pobiera zawartość po ścieżce
#[tauri::command]
pub async fn add_official_script_from_path(
    path: String,
    description: String,
    author: String,
    scripts_dir: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<String, String> {
    let file_name = Path::new(&path)
        .file_name()
        .ok_or_else(|| "Nieprawidłowa ścieżka".to_string())?
        .to_string_lossy()
        .to_string();

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Nie można odczytać pliku: {e}"))?;

    add_official_script(file_name, content, description, author, scripts_dir, state).await
}

#[tauri::command]
pub async fn delete_script(
    script_name: String,
    scripts_dir: String,
    subdir: Option<String>,
    state: tauri::State<'_, crate::AppState>,
) -> Result<String, String> {
    let folder = subdir.unwrap_or_else(|| "scripts".to_string());
    let scripts_path = Path::new(&scripts_dir);
    let script_dir = scripts_path.join(&folder).join(&script_name);

    if !script_dir.exists() {
        return Err("Script not found".to_string());
    }

    fs::remove_dir_all(&script_dir)
        .map_err(|e| format!("Failed to remove script: {}", e))?;

    commit_and_push(&scripts_path, &script_name)?;

    // Ensure all scripts' requirements are installed (rebuild after deletion)
    crate::dependency_manager::ensure_all_scripts_requirements(&std::path::PathBuf::from(&scripts_dir), &state.python_exec).await?;

    Ok(format!("Script '{}' deleted successfully", script_name))
}

fn analyze_dependencies(script_content: &str) -> Result<Vec<String>, String> {
    let mut dependencies = Vec::new();

    // Parse import statements
    for line in script_content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("import ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let module = parts[1].split('.').next().unwrap_or("");
                if !is_stdlib_module(module) {
                    dependencies.push(module.to_string());
                }
            }
        } else if trimmed.starts_with("from ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let module = parts[1].split('.').next().unwrap_or("");
                if !is_stdlib_module(module) {
                    dependencies.push(module.to_string());
                }
            }
        }
    }

    // Remove duplicates
    dependencies.sort();
    dependencies.dedup();

    Ok(dependencies)
}

fn is_stdlib_module(module: &str) -> bool {
    // Common Python stdlib modules
    let stdlib_modules = [
        "os",
        "sys",
        "re",
        "json",
        "time",
        "datetime",
        "random",
        "math",
        "collections",
        "itertools",
        "functools",
        "pathlib",
        "typing",
        "logging",
        "argparse",
        "subprocess",
        "threading",
        "multiprocessing",
        "socket",
        "urllib",
        "http",
        "email",
        "csv",
        "sqlite3",
        "pickle",
        "hashlib",
        "hmac",
        "secrets",
        "uuid",
        "base64",
        "io",
        "shutil",
        "tempfile",
        "glob",
        "fnmatch",
        "zipfile",
        "tarfile",
        "gzip",
    ];

    stdlib_modules.contains(&module)
}

fn commit_and_push(scripts_path: &Path, script_name: &str) -> Result<(), String> {
    // Git add
    let add_output = Command::new("git")
        .args(&["add", "."])
        .current_dir(scripts_path)
        .output()
        .map_err(|e| format!("Git add failed: {}", e))?;

    if !add_output.status.success() {
        return Err(format!(
            "Git add failed: {}",
            String::from_utf8_lossy(&add_output.stderr)
        ));
    }

    // Git commit
    let commit_msg = format!("Add script: {}", script_name);
    let commit_output = Command::new("git")
        .args(&["commit", "-m", &commit_msg])
        .current_dir(scripts_path)
        .output()
        .map_err(|e| format!("Git commit failed: {}", e))?;

    if !commit_output.status.success() {
        let stderr = String::from_utf8_lossy(&commit_output.stderr);
        // Ignore "nothing to commit" errors
        if !stderr.contains("nothing to commit") {
            return Err(format!("Git commit failed: {}", stderr));
        }
    }

    // Git push
    let push_output = Command::new("git")
        .args(&["push"])
        .current_dir(scripts_path)
        .output()
        .map_err(|e| format!("Git push failed: {}", e))?;

    if !push_output.status.success() {
        return Err(format!(
            "Git push failed: {}",
            String::from_utf8_lossy(&push_output.stderr)
        ));
    }

    Ok(())
}

#[tauri::command]
pub fn get_local_scripts(
    scripts_dir: String,
    subdir: Option<String>,
) -> Result<Vec<ScriptMetadata>, String> {
    let folder = subdir.unwrap_or_else(|| "scripts".to_string());
    let scripts_path = Path::new(&scripts_dir).join(folder);

    if !scripts_path.exists() {
        return Ok(Vec::new());
    }

    let mut scripts = Vec::new();

    let entries = fs::read_dir(&scripts_path)
        .map_err(|e| format!("Failed to read scripts directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let metadata_file = entry.path().join("metadata.json");

        if metadata_file.exists() {
            let metadata_content = fs::read_to_string(&metadata_file)
                .map_err(|e| format!("Failed to read metadata: {}", e))?;
            let metadata: ScriptMetadata = serde_json::from_str(&metadata_content)
                .map_err(|e| format!("Failed to parse metadata: {}", e))?;
            scripts.push(metadata);
        }
    }

    Ok(scripts)
}

fn add_script_internal(
    script_name: String,
    script_content: String,
    description: String,
    author: String,
    scripts_dir: String,
    subfolder: &str,
) -> Result<String, String> {
    let scripts_path = Path::new(&scripts_dir);
    let script_dir = scripts_path.join(subfolder).join(&script_name);

    fs::create_dir_all(&script_dir)
        .map_err(|e| format!("Failed to create script directory: {}", e))?;

    // Save main.py
    let script_file = script_dir.join("main.py");
    fs::write(&script_file, &script_content)
        .map_err(|e| format!("Failed to write script file: {}", e))?;

    // Analyze dependencies
    let dependencies = analyze_dependencies(&script_content)?;

    // Save requirements.txt
    if !dependencies.is_empty() {
        let requirements_file = script_dir.join("requirements.txt");
        fs::write(&requirements_file, dependencies.join("\n"))
            .map_err(|e| format!("Failed to write requirements: {}", e))?;
    }

    // Create metadata
    let now = Utc::now().to_rfc3339();
    let metadata = ScriptMetadata {
        name: script_name.clone(),
        description,
        author,
        version: "1.0.0".to_string(),
        created_at: now.clone(),
        last_modified: now,
    };

    let metadata_file = script_dir.join("metadata.json");
    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    fs::write(&metadata_file, metadata_json)
        .map_err(|e| format!("Failed to write metadata: {}", e))?;

    // Git commit and push
    commit_and_push(&scripts_path, &script_name)?;

    Ok(format!("Script '{}' added successfully!", script_name))
}
