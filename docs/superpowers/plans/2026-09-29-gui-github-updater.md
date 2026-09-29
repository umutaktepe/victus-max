# GUI GitHub Updater & Release Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish a robust GitHub release & update pipeline and overhaul the Victus Max GUI Update Center to provide seamless update checking (Canary/Git main commits and Stable GitHub Releases), persistent channel selection (`~/.config/victus-max/settings.json`), real-time progress streaming via Polkit, and reliable system re-installation.

**Architecture:** The solution introduces a dual-strategy update detection engine (`update_checker.rs`) capable of checking both GitHub Releases (when available) and GitHub commit history on `main` branch (for immediate Canary updates even before formal releases exist). User channel preference (`canary` vs `stable`) is persisted directly in `~/.config/victus-max/settings.json` so the user does not need to re-select it across sessions. At compile time, `build.rs` injects the local git commit hash and build date. Update installation is decoupled from fragile local paths by installing a dedicated system helper (`/usr/libexec/victus-max/victus-max-updater`) governed by a native Polkit action (`org.hp.victusmax.update.policy`), streaming structured `[STAGE:...]` markers to the GUI progress dialog. A GitHub Actions CI/CD release workflow (`release.yml`) and release helper script (`scripts/release.sh`) establish the missing release pipeline.

**Tech Stack:** Rust (GTK4, LibAdwaita, Tokio, Serde, Serde JSON), Polkit (`pkexec`), Bash, GitHub REST API v3, GitHub Actions CI/CD.

**Spec:** `docs/gui.md`, `docs/victus-max-wiki/decisions/adr-006-github-update-and-release-architecture.md`

## Global Constraints
- Adhere strictly to `AGENTS.md` LLM Wiki rules: kebab-case file names, no numeric prefixes, bidirectional wikilinks `[[...]]`, mandatory ADR-006, append-only log in `docs/victus-max-wiki/log.md`.
- Unprivileged GUI: The GUI itself must NEVER run with root privileges. Root operations are exclusively handled via `pkexec /usr/libexec/victus-max/victus-max-updater`.
- Persistent Channel Preference: Channel choice (`canary` or `stable`) must be stored in `~/.config/victus-max/settings.json` and automatically loaded on startup.
- Zero 404 Crashes: If GitHub returns 404 for `/releases/latest` (due to no published releases), the updater MUST gracefully handle it and fall back to checking commits on `main` branch with clear user messaging.
- Network Resilience: All GitHub API requests must include a valid `User-Agent: VictusMax/{version}` header and handle rate limits (HTTP 403) and offline states cleanly.

---

### Task 1: Build-Time Git Metadata & Compile-Time Env Variables

**Files:**
- Create: `src/victus-max-gui/build.rs`
- Modify: `src/victus-max-gui/Cargo.toml:1-12`
- Test: `src/victus-max-gui/src/version_info_test.rs`

**Interfaces:**
- Consumes: Host git repository state (`git rev-parse --short HEAD`).
- Produces: Environment variables `VICTUS_MAX_GIT_HASH` and `VICTUS_MAX_BUILD_DATE` at compile time, accessible via `env!("VICTUS_MAX_GIT_HASH")`.

- [ ] **Step 1: Write the failing test**

Create `src/victus-max-gui/src/version_info_test.rs`:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_git_metadata_present() {
        let hash = env!("VICTUS_MAX_GIT_HASH");
        assert!(!hash.is_empty(), "VICTUS_MAX_GIT_HASH must not be empty");
        assert!(
            hash.len() >= 7 || hash == "unknown",
            "Commit hash should be at least 7 characters or 'unknown'"
        );

        let build_date = env!("VICTUS_MAX_BUILD_DATE");
        assert!(!build_date.is_empty(), "VICTUS_MAX_BUILD_DATE must not be empty");
    }
}
```
And add `#[cfg(test)] mod version_info_test;` to `src/victus-max-gui/src/main.rs`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p victus-max-gui --bin victus-max test_git_metadata_present`
Expected: FAIL with compile error: `environment variable 'VICTUS_MAX_GIT_HASH' not defined at compile time`.

- [ ] **Step 3: Implement `build.rs` in `victus-max-gui`**

Create `src/victus-max-gui/build.rs`:
```rust
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs");

    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    let build_date = Command::new("date")
        .args(["-u", "+%Y-%m-%d"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=VICTUS_MAX_GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=VICTUS_MAX_BUILD_DATE={}", build_date);
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p victus-max-gui --bin victus-max test_git_metadata_present`
Expected: PASS with 1 passed, 0 failed.

- [ ] **Step 5: Commit**

```bash
git add src/victus-max-gui/build.rs src/victus-max-gui/src/version_info_test.rs src/victus-max-gui/src/main.rs
git commit -m "feat(gui): embed git commit hash and build date at compile time"
```

---

### Task 2: Core GitHub Update Checker & Config Persistence Module (`update_checker.rs`)

**Files:**
- Create: `src/victus-max-gui/src/update_checker.rs`
- Modify: `src/victus-max-gui/src/main.rs:16-20`
- Test: `src/victus-max-gui/src/update_checker.rs` (inline unit tests)

**Interfaces:**
- Consumes: GitHub REST API JSON payloads (Releases and Commits), local configuration at `~/.config/victus-max/settings.json`.
- Produces:
  - `pub fn load_update_channel() -> UpdateChannel`
  - `pub fn save_update_channel(channel: UpdateChannel) -> Result<(), std::io::Error>`
  - `pub async fn check_updates_async(channel: UpdateChannel) -> Result<UpdateCheckResult, UpdateCheckError>`

- [ ] **Step 1: Write the failing unit tests for parsing, comparison, and config persistence**

In `src/victus-max-gui/src/update_checker.rs`, define unit tests:
```rust
#[cfg(test)]
mod tests {
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
        let json_sample = r#"{
            "tag_name": "v2.2.0",
            "name": "Victus Max v2.2.0",
            "body": "### Changes\n- Fixed updater\n- Added Better Auto 2.0",
            "html_url": "https://github.com/umutaktepe/victus-max/releases/tag/v2.2.0",
            "published_at": "2026-09-30T12:00:00Z"
        }"#;

        let result = parse_release_json(json_sample, "2.1.3").unwrap();
        assert!(result.is_update_available);
        assert_eq!(result.target_version, "2.2.0");
        assert!(result.release_notes.contains("Fixed updater"));
    }

    #[test]
    fn test_parse_github_commit() {
        let json_sample = r#"{
            "sha": "d05a054abc1234567890",
            "commit": {
                "author": { "name": "Umut Aktepe", "date": "2026-09-29T21:00:00Z" },
                "message": "fix(gui): overhaul updater"
            },
            "html_url": "https://github.com/umutaktepe/victus-max/commit/d05a054abc1234567890"
        }"#;

        let result = parse_commit_json(json_sample, "a1b2c3d").unwrap();
        assert!(result.is_update_available);
        assert_eq!(result.target_version, "d05a054");
        assert_eq!(result.author, "Umut Aktepe");
        assert!(result.release_notes.contains("overhaul updater"));
    }

    #[test]
    fn test_channel_serialization() {
        let canary_str = serde_json::to_string(&UpdateChannel::Canary).unwrap();
        assert_eq!(canary_str, "\"canary\"");
        let stable_str = serde_json::to_string(&UpdateChannel::Stable).unwrap();
        assert_eq!(stable_str, "\"stable\"");

        let de_canary: UpdateChannel = serde_json::from_str("\"canary\"").unwrap();
        assert_eq!(de_canary, UpdateChannel::Canary);
        let de_stable: UpdateChannel = serde_json::from_str("\"stable\"").unwrap();
        assert_eq!(de_stable, UpdateChannel::Stable);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p victus-max-gui --bin victus-max update_checker::tests`
Expected: FAIL (module or functions not found).

- [ ] **Step 3: Implement `update_checker.rs`**

Write `src/victus-max-gui/src/update_checker.rs`:
```rust
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
    let clean = |s: &str| s.trim_start_matches('v').trim_matches(' ');
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
                let mut canary_res = check_updates_async(UpdateChannel::Canary).await?;
                canary_res.fallback_to_canary = true;
                Ok(canary_res)
            }
            Err(e) => Err(e),
        }
    } else {
        parse_commit_json(&body, current_hash)
    }
}
```

Add `pub mod update_checker;` in `src/victus-max-gui/src/main.rs`.

- [ ] **Step 4: Run unit tests to verify they pass**

Run: `cargo test -p victus-max-gui --bin victus-max update_checker::tests`
Expected: PASS with 4 passed, 0 failed.

- [ ] **Step 5: Commit**

```bash
git add src/victus-max-gui/src/update_checker.rs src/victus-max-gui/src/main.rs
git commit -m "feat(gui): implement update_checker with persistent channel configuration in settings.json"
```

---

### Task 3: System Updater Script & Polkit Security Policy

**Files:**
- Create: `scripts/victus-max-updater.sh`
- Create: `data/org.hp.victusmax.update.policy`
- Modify: `setup.sh:270-340` (install updater and polkit policy)
- Modify: `setup.sh:410-440` (uninstall updater and polkit policy)
- Test: Syntax check and dry-run tests for `victus-max-updater.sh`

**Interfaces:**
- Consumes: Invocation via `pkexec /usr/libexec/victus-max/victus-max-updater [stable|canary]`
- Produces: Emits stdout structured lines `[STAGE:PREPARE]`, `[STAGE:DOWNLOAD]`, `[STAGE:BUILD]`, `[STAGE:INSTALL]`, `[STAGE:RESTART]`, `[STAGE:COMPLETE]`.

- [ ] **Step 1: Create `scripts/victus-max-updater.sh`**

```bash
#!/usr/bin/env bash
# ==============================================================================
# Victus Max System Updater Helper
# Invoked with root privileges (typically via pkexec) from the GUI or CLI.
# Emits [STAGE:<TAG>] markers for real-time GUI progress tracking.
# ==============================================================================
set -euo pipefail

if [ "$EUID" -ne 0 ]; then
    echo "ERROR: victus-max-updater must be run as root." >&2
    exit 1
fi

CHANNEL="${1:-canary}"
REPO_URL="https://github.com/umutaktepe/victus-max.git"
WORK_DIR="/tmp/victus-max-update"

echo "[STAGE:PREPARE] Sistem bağımlılıkları ve çalışma ortamı hazırlanıyor..."

cleanup() {
    rm -rf "$WORK_DIR"
}
trap cleanup EXIT

echo "[STAGE:DOWNLOAD] GitHub üzerinden en son kaynak kodlar alınıyor (${CHANNEL})..."
rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR"

if command -v git &>/dev/null; then
    if [ "$CHANNEL" == "stable" ]; then
        # Check if tags exist
        LATEST_TAG=$(git ls-remote --tags --refs "$REPO_URL" | tail -n1 | awk '{print $2}' | sed 's|refs/tags/||' || true)
        if [ -n "$LATEST_TAG" ]; then
            echo "Latest release tag found: $LATEST_TAG"
            git clone --depth 1 -b "$LATEST_TAG" "$REPO_URL" "$WORK_DIR"
        else
            echo "No tags found. Falling back to main branch..."
            git clone --depth 1 -b main "$REPO_URL" "$WORK_DIR"
        fi
    else
        git clone --depth 1 -b main "$REPO_URL" "$WORK_DIR"
    fi
else
    echo "Git not installed. Downloading tarball archive from GitHub..."
    if [ "$CHANNEL" == "stable" ]; then
        ARCHIVE_URL="https://github.com/umutaktepe/victus-max/archive/refs/heads/main.tar.gz"
    else
        ARCHIVE_URL="https://github.com/umutaktepe/victus-max/archive/refs/heads/main.tar.gz"
    fi
    curl -fsSL "$ARCHIVE_URL" -o "$WORK_DIR/archive.tar.gz"
    tar -xzf "$WORK_DIR/archive.tar.gz" -C "$WORK_DIR" --strip-components=1
    rm -f "$WORK_DIR/archive.tar.gz"
fi

cd "$WORK_DIR"

if [ ! -f "setup.sh" ]; then
    echo "ERROR: setup.sh not found in downloaded source." >&2
    exit 1
fi

chmod +x setup.sh

echo "[STAGE:BUILD] Victus Max bileşenleri derleniyor (bu işlem bilgisayar hızına göre 1-3 dakika sürebilir)..."
./setup.sh update

echo "[STAGE:RESTART] Sistem servisleri ve arka plan daemon yeniden başlatılıyor..."
systemctl daemon-reload
systemctl restart victus-max-daemon.service || true

echo "[STAGE:COMPLETE] Victus Max başarıyla en güncel sürüme güncellendi!"
exit 0
```

- [ ] **Step 2: Create Polkit Action Policy `data/org.hp.victusmax.update.policy`**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE policyconfig PUBLIC "-//freedesktop//DTD PolicyKit Policy Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/PolicyKit/1.0/policyconfig.dtd">
<policyconfig>
  <vendor>Victus Max Project</vendor>
  <vendor_url>https://github.com/umutaktepe/victus-max</vendor_url>
  <icon_name>system-software-update</icon_name>

  <action id="org.hp.victusmax.update">
    <description>Update Victus Max system components and kernel drivers</description>
    <description xml:lang="tr">Victus Max sistem bileşenlerini ve çekirdek sürücülerini güncelle</description>
    <message>Authentication is required to update Victus Max</message>
    <message xml:lang="tr">Victus Max'i güncellemek için kimlik doğrulaması gerekiyor</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin_keep</allow_active>
    </defaults>
    <annotate key="org.freedesktop.policykit.exec.path">/usr/libexec/victus-max/victus-max-updater</annotate>
    <annotate key="org.freedesktop.policykit.exec.allow_gui">true</annotate>
  </action>
</policyconfig>
```

- [ ] **Step 3: Update `setup.sh` to install updater and policy**

Modify `setup.sh` around lines 270-325:
1. Copy `scripts/victus-max-updater.sh` to `/usr/libexec/victus-max/victus-max-updater` and `chmod 755`.
2. Symlink or copy to `/usr/share/victus-max/setup.sh` so legacy path calls don't 404.
3. Install `data/org.hp.victusmax.update.policy` to `/usr/share/polkit-1/actions/org.hp.victusmax.update.policy`.
4. In `do_uninstall`, remove `/usr/libexec/victus-max/victus-max-updater` and `/usr/share/polkit-1/actions/org.hp.victusmax.update.policy`.

- [ ] **Step 4: Verify script syntax**

Run: `bash -n scripts/victus-max-updater.sh`
Expected: Exits with 0 (syntax valid).

- [ ] **Step 5: Commit**

```bash
git add scripts/victus-max-updater.sh data/org.hp.victusmax.update.policy setup.sh
git commit -m "feat(system): add victus-max-updater script and polkit policy for safe root execution"
```

---

### Task 4: GUI Update Center Overhaul with Persistent Channel Selector (`updater.rs` & `i18n.rs`)

**Files:**
- Modify: `src/victus-max-gui/src/updater.rs`
- Modify: `src/victus-max-gui/src/i18n.rs`
- Test: GUI build & test suite (`cargo test -p victus-max-gui`)

**Interfaces:**
- Consumes: `update_checker::load_update_channel`, `update_checker::save_update_channel`, `update_checker::check_updates_async`.
- Produces: Interactive GTK4 Update Center with persistent Channel `adw::ComboRow`, live commit/release badges, stage-based progress bar, terminal toggle, and relaunch button.

- [ ] **Step 1: Add new Turkish and English i18n keys to `src/victus-max-gui/src/i18n.rs`**

Add keys:
- `"channel_canary"` => TR: `"Canary (Git main)"`, EN: `"Canary (Git main)"`
- `"channel_canary_sub"` => TR: `"Main dalındaki en güncel commitleri ve özellikleri takip eder"`, EN: `"Tracks bleeding-edge commits and features on main branch"`
- `"channel_stable_sub"` => TR: `"Doğrulanmış resmi GitHub sürümlerini takip eder"`, EN: `"Tracks verified official GitHub releases"`
- `"fallback_notice"` => TR: `"Henüz resmi bir GitHub Release yayınlanmadığından, Canary kanalı üzerinden en son değişiklikler denetlendi."`, EN: `"No official GitHub Release found yet; checked latest commits from Canary channel."`
- `"commit_by"` => TR: `"Geliştirici"`, EN: `"Author"`
- `"commit_date"` => TR: `"Tarih"`, EN: `"Date"`
- `"relaunch_app"` => TR: `"Uygulamayı Yeniden Başlat"`, EN: `"Restart Application"`
- `"rate_limit_err"` => TR: `"GitHub API istek limiti aşıldı. Lütfen birkaç dakika sonra tekrar deneyin."`, EN: `"GitHub API rate limit exceeded. Please wait a few minutes."`
- `"view_on_github"` => TR: `"GitHub'da Görüntüle"`, EN: `"View on GitHub"`

- [ ] **Step 2: Update `updater.rs` with persistent Channel selector and metadata badge**

In `build_page`:
1. Version badge row: Shows `v2.1.3 (hash)` + build date.
2. Channel row: `adw::ComboRow` allowing selection between `Canary (Git main)` and `Stable (Releases)`.
   - Initial selection populated using `crate::update_checker::load_update_channel()`.
   - Connect `notify::selected` signal:
     ```rust
     channel_row.connect_selected_notify(move |row| {
         let new_channel = match row.selected() {
             1 => crate::update_checker::UpdateChannel::Stable,
             _ => crate::update_checker::UpdateChannel::Canary,
         };
         let _ = crate::update_checker::save_update_channel(new_channel);
     });
     ```
   - This automatically writes `"update_channel"` to `~/.config/victus-max/settings.json`, eliminating repeated user selection across runs.
3. Check Updates button: Queries `crate::update_checker::load_update_channel()` and passes it to `show_app_update_modal`.

- [ ] **Step 3: Update `show_app_update_modal` in `updater.rs`**

1. Call `update_checker::check_updates_async(selected_channel)`.
2. When update is available:
   - Display version/commit pill.
   - If fallback happened, display an informational banner with `fallback_notice`.
   - Display commit author and timestamp.
   - Display formatted release notes / commit message in scrollable text view.
   - Add "GitHub'da Görüntüle" link button (`gtk::LinkButton`).
   - "Güncelle" button: invokes `start_update_process(vbox, dialog, selected_channel)`.
3. When up to date:
   - Display green checkmark icon, "Sisteminiz güncel", and the latest commit/release title.
4. When error occurs:
   - Display clear translated error and retry button.

- [ ] **Step 4: Update `start_update_process` in `updater.rs`**

1. Execute updater via `pkexec`:
   ```rust
   let updater_bin = if std::path::Path::new("/usr/libexec/victus-max/victus-max-updater").exists() {
       "/usr/libexec/victus-max/victus-max-updater".to_string()
   } else {
       // Dev fallback: use script in repository
       format!("{}/scripts/victus-max-updater.sh", env!("CARGO_MANIFEST_DIR"))
   };
   let channel_arg = selected_channel.as_str();
   ```
2. Parse stdout stream line-by-line for `[STAGE:...]`:
   - `[STAGE:PREPARE]`: set progress to 0.15, label = stage message.
   - `[STAGE:DOWNLOAD]`: set progress to 0.35, label = stage message.
   - `[STAGE:BUILD]`: set progress to 0.65, label = stage message.
   - `[STAGE:INSTALL]`: set progress to 0.85, label = stage message.
   - `[STAGE:RESTART]`: set progress to 0.95, label = stage message.
   - `[STAGE:COMPLETE]`: set progress to 1.0, label = `update_completed`.
3. Stream all lines to the monospace terminal text buffer.
4. On completion:
   - Add a suggested action button: `"Uygulamayı Yeniden Başlat"` (`relaunch_app`).
   - Clicking it executes `std::process::Command::new("victus-max").spawn()` and closes current application.

- [ ] **Step 5: Verify GUI compilation and tests**

Run: `cargo test -p victus-max-gui`
Expected: PASS with all tests passing.

- [ ] **Step 6: Commit**

```bash
git add src/victus-max-gui/src/updater.rs src/victus-max-gui/src/i18n.rs
git commit -m "feat(gui): overhaul Update Center with persistent channel selector, stage tracker, and GitHub commit integration"
```

---

### Task 5: GitHub Release Workflow & Release Automation

**Files:**
- Create: `.github/workflows/release.yml`
- Create: `scripts/release.sh`
- Test: Syntax check and dry run of release script

**Interfaces:**
- Consumes: Git tags in format `v*.*.*`.
- Produces: Automated GitHub Release with changelog and pre-packaged release tarball.

- [ ] **Step 1: Create `.github/workflows/release.yml`**

```yaml
name: Release Automation

on:
  push:
    tags:
      - 'v*.*.*'
  workflow_dispatch:

permissions:
  contents: write

jobs:
  build-release:
    name: Build and Package Release
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            build-essential pkg-config \
            libgtk-4-dev libadwaita-1-dev libsystemd-dev libdbus-1-dev \
            libgtk4-layer-shell-dev libhidapi-dev

      - name: Set up Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Run test suite
        run: cargo test --workspace

      - name: Build release binaries
        run: cargo build --release

      - name: Package release archive
        run: |
          TAG_NAME=${GITHUB_REF_NAME:-v$(sed -n 's/^version = "\(.*\)"/\1/p' src/victus-max-gui/Cargo.toml | head -1)}
          ARCHIVE_NAME="victus-max-${TAG_NAME}-x86_64.tar.gz"
          mkdir -p dist/bin dist/data dist/driver
          
          cp target/release/victus-max-daemon dist/bin/
          cp target/release/victus-max dist/bin/
          cp target/release/victus-max-cli dist/bin/
          cp target/release/victus-max-tray dist/bin/
          cp target/release/victus-max-overlay dist/bin/
          cp -r data/* dist/data/
          cp -r driver/* dist/driver/
          cp setup.sh install.sh scripts/victus-max-updater.sh dist/
          
          tar -czf "$ARCHIVE_NAME" -C dist .
          echo "ARCHIVE_NAME=$ARCHIVE_NAME" >> $GITHUB_ENV

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: ${{ env.ARCHIVE_NAME }}
          generate_release_notes: true
          draft: false
          prerelease: false
```

- [ ] **Step 2: Create release helper script `scripts/release.sh`**

Create `scripts/release.sh`:
```bash
#!/usr/bin/env bash
# ==============================================================================
# Victus Max Release Helper
# Synchronizes Cargo versions, tags the release, and prepares git push.
# ==============================================================================
set -euo pipefail

NEW_VER="${1:-}"

if [ -z "$NEW_VER" ]; then
    CURRENT_VER=$(sed -n 's/^version = "\(.*\)"/\1/p' src/victus-max-gui/Cargo.toml | head -1)
    echo "Usage: ./scripts/release.sh <new-version> (e.g. 2.1.4)"
    echo "Current version: $CURRENT_VER"
    exit 1
fi

CLEAN_VER="${NEW_VER#v}"
TAG="v${CLEAN_VER}"

echo "Preparing release $TAG..."

# Update versions in all Cargo.toml
for file in src/*/Cargo.toml; do
    sed -i "s/^version = \".*\"/version = \"$CLEAN_VER\"/" "$file"
    echo "Updated $file to $CLEAN_VER"
done

cargo check --workspace

git add src/*/Cargo.toml Cargo.lock
git commit -m "chore(release): bump version to $TAG"
git tag -a "$TAG" -m "Release $TAG"

echo ""
echo "✅ Release $TAG committed and tagged locally."
echo "To publish, run:"
echo "    git push origin main --tags"
```
Make executable: `chmod +x scripts/release.sh`.

- [ ] **Step 3: Verify scripts**

Run: `bash -n scripts/release.sh`
Expected: Syntax check passes with code 0.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/release.yml scripts/release.sh
git commit -m "ci(release): add GitHub Actions release workflow and release bump script"
```

---

### Task 6: Architectural Decision Record & Wiki Synchronization

**Files:**
- Create: `docs/victus-max-wiki/decisions/adr-006-github-update-and-release-architecture.md`
- Create: `docs/victus-max-wiki/user-experience/updater-service.md`
- Modify: `docs/victus-max-wiki/index.md`
- Modify: `docs/victus-max-wiki/log.md`
- Modify: `docs/gui.md:18-21`

**Interfaces:**
- Wiki graph alignment per `AGENTS.md`.

- [ ] **Step 1: Create `adr-006-github-update-and-release-architecture.md`**

Following the mandatory template with Status, Date, Context, Alternatives, Decision, Consequences, and Bidirectional Wikilinks:
- Links: `[[gui-application]]`, `[[adr-001-rust-daemon-client-split]]`, `[[polkit-dbus-security]]`, `[[updater-service]]`.

- [ ] **Step 2: Create `docs/victus-max-wiki/user-experience/updater-service.md`**

Details the Update Center UI, persistent channel selection logic (`settings.json`), dual-channel strategy (Canary/Git commits vs Stable Releases), Polkit privilege separation, and recovery behaviors.

- [ ] **Step 3: Update `docs/victus-max-wiki/index.md`**

Add `adr-006` and `updater-service` to the Map of Content (MOC).

- [ ] **Step 4: Append to `docs/victus-max-wiki/log.md`**

Append entry:
```markdown
## [2026-09-29] Geliştirme | GitHub Entegrasyonlu OTA Güncelleme ve Sürüm Mimarisi (ADR-006)
- GUI güncelleme sayfasının GitHub REST API (Releases & Commits) entegrasyonu tamamlandı.
- Kanal tercihi (Canary / Stable) `settings.json` içerisine kalıcı konfigürasyon olarak bağlandı.
- Release bulunmadığında Canary (Git main) dalına otomatik fallback sağlandı.
- Polkit destekli bağımsız `victus-max-updater` servisi ve GitHub Actions release iş akışı eklendi.
```

- [ ] **Step 5: Verify wiki links and git status**

Run: `git status`
Expected: All doc and code changes ready.

- [ ] **Step 6: Commit**

```bash
git add docs/victus-max-wiki/ docs/gui.md
git commit -m "docs(wiki): add ADR-006 and update center documentation per living architecture rules"
```

---

## Plan Review Checklist
1. **Spec Coverage**:
   - [x] Fix update checking broken due to missing GitHub releases (dual-strategy release + commit fallback).
   - [x] Persistent channel selection in `~/.config/victus-max/settings.json` so user never has to re-select it.
   - [x] Fix update execution broken due to missing `/usr/share/victus-max/setup.sh` (dedicated `/usr/libexec/victus-max/victus-max-updater` + Polkit action).
   - [x] Establish missing release structure and logic (GitHub Actions workflow + release script).
   - [x] Adhere to `AGENTS.md` Living Architecture rules (ADR-006, kebab-case, wikilinks, index, log).
2. **No Placeholders**: Every task contains concrete code, test commands, and exact paths.
3. **Type Consistency**: `UpdateChannel`, `UpdateCheckResult`, `UpdateCheckError`, `load_update_channel`, `save_update_channel` types match across tasks.
