use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn default_auto_update_enabled() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppSettings {
    pub dark_mode: bool,
    #[serde(default = "default_auto_update_enabled")]
    pub auto_update_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            dark_mode: false,
            auto_update_enabled: true,
        }
    }
}

pub fn settings_file() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let dir = PathBuf::from(appdata).join("script-runner");
        let _ = fs::create_dir_all(&dir);
        return dir.join("settings.json");
    }
    PathBuf::from("./settings.json")
}

pub fn load_settings() -> Result<AppSettings, String> {
    let path = settings_file();
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read settings: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = settings_file();
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write settings: {}", e))?;
    Ok(())
}

pub fn toggle_dark_mode() -> Result<bool, String> {
    let mut settings = load_settings()?;
    settings.dark_mode = !settings.dark_mode;
    save_settings(&settings)?;
    Ok(settings.dark_mode)
}

pub fn set_auto_update(enabled: bool) -> Result<bool, String> {
    let mut settings = load_settings()?;
    settings.auto_update_enabled = enabled;
    save_settings(&settings)?;
    Ok(settings.auto_update_enabled)
}
