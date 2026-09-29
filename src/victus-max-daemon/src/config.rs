use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use log::{info, warn};

pub fn default_min_fan_rpm() -> u32 {
    2600
}

pub fn default_acoustic_ceiling() -> usize {
    5
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FanConfig {
    pub fan_mode: String,
    pub custom_curve: String,
    pub thermal_protection_enabled: bool,
    #[serde(default = "default_min_fan_rpm")]
    pub min_fan_rpm: u32,
    #[serde(default = "default_acoustic_ceiling")]
    pub acoustic_ceiling: usize,
}

impl Default for FanConfig {
    fn default() -> Self {
        Self {
            fan_mode: "auto".to_string(),
            custom_curve: "[]".to_string(),
            thermal_protection_enabled: true,
            min_fan_rpm: default_min_fan_rpm(),
            acoustic_ceiling: default_acoustic_ceiling(),
        }
    }
}

pub struct ConfigManager {
    path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Self {
        let p_victus = PathBuf::from("/var/lib/victus-max-daemon/fan_config.json");
        let p_omen = PathBuf::from("/var/lib/omen-space-daemon/fan_config.json");
        let path = if p_victus.exists() || !p_omen.exists() {
            p_victus
        } else {
            p_omen
        };
        Self { path }
    }

    pub async fn load(&self) -> FanConfig {
        let p_victus = PathBuf::from("/var/lib/victus-max-daemon/fan_config.json");
        let p_omen = PathBuf::from("/var/lib/omen-space-daemon/fan_config.json");
        let load_path = if self.path.exists() {
            &self.path
        } else if p_victus.exists() {
            &p_victus
        } else if p_omen.exists() {
            &p_omen
        } else {
            &self.path
        };

        match fs::read_to_string(load_path).await {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(config) => {
                        info!("Loaded fan config from {:?}", load_path);
                        config
                    }
                    Err(e) => {
                        warn!("Failed to parse config file {:?}: {}. Using defaults.", load_path, e);
                        FanConfig::default()
                    }
                }
            }
            Err(_) => {
                info!("No existing config found at {:?}. Using defaults.", load_path);
                FanConfig::default()
            }
        }
    }

    pub async fn save(&self, config: &FanConfig) {
        let target_path = if self.path == PathBuf::from("/var/lib/omen-space-daemon/fan_config.json") {
            PathBuf::from("/var/lib/victus-max-daemon/fan_config.json")
        } else {
            self.path.clone()
        };

        if let Some(parent) = target_path.parent() {
            let _ = fs::create_dir_all(parent).await;
        }

        match serde_json::to_string_pretty(config) {
            Ok(json) => {
                if let Err(e) = fs::write(&target_path, json).await {
                    warn!("Failed to save fan config to {:?}: {}", target_path, e);
                } else {
                    info!("Saved fan config to {:?}", target_path);
                }
            }
            Err(e) => {
                warn!("Failed to serialize fan config: {}", e);
            }
        }
    }
}
