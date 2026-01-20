use reqwest;

pub async fn check_remote_status() -> Result<bool, String> {
    // Require internet connection - no offline mode
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/YOUR_USERNAME/script-runner-config/contents/kill_switch.json")
        .header("Accept", "application/vnd.github.v3.raw")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("No internet connection. Application requires network access: {}", e))?;
    
    let text = response.text().await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    let data = serde_json::from_str::<serde_json::Value>(&text)
        .map_err(|e| format!("Failed to parse kill switch data: {}", e))?;
    
    let is_blocked = data["blocked"].as_bool().unwrap_or(false);
    
    if is_blocked {
        log::error!("Application has been remotely blocked!");
        return Ok(true);
    }
    
    Ok(false)
}
