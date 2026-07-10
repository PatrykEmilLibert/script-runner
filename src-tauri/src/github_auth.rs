use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const GITHUB_API_BASE: &str = "https://api.github.com";
const ADMINS_REPO: &str = "PatrykEmilLibert/script-runner-config";

/// GitHub OAuth App client id (public identifier, not a secret) used for the
/// Device Authorization Flow. The OAuth App must have "Device Flow" enabled.
const GITHUB_CLIENT_ID: &str = "Ov23liI6qFrsfyIQZgal";
const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const ACCESS_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
/// Scope required to read the private admin config and push scripts.
const DEVICE_SCOPE: &str = "repo";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub name: Option<String>,
    pub avatar_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminConfig {
    pub version: u32,
    pub admins: Vec<AdminEntry>,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminEntry {
    pub github_username: String,
    pub role: String,
    pub added_at: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthSession {
    pub token: String,
    pub user: GitHubUser,
    pub is_admin: bool,
    pub created_at: String,
}

/// Response from GitHub's device/code endpoint - shown to the user so they can
/// authorize the app in their browser.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Raw response from GitHub's access_token endpoint during polling.
#[derive(Debug, Deserialize)]
struct DeviceTokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
    interval: Option<u64>,
}

/// Result of a single poll attempt against the access_token endpoint. The
/// frontend keeps polling on `Pending`/`SlowDown` until it gets `Authorized`.
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DeviceFlowPoll {
    /// User has not authorized yet - keep polling at the current interval.
    Pending,
    /// GitHub asks us to slow down - increase the polling interval (seconds).
    SlowDown { interval: u64 },
    /// User authorized the app - session is ready and saved.
    Authorized { session: AuthSession },
}

fn auth_file_path() -> PathBuf {
    if let Some(data_dir) = dirs::data_dir() {
        let auth_dir = data_dir.join("ScriptRunner").join("auth");
        let _ = fs::create_dir_all(&auth_dir);
        auth_dir.join("github_session.json")
    } else {
        PathBuf::from(".cache/github_session.json")
    }
}

/// Verify a token, resolve admin status, and persist the session locally.
/// Shared by both PAT login and the device flow.
async fn build_session(token: String) -> Result<AuthSession, String> {
    // 1. Verify token and get user info
    let user = fetch_github_user(&token).await?;

    // 2. Check if user is admin
    let is_admin = check_if_admin(&token, &user.login).await?;

    // 3. Create session
    let session = AuthSession {
        token,
        user,
        is_admin,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    // 4. Save session locally (encrypted would be better in production)
    save_session(&session)?;

    log::info!(
        "GitHub login successful: {} (admin: {})",
        session.user.login,
        session.is_admin
    );
    Ok(session)
}

/// Login with GitHub PAT - verifies token and checks admin status
pub async fn github_login(token: String) -> Result<AuthSession, String> {
    build_session(token).await
}

/// Step 1 of the device flow: request a device + user code from GitHub.
pub async fn start_device_flow() -> Result<DeviceCodeResponse, String> {
    let client = reqwest::Client::new();

    let response = client
        .post(DEVICE_CODE_URL)
        .header("Accept", "application/json")
        .header("User-Agent", "ScriptRunner")
        .form(&[("client_id", GITHUB_CLIENT_ID), ("scope", DEVICE_SCOPE)])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to GitHub: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!(
            "GitHub device flow request failed: {} - {}. \
             Make sure the OAuth App has Device Flow enabled.",
            status, body
        ));
    }

    let data: DeviceCodeResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse device code response: {}", e))?;

    log::info!(
        "Device flow started (user_code: {}, verification_uri: {})",
        data.user_code,
        data.verification_uri
    );
    Ok(data)
}

/// Step 2 of the device flow: poll once for the access token. The frontend
/// calls this repeatedly (respecting `interval`) until it gets `Authorized`.
pub async fn poll_device_flow(device_code: String) -> Result<DeviceFlowPoll, String> {
    let client = reqwest::Client::new();

    let response = client
        .post(ACCESS_TOKEN_URL)
        .header("Accept", "application/json")
        .header("User-Agent", "ScriptRunner")
        .form(&[
            ("client_id", GITHUB_CLIENT_ID),
            ("device_code", device_code.as_str()),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to GitHub: {}", e))?;

    let token_resp: DeviceTokenResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    // Success path
    if let Some(token) = token_resp.access_token {
        let session = build_session(token).await?;
        return Ok(DeviceFlowPoll::Authorized { session });
    }

    // Still waiting / recoverable states vs. terminal errors
    match token_resp.error.as_deref() {
        Some("authorization_pending") => Ok(DeviceFlowPoll::Pending),
        Some("slow_down") => Ok(DeviceFlowPoll::SlowDown {
            interval: token_resp.interval.unwrap_or(5),
        }),
        Some("expired_token") => {
            Err("The verification code expired. Please start the login again.".to_string())
        }
        Some("access_denied") => {
            Err("Authorization was cancelled or denied in the browser.".to_string())
        }
        Some(other) => Err(format!(
            "Device flow error: {}{}",
            other,
            token_resp
                .error_description
                .map(|d| format!(" - {}", d))
                .unwrap_or_default()
        )),
        None => Err("Unexpected response from GitHub during device flow.".to_string()),
    }
}

/// Logout - clear local session
pub fn github_logout() -> Result<(), String> {
    let path = auth_file_path();
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Failed to remove session file: {}", e))?;
    }
    log::info!("GitHub logout successful");
    Ok(())
}

/// Get current session if exists
pub fn get_current_session() -> Result<Option<AuthSession>, String> {
    let path = auth_file_path();

    if !path.exists() {
        return Ok(None);
    }

    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read session: {}", e))?;

    let session: AuthSession =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse session: {}", e))?;

    Ok(Some(session))
}

/// Check if currently logged in user is admin
pub async fn check_admin_status() -> Result<bool, String> {
    match get_current_session()? {
        Some(session) => {
            // Re-verify admin status from GitHub (in case admins.json changed)
            check_if_admin(&session.token, &session.user.login).await
        }
        None => Ok(false), // Not logged in = not admin
    }
}

/// Get current user info (or None if not logged in)
pub fn get_current_user() -> Result<Option<GitHubUser>, String> {
    match get_current_session()? {
        Some(session) => Ok(Some(session.user)),
        None => Ok(None),
    }
}

/// Fetch GitHub user info from API
async fn fetch_github_user(token: &str) -> Result<GitHubUser, String> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/user", GITHUB_API_BASE))
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "ScriptRunner")
        .header("Accept", "application/vnd.github.v3+json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to GitHub: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub API error: {} - Invalid token or insufficient permissions",
            response.status()
        ));
    }

    let user: GitHubUser = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub response: {}", e))?;

    Ok(user)
}

/// Fetch admins.json from private repo and check if user is admin
async fn check_if_admin(token: &str, username: &str) -> Result<bool, String> {
    let client = reqwest::Client::new();

    // Fetch admins.json from private repo
    let response = client
        .get(format!(
            "{}/repos/{}/contents/admins.json",
            GITHUB_API_BASE, ADMINS_REPO
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "ScriptRunner")
        .header("Accept", "application/vnd.github.v3.raw")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch admin config: {}", e))?;

    if !response.status().is_success() {
        log::warn!(
            "Could not fetch admins.json (status: {}). User '{}' treated as non-admin.",
            response.status(),
            username
        );
        return Ok(false);
    }

    let content = response
        .text()
        .await
        .map_err(|e| format!("Failed to read admin config: {}", e))?;

    let config: AdminConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse admin config: {}", e))?;

    // Check if username is in admin list
    let is_admin = config
        .admins
        .iter()
        .any(|admin| admin.github_username.eq_ignore_ascii_case(username));

    log::info!("Admin check for '{}': {}", username, is_admin);
    Ok(is_admin)
}

/// Save session to local file
fn save_session(session: &AuthSession) -> Result<(), String> {
    let path = auth_file_path();

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create auth directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(session)
        .map_err(|e| format!("Failed to serialize session: {}", e))?;

    fs::write(&path, content).map_err(|e| format!("Failed to write session file: {}", e))?;

    Ok(())
}

/// Refresh admin status for current session (call after changes to admins.json)
pub async fn refresh_admin_status() -> Result<bool, String> {
    match get_current_session()? {
        Some(mut session) => {
            let is_admin = check_if_admin(&session.token, &session.user.login).await?;
            session.is_admin = is_admin;
            save_session(&session)?;
            Ok(is_admin)
        }
        None => Ok(false),
    }
}
