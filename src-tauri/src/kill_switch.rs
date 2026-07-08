use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CACHE_FILE: &str = "kill_switch_cache.json";
const CACHE_DURATION_HOURS: i64 = 24;
// Kill switch config is hosted in the PUBLIC script-runner-scripts repo so every
// client can read it with no token. admins.json stays in the private
// script-runner-config repo; the two are intentionally separated.
const DEFAULT_GITHUB_KILL_SWITCH_API_URL: &str =
    "https://api.github.com/repos/PatrykEmilLibert/script-runner-scripts/contents/kill_switch.json";
const DEFAULT_PUBLIC_KILL_SWITCH_URL: &str =
    "https://raw.githubusercontent.com/PatrykEmilLibert/script-runner-scripts/main/kill_switch.json";
const COMPILED_KILL_SWITCH_URL: Option<&str> = option_env!("SR_KILL_SWITCH_URL");
const COMPILED_KILL_SWITCH_READ_TOKEN: Option<&str> = option_env!("SR_KILL_SWITCH_READ_TOKEN");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KillSwitchConfig {
    pub blocked: bool,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_until: Option<String>, // ISO 8601 timestamp
    #[serde(default)]
    pub whitelist: Vec<String>, // machine IDs
    #[serde(default)]
    pub message: String, // custom message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_at: Option<String>, // internal use for cache validation
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

fn resolve_github_token() -> Option<String> {
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.trim().is_empty() {
            return Some(token);
        }
    }

    if let Ok(Some(session)) = crate::github_auth::get_current_session() {
        if !session.token.trim().is_empty() {
            return Some(session.token);
        }
    }

    None
}

fn resolve_kill_switch_read_token() -> Option<String> {
    if let Some(token) = resolve_github_token() {
        return Some(token);
    }

    if let Ok(token) = std::env::var("SR_KILL_SWITCH_READ_TOKEN") {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    COMPILED_KILL_SWITCH_READ_TOKEN
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_string)
}

fn resolve_primary_kill_switch_url() -> Option<String> {
    if let Ok(url) = std::env::var("SR_KILL_SWITCH_URL") {
        let trimmed = url.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    COMPILED_KILL_SWITCH_URL
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .map(str::to_string)
}

fn candidate_kill_switch_urls() -> Vec<String> {
    let mut urls = Vec::new();

    if let Some(primary) = resolve_primary_kill_switch_url() {
        urls.push(primary);
    }

    urls.push(DEFAULT_GITHUB_KILL_SWITCH_API_URL.to_string());
    urls.push(DEFAULT_PUBLIC_KILL_SWITCH_URL.to_string());

    urls
}

async fn fetch_config_from_url(
    client: &reqwest::Client,
    url: &str,
    read_token: Option<&str>,
) -> Result<KillSwitchConfig, String> {
    let mut request = client
        .get(url)
        .header("Accept", "application/vnd.github.v3.raw")
        .header("User-Agent", "ScriptRunner-App")
        .timeout(std::time::Duration::from_secs(6));

    if let Some(token) = read_token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("{} (source: {})", e, url))?;

    if !response.status().is_success() {
        let status = response.status();
        let response_body = response.text().await.unwrap_or_default();
        return Err(format!(
            "HTTP {} from {}{}",
            status,
            url,
            if response_body.is_empty() {
                String::new()
            } else {
                format!(". Body: {}", response_body)
            }
        ));
    }

    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed reading body from {}: {}", url, e))?;

    serde_json::from_str::<KillSwitchConfig>(&text)
        .map_err(|e| format!("Invalid JSON from {}: {}", url, e))
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
    let read_token = resolve_kill_switch_read_token();
    let mut last_error: Option<String> = None;

    for url in candidate_kill_switch_urls() {
        match fetch_config_from_url(&client, &url, read_token.as_deref()).await {
            Ok(mut config) => {
                config.cached_at = Some(chrono::Utc::now().to_rfc3339());
                let _ = cache_kill_switch_status(&config);
                log::info!("Kill switch config loaded from {}", url);
                return Ok(config);
            }
            Err(err) => {
                log::warn!("Kill switch source unavailable: {}", err);
                last_error = Some(err);
            }
        }
    }

    if let Some(cached) = get_cached_status() {
        log::info!("Using cached kill switch status after remote fetch failure");
        return Ok(cached);
    }

    if read_token.is_none() {
        log::warn!(
            "Kill switch fetched with no token. Configure SR_KILL_SWITCH_URL (public JSON) or SR_KILL_SWITCH_READ_TOKEN for private config repo access."
        );
    }

    if let Some(err) = last_error {
        log::warn!("No kill switch source succeeded: {}", err);
    }

    Ok(KillSwitchConfig::default())
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
                if now
                    .signed_duration_since(until_time.with_timezone(&chrono::Utc))
                    .num_seconds()
                    > 0
                {
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
        uuid::Uuid::new_v4()
            .to_string()
            .split('-')
            .next()
            .unwrap_or("unknown")
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

    fs::write(&cache_file, json).map_err(|e| format!("Failed to write cache file: {}", e))?;

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
                            let duration =
                                now.signed_duration_since(cached_time.with_timezone(&chrono::Utc));

                            if duration.num_hours() < CACHE_DURATION_HOURS {
                                log::info!(
                                    "Using valid cached kill switch status (age: {} hours)",
                                    duration.num_hours()
                                );
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
