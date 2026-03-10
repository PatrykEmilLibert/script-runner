use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use git2::{Cred, IndexAddOption, PushOptions, RemoteCallbacks, Repository, Signature};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const COMPILED_SCRIPTS_PUSH_TOKEN: Option<&str> = option_env!("SR_SCRIPTS_PUSH_TOKEN");
const COMPILED_SCRIPTS_PUSH_TOKEN_B64: Option<&str> = option_env!("SR_SCRIPTS_PUSH_TOKEN_B64");

fn decode_base64_token(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let normalized = trimmed.strip_prefix("b64:").unwrap_or(trimmed);
    let decoded = general_purpose::STANDARD.decode(normalized).ok()?;
    let token = String::from_utf8(decoded).ok()?.trim().to_string();

    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

fn read_non_empty_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn resolve_service_push_token() -> Option<String> {
    read_non_empty_env("SR_SCRIPTS_PUSH_TOKEN")
        .or_else(|| {
            read_non_empty_env("SR_SCRIPTS_PUSH_TOKEN_B64")
                .and_then(|value| decode_base64_token(&value))
        })
        .or_else(|| {
            COMPILED_SCRIPTS_PUSH_TOKEN
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .or_else(|| COMPILED_SCRIPTS_PUSH_TOKEN_B64.and_then(decode_base64_token))
}

fn apply_no_console_window(cmd: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = cmd;
    }
}

fn detect_hardcoded_paths(script_content: &str) -> Vec<String> {
    let windows_abs = Regex::new(r#"(?i)[a-z]:\\[^\"'\s]+"#).unwrap();
    let unix_abs = Regex::new(r#"(?i)/(users|home|var|etc|opt|tmp)/[^\"'\s]+"#).unwrap();

    let mut findings = Vec::new();

    for (index, line) in script_content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }

        for m in windows_abs.find_iter(line) {
            findings.push(format!("L{}: {}", index + 1, m.as_str()));
            if findings.len() >= 20 {
                return findings;
            }
        }

        for m in unix_abs.find_iter(line) {
            findings.push(format!("L{}: {}", index + 1, m.as_str()));
            if findings.len() >= 20 {
                return findings;
            }
        }
    }

    findings
}

fn format_portability_warning(findings: &[String]) -> String {
    if findings.is_empty() {
        return String::new();
    }

    format!(
        "\n\n⚠️ Wykryto potencjalne hardcoded ścieżki ({}):\n{}\n\n💡 Zalecenie: używaj Path(__file__).parent dla configów i plików lokalnych.",
        findings.len(),
        findings.join("\n")
    )
}

fn extract_basename(path: &str) -> Option<String> {
    path.rsplit(['\\', '/'])
        .find(|part| !part.trim().is_empty())
        .map(|part| part.trim().to_string())
}

fn ensure_pathlib_import(content: &str) -> String {
    if content.contains("from pathlib import Path") {
        return content.to_string();
    }

    let mut lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
    let mut insert_at = 0usize;

    if lines
        .first()
        .map(|line| line.starts_with("#!"))
        .unwrap_or(false)
    {
        insert_at = 1;
    }

    if lines
        .get(insert_at)
        .map(|line| line.contains("coding:"))
        .unwrap_or(false)
    {
        insert_at += 1;
    }

    lines.insert(insert_at, "from pathlib import Path".to_string());
    lines.join("\n")
}

fn auto_rewrite_paths_to_script_dir(script_content: &str) -> (String, usize) {
    let win_double = Regex::new(r#"\"(?i:[a-z]:\\[^\"\n]+)\""#).unwrap();
    let win_single = Regex::new(r#"'(?i:[a-z]:\\[^'\n]+)'"#).unwrap();
    let unix_double = Regex::new(r#"\"(?i:/(users|home|var|etc|opt|tmp)/[^\"\n]+)\""#).unwrap();
    let unix_single = Regex::new(r#"'(?i:/(users|home|var|etc|opt|tmp)/[^'\n]+)'"#).unwrap();

    let patterns = [win_double, win_single, unix_double, unix_single];

    let mut updated = script_content.to_string();
    let mut replacements = 0usize;

    for regex in patterns {
        let source = updated.clone();
        updated = regex
            .replace_all(&source, |caps: &regex::Captures| {
                let full = caps.get(0).map(|m| m.as_str()).unwrap_or_default();
                if full.len() < 2 {
                    return full.to_string();
                }

                let inner = &full[1..full.len() - 1];
                let Some(base) = extract_basename(inner) else {
                    return full.to_string();
                };

                replacements += 1;
                format!("str((Path(__file__).parent / {:?}).resolve())", base)
            })
            .to_string();
    }

    if replacements > 0 {
        (ensure_pathlib_import(&updated), replacements)
    } else {
        (updated, 0)
    }
}

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

    let (normalized_content, rewrites) = auto_rewrite_paths_to_script_dir(&script_content);
    let portability_findings = detect_hardcoded_paths(&normalized_content);
    let was_encrypted = script_dir.join("main.py.enc").exists();
    write_script_and_requirements(&script_dir, &normalized_content)?;

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

    let rewrite_note = if rewrites > 0 {
        format!(
            "\n\n✅ Auto-poprawka: zamieniono {} hardcoded ścieżek na Path(__file__).parent.",
            rewrites
        )
    } else {
        String::new()
    };

    Ok(format!(
        "Official script '{}' updated{}{}",
        script_name,
        rewrite_note,
        format_portability_warning(&portability_findings)
    ))
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

    let (normalized_content, rewrites) = auto_rewrite_paths_to_script_dir(&script_content);
    let portability_findings = detect_hardcoded_paths(&normalized_content);
    let was_encrypted = script_dir.join("main.py.enc").exists();
    let previous_metadata = load_script_metadata(&script_dir, &new_name);
    write_script_and_requirements(&script_dir, &normalized_content)?;

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

    let rewrite_note = if rewrites > 0 {
        format!(
            "\n\n✅ Auto-poprawka: zamieniono {} hardcoded ścieżek na Path(__file__).parent.",
            rewrites
        )
    } else {
        String::new()
    };

    Ok(format!(
        "Official script '{}' saved{}{}",
        new_name,
        rewrite_note,
        format_portability_warning(&portability_findings)
    ))
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
        "__future__",
        "asyncio",
        "concurrent",
        "configparser",
        "ctypes",
        "queue",
        "subprocess",
        "threading",
        "multiprocessing",
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
    match commit_and_push_via_cli(scripts_path, &commit_msg) {
        Ok(()) => Ok(()),
        Err(cli_error) => {
            log::warn!(
                "Git CLI commit/push failed, trying libgit2 fallback: {}",
                cli_error
            );

            match commit_and_push_via_libgit2(scripts_path, &commit_msg) {
                Ok(()) => Ok(()),
                Err(libgit2_error) => {
                    log::warn!(
                        "Push skipped. Changes saved locally only. Git CLI error: {}. libgit2 error: {}",
                        cli_error,
                        libgit2_error
                    );
                    Ok(())
                }
            }
        }
    }
}

fn commit_and_push_via_cli(scripts_path: &Path, commit_msg: &str) -> Result<(), String> {
    // Git add
    let mut add_cmd = Command::new("git");
    add_cmd.args(["add", "."]).current_dir(scripts_path);
    apply_no_console_window(&mut add_cmd);
    let add_output = add_cmd
        .output()
        .map_err(|e| format!("Git add failed: {}", e))?;

    if !add_output.status.success() {
        return Err(format!(
            "Git add failed: {}",
            String::from_utf8_lossy(&add_output.stderr)
        ));
    }

    // Git commit
    let mut commit_cmd = Command::new("git");
    commit_cmd
        .args(["commit", "-m", commit_msg])
        .current_dir(scripts_path);
    apply_no_console_window(&mut commit_cmd);
    let commit_output = commit_cmd
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
    let mut push_cmd = Command::new("git");
    push_cmd.args(["push"]).current_dir(scripts_path);
    apply_no_console_window(&mut push_cmd);
    let push_output = push_cmd
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

fn default_git_signature() -> Result<Signature<'static>, String> {
    if let Ok(Some(session)) = crate::github_auth::get_current_session() {
        let name = if session.user.login.trim().is_empty() {
            "script-runner".to_string()
        } else {
            session.user.login.clone()
        };
        let email = format!("{}@users.noreply.github.com", name);
        return Signature::now(&name, &email)
            .map_err(|e| format!("Failed to create git signature: {}", e));
    }

    Signature::now("script-runner", "script-runner@local")
        .map_err(|e| format!("Failed to create fallback git signature: {}", e))
}

fn commit_and_push_via_libgit2(scripts_path: &Path, commit_msg: &str) -> Result<(), String> {
    let repo = Repository::open(scripts_path)
        .map_err(|e| format!("Failed to open repository with libgit2: {}", e))?;

    let mut index = repo
        .index()
        .map_err(|e| format!("Failed to open git index: {}", e))?;
    index
        .add_all(["*"], IndexAddOption::DEFAULT, None)
        .map_err(|e| format!("Failed to stage changes: {}", e))?;
    index
        .write()
        .map_err(|e| format!("Failed to write git index: {}", e))?;

    let statuses = repo
        .statuses(None)
        .map_err(|e| format!("Failed to read git status: {}", e))?;

    if !statuses.is_empty() {
        let tree_id = index
            .write_tree()
            .map_err(|e| format!("Failed to write git tree: {}", e))?;
        let tree = repo
            .find_tree(tree_id)
            .map_err(|e| format!("Failed to find git tree: {}", e))?;

        let signature = repo.signature().or_else(|_| default_git_signature())?;

        if let Ok(head) = repo.head() {
            if let Some(parent_oid) = head.target() {
                let parent = repo
                    .find_commit(parent_oid)
                    .map_err(|e| format!("Failed to find parent commit: {}", e))?;
                repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    commit_msg,
                    &tree,
                    &[&parent],
                )
                .map_err(|e| format!("Failed to create commit: {}", e))?;
            } else {
                repo.commit(Some("HEAD"), &signature, &signature, commit_msg, &tree, &[])
                    .map_err(|e| format!("Failed to create initial commit: {}", e))?;
            }
        } else {
            repo.commit(Some("HEAD"), &signature, &signature, commit_msg, &tree, &[])
                .map_err(|e| format!("Failed to create initial commit: {}", e))?;
        }
    }

    let mut callbacks = RemoteCallbacks::new();

    let token = crate::github_auth::get_current_session()
        .ok()
        .flatten()
        .map(|session| session.token)
        .or_else(resolve_service_push_token)
        .unwrap_or_default();

    if token.is_empty() {
        return Err(
            "Git CLI unavailable and no service push token configured (SR_SCRIPTS_PUSH_TOKEN or SR_SCRIPTS_PUSH_TOKEN_B64)."
                .to_string(),
        );
    }

    callbacks.credentials(move |_url, username_from_url, _allowed_types| {
        Cred::userpass_plaintext(username_from_url.unwrap_or("x-access-token"), &token)
    });

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(callbacks);

    let branch = repo
        .head()
        .ok()
        .and_then(|head| head.shorthand().map(|s| s.to_string()))
        .unwrap_or_else(|| "main".to_string());

    let refspec = format!("refs/heads/{0}:refs/heads/{0}", branch);

    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| format!("Failed to find remote origin: {}", e))?;

    remote
        .push(&[&refspec], Some(&mut push_options))
        .map_err(|e| format!("Failed to push via libgit2: {}", e))?;

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

    let (normalized_content, rewrites, portability_findings) = if subfolder == "official" {
        let (normalized, rewrite_count) = auto_rewrite_paths_to_script_dir(&script_content);
        let findings = detect_hardcoded_paths(&normalized);
        (normalized, rewrite_count, findings)
    } else {
        (script_content.clone(), 0, Vec::new())
    };

    fs::create_dir_all(&script_dir)
        .map_err(|e| format!("Failed to create script directory: {}", e))?;

    // Save main.py
    let script_file = script_dir.join("main.py");
    fs::write(&script_file, &normalized_content)
        .map_err(|e| format!("Failed to write script file: {}", e))?;

    // Analyze dependencies
    let dependencies = analyze_dependencies(&normalized_content)?;

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

    let rewrite_note = if rewrites > 0 {
        format!(
            "\n\n✅ Auto-poprawka: zamieniono {} hardcoded ścieżek na Path(__file__).parent.",
            rewrites
        )
    } else {
        String::new()
    };

    Ok(format!(
        "Script '{}' added successfully!{}{}",
        script_name,
        rewrite_note,
        format_portability_warning(&portability_findings)
    ))
}
