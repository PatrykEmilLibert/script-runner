use reqwest;

pub async fn check_remote_status() -> Result<bool, String> {
    // Try to check kill switch, but allow app to run if it fails
    let client = reqwest::Client::new();
    
    match client
        .get("https://api.github.com/repos/PatrykEmilLibert/script-runner-config/contents/kill_switch.json")
        .header("Accept", "application/vnd.github.v3.raw")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => {
            match response.text().await {
                Ok(text) => {
                    match serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(data) => {
                            let is_blocked = data["blocked"].as_bool().unwrap_or(false);
                            if is_blocked {
                                log::error!("Application has been remotely blocked!");
                                return Ok(true);
                            }
                            Ok(false)
                        }
                        Err(e) => {
                            log::warn!("Failed to parse kill switch data: {}. Allowing app to run.", e);
                            Ok(false)
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to read kill switch response: {}. Allowing app to run.", e);
                    Ok(false)
                }
            }
        }
        Err(e) => {
            log::warn!("No internet connection or kill switch unreachable: {}. Allowing app to run.", e);
            Ok(false)
        }
    }
}
