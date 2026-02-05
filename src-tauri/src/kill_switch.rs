use reqwest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

const CACHE_FILE: &str = "kill_switch_cache.json";
const CACHE_DURATION_HOURS: i64 = 24;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillSwitchConfig {
    pub blocked: bool,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_until: Option<String>, // ISO 8601 timestamp
    #[serde(default)]
    pub whitelist: Vec<String>,        // machine IDs
    #[serde(default)]
    pub message: String,               // custom message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_at: Option<String>,     // internal use for cache validation
}

impl Default for KillSwitchConfig {
    fn default() -> Self {
        Self {
            blocked: false,
            reason: String::new(),
            blocked_until: None,
            whitelist: Vec::new(),
            message: String::from("Application is running normally."),
            redirect_url: None,
            cached_at: None,
        }
    }
}

/// Legacy function for backward compatibility
pub async fn check_remote_status() -> Result<bool, String> {
    match check_remote_status_advanced().await {
        Ok(config) => Ok(should_block(&config)),
        Err(e) => {
            log::warn!("Failed to check kill switch: {}. Allowing app to run.", e);
            Ok(false)
        }
    }
}

/// Fetches full kill switch configuration from GitHub
pub async fn check_remote_status_advanced() -> Result<KillSwitchConfig, String> {
    let client = reqwest::Client::new();
    
    match client
        .get("https://api.github.com/repos/PatrykEmilLibert/script-runner-config/contents/kill_switch.json")
        .header("Accept", "application/vnd.github.v3.raw")
        .header("User-Agent", "ScriptRunner-App")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => {
            match response.text().await {
                Ok(text) => {
                    match serde_json::from_str::<KillSwitchConfig>(&text) {
                        Ok(mut config) => {
                            // Cache successful fetch
                            config.cached_at = Some(chrono::Utc::now().to_rfc3339());
                            let _ = cache_kill_switch_status(&config);
                            Ok(config)
                        }
                        Err(e) => {
                            log::warn!("Failed to parse kill switch config: {}", e);
                            // Try to use cached version
                            if let Some(cached) = get_cached_status() {
                                log::info!("Using cached kill switch status");
                                Ok(cached)
                            } else {
                                Ok(KillSwitchConfig::default())
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to read kill switch response: {}", e);
                    if let Some(cached) = get_cached_status() {
                        log::info!("Using cached kill switch status");
                        Ok(cached)
                    } else {
                        Ok(KillSwitchConfig::default())
                    }
                }
            }
        }
        Err(e) => {
            log::warn!("No internet connection or kill switch unreachable: {}", e);
            if let Some(cached) = get_cached_status() {
                log::info!("Using cached kill switch status (offline mode)");
                Ok(cached)
            } else {
                Ok(KillSwitchConfig::default())
            }
        }
    }
}

/// Checks if the current machine is whitelisted
pub fn is_machine_whitelisted(config: &KillSwitchConfig) -> bool {
    let machine_id = get_machine_id();
    config.whitelist.iter().any(|id| id == &machine_id)
}

/// Determines if the application should be blocked based on config
pub fn should_block(config: &KillSwitchConfig) -> bool {
    // Check if whitelisted first
    if is_machine_whitelisted(config) {
        log::info!("Machine is whitelisted, bypassing kill switch");
        return false;
    }

    // Check if blocked
    if !config.blocked {
        return false;
    }

    // Check time-based blocking
    if let Some(until) = &config.blocked_until {
        match chrono::DateTime::parse_from_rfc3339(until) {
            Ok(until_time) => {
                let now = chrono::Utc::now();
                if now.signed_duration_since(until_time.with_timezone(&chrono::Utc)).num_seconds() > 0 {
                    log::info!("Block time expired, allowing app to run");
                    return false;
                }
            }
            Err(e) => {
                log::warn!("Invalid blocked_until timestamp: {}", e);
            }
        }
    }

    log::error!("Application is blocked: {}", config.reason);
    true
}

/// Generates or retrieves a unique machine ID
pub fn get_machine_id() -> String {
    let cache_dir = get_cache_dir();
    let machine_id_file = cache_dir.join("machine_id.txt");

    // Try to read existing ID
    if let Ok(id) = fs::read_to_string(&machine_id_file) {
        let trimmed = id.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    // Generate new ID
    let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "unknown-host".to_string());
    let new_id = format!(
        "{}-{}-{}",
        hostname,
        whoami::username(),
        uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown")
    );

    // Save for future use
    let _ = fs::create_dir_all(&cache_dir);
    let _ = fs::write(&machine_id_file, &new_id);

    new_id
}

/// Caches kill switch status locally
pub fn cache_kill_switch_status(config: &KillSwitchConfig) -> Result<(), String> {
    let cache_dir = get_cache_dir();
    let cache_file = cache_dir.join(CACHE_FILE);

    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;

    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&cache_file, json)
        .map_err(|e| format!("Failed to write cache file: {}", e))?;

    log::info!("Kill switch status cached successfully");
    Ok(())
}

/// Retrieves cached kill switch status if valid
pub fn get_cached_status() -> Option<KillSwitchConfig> {
    let cache_dir = get_cache_dir();
    let cache_file = cache_dir.join(CACHE_FILE);

    if !cache_file.exists() {
        return None;
    }

    match fs::read_to_string(&cache_file) {
        Ok(content) => {
            match serde_json::from_str::<KillSwitchConfig>(&content) {
                Ok(config) => {
                    // Check if cache is still valid (24 hours)
                    if let Some(cached_at) = &config.cached_at {
                        if let Ok(cached_time) = chrono::DateTime::parse_from_rfc3339(cached_at) {
                            let now = chrono::Utc::now();
                            let duration = now.signed_duration_since(cached_time.with_timezone(&chrono::Utc));
                            
                            if duration.num_hours() < CACHE_DURATION_HOURS {
                                log::info!("Using valid cached kill switch status (age: {} hours)", duration.num_hours());
                                return Some(config);
                            } else {
                                log::warn!("Cache expired (age: {} hours)", duration.num_hours());
                            }
                        }
                    }
                    // Return even if expired - better than nothing in offline mode
                    Some(config)
                }
                Err(e) => {
                    log::warn!("Failed to parse cached config: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to read cache file: {}", e);
            None
        }
    }
}

/// Gets the cache directory path
fn get_cache_dir() -> PathBuf {
    if let Some(data_dir) = dirs::data_dir() {
        data_dir.join("ScriptRunner").join("cache")
    } else {
        PathBuf::from(".cache")
    }
}
