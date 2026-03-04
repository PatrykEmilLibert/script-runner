use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
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

#[derive(Serialize, Deserialize)]
pub struct ScriptInfo {
    pub name: String,
    pub subdir: String,
    pub path: String,
    pub size: u64,
    pub modified: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub encrypted: bool,
}

#[derive(Serialize)]
pub struct BulkScriptOperationResult {
    pub requested: usize,
    pub processed: usize,
    pub skipped: Vec<String>,
}

fn sanitize_scope_component(raw: &str) -> String {
    let sanitized: String = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect();

    let lower = sanitized.to_lowercase();
    if lower.trim_matches('_').is_empty() {
        "unknown".to_string()
    } else {
        lower
    }
}

pub fn current_user_namespace() -> String {
    if let Ok(Some(user)) = crate::github_auth::get_current_user() {
        return format!("gh_{}", sanitize_scope_component(&user.login));
    }

    let machine = sanitize_scope_component(&crate::kill_switch::get_machine_id());
    format!("local_{}", machine)
}

pub fn get_user_scripts_root(scripts_path: &Path) -> PathBuf {
    scripts_path.join("scripts").join(current_user_namespace())
}

pub fn get_user_script_dir(scripts_path: &Path, script_name: &str) -> PathBuf {
    get_user_scripts_root(scripts_path).join(script_name)
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
    crate::dependency_manager::ensure_all_scripts_requirements(
        &std::path::PathBuf::from(&scripts_dir),
        &state.python_exec,
    )
    .await?;
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
    crate::dependency_manager::ensure_all_scripts_requirements(
        &std::path::PathBuf::from(&scripts_dir),
        &state.python_exec,
    )
    .await?;
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

    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Nie można odczytać pliku: {e}"))?;

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
    let script_dir = if folder == "scripts" {
        get_user_script_dir(scripts_path, &script_name)
    } else {
        scripts_path.join(&folder).join(&script_name)
    };

    if !script_dir.exists() {
        return Err("Script not found".to_string());
    }

    fs::remove_dir_all(&script_dir).map_err(|e| format!("Failed to remove script: {}", e))?;

    let commit_msg = format!("Delete script: {}", script_name);
    commit_and_push(scripts_path, commit_msg)?;

    // Ensure all scripts' requirements are installed (rebuild after deletion)
    crate::dependency_manager::ensure_all_scripts_requirements(
        &std::path::PathBuf::from(&scripts_dir),
        &state.python_exec,
    )
    .await?;

    Ok(format!("Script '{}' deleted successfully", script_name))
}

fn load_script_metadata(script_dir: &Path, script_name: &str) -> ScriptMetadata {
    let metadata_file = script_dir.join("metadata.json");

    if metadata_file.exists() {
        if let Ok(metadata_content) = fs::read_to_string(&metadata_file) {
            if let Ok(metadata) = serde_json::from_str::<ScriptMetadata>(&metadata_content) {
                return metadata;
            }
        }
    }

    let now = Utc::now().to_rfc3339();
    ScriptMetadata {
        name: script_name.to_string(),
        description: String::new(),
        author: String::new(),
        version: "1.0.0".to_string(),
        created_at: now.clone(),
        last_modified: now,
    }
}

fn write_script_metadata(script_dir: &Path, metadata: &ScriptMetadata) -> Result<(), String> {
    let metadata_file = script_dir.join("metadata.json");
    let metadata_json = serde_json::to_string_pretty(metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    fs::write(metadata_file, metadata_json).map_err(|e| format!("Failed to write metadata: {}", e))
}

fn write_script_and_requirements(script_dir: &Path, script_content: &str) -> Result<(), String> {
    let script_file = script_dir.join("main.py");
    fs::write(&script_file, script_content)
        .map_err(|e| format!("Failed to write script file: {}", e))?;

    let dependencies = analyze_dependencies(script_content)?;
    let requirements_file = script_dir.join("requirements.txt");

    if dependencies.is_empty() {
        if requirements_file.exists() {
            fs::remove_file(&requirements_file)
                .map_err(|e| format!("Failed to remove requirements: {}", e))?;
        }
    } else {
        fs::write(&requirements_file, dependencies.join("\n"))
            .map_err(|e| format!("Failed to write requirements: {}", e))?;
    }

    Ok(())
}

fn read_script_source_internal(script_dir: &Path) -> Result<String, String> {
    let main_enc = script_dir.join("main.py.enc");
    let main_py = script_dir.join("main.py");

    if main_enc.exists() {
        return crate::script_encryption::decrypt_script(&main_enc);
    }

    if main_py.exists() {
        return fs::read_to_string(&main_py)
            .map_err(|e| format!("Failed to read script file: {}", e));
    }

    Err("Script file not found (main.py or main.py.enc)".to_string())
}

fn should_encrypt_official_scripts() -> bool {
    std::env::var("SR_ENCRYPT_OFFICIAL_LOCAL")
        .map(|value| value != "0")
        .unwrap_or(true)
}

fn collect_scripts_from_subdir(
    scripts_path: &Path,
    subdir: &str,
) -> Result<Vec<ScriptInfo>, String> {
    let (root, path_prefix) = if subdir == "scripts" {
        let namespace = current_user_namespace();
        (
            scripts_path.join("scripts").join(&namespace),
            format!("scripts/{}", namespace),
        )
    } else {
        (scripts_path.join(subdir), subdir.to_string())
    };

    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut result = Vec::new();
    let entries = fs::read_dir(&root).map_err(|e| format!("Failed to read {}: {}", subdir, e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry in {}: {}", subdir, e))?;
        let script_dir = entry.path();
        if !script_dir.is_dir() {
            continue;
        }

        let script_name = entry.file_name().to_string_lossy().to_string();
        let main_py = script_dir.join("main.py");
        let main_enc = script_dir.join("main.py.enc");
        let encrypted = main_enc.exists();

        if !main_py.exists() && !encrypted {
            continue;
        }

        let metadata = load_script_metadata(&script_dir, &script_name);
        let source_path = if encrypted { main_enc } else { main_py };
        let size = fs::metadata(&source_path).map(|m| m.len()).unwrap_or(0);

        result.push(ScriptInfo {
            name: metadata.name.clone(),
            subdir: subdir.to_string(),
            path: format!("{}/{}", path_prefix, script_name),
            size,
            modified: metadata.last_modified.clone(),
            description: metadata.description,
            author: metadata.author,
            version: metadata.version,
            encrypted,
        });
    }

    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(result)
}

#[tauri::command]
pub fn list_official_scripts(scripts_dir: String) -> Result<Vec<String>, String> {
    let scripts_path = Path::new(&scripts_dir);
    let info = collect_scripts_from_subdir(scripts_path, "official")?;
    Ok(info.into_iter().map(|item| item.name).collect())
}

#[tauri::command]
pub fn get_all_scripts_info(scripts_dir: String) -> Result<Vec<ScriptInfo>, String> {
    let scripts_path = Path::new(&scripts_dir);
    let mut result = collect_scripts_from_subdir(scripts_path, "official")?;
    result.extend(collect_scripts_from_subdir(scripts_path, "scripts")?);
    Ok(result)
}

#[tauri::command]
pub fn get_script_source(
    script_name: String,
    scripts_dir: String,
    subdir: Option<String>,
) -> Result<String, String> {
    let folder = subdir.unwrap_or_else(|| "official".to_string());
    let scripts_path = Path::new(&scripts_dir);
    let script_dir = if folder == "scripts" {
        get_user_script_dir(scripts_path, &script_name)
    } else {
        scripts_path.join(folder).join(&script_name)
    };

    if !script_dir.exists() {
        return Err(format!("Script not found: {}", script_name));
    }

    read_script_source_internal(&script_dir)
}

#[tauri::command]
pub async fn replace_official_script_content(
    script_name: String,
    script_content: String,
    scripts_dir: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<String, String> {
    let script_name = script_name.trim();
    if script_name.is_empty() {
        return Err("Script name cannot be empty".to_string());
    }

    let scripts_path = Path::new(&scripts_dir);
    let script_dir = scripts_path.join("official").join(script_name);
    if !script_dir.exists() {
        return Err(format!("Official script not found: {}", script_name));
    }

    let was_encrypted = script_dir.join("main.py.enc").exists();
    write_script_and_requirements(&script_dir, &script_content)?;

    let mut metadata = load_script_metadata(&script_dir, script_name);
    metadata.last_modified = Utc::now().to_rfc3339();
    write_script_metadata(&script_dir, &metadata)?;

    if was_encrypted || should_encrypt_official_scripts() {
        let main_py = script_dir.join("main.py");
        if main_py.exists() {
            crate::script_encryption::encrypt_script(&main_py)?;
        }
    }

    let commit_msg = format!("Update official script code: {}", script_name);
    commit_and_push(scripts_path, commit_msg)?;

    crate::dependency_manager::ensure_all_scripts_requirements(
        &std::path::PathBuf::from(&scripts_dir),
        &state.python_exec,
    )
    .await?;

    Ok(format!("Official script '{}' updated", script_name))
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn update_official_script_full(
    current_name: String,
    new_name: String,
    script_content: String,
    description: String,
    author: String,
    version: String,
    scripts_dir: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<String, String> {
    let current_name = current_name.trim().to_string();
    let new_name = new_name.trim().to_string();

    if current_name.is_empty() || new_name.is_empty() {
        return Err("Script name cannot be empty".to_string());
    }

    let scripts_path = Path::new(&scripts_dir);
    let official_root = scripts_path.join("official");
    let current_dir = official_root.join(&current_name);
    if !current_dir.exists() {
        return Err(format!("Official script not found: {}", current_name));
    }

    let target_dir = official_root.join(&new_name);
    if current_name != new_name {
        if target_dir.exists() {
            return Err(format!("Target script name already exists: {}", new_name));
        }
        fs::rename(&current_dir, &target_dir)
            .map_err(|e| format!("Failed to rename script directory: {}", e))?;
    }

    let script_dir = if current_name == new_name {
        current_dir
    } else {
        target_dir
    };

    let was_encrypted = script_dir.join("main.py.enc").exists();
    let previous_metadata = load_script_metadata(&script_dir, &new_name);
    write_script_and_requirements(&script_dir, &script_content)?;

    let now = Utc::now().to_rfc3339();
    let metadata = ScriptMetadata {
        name: new_name.clone(),
        description,
        author,
        version: if version.trim().is_empty() {
            "1.0.0".to_string()
        } else {
            version.trim().to_string()
        },
        created_at: if previous_metadata.created_at.trim().is_empty() {
            now.clone()
        } else {
            previous_metadata.created_at
        },
        last_modified: now,
    };
    write_script_metadata(&script_dir, &metadata)?;

    if was_encrypted || should_encrypt_official_scripts() {
        let main_py = script_dir.join("main.py");
        if main_py.exists() {
            crate::script_encryption::encrypt_script(&main_py)?;
        }
    }

    let commit_msg = if current_name == new_name {
        format!("Update official script: {}", current_name)
    } else {
        format!(
            "Rename and update official script: {} -> {}",
            current_name, new_name
        )
    };
    commit_and_push(scripts_path, commit_msg)?;

    crate::dependency_manager::ensure_all_scripts_requirements(
        &std::path::PathBuf::from(&scripts_dir),
        &state.python_exec,
    )
    .await?;

    Ok(format!("Official script '{}' saved", new_name))
}

fn normalize_unique_script_names(script_names: Vec<String>) -> Vec<String> {
    let mut unique = BTreeSet::new();

    for name in script_names {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            unique.insert(trimmed.to_string());
        }
    }

    unique.into_iter().collect()
}

#[tauri::command]
pub async fn bulk_delete_official_scripts(
    script_names: Vec<String>,
    scripts_dir: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<BulkScriptOperationResult, String> {
    let normalized = normalize_unique_script_names(script_names);
    if normalized.is_empty() {
        return Err("No scripts selected".to_string());
    }

    let scripts_path = Path::new(&scripts_dir);
    let official_root = scripts_path.join("official");

    let mut processed = 0usize;
    let mut skipped = Vec::new();

    for script_name in &normalized {
        let script_dir = official_root.join(script_name);
        if !script_dir.exists() {
            skipped.push(format!("{} (not found)", script_name));
            continue;
        }

        match fs::remove_dir_all(&script_dir) {
            Ok(_) => {
                processed += 1;
            }
            Err(e) => {
                skipped.push(format!("{} ({})", script_name, e));
            }
        }
    }

    if processed > 0 {
        commit_and_push(
            scripts_path,
            format!("Bulk delete official scripts ({} items)", processed),
        )?;

        crate::dependency_manager::ensure_all_scripts_requirements(
            &std::path::PathBuf::from(&scripts_dir),
            &state.python_exec,
        )
        .await?;
    }

    Ok(BulkScriptOperationResult {
        requested: normalized.len(),
        processed,
        skipped,
    })
}

#[tauri::command]
pub fn bulk_encrypt_official_scripts(
    script_names: Vec<String>,
    scripts_dir: String,
) -> Result<BulkScriptOperationResult, String> {
    let normalized = normalize_unique_script_names(script_names);
    if normalized.is_empty() {
        return Err("No scripts selected".to_string());
    }

    let scripts_path = Path::new(&scripts_dir);
    let official_root = scripts_path.join("official");

    let mut processed = 0usize;
    let mut skipped = Vec::new();

    for script_name in &normalized {
        let script_dir = official_root.join(script_name);
        if !script_dir.exists() {
            skipped.push(format!("{} (not found)", script_name));
            continue;
        }

        let main_py = script_dir.join("main.py");
        if !main_py.exists() {
            if script_dir.join("main.py.enc").exists() {
                skipped.push(format!("{} (already encrypted)", script_name));
            } else {
                skipped.push(format!("{} (missing main.py)", script_name));
            }
            continue;
        }

        match crate::script_encryption::encrypt_script(&main_py) {
            Ok(_) => {
                processed += 1;
            }
            Err(e) => {
                skipped.push(format!("{} ({})", script_name, e));
            }
        }
    }

    Ok(BulkScriptOperationResult {
        requested: normalized.len(),
        processed,
        skipped,
    })
}

#[tauri::command]
pub fn bulk_update_official_metadata(
    script_names: Vec<String>,
    scripts_dir: String,
    author: Option<String>,
    version: Option<String>,
    description_prefix: Option<String>,
) -> Result<BulkScriptOperationResult, String> {
    let normalized = normalize_unique_script_names(script_names);
    if normalized.is_empty() {
        return Err("No scripts selected".to_string());
    }

    let author = author.unwrap_or_default().trim().to_string();
    let version = version.unwrap_or_default().trim().to_string();
    let description_prefix = description_prefix.unwrap_or_default().trim().to_string();

    if author.is_empty() && version.is_empty() && description_prefix.is_empty() {
        return Err("No metadata changes provided".to_string());
    }

    let scripts_path = Path::new(&scripts_dir);
    let official_root = scripts_path.join("official");

    let mut processed = 0usize;
    let mut skipped = Vec::new();

    for script_name in &normalized {
        let script_dir = official_root.join(script_name);
        if !script_dir.exists() {
            skipped.push(format!("{} (not found)", script_name));
            continue;
        }

        let mut metadata = load_script_metadata(&script_dir, script_name);

        if !author.is_empty() {
            metadata.author = author.clone();
        }
        if !version.is_empty() {
            metadata.version = version.clone();
        }
        if !description_prefix.is_empty() {
            let current = metadata.description.trim();
            metadata.description = if current.is_empty() {
                description_prefix.clone()
            } else if current.starts_with(&description_prefix) {
                current.to_string()
            } else {
                format!("{} {}", description_prefix, current)
                    .trim()
                    .to_string()
            };
        }

        metadata.last_modified = Utc::now().to_rfc3339();

        match write_script_metadata(&script_dir, &metadata) {
            Ok(_) => {
                processed += 1;
            }
            Err(e) => {
                skipped.push(format!("{} ({})", script_name, e));
            }
        }
    }

    if processed > 0 {
        commit_and_push(
            scripts_path,
            format!("Bulk update official script metadata ({} items)", processed),
        )?;
    }

    Ok(BulkScriptOperationResult {
        requested: normalized.len(),
        processed,
        skipped,
    })
}

fn analyze_dependencies(script_content: &str) -> Result<Vec<String>, String> {
    let mut dependencies = Vec::new();

    // Parse import statements
    for line in script_content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
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

fn commit_and_push(scripts_path: &Path, commit_msg: String) -> Result<(), String> {
    // Git add
    let add_output = Command::new("git")
        .args(["add", "."])
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
    let commit_output = Command::new("git")
        .args(["commit", "-m", &commit_msg])
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
        .args(["push"])
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
    let scripts_root = Path::new(&scripts_dir);
    let scripts_path = if folder == "scripts" {
        get_user_scripts_root(scripts_root)
    } else {
        scripts_root.join(folder)
    };

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
    let script_dir = if subfolder == "scripts" {
        get_user_script_dir(scripts_path, &script_name)
    } else {
        scripts_path.join(subfolder).join(&script_name)
    };

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
    let commit_msg = format!("Add script: {}", script_name);
    commit_and_push(scripts_path, commit_msg)?;

    Ok(format!("Script '{}' added successfully!", script_name))
}
