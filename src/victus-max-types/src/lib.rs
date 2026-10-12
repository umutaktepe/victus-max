use serde::{Deserialize, Serialize};
use zbus::proxy;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_temp: i32,
    pub cpu_load: f64,
    pub cpu_pwr: f64,
    pub fan_rpm: i32,
    pub fan1_rpm: i32,
    pub fan2_rpm: i32,
    pub gpu_temp: i32,
    pub gpu_load: f64,
    pub gpu_pwr: f64,
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
    pub ram_frac: f64,
    pub disk_used_gb: f64,
    pub disk_total_gb: f64,
    pub disk_frac: f64,
    pub total_pwr: f64,
    pub cpu_throttle_count: u32,
    #[serde(default)]
    pub chassis_temp: i32,
    #[serde(default)]
    pub board_verified: bool,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSpecs {
    pub product_name: String,
    pub cpu_spec: String,
    pub gpu_spec: String,
    pub ram_spec: String,
    pub ssd_spec: String,
    pub os_spec: String,
    pub bios_version: String,
    pub ec_version: String,
    pub vbios_version: String,
    pub nvidia_driver: String,
    pub kernel_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerModeLimits {
    pub stapm_w: u32,
    pub slow_ppt_w: u32,
    pub fast_ppt_w: u32,
    pub tctl_temp: u32,
}

impl Default for PowerModeLimits {
    fn default() -> Self {
        Self {
            stapm_w: 45,
            slow_ppt_w: 45,
            fast_ppt_w: 65,
            tctl_temp: 95,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PowerModeProfilesConfig {
    pub power_saver: PowerModeLimits,
    pub balanced: PowerModeLimits,
    pub performance: PowerModeLimits,
    pub active_mode: Option<String>,
    #[serde(default)]
    pub manual_override: bool,
}

// ── Power Service Proxy ────────────────────────────────────────────────────────
#[proxy(
    interface = "org.hp.omen.Power",
    default_service = "org.hp.omen",
    default_path = "/org/hp/omen/Power"
)]
pub trait Power {
    async fn set_power_profile(&self, profile: &str) -> zbus::Result<String>;
    async fn get_power_profile(&self) -> zbus::Result<String>;
    async fn get_power_limits_info(&self) -> zbus::Result<String>;
    async fn set_power_limits(&self, enabled: bool, pl1: i32, pl2: i32) -> zbus::Result<String>;
    async fn set_ryzen_power_limits(&self, enabled: bool, stapm: i32, slow: i32, fast: i32) -> zbus::Result<String>;
    async fn set_app_profiles_enabled(&self, enabled: bool) -> zbus::Result<String>;
    async fn get_power_presets(&self) -> zbus::Result<String>;
    async fn set_power_preset_mode(&self, mode: &str) -> zbus::Result<String>;
    async fn save_power_preset(&self, mode: &str, stapm_w: u32, slow_ppt_w: u32, fast_ppt_w: u32, tctl_temp: u32) -> zbus::Result<String>;
    async fn reset_power_presets(&self) -> zbus::Result<String>;
}

// ── Fan Service Proxy ─────────────────────────────────────────────────────────
#[proxy(
    interface = "org.hp.omen.Fan",
    default_service = "org.hp.omen",
    default_path = "/org/hp/omen/Fan"
)]
pub trait Fan {
    async fn set_fan_mode(&self, mode: &str) -> zbus::Result<String>;
    async fn get_fan_mode(&self) -> zbus::Result<String>;
    async fn get_fan_info(&self) -> zbus::Result<String>;
    async fn save_custom_curve(&self, curve_json: &str) -> zbus::Result<String>;
    async fn set_thermal_protection(&self, enabled: bool) -> zbus::Result<String>;
    async fn get_min_fan_rpm(&self) -> zbus::Result<u32>;
    async fn set_min_fan_rpm(&self, rpm: u32) -> zbus::Result<bool>;
    async fn get_acoustic_ceiling(&self) -> zbus::Result<u32>;
    async fn set_acoustic_ceiling(&self, level: u32) -> zbus::Result<bool>;
    
    #[zbus(signal)]
    fn thermal_protection_alert(&self, active: bool) -> zbus::Result<()>;
}

// ── SysMon Service Proxy ──────────────────────────────────────────────────────
#[proxy(
    interface = "org.hp.omen.SysMon",
    default_service = "org.hp.omen",
    default_path = "/org/hp/omen/SysMon"
)]
pub trait SysMon {
    async fn get_diagnostics(&self) -> zbus::Result<String>;
    async fn get_hardware_specs(&self) -> zbus::Result<String>;
    async fn generate_diagnostic_report(&self) -> zbus::Result<String>;
    async fn generate_rgb_issue(&self) -> zbus::Result<String>;
    
    #[zbus(signal)]
    fn telemetry_updated(&self, json_stats: &str) -> zbus::Result<()>;
}

// ── Rgb Service Proxy ─────────────────────────────────────────────────────────
#[proxy(
    interface = "org.hp.omen.Rgb",
    default_service = "org.hp.omen",
    default_path = "/org/hp/omen/Rgb"
)]
pub trait Rgb {
    async fn set_color(&self, zone_val: i32, hex_color: &str) -> zbus::Result<String>;
    async fn set_mode(&self, mode_str: &str, speed_val: i32) -> zbus::Result<String>;
    async fn set_global(&self, power_val: bool, brightness_val: i32, direction_str: &str) -> zbus::Result<String>;
    async fn get_state(&self) -> zbus::Result<String>;
    async fn set_per_key_colors(&self, colors_json: &str) -> zbus::Result<String>;
    async fn start_per_key_wizard(&self) -> zbus::Result<String>;
    async fn light_key_index(&self, index: u32, hex_color: &str) -> zbus::Result<String>;
    async fn record_key_mapping(&self, index: u32, key_name: &str) -> zbus::Result<String>;
    async fn export_keymap_report(&self) -> zbus::Result<String>;
}

// ── Platform Service Proxy ───────────────────────────────────────────────────
#[proxy(
    interface = "org.hp.omen.Platform",
    default_service = "org.hp.omen",
    default_path = "/org/hp/omen/Platform"
)]
pub trait Platform {
    async fn set_battery_care(&self, limit: u32) -> zbus::Result<String>;
    async fn run_fan_cleaning(&self) -> zbus::Result<String>;
    async fn toggle_overlay(&self) -> zbus::Result<String>;
    async fn generate_triage_bundle(&self) -> zbus::Result<String>;
    #[zbus(signal)]
    fn macro_key_pressed(&self, key_name: &str) -> zbus::Result<()>;
}

// ── Mux Service Proxy ────────────────────────────────────────────────────────
#[proxy(
    interface = "org.hp.omen.Mux",
    default_service = "org.hp.omen",
    default_path = "/org/hp/omen/Mux"
)]
pub trait Mux {
    async fn set_gpu_mode(&self, mode: &str) -> zbus::Result<String>;
    async fn get_gpu_info(&self) -> zbus::Result<String>;
}

// ── Undervolt Service Proxy ──────────────────────────────────────────────────
#[proxy(
    interface = "org.hp.omen.Undervolt",
    default_service = "org.hp.omen",
    default_path = "/org/hp/omen/Undervolt"
)]
pub trait Undervolt {
    async fn set_offset(&self, plane: &str, offset_mv: i32) -> zbus::Result<String>;
    async fn set_tcc_offset(&self, val: i32) -> zbus::Result<String>;
    async fn get_state(&self) -> zbus::Result<String>;
}

// ── AppProfiles Service Proxy ──────────────────────────────────────────────────
#[proxy(
    interface = "org.hp.omen.AppProfiles",
    default_service = "org.hp.omen",
    default_path = "/org/hp/omen/AppProfiles"
)]
pub trait AppProfiles {
    async fn get_profiles(&self) -> zbus::Result<String>;
    async fn add_profile(&self, process_name: &str, power_profile: &str, fan_mode: &str) -> zbus::Result<String>;
    async fn remove_profile(&self, process_name: &str) -> zbus::Result<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_mode_limits_serde() {
        let limits = PowerModeLimits {
            stapm_w: 35,
            slow_ppt_w: 35,
            fast_ppt_w: 35,
            tctl_temp: 82,
        };
        let json = serde_json::to_string(&limits).expect("serialize limits");
        let parsed: PowerModeLimits = serde_json::from_str(&json).expect("deserialize limits");
        assert_eq!(parsed, limits);
    }

    #[test]
    fn test_power_mode_profiles_config_serde() {
        let config = PowerModeProfilesConfig {
            power_saver: PowerModeLimits { stapm_w: 20, slow_ppt_w: 20, fast_ppt_w: 25, tctl_temp: 70 },
            balanced: PowerModeLimits { stapm_w: 35, slow_ppt_w: 35, fast_ppt_w: 35, tctl_temp: 82 },
            performance: PowerModeLimits { stapm_w: 45, slow_ppt_w: 45, fast_ppt_w: 45, tctl_temp: 88 },
            active_mode: Some("balanced".to_string()),
            manual_override: false,
        };
        let json = serde_json::to_string(&config).expect("serialize config");
        let parsed: PowerModeProfilesConfig = serde_json::from_str(&json).expect("deserialize config");
        assert_eq!(parsed, config);
        assert_eq!(parsed.active_mode.as_deref(), Some("balanced"));
    }
}
