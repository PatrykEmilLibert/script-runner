use chrono::Utc;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct AdminKeyFile {
    pub version: u32,
    pub key: String, // 256 hex characters
    pub created_at: String,
}

pub fn desktop_key_path() -> PathBuf {
    // 1. Check env override first
    if let Ok(custom) = env::var("SR_ADMIN_KEY_PATH") {
        return PathBuf::from(custom);
    }

    // 2. Try dirs crate (most reliable on normal systems)
    if let Some(desktop) = dirs::desktop_dir() {
        return desktop.join("sr-admin.key");
    }

    // 3. Windows: try USERPROFILE\Desktop
    #[cfg(target_os = "windows")]
    {
        if let Ok(userprofile) = env::var("USERPROFILE") {
            let path = PathBuf::from(userprofile)
                .join("Desktop")
                .join("sr-admin.key");
            if path.parent().map_or(false, |p| p.exists()) {
                return path;
            }
        }

        // Fallback: common Windows paths
        let common_paths = vec![
            PathBuf::from("C:\\Users\\Public\\Desktop\\sr-admin.key"),
            PathBuf::from("~/Desktop/sr-admin.key"),
        ];
        for path in common_paths {
            if let Some(parent) = path.parent() {
                if parent.exists() {
                    return path;
                }
            }
        }

        // Last resort: Program Files or AppData
        if let Ok(appdata) = env::var("APPDATA") {
            return PathBuf::from(appdata)
                .join("script-runner")
                .join("sr-admin.key");
        }

        PathBuf::from("C:\\sr-admin.key")
    }

    // 4. macOS/Linux: try common paths
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home).join("Desktop").join("sr-admin.key");
        }
        PathBuf::from("/tmp/sr-admin.key")
    }
}

#[allow(dead_code)]
pub fn generate_hex_key_256() -> String {
    // 128 random bytes -> 256 hex characters
    let mut bytes = [0u8; 128];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[allow(dead_code)]
pub fn write_key_file(path: &Path) -> Result<AdminKeyFile, String> {
    let key_hex = generate_hex_key_256();
    let payload = AdminKeyFile {
        version: 1,
        key: key_hex,
        created_at: Utc::now().to_rfc3339(),
    };
    let content =
        serde_json::to_string(&payload).map_err(|e| format!("Failed to serialize key: {}", e))?;
    fs::write(path, content).map_err(|e| format!("Failed to write key file: {}", e))?;
    Ok(payload)
}

pub fn validate_key_file(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    match fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str::<AdminKeyFile>(&content) {
            Ok(parsed) => {
                parsed.version == 1
                    && parsed.key.len() == 256
                    && parsed.key.chars().all(|c| c.is_ascii_hexdigit())
            }
            Err(_) => false,
        },
        Err(_) => false,
    }
}

pub fn verify_admin_key(provided_key: &str) -> Result<bool, String> {
    let key_path = desktop_key_path();
    
    if !key_path.exists() {
        return Err("Admin key file not found. Generate a key first.".to_string());
    }
    
    let content = fs::read_to_string(&key_path)
        .map_err(|e| format!("Failed to read key file: {}", e))?;
    
    let parsed: AdminKeyFile = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse key file: {}", e))?;
    
    Ok(provided_key == parsed.key)
}
