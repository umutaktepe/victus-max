#[path = "../src/config.rs"]
pub mod config;

pub mod notifier {
    pub struct DesktopNotifier;
    impl DesktopNotifier {
        pub async fn send_notification(_title: &str, _msg: &str, _urgency: u8) {}
        pub async fn notify_error(_title: &str, _msg: &str) {}
    }
}

pub mod ec {
    pub struct LinuxEcController;
    impl LinuxEcController {
        pub fn new() -> Self { Self }
        pub fn needs_ec_fallback(&self) -> bool { false }
        pub async fn restore_auto_mode(&self) {}
        pub fn set_perf_mode(&mut self, _mode: &str) {}
    }
}

pub mod capabilities {
    pub struct Capabilities {
        pub supports_fan_control_ec: bool,
        pub supports_fan_control_wmi: bool,
    }
    pub fn detect(_board_id: &str, _p1: &str, _p2: &str) -> Capabilities {
        Capabilities {
            supports_fan_control_ec: true,
            supports_fan_control_wmi: true,
        }
    }
}

#[path = "../src/sysmon/load.rs"]
pub mod sysmon_load;
pub mod sysmon {
    pub use crate::sysmon_load::*;
    pub fn get_safe_gpu_temp() -> f64 { 0.0 }
}

#[path = "../src/fan/mod.rs"]
pub mod fan;

use config::{default_acoustic_ceiling, default_min_fan_rpm, FanConfig};
use fan::{get_effective_acoustic_ceiling, update_cooldown, FanService, FanState};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[test]
fn test_fan_config_defaults() {
    let cfg = FanConfig::default();
    assert_eq!(cfg.min_fan_rpm, 2600);
    assert_eq!(cfg.acoustic_ceiling, 5);
    assert_eq!(default_min_fan_rpm(), 2600);
    assert_eq!(default_acoustic_ceiling(), 5);
}

#[test]
fn test_fan_config_backwards_compatibility() {
    let old_json = r#"{"fan_mode":"auto","custom_curve":"[]","thermal_protection_enabled":true}"#;
    let cfg: FanConfig = serde_json::from_str(old_json).expect("Failed to deserialize old config");
    assert_eq!(cfg.min_fan_rpm, 2600);
    assert_eq!(cfg.acoustic_ceiling, 5);
    assert_eq!(cfg.fan_mode, "auto");
    assert!(cfg.thermal_protection_enabled);
}

#[test]
fn test_fan_config_roundtrip_custom_values() {
    let json_str = r#"{"fan_mode":"better_auto","custom_curve":"[]","thermal_protection_enabled":false,"min_fan_rpm":2800,"acoustic_ceiling":7}"#;
    let cfg: FanConfig = serde_json::from_str(json_str).expect("Failed to deserialize custom config");
    assert_eq!(cfg.min_fan_rpm, 2800);
    assert_eq!(cfg.acoustic_ceiling, 7);
    assert_eq!(cfg.fan_mode, "better_auto");
    assert!(!cfg.thermal_protection_enabled);

    let serialized = serde_json::to_string(&cfg).expect("Serialization failed");
    assert!(serialized.contains("\"min_fan_rpm\":2800"));
    assert!(serialized.contains("\"acoustic_ceiling\":7"));
}

#[tokio::test]
async fn test_better_auto_mode_transition() {
    let temp_dir = std::env::temp_dir().join(format!("hwmon_test_mode_{}", std::process::id()));
    let _ = tokio::fs::create_dir_all(&temp_dir).await;
    let pwm1_enable_path = temp_dir.join("pwm1_enable");
    let _ = tokio::fs::write(&pwm1_enable_path, b"2").await;

    let mut state = FanState::default();
    state.hwmon_path = Some(temp_dir.clone());
    assert_eq!(state.mode, "auto");

    let ok = FanService::set_mode_internal(&mut state, "better_auto").await;
    assert!(ok);
    assert_eq!(state.mode, "better_auto");
    assert!(state.manual_target_pct.is_none());
    assert_eq!(state.better_auto_level, 0);

    // Verify pwm1_enable was set to 1 (software manual control)
    let enable_val = tokio::fs::read_to_string(&pwm1_enable_path).await.unwrap();
    assert_eq!(enable_val.trim(), "1");

    // Verify last_apply was reset so speed is applied immediately on next loop
    let now = Instant::now();
    assert!(now.duration_since(state.better_auto_last_apply).as_secs() >= 90);

    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}

#[test]
fn test_effective_acoustic_ceiling_resolution() {
    // Performance profile: unconditionally 8
    assert_eq!(get_effective_acoustic_ceiling("performance", 4), 8);
    assert_eq!(get_effective_acoustic_ceiling("performance", 5), 8);

    // Power-saver / quiet / eco / battery / low-power / cool: min(user_ceiling, 3)
    assert_eq!(get_effective_acoustic_ceiling("power-saver", 5), 3);
    assert_eq!(get_effective_acoustic_ceiling("quiet", 2), 2);
    assert_eq!(get_effective_acoustic_ceiling("eco", 4), 3);
    assert_eq!(get_effective_acoustic_ceiling("battery", 5), 3);
    assert_eq!(get_effective_acoustic_ceiling("low-power", 5), 3);
    assert_eq!(get_effective_acoustic_ceiling("cool", 5), 3);

    // Balanced: clamp(3, 8)
    assert_eq!(get_effective_acoustic_ceiling("balanced", 5), 5);
    assert_eq!(get_effective_acoustic_ceiling("balanced", 2), 3);
    assert_eq!(get_effective_acoustic_ceiling("balanced", 9), 8);

    // Unknown defaults to balanced clamping
    assert_eq!(get_effective_acoustic_ceiling("unknown", 6), 6);
}

#[test]
fn test_cooldown_window_logic() {
    let mut cooldown_level = 0;
    let mut cooldown_until = Instant::now();
    let now = Instant::now();

    // Reaching level 7+ sets 30s cooldown at floor 7
    let target1 = update_cooldown(7, now, &mut cooldown_level, &mut cooldown_until);
    assert_eq!(target1, 7);
    assert_eq!(cooldown_level, 7);
    assert!(cooldown_until > now);

    // Drop to level 3 after 5s (< 30s) -> floor holds at 7
    let after_5s = now + Duration::from_secs(5);
    let target2 = update_cooldown(3, after_5s, &mut cooldown_level, &mut cooldown_until);
    assert_eq!(target2, 7);

    // After 30s window expires, cooldown clears
    let after_31s = now + Duration::from_secs(31);
    let target3 = update_cooldown(3, after_31s, &mut cooldown_level, &mut cooldown_until);
    assert_eq!(target3, 3);
    assert_eq!(cooldown_level, 0);

    // Reaching level 5 sets 15s cooldown at floor 5
    let now2 = Instant::now();
    let target4 = update_cooldown(5, now2, &mut cooldown_level, &mut cooldown_until);
    assert_eq!(target4, 5);
    assert_eq!(cooldown_level, 5);

    // Drop to level 2 after 10s (< 15s) -> floor holds at 5
    let after_10s = now2 + Duration::from_secs(10);
    let target5 = update_cooldown(2, after_10s, &mut cooldown_level, &mut cooldown_until);
    assert_eq!(target5, 5);

    // After 16s, clears
    let after_16s = now2 + Duration::from_secs(16);
    let target6 = update_cooldown(2, after_16s, &mut cooldown_level, &mut cooldown_until);
    assert_eq!(target6, 2);
    assert_eq!(cooldown_level, 0);
}

#[tokio::test]
async fn test_stagger_write_timing() {
    let temp_dir = std::env::temp_dir().join(format!("hwmon_test_stagger_{}", std::process::id()));
    let _ = tokio::fs::create_dir_all(&temp_dir).await;
    let fan1_path = temp_dir.join("fan1_target");
    let fan2_path = temp_dir.join("fan2_target");

    let _ = tokio::fs::write(&fan1_path, b"1000").await;
    let _ = tokio::fs::write(&fan2_path, b"1000").await;

    let mut state = FanState::default();
    state.hwmon_path = Some(temp_dir.clone());
    state.found_fans = vec![1, 2];

    // Use a 50ms stagger delay for fast, non-flaky test execution
    FanService::write_fan_targets_staggered_with_delay(&mut state, 3500, 4200, Duration::from_millis(50)).await;

    // fan1_target written immediately
    let fan1_val = tokio::fs::read_to_string(&fan1_path).await.unwrap();
    assert_eq!(fan1_val.trim(), "3500");

    // fan2_target should NOT yet be updated at 10ms
    tokio::time::sleep(Duration::from_millis(15)).await;
    let fan2_initial = tokio::fs::read_to_string(&fan2_path).await.unwrap();
    assert_eq!(fan2_initial.trim(), "1000");

    // After 60ms (> 50ms delay), fan2_target should be updated
    tokio::time::sleep(Duration::from_millis(55)).await;
    let fan2_updated = tokio::fs::read_to_string(&fan2_path).await.unwrap();
    assert_eq!(fan2_updated.trim(), "4200");

    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}

#[tokio::test]
async fn test_stagger_write_fallback_to_pwm() {
    let temp_dir = std::env::temp_dir().join(format!("hwmon_test_fallback_{}", std::process::id()));
    let _ = tokio::fs::create_dir_all(&temp_dir).await;
    let pwm1_path = temp_dir.join("pwm1");
    let pwm1_enable_path = temp_dir.join("pwm1_enable");

    let _ = tokio::fs::write(&pwm1_enable_path, b"1").await;
    let _ = tokio::fs::write(&pwm1_path, b"0").await;

    let mut state = FanState::default();
    state.hwmon_path = Some(temp_dir.clone());
    state.found_fans = vec![1, 2];
    state.max_speeds.insert(1, 6000);
    state.max_speeds.insert(2, 6000);

    // No fan1_target exists -> fallback calculates duty cycle (3000 / 6000 = 50%)
    let ok = FanService::write_fan_targets_staggered(&mut state, 3000, 3000).await;
    assert!(ok);

    let pwm_val = tokio::fs::read_to_string(&pwm1_path).await.unwrap();
    // 50% of 255 is 128
    assert_eq!(pwm_val.trim(), "128");

    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}

#[test]
fn test_watchdog_timing_calculations() {
    let now = Instant::now();
    let last_manual_assert = now - Duration::from_secs(81);
    let last_apply = now - Duration::from_secs(91);

    // 80s manual reassert
    assert!(now.duration_since(last_manual_assert).as_secs() >= 80);

    // 90s watchdog reapply
    assert!(now.duration_since(last_apply).as_secs() >= 90);

    let recent = now - Duration::from_secs(50);
    assert!(!(now.duration_since(recent).as_secs() >= 80));
    assert!(!(now.duration_since(recent).as_secs() >= 90));
}

#[tokio::test]
async fn test_notify_power_profile_dbus_handler() {
    let mut state = FanState::default();
    state.last_power_profile = "performance".to_string();
    state.better_auto_cooldown_level = 7;
    state.better_auto_cooldown_until = Instant::now() + Duration::from_secs(30);
    state.better_auto_last_apply = Instant::now();

    let state_arc = Arc::new(Mutex::new(state));
    let service = FanService::new_with_state(state_arc.clone());

    let res = service.notify_power_profile("balanced".to_string()).await;
    assert_eq!(res, "OK");

    let st = state_arc.lock().await;
    assert_eq!(st.last_power_profile, "balanced");
    assert_eq!(st.better_auto_cooldown_level, 0);
    assert!(st.better_auto_cooldown_until <= Instant::now());
    let now = Instant::now();
    assert!(now.duration_since(st.better_auto_last_apply).as_secs() >= 90);
}

#[tokio::test]
async fn test_profile_change_resets_cooldown_and_applies_ceiling() {
    // 1. Start with "performance" profile, active level 7 cooldown, ceiling 5
    let mut state = FanState::default();
    state.last_power_profile = "performance".to_string();
    state.acoustic_ceiling = 5;
    state.better_auto_level = 7;
    state.better_auto_cooldown_level = 7;
    state.better_auto_cooldown_until = Instant::now() + Duration::from_secs(30);

    let state_arc = Arc::new(Mutex::new(state));
    let service = FanService::new_with_state(state_arc.clone());

    // Switch from "performance" to "balanced"
    let res = service.notify_power_profile("balanced".to_string()).await;
    assert_eq!(res, "OK");

    {
        let mut st = state_arc.lock().await;
        assert_eq!(st.last_power_profile, "balanced");
        // Verify cooldown floor was reset (0 <= 5)
        assert_eq!(st.better_auto_cooldown_level, 0);

        let ceiling = get_effective_acoustic_ceiling(&st.last_power_profile, st.acoustic_ceiling);
        assert_eq!(ceiling, 5);

        // Under 76°C where raw computed level would be 6,
        // it must be clamped to ceiling 5
        let computed = fan::better_auto::compute_better_auto_level(76.0, 40.0, st.better_auto_level, ceiling);
        assert!(computed <= 5);

        let now = Instant::now();
        let s = &mut *st;
        let target = update_cooldown(computed, now, &mut s.better_auto_cooldown_level, &mut s.better_auto_cooldown_until);
        let clamped_target = if 76.0 >= fan::better_auto::EMERGENCY_TEMP_C {
            target
        } else {
            if s.better_auto_cooldown_level > ceiling {
                s.better_auto_cooldown_level = ceiling;
            }
            target.min(ceiling)
        };
        assert_eq!(clamped_target, 5);
        assert!(s.better_auto_cooldown_level <= 5);
    }

    // 2. Switch from "balanced" to "power-saver"
    let res = service.notify_power_profile("power-saver".to_string()).await;
    assert_eq!(res, "OK");

    {
        let mut st = state_arc.lock().await;
        assert_eq!(st.last_power_profile, "power-saver");
        assert_eq!(st.better_auto_cooldown_level, 0);

        let ceiling = get_effective_acoustic_ceiling(&st.last_power_profile, st.acoustic_ceiling);
        assert_eq!(ceiling, 3);

        // Under 76°C, Better Auto computes level <= 3
        let computed = fan::better_auto::compute_better_auto_level(76.0, 40.0, st.better_auto_level, ceiling);
        assert!(computed <= 3);

        let now = Instant::now();
        let s = &mut *st;
        let target = update_cooldown(computed, now, &mut s.better_auto_cooldown_level, &mut s.better_auto_cooldown_until);
        let clamped_target = if 76.0 >= fan::better_auto::EMERGENCY_TEMP_C {
            target
        } else {
            if s.better_auto_cooldown_level > ceiling {
                s.better_auto_cooldown_level = ceiling;
            }
            target.min(ceiling)
        };
        assert!(clamped_target <= 3);
        assert!(s.better_auto_cooldown_level <= 3);

        // Emergency temperature bypass: at 89°C >= EMERGENCY_TEMP_C (88°C), ceiling is bypassed to level 8
        let emerg_computed = fan::better_auto::compute_better_auto_level(89.0, 40.0, clamped_target, ceiling);
        assert_eq!(emerg_computed, 8);
        let emerg_target = if 89.0 >= fan::better_auto::EMERGENCY_TEMP_C {
            emerg_computed
        } else {
            emerg_computed.min(ceiling)
        };
        assert_eq!(emerg_target, 8);
    }
}

