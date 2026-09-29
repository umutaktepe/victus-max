use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateChannel {
    Canary, // Tracks commits on 'main' branch
    Stable, // Tracks GitHub Releases (falls back to Canary if no releases exist)
}

impl Default for UpdateChannel {
    fn default() -> Self {
        UpdateChannel::Canary
    }
}

impl UpdateChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Canary => "canary",
            Self::Stable => "stable",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "stable" => Self::Stable,
            _ => Self::Canary,
        }
    }
}

pub fn load_update_channel() -> UpdateChannel {
    if let Ok(home) = std::env::var("HOME") {
        let p_victus = format!("{}/.config/victus-max/settings.json", home);
        let p_fallback = format!("{}/.config/omenspace/settings.json", home);
        let content = std::fs::read_to_string(&p_victus).or_else(|_| std::fs::read_to_string(&p_fallback));
        if let Ok(json_str) = content {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                if let Some(ch_str) = json.get("update_channel").and_then(|v| v.as_str()) {
                    return UpdateChannel::from_str_loose(ch_str);
                }
            }
        }
    }
    UpdateChannel::Canary
}

pub fn save_update_channel(channel: UpdateChannel) -> Result<(), std::io::Error> {
    if let Ok(home) = std::env::var("HOME") {
        let dir = format!("{}/.config/victus-max", home);
        let _ = std::fs::create_dir_all(&dir);
        let path = format!("{}/settings.json", dir);
        let path_fallback = format!("{}/.config/omenspace/settings.json", home);
        let mut json = serde_json::json!({});
        let content = std::fs::read_to_string(&path).or_else(|_| std::fs::read_to_string(&path_fallback));
        if let Ok(js) = content {
            if let Ok(j) = serde_json::from_str::<serde_json::Value>(&js) {
                json = j;
            }
        }
        json["update_channel"] = serde_json::json!(channel.as_str());
        std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap_or_default())?;
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub channel: UpdateChannel,
    pub current_version: String,
    pub current_commit: String,
    pub target_version: String,
    pub is_update_available: bool,
    pub title: String,
    pub release_notes: String,
    pub html_url: String,
    pub author: String,
    pub date: String,
    pub fallback_to_canary: bool,
}

#[derive(Debug, Clone)]
pub enum UpdateCheckError {
    NetworkError(String),
    RateLimitExceeded,
    InvalidJson(String),
    NoReleaseFound,
}

impl std::fmt::Display for UpdateCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(e) => write!(f, "Network error: {}", e),
            Self::RateLimitExceeded => write!(f, "GitHub API rate limit exceeded. Please try again later."),
            Self::InvalidJson(e) => write!(f, "Invalid API response: {}", e),
            Self::NoReleaseFound => write!(f, "No published release found."),
        }
    }
}

pub fn is_newer_semver(remote: &str, current: &str) -> bool {
    fn clean(s: &str) -> &str {
        s.trim_start_matches('v').trim_matches(' ')
    }
    let parse = |s: &str| -> Vec<u32> {
        clean(s).split('.').filter_map(|p| p.parse().ok()).collect()
    };
    let r = parse(remote);
    let c = parse(current);
    for i in 0..std::cmp::max(r.len(), c.len()) {
        let rv = r.get(i).copied().unwrap_or(0);
        let cv = c.get(i).copied().unwrap_or(0);
        if rv > cv { return true; }
        if cv > rv { return false; }
    }
    false
}

pub fn parse_release_json(json_str: &str, current_ver: &str) -> Result<UpdateCheckResult, UpdateCheckError> {
    let parsed: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| UpdateCheckError::InvalidJson(e.to_string()))?;

    if let Some(msg) = parsed["message"].as_str() {
        if msg.contains("API rate limit exceeded") {
            return Err(UpdateCheckError::RateLimitExceeded);
        }
        if msg == "Not Found" {
            return Err(UpdateCheckError::NoReleaseFound);
        }
    }

    let tag = parsed["tag_name"].as_str().ok_or(UpdateCheckError::NoReleaseFound)?;
    let clean_tag = tag.trim_start_matches('v');
    let title = parsed["name"].as_str().unwrap_or(tag).to_string();
    let body = parsed["body"].as_str().unwrap_or("No release notes provided.").to_string();
    let html_url = parsed["html_url"].as_str().unwrap_or("").to_string();
    let date = parsed["published_at"].as_str().unwrap_or("").to_string();

    let is_newer = is_newer_semver(clean_tag, current_ver);

    Ok(UpdateCheckResult {
        channel: UpdateChannel::Stable,
        current_version: current_ver.to_string(),
        current_commit: env!("VICTUS_MAX_GIT_HASH").to_string(),
        target_version: clean_tag.to_string(),
        is_update_available: is_newer,
        title,
        release_notes: body,
        html_url,
        author: "Victus Max Project".to_string(),
        date,
        fallback_to_canary: false,
    })
}

pub fn parse_commit_json(json_str: &str, current_commit: &str) -> Result<UpdateCheckResult, UpdateCheckError> {
    let parsed: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| UpdateCheckError::InvalidJson(e.to_string()))?;

    if let Some(msg) = parsed["message"].as_str() {
        if msg.contains("API rate limit exceeded") {
            return Err(UpdateCheckError::RateLimitExceeded);
        }
    }

    let sha = parsed["sha"].as_str().ok_or_else(|| {
        UpdateCheckError::InvalidJson("Missing 'sha' field in commit object".to_string())
    })?;

    let short_sha = if sha.len() >= 7 { &sha[..7] } else { sha };
    let author_name = parsed["commit"]["author"]["name"].as_str().unwrap_or("Unknown").to_string();
    let date = parsed["commit"]["author"]["date"].as_str().unwrap_or("").to_string();
    let message = parsed["commit"]["message"].as_str().unwrap_or("No commit message").to_string();
    let html_url = parsed["html_url"].as_str().unwrap_or("").to_string();

    let is_newer = current_commit != "unknown" && short_sha != current_commit;

    Ok(UpdateCheckResult {
        channel: UpdateChannel::Canary,
        current_version: env!("CARGO_PKG_VERSION").to_string(),
        current_commit: current_commit.to_string(),
        target_version: short_sha.to_string(),
        is_update_available: is_newer,
        title: format!("Commit {}", short_sha),
        release_notes: message,
        html_url,
        author: author_name,
        date,
        fallback_to_canary: false,
    })
}

/// Executes GitHub API check via curl with proper User-Agent headers
pub async fn check_updates_async(channel: UpdateChannel) -> Result<UpdateCheckResult, UpdateCheckError> {
    let current_ver = env!("CARGO_PKG_VERSION");
    let current_hash = env!("VICTUS_MAX_GIT_HASH");
    let user_agent = format!("VictusMax-Updater/{} (Linux; x86_64)", current_ver);

    let (url, is_stable) = match channel {
        UpdateChannel::Stable => (
            "https://api.github.com/repos/umutaktepe/victus-max/releases/latest",
            true,
        ),
        UpdateChannel::Canary => (
            "https://api.github.com/repos/umutaktepe/victus-max/commits/main",
            false,
        ),
    };

    let output = tokio::process::Command::new("curl")
        .args([
            "-sSL",
            "-H", &format!("User-Agent: {}", user_agent),
            "-H", "Accept: application/vnd.github.v3+json",
            "--connect-timeout", "10",
            "--max-time", "15",
            url,
        ])
        .output()
        .await
        .map_err(|e| UpdateCheckError::NetworkError(e.to_string()))?;

    if !output.status.success() {
        return Err(UpdateCheckError::NetworkError("curl process exited with error".to_string()));
    }

    let body = String::from_utf8(output.stdout)
        .map_err(|e| UpdateCheckError::InvalidJson(e.to_string()))?;

    if is_stable {
        match parse_release_json(&body, current_ver) {
            Ok(res) => Ok(res),
            Err(UpdateCheckError::NoReleaseFound) => {
                // Seamless fallback to Canary if no releases exist yet
                let mut canary_res = Box::pin(check_updates_async(UpdateChannel::Canary)).await?;
                canary_res.fallback_to_canary = true;
                Ok(canary_res)
            }
            Err(e) => Err(e),
        }
    } else {
        parse_commit_json(&body, current_hash)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_semver_comparison() {
        assert!(is_newer_semver("2.1.4", "2.1.3"));
        assert!(is_newer_semver("v2.2.0", "2.1.3"));
        assert!(!is_newer_semver("2.1.3", "2.1.3"));
        assert!(!is_newer_semver("2.1.2", "2.1.3"));
    }

    #[test]
    fn test_parse_github_release() {
        let payload = r#"{
            "tag_name": "v2.1.4",
            "name": "Victus Max 2.1.4 Release",
            "body": "Bug fixes and improvements",
            "published_at": "2026-09-29T10:00:00Z",
            "html_url": "https://github.com/umutaktepe/victus-max/releases/tag/v2.1.4"
        }"#;

        let result = parse_release_json(payload, "2.1.3").expect("Failed to parse release");
        assert_eq!(result.channel, UpdateChannel::Stable);
        assert_eq!(result.current_version, "2.1.3");
        assert_eq!(result.target_version, "2.1.4");
        assert!(result.is_update_available);
        assert_eq!(result.title, "Victus Max 2.1.4 Release");
        assert_eq!(result.release_notes, "Bug fixes and improvements");
        assert_eq!(result.date, "2026-09-29T10:00:00Z");
        assert_eq!(result.html_url, "https://github.com/umutaktepe/victus-max/releases/tag/v2.1.4");
        assert_eq!(result.author, "Victus Max Project");
        assert!(!result.fallback_to_canary);

        let result_same = parse_release_json(payload, "2.1.4").expect("Failed to parse release");
        assert!(!result_same.is_update_available);
    }

    #[test]
    fn test_parse_github_commit() {
        let payload = r#"{
            "sha": "9f8e7d6c5b4a3210fedcba09876543210abcdef1",
            "commit": {
                "author": {
                    "name": "Jane Developer",
                    "date": "2026-09-29T18:00:00Z"
                },
                "message": "fix: resolve fan curve hysteresis"
            },
            "html_url": "https://github.com/umutaktepe/victus-max/commit/9f8e7d6c5b4a3210fedcba09876543210abcdef1"
        }"#;

        let result = parse_commit_json(payload, "1234567").expect("Failed to parse commit");
        assert_eq!(result.channel, UpdateChannel::Canary);
        assert_eq!(result.target_version, "9f8e7d6");
        assert_eq!(result.current_commit, "1234567");
        assert!(result.is_update_available);
        assert_eq!(result.title, "Commit 9f8e7d6");
        assert_eq!(result.release_notes, "fix: resolve fan curve hysteresis");
        assert_eq!(result.author, "Jane Developer");
        assert_eq!(result.date, "2026-09-29T18:00:00Z");
        assert_eq!(result.html_url, "https://github.com/umutaktepe/victus-max/commit/9f8e7d6c5b4a3210fedcba09876543210abcdef1");
        assert!(!result.fallback_to_canary);

        let result_same = parse_commit_json(payload, "9f8e7d6").expect("Failed to parse commit");
        assert!(!result_same.is_update_available);
    }

    #[test]
    fn test_channel_serialization() {
        let canary_str = serde_json::to_string(&UpdateChannel::Canary).unwrap();
        assert_eq!(canary_str, "\"canary\"");
        let deserialized_canary: UpdateChannel = serde_json::from_str(&canary_str).unwrap();
        assert_eq!(deserialized_canary, UpdateChannel::Canary);

        let stable_str = serde_json::to_string(&UpdateChannel::Stable).unwrap();
        assert_eq!(stable_str, "\"stable\"");
        let deserialized_stable: UpdateChannel = serde_json::from_str(&stable_str).unwrap();
        assert_eq!(deserialized_stable, UpdateChannel::Stable);
    }

    #[test]
    fn test_channel_config_persistence() {
        let orig_home = std::env::var("HOME").ok();
        let unique_suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("victus_test_cfg_{}", unique_suffix));
        std::fs::create_dir_all(&temp_dir).unwrap();

        unsafe {
            std::env::set_var("HOME", &temp_dir);
        }

        // Initially no file exists, should default to Canary
        assert_eq!(load_update_channel(), UpdateChannel::Canary);

        // Save Stable and verify
        assert!(save_update_channel(UpdateChannel::Stable).is_ok());
        assert_eq!(load_update_channel(), UpdateChannel::Stable);

        // Save Canary and verify
        assert!(save_update_channel(UpdateChannel::Canary).is_ok());
        assert_eq!(load_update_channel(), UpdateChannel::Canary);

        // Verify existing keys are preserved
        let settings_path = temp_dir.join(".config/victus-max/settings.json");
        let initial_json = r#"{"theme": "dark", "start_minimized": true}"#;
        std::fs::write(&settings_path, initial_json).unwrap();

        assert!(save_update_channel(UpdateChannel::Stable).is_ok());
        assert_eq!(load_update_channel(), UpdateChannel::Stable);

        let saved_content = std::fs::read_to_string(&settings_path).unwrap();
        let saved_val: serde_json::Value = serde_json::from_str(&saved_content).unwrap();
        assert_eq!(saved_val["theme"], "dark");
        assert_eq!(saved_val["start_minimized"], true);
        assert_eq!(saved_val["update_channel"], "stable");

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
        unsafe {
            if let Some(h) = orig_home {
                std::env::set_var("HOME", h);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }
}
