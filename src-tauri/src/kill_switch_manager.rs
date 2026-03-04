use crate::kill_switch::KillSwitchConfig;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::json;
use std::env;

const GITHUB_API_BASE: &str = "https://api.github.com/repos/PatrykEmilLibert/script-runner-config";

fn resolve_github_token() -> Result<String, String> {
    if let Ok(token) = env::var("GITHUB_TOKEN") {
        if !token.trim().is_empty() {
            return Ok(token);
        }
    }

    if let Ok(Some(session)) = crate::github_auth::get_current_session() {
        if !session.token.trim().is_empty() {
            return Ok(session.token);
        }
    }

    Err("No GitHub token available. Log in as GitHub admin or set GITHUB_TOKEN.".to_string())
}

fn cache_local_config(config: &KillSwitchConfig) {
    let mut cached = config.clone();
    cached.cached_at = Some(chrono::Utc::now().to_rfc3339());
    let _ = crate::kill_switch::cache_kill_switch_status(&cached);
}

/// Toggle kill switch on or off
pub async fn toggle_kill_switch(blocked: bool, reason: String) -> Result<String, String> {
    let mut config = fetch_current_config().await?;

    config.blocked = blocked;
    config.reason = reason.clone();

    if !blocked {
        // Clear time-based blocking when unblocking
        config.blocked_until = None;
    }

    push_config_to_github(config).await?;

    // Keep local cache in sync (important for post-logout checks)
    let mut cached_config = fetch_current_config().await.unwrap_or(KillSwitchConfig {
        blocked,
        reason: reason.clone(),
        blocked_until: None,
        whitelist: Vec::new(),
        message: String::new(),
        redirect_url: None,
        cached_at: None,
    });
    cached_config.blocked = blocked;
    cached_config.reason = reason.clone();
    if !blocked {
        cached_config.blocked_until = None;
    }
    cache_local_config(&cached_config);

    let status = if blocked { "enabled" } else { "disabled" };
    log::info!("Kill switch {} with reason: {}", status, reason);
    Ok(format!("Kill switch {} successfully", status))
}

/// Schedule a time-based block
pub async fn schedule_block(until: String, reason: String) -> Result<String, String> {
    // Validate timestamp
    chrono::DateTime::parse_from_rfc3339(&until).map_err(|e| {
        format!(
            "Invalid timestamp format: {}. Use ISO 8601 (e.g., 2026-02-10T12:00:00Z)",
            e
        )
    })?;

    let mut config = fetch_current_config().await?;

    config.blocked = true;
    config.blocked_until = Some(until.clone());
    config.reason = reason.clone();

    push_config_to_github(config).await?;

    let mut cached_config = fetch_current_config().await.unwrap_or(KillSwitchConfig {
        blocked: true,
        reason: reason.clone(),
        blocked_until: Some(until.clone()),
        whitelist: Vec::new(),
        message: String::new(),
        redirect_url: None,
        cached_at: None,
    });
    cached_config.blocked = true;
    cached_config.reason = reason.clone();
    cached_config.blocked_until = Some(until.clone());
    cache_local_config(&cached_config);

    log::info!(
        "Kill switch scheduled until {} with reason: {}",
        until,
        reason
    );
    Ok(format!("Kill switch scheduled until {}", until))
}

/// Add machine ID to whitelist
pub async fn add_to_whitelist(machine_id: String) -> Result<String, String> {
    let mut config = fetch_current_config().await?;

    if config.whitelist.contains(&machine_id) {
        return Ok(format!("Machine {} is already whitelisted", machine_id));
    }

    config.whitelist.push(machine_id.clone());
    push_config_to_github(config).await?;

    let mut cached_config = fetch_current_config().await.unwrap_or(KillSwitchConfig::default());
    if !cached_config.whitelist.contains(&machine_id) {
        cached_config.whitelist.push(machine_id.clone());
    }
    cache_local_config(&cached_config);

    log::info!("Added machine {} to whitelist", machine_id);
    Ok(format!("Machine {} added to whitelist", machine_id))
}

/// Remove machine ID from whitelist
pub async fn remove_from_whitelist(machine_id: String) -> Result<String, String> {
    let mut config = fetch_current_config().await?;

    let original_len = config.whitelist.len();
    config.whitelist.retain(|id| id != &machine_id);

    if config.whitelist.len() == original_len {
        return Ok(format!("Machine {} was not in whitelist", machine_id));
    }

    push_config_to_github(config).await?;

    let mut cached_config = fetch_current_config().await.unwrap_or(KillSwitchConfig::default());
    cached_config.whitelist.retain(|id| id != &machine_id);
    cache_local_config(&cached_config);

    log::info!("Removed machine {} from whitelist", machine_id);
    Ok(format!("Machine {} removed from whitelist", machine_id))
}

/// Set custom message for blocked users
pub async fn set_custom_message(
    message: String,
    redirect_url: Option<String>,
) -> Result<String, String> {
    let mut config = fetch_current_config().await?;

    config.message = message.clone();
    config.redirect_url = redirect_url.clone();

    push_config_to_github(config).await?;

    let mut cached_config = fetch_current_config().await.unwrap_or(KillSwitchConfig::default());
    cached_config.message = message;
    cached_config.redirect_url = redirect_url;
    cache_local_config(&cached_config);

    log::info!("Updated kill switch message");
    Ok("Custom message updated successfully".to_string())
}

/// Fetch current configuration from GitHub
async fn fetch_current_config() -> Result<KillSwitchConfig, String> {
    let client = reqwest::Client::new();
    let maybe_token = resolve_github_token().ok();

    let mut request = client
        .get(format!("{}/contents/kill_switch.json", GITHUB_API_BASE))
        .header("Accept", "application/vnd.github.v3.raw")
        .header("User-Agent", "ScriptRunner-Admin")
        .timeout(std::time::Duration::from_secs(10));

    if let Some(token) = maybe_token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("Failed to fetch config from GitHub: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!(
            "Failed to fetch kill switch config (status: {}). {}",
            status, body
        ));
    }

    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    serde_json::from_str::<KillSwitchConfig>(&text)
        .map_err(|e| format!("Failed to parse config: {}", e))
}

/// Push updated configuration to GitHub
pub async fn push_config_to_github(mut config: KillSwitchConfig) -> Result<(), String> {
    let github_token = resolve_github_token()?;

    // Remove cached_at before pushing
    config.cached_at = None;

    let client = reqwest::Client::new();

    // First, get the current file to obtain its SHA
    let current_file_response = client
        .get(format!("{}/contents/kill_switch.json", GITHUB_API_BASE))
        .header("Authorization", format!("Bearer {}", github_token))
        .header("User-Agent", "ScriptRunner-Admin")
        .header("Accept", "application/vnd.github.v3+json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch current file: {}", e))?;

    let current_file: serde_json::Value = current_file_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse current file response: {}", e))?;

    let sha = current_file["sha"]
        .as_str()
        .ok_or("Failed to get file SHA from GitHub response")?;

    // Serialize the new config
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    // Encode content to base64
    let encoded_content = STANDARD.encode(&config_json);

    // Create update payload
    let update_payload = json!({
        "message": format!("Update kill switch: {}", config.reason),
        "content": encoded_content,
        "sha": sha,
        "branch": "main"
    });

    // Push the update
    let update_response = client
        .put(format!("{}/contents/kill_switch.json", GITHUB_API_BASE))
        .header("Authorization", format!("Bearer {}", github_token))
        .header("User-Agent", "ScriptRunner-Admin")
        .header("Accept", "application/vnd.github.v3+json")
        .json(&update_payload)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("Failed to push config to GitHub: {}", e))?;

    if !update_response.status().is_success() {
        let error_text = update_response.text().await.unwrap_or_default();
        return Err(format!("GitHub API error: {}", error_text));
    }

    log::info!("Successfully pushed kill switch config to GitHub");
    Ok(())
}

/// Emergency local override (admin only, temporary)
pub fn create_local_override(allow: bool) -> Result<String, String> {
    let override_file = if let Some(data_dir) = dirs::data_dir() {
        data_dir
            .join("ScriptRunner")
            .join("cache")
            .join("kill_switch_override.json")
    } else {
        std::path::PathBuf::from(".cache/kill_switch_override.json")
    };

    let override_data = json!({
        "override_enabled": true,
        "allow_execution": allow,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "expires_at": chrono::Utc::now().checked_add_signed(chrono::Duration::hours(1))
            .map(|t| t.to_rfc3339())
    });

    std::fs::create_dir_all(override_file.parent().unwrap())
        .map_err(|e| format!("Failed to create override directory: {}", e))?;

    std::fs::write(
        &override_file,
        serde_json::to_string_pretty(&override_data).unwrap(),
    )
    .map_err(|e| format!("Failed to write override file: {}", e))?;

    log::warn!("Local kill switch override created (expires in 1 hour)");
    Ok("Local override created successfully (valid for 1 hour)".to_string())
}

/// Check if local override is active and valid
pub fn check_local_override() -> Option<bool> {
    let override_file = if let Some(data_dir) = dirs::data_dir() {
        data_dir
            .join("ScriptRunner")
            .join("cache")
            .join("kill_switch_override.json")
    } else {
        std::path::PathBuf::from(".cache/kill_switch_override.json")
    };

    if !override_file.exists() {
        return None;
    }

    match std::fs::read_to_string(&override_file) {
        Ok(content) => {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                // Check expiration
                if let Some(expires_at) = data["expires_at"].as_str() {
                    if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                        if chrono::Utc::now() > expires.with_timezone(&chrono::Utc) {
                            log::info!("Local override expired, removing");
                            let _ = std::fs::remove_file(&override_file);
                            return None;
                        }

                        return data["allow_execution"].as_bool();
                    }
                }
            }
            None
        }
        Err(_) => None,
    }
}
