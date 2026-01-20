use chrono::Utc;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct AdminKeyFile {
    pub version: u32,
    pub key: String, // 256 hex characters
    pub created_at: String,
}

pub fn desktop_key_path() -> PathBuf {
    if let Some(desktop) = dirs::desktop_dir() {
        return desktop.join("sr-admin.key");
    }
    #[cfg(target_os = "windows")]
    {
        PathBuf::from("C:/Users/Public/Desktop/sr-admin.key")
    }
    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("/tmp/sr-admin.key")
    }
}

pub fn generate_hex_key_256() -> String {
    // 128 random bytes -> 256 hex characters
    let mut bytes = [0u8; 128];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

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
