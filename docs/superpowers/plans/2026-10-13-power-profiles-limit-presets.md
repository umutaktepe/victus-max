# Power Profiles Limit Presets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Integrate hardware power limit presets (Power Save, Balanced, Performance) into Victus Max across AMD Ryzen and Intel, with an in-daemon 30-second safety lock (watchdog), AC plug/unplug trigger, suspend/resume handler, PPD synchronization, and GTK4/CLI management.

**Architecture:** Extend `victus-max-types` with `PowerModeLimits` and `PowerModeProfilesConfig`. In `victus-max-daemon`, store presets in `power.json`, provide CPU-tier calibrated factory defaults via `cpu_tier.rs`, enforce limits silently every 30 seconds, and link AC/sleep/PPD events via `power_automation.rs`. In `victus-max-gui`, add 3 interactive mode cards with an architecture-aware edit dialog at the bottom of the Advanced Power page and synchronize with the sliders. In `victus-max-cli`, expose preset listing and activation commands.

**Tech Stack:** Rust (edition 2021), GTK4, Libadwaita 1, zbus 4, tokio, serde/serde_json, ryzenadj (AMD), intel-rapl / msr (Intel).

**Spec:** `docs/superpowers/specs/2026-10-13-power-profiles-limit-presets-design.md`

## Global Constraints

- Never use raw numbers in filenames; follow the living wiki rules in `AGENTS.md` (kebab-case, obsidian wikilinks `[[...]]`, ADRs, append-only `log.md`).
- Both AMD Ryzen (STAPM, Slow PPT, Fast PPT, Tctl Temp via `ryzenadj`) and Intel (PL1, PL2, TCC Offset via RAPL sysfs & MSR) must be fully supported.
- Holding `RYZENADJ_MUTEX` with a 40ms breather between SMU commands is mandatory on AMD platforms.
- 30-second watchdog execution must be completely silent (no desktop notification spam); desktop notifications are reserved for explicit mode changes and AC adapter plug/unplug events.
- All workspace crates must compile cleanly (`cargo check --workspace`) with passing tests (`cargo test --workspace`).

---

### Task 1: Shared Data Types and D-Bus Proxy Update (`victus-max-types`)

**Files:**
- Modify: `src/victus-max-types/src/lib.rs:40-60`
- Test: `src/victus-max-types/src/lib.rs` (inline unit tests)

**Interfaces:**
- Consumes: `serde::{Serialize, Deserialize}`, `zbus::proxy`
- Produces: `PowerModeLimits`, `PowerModeProfilesConfig`, updated `Power` D-Bus trait with `get_power_presets`, `set_power_preset_mode`, `save_power_preset`, `reset_power_presets`

- [ ] **Step 1: Write failing unit tests for `PowerModeProfilesConfig` serialization and deserialization**

In `src/victus-max-types/src/lib.rs`:
```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p victus-max-types`
Expected: Compilation failure due to missing `PowerModeLimits` and `PowerModeProfilesConfig`.

- [ ] **Step 3: Define structs and update `Power` proxy trait**

In `src/victus-max-types/src/lib.rs`:
```rust
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
```
And add proxy methods to `pub trait Power`:
```rust
    async fn get_power_presets(&self) -> zbus::Result<String>;
    async fn set_power_preset_mode(&self, mode: &str) -> zbus::Result<String>;
    async fn save_power_preset(&self, mode: &str, stapm_w: u32, slow_ppt_w: u32, fast_ppt_w: u32, tctl_temp: u32) -> zbus::Result<String>;
    async fn reset_power_presets(&self) -> zbus::Result<String>;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p victus-max-types`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/victus-max-types/src/lib.rs
git commit -m "feat(types): add power mode limit presets structs and proxy methods"
```

---

### Task 2: CPU-Tier Calibrated Factory Presets (`cpu_tier.rs`)

**Files:**
- Modify: `src/victus-max-daemon/src/cpu_tier.rs`
- Test: `src/victus-max-daemon/src/cpu_tier.rs` (inline unit tests)

**Interfaces:**
- Consumes: `CpuVendor`, `CpuTier`, `classify_cpu`, `PowerModeProfilesConfig`, `PowerModeLimits`
- Produces: `pub fn build_default_power_presets(cpu_spec: &str) -> PowerModeProfilesConfig`

- [ ] **Step 1: Write unit tests for `build_default_power_presets`**

In `src/victus-max-daemon/src/cpu_tier.rs`:
```rust
#[test]
fn test_default_power_presets_ryzen_hs() {
    let presets = build_default_power_presets("AMD Ryzen 7 7840HS w/ Radeon 780M Graphics");
    assert_eq!(presets.power_saver.stapm_w, 20);
    assert_eq!(presets.power_saver.fast_ppt_w, 25);
    assert_eq!(presets.power_saver.tctl_temp, 70);

    assert_eq!(presets.balanced.stapm_w, 35);
    assert_eq!(presets.balanced.slow_ppt_w, 35);
    assert_eq!(presets.balanced.fast_ppt_w, 35);
    assert_eq!(presets.balanced.tctl_temp, 82);

    assert_eq!(presets.performance.stapm_w, 45);
    assert_eq!(presets.performance.fast_ppt_w, 45);
    assert_eq!(presets.performance.tctl_temp, 88);
}

#[test]
fn test_default_power_presets_intel_standard() {
    let presets = build_default_power_presets("13th Gen Intel(R) Core(TM) i5-13500H");
    assert_eq!(presets.power_saver.stapm_w, 25);
    assert_eq!(presets.power_saver.fast_ppt_w, 35);
    assert_eq!(presets.power_saver.tctl_temp, 75);

    assert_eq!(presets.balanced.stapm_w, 45);
    assert_eq!(presets.balanced.fast_ppt_w, 65);
    assert_eq!(presets.balanced.tctl_temp, 85);

    assert_eq!(presets.performance.stapm_w, 55);
    assert_eq!(presets.performance.fast_ppt_w, 95);
    assert_eq!(presets.performance.tctl_temp, 97);
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p victus-max-daemon --lib cpu_tier::tests::test_default_power_presets_ryzen_hs`
Expected: FAIL (`build_default_power_presets` not found).

- [ ] **Step 3: Implement `build_default_power_presets`**

In `src/victus-max-daemon/src/cpu_tier.rs`:
```rust
use omen_types::{PowerModeLimits, PowerModeProfilesConfig};

pub fn build_default_power_presets(cpu_spec: &str) -> PowerModeProfilesConfig {
    let (vendor, tier, ranges) = classify_cpu(cpu_spec);

    match vendor {
        CpuVendor::Amd => match tier {
            CpuTier::AmdExtremeHx => PowerModeProfilesConfig {
                power_saver: PowerModeLimits { stapm_w: 25, slow_ppt_w: 25, fast_ppt_w: 35, tctl_temp: 72 },
                balanced: PowerModeLimits { stapm_w: 45, slow_ppt_w: 45, fast_ppt_w: 55, tctl_temp: 84 },
                performance: PowerModeLimits { stapm_w: 65, slow_ppt_w: 65, fast_ppt_w: 80, tctl_temp: 92 },
                active_mode: Some("balanced".to_string()),
                manual_override: false,
            },
            CpuTier::AmdUltraLow => PowerModeProfilesConfig {
                power_saver: PowerModeLimits { stapm_w: 12, slow_ppt_w: 12, fast_ppt_w: 15, tctl_temp: 68 },
                balanced: PowerModeLimits { stapm_w: 18, slow_ppt_w: 18, fast_ppt_w: 22, tctl_temp: 78 },
                performance: PowerModeLimits { stapm_w: 25, slow_ppt_w: 25, fast_ppt_w: 30, tctl_temp: 85 },
                active_mode: Some("balanced".to_string()),
                manual_override: false,
            },
            _ => PowerModeProfilesConfig {
                power_saver: PowerModeLimits { stapm_w: 20, slow_ppt_w: 20, fast_ppt_w: 25, tctl_temp: 70 },
                balanced: PowerModeLimits { stapm_w: 35, slow_ppt_w: 35, fast_ppt_w: 35, tctl_temp: 82 },
                performance: PowerModeLimits { stapm_w: 45, slow_ppt_w: 45, fast_ppt_w: 45, tctl_temp: 88 },
                active_mode: Some("balanced".to_string()),
                manual_override: false,
            },
        },
        CpuVendor::Intel => match tier {
            CpuTier::IntelExtreme => PowerModeProfilesConfig {
                power_saver: PowerModeLimits { stapm_w: 35, slow_ppt_w: 35, fast_ppt_w: 55, tctl_temp: 75 },
                balanced: PowerModeLimits { stapm_w: 55, slow_ppt_w: 55, fast_ppt_w: 95, tctl_temp: 88 },
                performance: PowerModeLimits { stapm_w: 80, slow_ppt_w: 80, fast_ppt_w: 125, tctl_temp: 97 },
                active_mode: Some("balanced".to_string()),
                manual_override: false,
            },
            _ => PowerModeProfilesConfig {
                power_saver: PowerModeLimits { stapm_w: 25, slow_ppt_w: 25, fast_ppt_w: 35, tctl_temp: 75 },
                balanced: PowerModeLimits { stapm_w: 45, slow_ppt_w: 45, fast_ppt_w: 65, tctl_temp: 85 },
                performance: PowerModeLimits { stapm_w: 55, slow_ppt_w: 55, fast_ppt_w: 95, tctl_temp: 97 },
                active_mode: Some("balanced".to_string()),
                manual_override: false,
            },
        },
        CpuVendor::Unknown => PowerModeProfilesConfig {
            power_saver: PowerModeLimits { stapm_w: ranges.sustained_min, slow_ppt_w: ranges.sustained_min, fast_ppt_w: ranges.burst_min, tctl_temp: ranges.temp_min },
            balanced: PowerModeLimits { stapm_w: ranges.sustained_default, slow_ppt_w: ranges.sustained_default, fast_ppt_w: ranges.burst_default, tctl_temp: 85 },
            performance: PowerModeLimits { stapm_w: ranges.sustained_max, slow_ppt_w: ranges.sustained_max, fast_ppt_w: ranges.burst_max, tctl_temp: ranges.temp_max },
            active_mode: Some("balanced".to_string()),
            manual_override: false,
        },
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p victus-max-daemon --lib cpu_tier::tests`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/victus-max-daemon/src/cpu_tier.rs
git commit -m "feat(daemon): add CPU-tier calibrated power profile defaults"
```

---

### Task 3: Backend PowerConfig & Persistence (`power.rs`)

**Files:**
- Modify: `src/victus-max-daemon/src/power.rs:44-95`
- Test: `src/victus-max-daemon/src/power.rs` (inline test for config loading/saving with presets)

**Interfaces:**
- Consumes: `omen_types::{PowerModeLimits, PowerModeProfilesConfig}`
- Produces: `PowerConfig.limit_presets`, `PowerConfig.active_preset_mode`, `PowerConfig.manual_override`, `apply_preset_limits(&self, mode: &str)`

- [ ] **Step 1: Write unit test for `PowerConfig` backward compatibility and presets migration**

In `src/victus-max-daemon/src/power.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_config_default_migration() {
        let legacy_json = r#"{
            "power_profile": "performance",
            "app_profiles_enabled": false,
            "app_profiles": {},
            "undervolt_mv": 0,
            "tcc_offset": 0,
            "pl1_w": 45,
            "slow_ppt_w": 45,
            "pl2_w": 80,
            "pl_enabled": true,
            "gpu_w": 0
        }"#;
        let mut cfg: PowerConfig = serde_json::from_str(legacy_json).expect("parse legacy config");
        assert!(cfg.limit_presets.is_none());
        assert_eq!(cfg.manual_override, false);
    }
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p victus-max-daemon --lib power::tests::test_power_config_default_migration`
Expected: Compilation failure due to missing fields in `PowerConfig`.

- [ ] **Step 3: Update `PowerConfig` struct and migration logic**

In `src/victus-max-daemon/src/power.rs`:
```rust
use omen_types::{PowerModeLimits, PowerModeProfilesConfig};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PowerConfig {
    power_profile: String,
    app_profiles_enabled: bool,
    app_profiles: HashMap<String, serde_json::Value>,
    undervolt_mv: i32,
    tcc_offset: i32,
    pl1_w: u32,
    #[serde(default)]
    slow_ppt_w: u32,
    pl2_w: u32,
    pl_enabled: bool,
    gpu_w: u32,
    #[serde(default)]
    limit_presets: Option<PowerModeProfilesConfig>,
    #[serde(default)]
    manual_override: bool,
}
```
In `PowerService::new()`:
```rust
let specs = crate::sysmon::get_hardware_specs();
if config.limit_presets.is_none() {
    let mut defaults = crate::cpu_tier::build_default_power_presets(&specs.cpu_spec);
    defaults.active_mode = Some(config.power_profile.clone());
    config.limit_presets = Some(defaults);
    config.manual_override = false;
    config.save();
}
```
Add helper method `apply_mode_limits`:
```rust
impl PowerService {
    pub async fn apply_preset_limits(&self, mode: &str) -> bool {
        let (limits, is_amd) = {
            let cfg = self.config.lock().await;
            let presets = cfg.limit_presets.as_ref();
            let limits = match mode {
                "power-saver" => presets.map(|p| p.power_saver),
                "performance" => presets.map(|p| p.performance),
                _ => presets.map(|p| p.balanced),
            };
            let specs = crate::sysmon::get_hardware_specs();
            let (vendor, _, _) = crate::cpu_tier::classify_cpu(&specs.cpu_spec);
            (limits, vendor == crate::cpu_tier::CpuVendor::Amd)
        };

        if let Some(l) = limits {
            if is_amd {
                let specs = crate::sysmon::get_hardware_specs();
                let (_, _, ranges) = crate::cpu_tier::classify_cpu(&specs.cpu_spec);
                let (norm_stapm, norm_slow, norm_fast) = crate::cpu_tier::normalize_ryzen_limits(
                    l.stapm_w,
                    l.slow_ppt_w,
                    l.fast_ppt_w,
                    ranges.stapm_fast_linked,
                );
                let arg_stapm = format!("--stapm-limit={}", norm_stapm * 1000);
                let arg_slow = format!("--slow-limit={}", norm_slow * 1000);
                let arg_fast = format!("--fast-limit={}", norm_fast * 1000);
                let arg_temp = format!("--tctl-temp={}", l.tctl_temp.clamp(70, 105));
                let _ = crate::cpu_tier::run_ryzenadj(&[&arg_stapm, &arg_slow, &arg_fast, &arg_temp]).await;
                info!("Enforced AMD Preset '{}': STAPM={}W Slow={}W Fast={}W Temp={}°C", mode, norm_stapm, norm_slow, norm_fast, l.tctl_temp);
            } else {
                let rapl1 = "/sys/class/powercap/intel-rapl/intel-rapl:0/constraint_0_power_limit_uw";
                let rapl2 = "/sys/class/powercap/intel-rapl/intel-rapl:0/constraint_1_power_limit_uw";
                if sysfs_exists(rapl1) {
                    let _ = sysfs_write_async(rapl1, &(l.stapm_w * 1_000_000).to_string()).await;
                }
                if sysfs_exists(rapl2) {
                    let _ = sysfs_write_async(rapl2, &(l.fast_ppt_w * 1_000_000).to_string()).await;
                }
                // TCC offset calculation (100 - tctl_temp)
                let tcc_offset = 100i32.saturating_sub(l.tctl_temp as i32).clamp(0, 15);
                crate::undervolt::write_intel_tcc_offset(tcc_offset);
                info!("Enforced Intel Preset '{}': PL1={}W PL2={}W TempTarget={}°C (TCC={})", mode, l.stapm_w, l.fast_ppt_w, l.tctl_temp, tcc_offset);
            }
            true
        } else {
            false
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p victus-max-daemon --lib power::tests`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/victus-max-daemon/src/power.rs
git commit -m "feat(daemon): add limit_presets to PowerConfig and apply_preset_limits"
```

---

### Task 4: D-Bus Methods and 30-Second Watchdog Loop (`power.rs`)

**Files:**
- Modify: `src/victus-max-daemon/src/power.rs:600-833`
- Test: `src/victus-max-daemon/src/power.rs` (unit tests for D-Bus methods behavior)

**Interfaces:**
- Consumes: `apply_preset_limits`, `set_power_limits`, `set_ryzen_power_limits`
- Produces: `get_power_presets`, `set_power_preset_mode`, `save_power_preset`, `reset_power_presets`, 30s background loop

- [ ] **Step 1: Write failing unit test for `manual_override` logic when sliders apply**

In `src/victus-max-daemon/src/power.rs`:
```rust
#[test]
fn test_slider_override_clears_active_preset() {
    let mut config = PowerConfig::default();
    let mut presets = PowerModeProfilesConfig::default();
    presets.active_mode = Some("balanced".to_string());
    config.limit_presets = Some(presets);
    config.manual_override = false;

    // Simulate slider apply
    config.manual_override = true;
    if let Some(ref mut p) = config.limit_presets {
        p.active_mode = None;
        p.manual_override = true;
    }

    assert_eq!(config.manual_override, true);
    assert_eq!(config.limit_presets.unwrap().active_mode, None);
}
```

- [ ] **Step 2: Run test to verify it compiles and runs**

Run: `cargo test -p victus-max-daemon --lib power::tests::test_slider_override_clears_active_preset`
Expected: PASS.

- [ ] **Step 3: Implement D-Bus methods and 30-second watchdog**

In `src/victus-max-daemon/src/power.rs`:
Expose on `#[interface(name = "org.hp.omen.Power")] impl PowerService`:
```rust
    async fn get_power_presets(&self) -> String {
        let cfg = self.config.lock().await.clone();
        let specs = crate::sysmon::get_hardware_specs();
        let (vendor, _, _) = crate::cpu_tier::classify_cpu(&specs.cpu_spec);
        let defaults = crate::cpu_tier::build_default_power_presets(&specs.cpu_spec);
        let current_presets = cfg.limit_presets.unwrap_or_else(|| defaults.clone());

        let json = serde_json::json!({
            "is_amd": vendor == crate::cpu_tier::CpuVendor::Amd,
            "presets": current_presets,
            "defaults": defaults,
            "active_mode": current_presets.active_mode,
            "manual_override": cfg.manual_override,
            "system_profile": cfg.power_profile,
        });
        json.to_string()
    }

    async fn set_power_preset_mode(&self, mode: String) -> String {
        let norm_mode = Self::normalize_profile(&mode);
        {
            let mut cfg = self.config.lock().await;
            cfg.power_profile = norm_mode.clone();
            cfg.manual_override = false;
            if let Some(ref mut p) = cfg.limit_presets {
                p.active_mode = Some(norm_mode.clone());
                p.manual_override = false;
            }
            cfg.save();
        }
        self.apply_preset_limits(&norm_mode).await;
        let _ = Self::sync_omen_profile(&norm_mode).await;

        // Desktop Notification on mode change
        let title = format!("Güç Modu: {}", norm_mode.to_uppercase());
        let desc = format!("{} profili güç limitleri başarıyla uygulandı ve kilitlendi.", norm_mode);
        crate::notifier::DesktopNotifier::send_notification(&title, &desc, 0).await;

        "OK".to_string()
    }

    async fn save_power_preset(&self, mode: String, stapm_w: u32, slow_ppt_w: u32, fast_ppt_w: u32, tctl_temp: u32) -> String {
        let norm_mode = Self::normalize_profile(&mode);
        let mut is_active = false;
        {
            let mut cfg = self.config.lock().await;
            if let Some(ref mut p) = cfg.limit_presets {
                let target = match norm_mode.as_str() {
                    "power-saver" => &mut p.power_saver,
                    "performance" => &mut p.performance,
                    _ => &mut p.balanced,
                };
                target.stapm_w = stapm_w;
                target.slow_ppt_w = slow_ppt_w;
                target.fast_ppt_w = fast_ppt_w;
                target.tctl_temp = tctl_temp;

                if p.active_mode.as_deref() == Some(&norm_mode) && !cfg.manual_override {
                    is_active = true;
                }
            }
            cfg.save();
        }

        if is_active {
            self.apply_preset_limits(&norm_mode).await;
        }

        "OK".to_string()
    }

    async fn reset_power_presets(&self) -> String {
        let specs = crate::sysmon::get_hardware_specs();
        let defaults = crate::cpu_tier::build_default_power_presets(&specs.cpu_spec);
        let active = {
            let mut cfg = self.config.lock().await;
            cfg.limit_presets = Some(defaults.clone());
            cfg.save();
            defaults.active_mode.unwrap_or_else(|| "balanced".to_string())
        };
        self.apply_preset_limits(&active).await;
        "OK".to_string()
    }
```
Update `set_power_limits` and `set_ryzen_power_limits` in `PowerService`:
```rust
    // In set_power_limits and set_ryzen_power_limits:
    let mut cfg = self.config.lock().await;
    cfg.manual_override = true;
    if let Some(ref mut p) = cfg.limit_presets {
        p.active_mode = None;
        p.manual_override = true;
    }
    cfg.save();
```
And spawn the 30-second silent watchdog loop in `PowerService::new()`:
```rust
let svc_watchdog = svc.clone();
tokio::spawn(async move {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
    loop {
        interval.tick().await;
        let (active_mode, manual_override, pl1, pl2, slow, pl_enabled) = {
            let cfg = svc_watchdog.config.lock().await;
            let active = cfg.limit_presets.as_ref().and_then(|p| p.active_mode.clone());
            (active, cfg.manual_override, cfg.pl1_w, cfg.pl2_w, cfg.slow_ppt_w, cfg.pl_enabled)
        };

        if manual_override {
            if pl_enabled {
                Self::apply_rapl_limits(pl1, pl2, Some(slow)).await;
            }
        } else if let Some(ref mode) = active_mode {
            svc_watchdog.apply_preset_limits(mode).await;
        }
    }
});
```

- [ ] **Step 4: Run compiler check**

Run: `cargo check -p victus-max-daemon`
Expected: SUCCESS with 0 errors.

- [ ] **Step 5: Commit**

```bash
git add src/victus-max-daemon/src/power.rs
git commit -m "feat(daemon): implement preset D-Bus methods and 30s silent safety watchdog"
```

---

### Task 5: Hardware Events, AC Adapter, and Sleep Wake Handler (`power_automation.rs`)

**Files:**
- Modify: `src/victus-max-daemon/src/power_automation.rs:1-116`
- Test: `src/victus-max-daemon/src/power_automation.rs`

**Interfaces:**
- Consumes: `org.freedesktop.login1.Manager.PrepareForSleep`, `net.hadess.PowerProfiles`, `PowerService`
- Produces: Seamless mode transition on AC plug/unplug, PPD change, and sleep resume

- [ ] **Step 1: Write test for profile mapping from PPD string**

In `src/victus-max-daemon/src/power_automation.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppd_mode_normalization() {
        assert_eq!(normalize_ppd_mode("power-saver"), "power-saver");
        assert_eq!(normalize_ppd_mode("performance"), "performance");
        assert_eq!(normalize_ppd_mode("balanced"), "balanced");
        assert_eq!(normalize_ppd_mode("quiet"), "power-saver");
    }
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p victus-max-daemon --lib power_automation::tests::test_ppd_mode_normalization`
Expected: FAIL (`normalize_ppd_mode` not found).

- [ ] **Step 3: Implement `normalize_ppd_mode`, sleep/resume listener, and AC trigger**

In `src/victus-max-daemon/src/power_automation.rs`:
```rust
pub fn normalize_ppd_mode(s: &str) -> &'static str {
    match s.to_lowercase().as_str() {
        "performance" => "performance",
        "power-saver" | "quiet" | "low-power" => "power-saver",
        _ => "balanced",
    }
}
```
In `start_monitor()`:
1. When PPD `ActiveProfile` changes:
```rust
let norm = normalize_ppd_mode(profile_str);
if let Ok(pwr_proxy) = omen_types::PowerProxy::new(&conn).await {
    let _ = pwr_proxy.set_power_preset_mode(norm).await;
}
```
2. When AC connects:
```rust
if let Ok(pwr_proxy) = omen_types::PowerProxy::new(&conn).await {
    let _ = pwr_proxy.set_power_preset_mode("performance").await;
}
```
When Battery connects:
```rust
if let Ok(pwr_proxy) = omen_types::PowerProxy::new(&conn).await {
    let _ = pwr_proxy.set_power_preset_mode("power-saver").await;
}
```
3. Add `systemd-logind` `PrepareForSleep` signal subscriber:
```rust
tokio::spawn(async move {
    if let Ok(conn) = zbus::Connection::system().await {
        if let Ok(proxy) = zbus::Proxy::new(
            &conn,
            "org.freedesktop.login1",
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
        ).await {
            use futures::StreamExt;
            if let Ok(mut stream) = proxy.receive_signal("PrepareForSleep").await {
                while let Some(sig) = stream.next().await {
                    if let Ok(sleeping) = sig.body().deserialize::<bool>() {
                        if !sleeping {
                            // System waking up from sleep/hibernate
                            info!("System resumed from sleep. Re-enforcing power limits in 1.5s...");
                            tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                            if let Ok(pwr) = omen_types::PowerProxy::new(&conn).await {
                                if let Ok(info_json) = pwr.get_power_presets().await {
                                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&info_json) {
                                        if let Some(active) = parsed.get("active_mode").and_then(|v| v.as_str()) {
                                            let _ = pwr.set_power_preset_mode(active).await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
});
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p victus-max-daemon --lib power_automation::tests`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/victus-max-daemon/src/power_automation.rs
git commit -m "feat(daemon): connect PPD signals, AC plug/unplug, and sleep resume to power presets"
```

---

### Task 6: GUI Daemon Client Updates (`daemon_client.rs`)

**Files:**
- Modify: `src/victus-max-gui/src/daemon_client.rs:690-760`
- Test: `cargo check -p victus-max-gui`

**Interfaces:**
- Consumes: `omen_types::PowerProxy`
- Produces: `get_power_presets_async()`, `set_power_preset_mode_async()`, `save_power_preset_async()`, `reset_power_presets_async()`

- [ ] **Step 1: Implement async and sync helper methods in `daemon_client.rs`**

In `src/victus-max-gui/src/daemon_client.rs`:
```rust
pub async fn get_power_presets_async() -> Result<String, Box<dyn std::error::Error>> {
    let conn = get_conn().await?;
    let proxy = PowerProxy::new(&conn).await?;
    let res = proxy.get_power_presets().await?;
    Ok(res)
}

pub fn set_power_preset_mode_sync(mode: String) {
    let rt = get_runtime();
    rt.spawn(async move {
        if let Ok(conn) = get_conn().await {
            if let Ok(proxy) = PowerProxy::new(&conn).await {
                if let Err(e) = proxy.set_power_preset_mode(&mode).await {
                    notify_dbus_error("Güç modu uygulanamadı", &e).await;
                }
            }
        }
    });
}

pub fn save_power_preset_sync(mode: String, stapm: u32, slow: u32, fast: u32, temp: u32) {
    let rt = get_runtime();
    rt.spawn(async move {
        if let Ok(conn) = get_conn().await {
            if let Ok(proxy) = PowerProxy::new(&conn).await {
                if let Err(e) = proxy.save_power_preset(&mode, stapm, slow, fast, temp).await {
                    notify_dbus_error("Güç modu profili kaydedilemedi", &e).await;
                }
            }
        }
    });
}

pub fn reset_power_presets_sync() {
    let rt = get_runtime();
    rt.spawn(async move {
        if let Ok(conn) = get_conn().await {
            if let Ok(proxy) = PowerProxy::new(&conn).await {
                if let Err(e) = proxy.reset_power_presets().await {
                    notify_dbus_error("Güç profilleri sıfırlanamadı", &e).await;
                }
            }
        }
    });
}
```

- [ ] **Step 2: Run compiler check**

Run: `cargo check -p victus-max-gui`
Expected: SUCCESS.

- [ ] **Step 3: Commit**

```bash
git add src/victus-max-gui/src/daemon_client.rs
git commit -m "feat(gui): add power presets D-Bus client bindings"
```

---

### Task 7: GTK4 / Libadwaita Advanced Power Page Integration (`undervolt.rs`)

**Files:**
- Modify: `src/victus-max-gui/src/undervolt.rs`
- Test: GUI compilation and simulation tests in `undervolt.rs`

**Interfaces:**
- Consumes: `daemon_client::get_power_presets_async`, `set_power_preset_mode_sync`, `save_power_preset_sync`
- Produces: 3 Mode Cards with Edit dialogs and bidirectional slider synchronization.

- [ ] **Step 1: Write simulation test for preset activation updating slider state**

In `src/victus-max-gui/src/undervolt.rs`:
```rust
#[test]
fn test_preset_mode_overrides_sliders_values() {
    let mut sim = SlidersSim::new(true, true, 45.0, 45.0, 65.0);
    // Simulate Balanced preset values (35W STAPM / 35W Slow / 35W Fast)
    sim.on_pl1_changed(35.0);
    sim.on_slow_changed(35.0);
    sim.on_pl2_changed(35.0);

    assert_eq!(sim.pl1, 35.0);
    assert_eq!(sim.slow, 35.0);
    assert_eq!(sim.pl2, 35.0);
}
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo test -p victus-max-gui --lib undervolt::tests::test_preset_mode_overrides_sliders_values`
Expected: PASS.

- [ ] **Step 3: Build the UI section at the bottom of `build_page()`**

In `src/victus-max-gui/src/undervolt.rs`:
1. Build `presets_group` with `adw::PreferencesGroup`.
2. Create 3 cards: `build_mode_card(mode_id, title, icon_name, default_sub)`.
3. Add Edit icon button to each card.
4. Implement `show_mode_edit_dialog(parent_window, is_amd, mode_id, current_limits, on_save, on_reset)`:
   - Contains Sliders with live value badges:
     - If AMD: STAPM, Slow PPT, Fast PPT, Tctl Temp.
     - If Intel: PL1, PL2, TCC Offset / Temp.
   - Buttons: "Varsayılana Sıfırla" (Reset to default), "İptal" (Cancel), "Kaydet" (Save).
5. Hook card activation:
   - When clicked: calls `daemon_client::set_power_preset_mode_sync(mode_id)`.
   - Sliders above update immediately to the preset's values.
6. Hook slider "Apply & Save":
   - When user clicks "Apply & Save": clears active card styling and marks state as "Manuel Override".

- [ ] **Step 4: Run compiler check and UI tests**

Run: `cargo test -p victus-max-gui`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/victus-max-gui/src/undervolt.rs
git commit -m "feat(gui): integrate power mode preset cards and edit dialog into Advanced Power page"
```

---

### Task 8: CLI Commands (`victus-max-cli`)

**Files:**
- Modify: `src/victus-max-cli/src/commands/power.rs`
- Modify: `src/victus-max-cli/src/dbus_proxy.rs`
- Test: `cargo test -p victus-max-cli`

**Interfaces:**
- Consumes: `victus_max_types::PowerProxy`
- Produces: `victus-max-cli power preset list`, `victus-max-cli power preset set <mode>`, `victus-max-cli power limits`

- [ ] **Step 1: Write unit test for CLI argument parsing**

In `src/victus-max-cli/src/commands/power.rs`:
Add test for subcommands `presets` and `limits`.

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p victus-max-cli`
Expected: FAIL.

- [ ] **Step 3: Implement CLI commands**

In `src/victus-max-cli/src/commands/power.rs`:
```rust
pub async fn handle_presets(action: Option<String>, mode: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let conn = zbus::Connection::system().await?;
    let proxy = omen_types::PowerProxy::new(&conn).await?;

    match action.as_deref() {
        Some("set") => {
            if let Some(m) = mode {
                let res = proxy.set_power_preset_mode(&m).await?;
                println!("Power preset set to '{}': {}", m, res);
            } else {
                eprintln!("Usage: victus-max-cli power presets set <power-saver|balanced|performance>");
            }
        }
        _ => {
            let json_str = proxy.get_power_presets().await?;
            let data: serde_json::Value = serde_json::from_str(&json_str)?;
            println!("=== Victus Max Güç Modu Profilleri ===");
            println!("Aktif Mod: {}", data.get("active_mode").and_then(|v| v.as_str()).unwrap_or("Manuel Override"));
            println!("Manuel Override: {}", data.get("manual_override").and_then(|v| v.as_bool()).unwrap_or(false));
            println!("{:#}", data.get("presets").unwrap_or(&serde_json::Value::Null));
        }
    }
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p victus-max-cli`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/victus-max-cli/
git commit -m "feat(cli): add power presets and limits query commands"
```

---

### Task 9: ADR-008 & LLM Wiki Living Architecture (`AGENTS.md`)

**Files:**
- Create: `docs/victus-max-wiki/decisions/adr-008-power-profile-limit-presets.md`
- Modify: `docs/victus-max-wiki/system-daemon/power-service.md`
- Modify: `docs/victus-max-wiki/index.md`
- Modify: `docs/victus-max-wiki/log.md`

- [ ] **Step 1: Write `adr-008-power-profile-limit-presets.md`**

Include: Bağlam, Alternatifler, Karar, Sonuçlar (Trade-offs), İlgili Wikilinkler (`[[adr-004-native-msr-and-smu-mailbox-tuning]]`, `[[adr-007-adaptive-ryzen-power-limits]]`).

- [ ] **Step 2: Update `index.md` (MOC) with ADR-008 and link**

- [ ] **Step 3: Append to `log.md` with format `## [2026-10-13] Özellik | Güç Modu Profilleri ve Donanımsal Koruma Mimarisi`**

- [ ] **Step 4: Commit**

```bash
git add docs/victus-max-wiki/
git commit -m "docs(wiki): add ADR-008 and update knowledge graph for power presets"
```

---

### Task 10: End-to-End Build and Verification Pass

**Files:**
- Verification only

- [ ] **Step 1: Run full test suite**

Run: `cargo test --workspace`
Expected: All tests PASS across `victus-max-types`, `victus-max-daemon`, `victus-max-gui`, `victus-max-cli`.

- [ ] **Step 2: Run full workspace build**

Run: `cargo build --workspace`
Expected: Zero compilation errors.

- [ ] **Step 3: Verification of wikilinks and graph hygiene**

Verify no dangling wikilinks in docs.
