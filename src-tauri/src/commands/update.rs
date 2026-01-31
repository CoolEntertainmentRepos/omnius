use serde::{Deserialize, Serialize};

const GITHUB_OWNER: &str = "flakerim";
const GITHUB_REPO: &str = "streamer";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let v = version.trim_start_matches('v');
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() >= 3 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;
        Some((major, minor, patch))
    } else {
        None
    }
}

fn is_newer_version(current: &str, latest: &str) -> bool {
    match (parse_version(current), parse_version(latest)) {
        (Some((cm, cn, cp)), Some((lm, ln, lp))) => {
            (lm, ln, lp) > (cm, cn, cp)
        }
        _ => false,
    }
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    println!("[check_for_updates] Checking for updates, current version: {}", CURRENT_VERSION);

    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        GITHUB_OWNER, GITHUB_REPO
    );

    let client = reqwest::Client::builder()
        .user_agent("Streamer-App")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch release info: {}", e))?;

    if !response.status().is_success() {
        // No releases yet or repo doesn't exist
        println!("[check_for_updates] No releases found or API error: {}", response.status());
        return Ok(UpdateInfo {
            current_version: CURRENT_VERSION.to_string(),
            latest_version: CURRENT_VERSION.to_string(),
            update_available: false,
            download_url: None,
            release_notes: None,
        });
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release info: {}", e))?;

    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let update_available = is_newer_version(CURRENT_VERSION, &latest_version);

    // Find APK asset
    let download_url = release
        .assets
        .iter()
        .find(|a| a.name.ends_with(".apk"))
        .map(|a| a.browser_download_url.clone());

    println!(
        "[check_for_updates] Latest: {}, Current: {}, Update available: {}",
        latest_version, CURRENT_VERSION, update_available
    );

    Ok(UpdateInfo {
        current_version: CURRENT_VERSION.to_string(),
        latest_version,
        update_available,
        download_url,
        release_notes: release.body,
    })
}

#[tauri::command]
pub fn get_app_version() -> String {
    CURRENT_VERSION.to_string()
}
