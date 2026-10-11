# Adaptive AMD Ryzen Power Limits & Live Telemetry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement adaptive CPU architecture detection in Victus Max to display dedicated AMD Ryzen power limits (STAPM, Fast PPT, Tctl Max Temperature) with live hardware state reading via RyzenAdj/SMU and CPU tier-aware slider ranges, while preserving Intel RAPL controls for Intel processors.

**Architecture:** 
1. CPU Tier & Arch Classifier: Detect processor vendor and tier (U-series, HS/H-series, HX-series) from `/proc/cpuinfo` / hardware specs to define safe minimum, maximum, and default power envelopes.
2. Daemon Hardware Telemetry: Extend `victus-max-daemon`'s power service to query live power limits via `ryzenadj -i` / PMTable parser when running on AMD Ryzen, falling back gracefully to persistent config (`/etc/victus-max/power.json`).
3. D-Bus IPC & Types: Surface AMD-specific power limits (`stapm_w`, `fast_ppt_w`, `slow_ppt_w`, `tctl_temp_c`, `cpu_tier`) across `org.hp.omen.Power` and `victus-max-types`.
4. Adaptive GUI: Dynamically rebrand and adapt the "Power Limits" group in `undervolt.rs` based on CPU architecture—showing AMD STAPM/Fast PPT/Tctl Temp sliders with live loaded values for Ryzen, and PL1/PL2/TCC Offset for Intel.

**Tech Stack:** Rust (Tokio, Zbus, Serde, Libadwaita, GTK4), RyzenAdj CLI / SMU mailbox, Linux sysfs (`intel-rapl`), i18n.

**Spec:** `docs/victus-max-wiki/hardware-drivers/amd-ryzen-smu.md`, `docs/victus-max-wiki/decisions/adr-004-native-msr-and-smu-mailbox-tuning.md`

## Global Constraints

- Never break backward compatibility for Intel 12th/13th/14th Gen RAPL power limits and MSR undervolting.
- Zero-panic rule: All subprocess calls (`ryzenadj -i`, `ryzenadj --stapm-limit`, etc.) must have timeouts (max 500ms) and graceful fallbacks to persisted configuration on failure or parse errors.
- AMD Temperature Display: Tctl temperature must be presented to the user directly in degrees Celsius (e.g., 75°C - 100°C), converting transparently to/from daemon TCC offset if legacy interfaces are used.
- Adhere to `AGENTS.md`: Update wiki pages and `log.md` with appropriate format and wikilinks.

---

### Task 1: CPU Architecture & TDP Tier Classifier Module

**Files:**
- Create: `src/victus-max-daemon/src/cpu_tier.rs`
- Modify: `src/victus-max-daemon/src/main.rs:1-30`
- Test: `src/victus-max-daemon/tests/cpu_tier_test.rs`

**Interfaces:**
- Produces:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
  pub enum CpuVendor {
      Intel,
      Amd,
      Unknown,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
  pub enum CpuTier {
      AmdUltraLow,   // U-series: 15W - 30W
      AmdStandardHs, // HS/H-series: 35W - 54W+ (e.g. 7840HS)
      AmdExtremeHx,  // HX-series: 55W - 100W+ (e.g. 7945HX)
      IntelStandard, // Intel H/P/U
      IntelExtreme,  // Intel HX/HK
      Generic,
  }

  pub struct CpuPowerProfileRanges {
      pub sustained_min: u32,
      pub sustained_max: u32,
      pub sustained_default: u32,
      pub burst_min: u32,
      pub burst_max: u32,
      pub burst_default: u32,
      pub temp_min: u32,
      pub temp_max: u32,
      pub temp_default: u32,
  }

  pub fn classify_cpu(cpu_name: &str) -> (CpuVendor, CpuTier, CpuPowerProfileRanges);
  ```

- [ ] **Step 1: Write failing unit test for CPU classification**

Create `src/victus-max-daemon/tests/cpu_tier_test.rs`:
```rust
use victus_max_daemon::cpu_tier::{classify_cpu, CpuTier, CpuVendor};

#[test]
fn test_classify_amd_7840hs() {
    let (vendor, tier, ranges) = classify_cpu("AMD Ryzen 7 7840HS w/ Radeon 780M Graphics");
    assert_eq!(vendor, CpuVendor::Amd);
    assert_eq!(tier, CpuTier::AmdStandardHs);
    assert_eq!(ranges.sustained_default, 45);
    assert_eq!(ranges.burst_default, 65);
    assert_eq!(ranges.sustained_min, 25);
    assert_eq!(ranges.sustained_max, 75);
}

#[test]
fn test_classify_amd_u_series() {
    let (vendor, tier, ranges) = classify_cpu("AMD Ryzen 5 5625U with Radeon Graphics");
    assert_eq!(vendor, CpuVendor::Amd);
    assert_eq!(tier, CpuTier::AmdUltraLow);
    assert_eq!(ranges.sustained_default, 15);
    assert_eq!(ranges.sustained_min, 10);
    assert_eq!(ranges.sustained_max, 35);
}

#[test]
fn test_classify_intel_13700h() {
    let (vendor, tier, ranges) = classify_cpu("13th Gen Intel(R) Core(TM) i7-13700H");
    assert_eq!(vendor, CpuVendor::Intel);
    assert_eq!(tier, CpuTier::IntelStandard);
    assert_eq!(ranges.sustained_default, 45);
    assert_eq!(ranges.burst_default, 95);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cpu_tier_test -p victus-max-daemon`
Expected: FAIL (module `cpu_tier` not found)

- [ ] **Step 3: Implement `src/victus-max-daemon/src/cpu_tier.rs` and expose in `main.rs`**

Create `src/victus-max-daemon/src/cpu_tier.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpuVendor {
    Intel,
    Amd,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpuTier {
    AmdUltraLow,
    AmdStandardHs,
    AmdExtremeHx,
    IntelStandard,
    IntelExtreme,
    Generic,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CpuPowerProfileRanges {
    pub sustained_min: u32,
    pub sustained_max: u32,
    pub sustained_default: u32,
    pub burst_min: u32,
    pub burst_max: u32,
    pub burst_default: u32,
    pub temp_min: u32,
    pub temp_max: u32,
    pub temp_default: u32,
}

pub fn classify_cpu(cpu_name: &str) -> (CpuVendor, CpuTier, CpuPowerProfileRanges) {
    let name_upper = cpu_name.to_uppercase();

    if name_upper.contains("AMD") || name_upper.contains("RYZEN") {
        if name_upper.contains("HX") {
            (
                CpuVendor::Amd,
                CpuTier::AmdExtremeHx,
                CpuPowerProfileRanges {
                    sustained_min: 35,
                    sustained_max: 100,
                    sustained_default: 55,
                    burst_min: 45,
                    burst_max: 125,
                    burst_default: 80,
                    temp_min: 75,
                    temp_max: 100,
                    temp_default: 95,
                },
            )
        } else if name_upper.contains('U') {
            (
                CpuVendor::Amd,
                CpuTier::AmdUltraLow,
                CpuPowerProfileRanges {
                    sustained_min: 10,
                    sustained_max: 35,
                    sustained_default: 15,
                    burst_min: 15,
                    burst_max: 45,
                    burst_default: 25,
                    temp_min: 70,
                    temp_max: 95,
                    temp_default: 85,
                },
            )
        } else {
            // Default AMD laptop (HS / H / standard 35-54W)
            (
                CpuVendor::Amd,
                CpuTier::AmdStandardHs,
                CpuPowerProfileRanges {
                    sustained_min: 25,
                    sustained_max: 75,
                    sustained_default: 45,
                    burst_min: 35,
                    burst_max: 95,
                    burst_default: 65,
                    temp_min: 75,
                    temp_max: 100,
                    temp_default: 95,
                },
            )
        }
    } else if name_upper.contains("INTEL") {
        if name_upper.contains("HX") || name_upper.contains("HK") {
            (
                CpuVendor::Intel,
                CpuTier::IntelExtreme,
                CpuPowerProfileRanges {
                    sustained_min: 35,
                    sustained_max: 125,
                    sustained_default: 55,
                    burst_min: 55,
                    burst_max: 155,
                    burst_default: 115,
                    temp_min: 80,
                    temp_max: 100,
                    temp_default: 97,
                },
            )
        } else {
            (
                CpuVendor::Intel,
                CpuTier::IntelStandard,
                CpuPowerProfileRanges {
                    sustained_min: 25,
                    sustained_max: 75,
                    sustained_default: 45,
                    burst_min: 45,
                    burst_max: 115,
                    burst_default: 95,
                    temp_min: 80,
                    temp_max: 100,
                    temp_default: 97,
                },
            )
        }
    } else {
        (
            CpuVendor::Unknown,
            CpuTier::Generic,
            CpuPowerProfileRanges {
                sustained_min: 15,
                sustained_max: 95,
                sustained_default: 45,
                burst_min: 25,
                burst_max: 115,
                burst_default: 65,
                temp_min: 70,
                temp_max: 100,
                temp_default: 95,
            },
        )
    }
}
```

In `src/victus-max-daemon/src/main.rs`:
Add `pub mod cpu_tier;`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cpu_tier_test -p victus-max-daemon`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/victus-max-daemon/src/cpu_tier.rs src/victus-max-daemon/src/main.rs src/victus-max-daemon/tests/cpu_tier_test.rs
git commit -m "feat(daemon): add CPU architecture and TDP tier classifier"
```

---

### Task 2: Live RyzenAdj Hardware Telemetry Parser & Daemon Power Service Integration

**Files:**
- Modify: `src/victus-max-daemon/src/power.rs:500-600`
- Test: `src/victus-max-daemon/tests/ryzen_telemetry_test.rs`

**Interfaces:**
- Produces in `PowerService`:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct RyzenLiveMetrics {
      pub stapm_w: Option<u32>,
      pub fast_limit_w: Option<u32>,
      pub slow_limit_w: Option<u32>,
      pub tctl_temp_c: Option<u32>,
  }

  pub fn parse_ryzenadj_info_output(stdout: &str) -> RyzenLiveMetrics;
  ```
- Produces in `GetPowerProfile` D-Bus response:
  ```json
  {
    "available": true,
    "active": "balanced",
    "is_amd": true,
    "cpu_tier": "AmdStandardHs",
    "pl1_w": 45,
    "pl2_w": 65,
    "stapm_w": 45,
    "fast_ppt_w": 65,
    "slow_ppt_w": 45,
    "tctl_temp": 95,
    "ranges": { ... }
  }
  ```

- [ ] **Step 1: Write failing unit test for `parse_ryzenadj_info_output`**

Create `src/victus-max-daemon/tests/ryzen_telemetry_test.rs`:
```rust
use victus_max_daemon::power::parse_ryzenadj_info_output;

#[test]
fn test_parse_ryzenadj_info() {
    let sample = r#"
CPU Family: Phoenix Point
SMU BIOS Interface Version: 18
|   Name                   |   Value   |   Parameter                |
|--------------------------|-----------|----------------------------|
| STAPM LIMIT              |    45.000 | stapm-limit                 |
| STAPM VALUE              |    14.231 |                            |
| PPT LIMIT FAST           |    65.000 | fast-limit                  |
| PPT VALUE FAST           |    14.300 |                            |
| PPT LIMIT SLOW           |    45.000 | slow-limit                  |
| PPT VALUE SLOW           |    14.280 |                            |
| THM LIMIT CORE           |    95.000 | tctl-temp                   |
"#;
    let metrics = parse_ryzenadj_info_output(sample);
    assert_eq!(metrics.stapm_w, Some(45));
    assert_eq!(metrics.fast_limit_w, Some(65));
    assert_eq!(metrics.slow_limit_w, Some(45));
    assert_eq!(metrics.tctl_temp_c, Some(95));
}

#[test]
fn test_parse_ryzenadj_info_with_nan() {
    let sample = r#"
| STAPM LIMIT              |       nan | stapm-limit                 |
| PPT LIMIT FAST           |    65.000 | fast-limit                  |
"#;
    let metrics = parse_ryzenadj_info_output(sample);
    assert_eq!(metrics.stapm_w, None);
    assert_eq!(metrics.fast_limit_w, Some(65));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test ryzen_telemetry_test -p victus-max-daemon`
Expected: FAIL (`parse_ryzenadj_info_output` not found)

- [ ] **Step 3: Implement `parse_ryzenadj_info_output` and async live query in `src/victus-max-daemon/src/power.rs`**

In `src/victus-max-daemon/src/power.rs`:
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RyzenLiveMetrics {
    pub stapm_w: Option<u32>,
    pub fast_limit_w: Option<u32>,
    pub slow_limit_w: Option<u32>,
    pub tctl_temp_c: Option<u32>,
}

pub fn parse_ryzenadj_info_output(stdout: &str) -> RyzenLiveMetrics {
    let mut metrics = RyzenLiveMetrics::default();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        if parts.len() >= 3 {
            let key = parts[1].to_uppercase();
            let val_str = parts[2];
            if let Ok(val_f) = val_str.parse::<f64>() {
                if !val_f.is_nan() && val_f > 0.0 {
                    let val_u = val_f.round() as u32;
                    if key.contains("STAPM LIMIT") {
                        metrics.stapm_w = Some(val_u);
                    } else if key.contains("PPT LIMIT FAST") {
                        metrics.fast_limit_w = Some(val_u);
                    } else if key.contains("PPT LIMIT SLOW") {
                        metrics.slow_limit_w = Some(val_u);
                    } else if key.contains("THM LIMIT CORE") {
                        metrics.tctl_temp_c = Some(val_u);
                    }
                }
            }
        }
    }

    metrics
}

pub async fn fetch_live_ryzen_metrics() -> Option<RyzenLiveMetrics> {
    let child = tokio::process::Command::new("ryzenadj")
        .arg("-i")
        .output();

    match tokio::time::timeout(tokio::time::Duration::from_millis(500), child).await {
        Ok(Ok(output)) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Some(parse_ryzenadj_info_output(&stdout))
        }
        _ => None,
    }
}
```

In `get_power_profile(&self)` of `PowerService`:
Update to incorporate AMD live query and classifier:
```rust
        let specs = crate::sysmon::get_hardware_specs();
        let (vendor, tier, ranges) = crate::cpu_tier::classify_cpu(&specs.cpu_spec);
        let is_amd = vendor == crate::cpu_tier::CpuVendor::Amd;

        let mut real_pl1 = cfg.pl1_w;
        let mut real_pl2 = cfg.pl2_w;
        let mut live_tctl = None;

        if is_amd {
            if let Some(metrics) = fetch_live_ryzen_metrics().await {
                if let Some(stapm) = metrics.stapm_w { real_pl1 = stapm; }
                if let Some(fast) = metrics.fast_limit_w { real_pl2 = fast; }
                if let Some(tctl) = metrics.tctl_temp_c { live_tctl = Some(tctl); }
            }
        } else {
            if let Some(val) = sysfs_read_async("/sys/class/powercap/intel-rapl/intel-rapl:0/constraint_0_power_limit_uw").await {
                if let Ok(uw) = val.parse::<u32>() { real_pl1 = uw / 1_000_000; }
            }
            if let Some(val) = sysfs_read_async("/sys/class/powercap/intel-rapl/intel-rapl:0/constraint_1_power_limit_uw").await {
                if let Ok(uw) = val.parse::<u32>() { real_pl2 = uw / 1_000_000; }
            }
        }

        let json = serde_json::json!({
            "available": true,
            "active": active,
            "profiles": available_profiles,
            "is_amd": is_amd,
            "cpu_tier": format!("{:?}", tier),
            "ranges": ranges,
            "pl1_w": real_pl1,
            "pl2_w": real_pl2,
            "stapm_w": real_pl1,
            "fast_ppt_w": real_pl2,
            "tctl_temp": live_tctl.unwrap_or(if cfg.tcc_offset > 0 { 100 - cfg.tcc_offset as u32 } else { ranges.temp_default }),
            "pl_enabled": cfg.pl_enabled,
            "undervolt_mv": cfg.undervolt_mv,
            "tcc_offset": cfg.tcc_offset,
            "gpu_w": cfg.gpu_w,
        });
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test ryzen_telemetry_test -p victus-max-daemon`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/victus-max-daemon/src/power.rs src/victus-max-daemon/tests/ryzen_telemetry_test.rs
git commit -m "feat(daemon): add live RyzenAdj hardware telemetry parser and power profile payload"
```

---

### Task 3: D-Bus Client Data Structures & Proxy Extensions

**Files:**
- Modify: `src/victus-max-gui/src/daemon_client.rs`

**Interfaces:**
- Produces in `daemon_client.rs`:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct PowerProfileInfo {
      pub is_amd: bool,
      pub cpu_tier: String,
      pub pl1_w: u32,
      pub pl2_w: u32,
      pub stapm_w: u32,
      pub fast_ppt_w: u32,
      pub tctl_temp: u32,
      pub ranges: Option<CpuPowerProfileRanges>,
  }

  pub async fn get_power_profile_info_async() -> Result<PowerProfileInfo, Box<dyn std::error::Error>>;
  ```

- [ ] **Step 1: Implement `PowerProfileInfo` parsing in `src/victus-max-gui/src/daemon_client.rs`**

Add helper function to parse full json:
```rust
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CpuPowerProfileRanges {
    pub sustained_min: u32,
    pub sustained_max: u32,
    pub sustained_default: u32,
    pub burst_min: u32,
    pub burst_max: u32,
    pub burst_default: u32,
    pub temp_min: u32,
    pub temp_max: u32,
    pub temp_default: u32,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PowerProfileInfo {
    #[serde(default)]
    pub is_amd: bool,
    #[serde(default)]
    pub cpu_tier: String,
    #[serde(default)]
    pub pl1_w: u32,
    #[serde(default)]
    pub pl2_w: u32,
    #[serde(default)]
    pub stapm_w: u32,
    #[serde(default)]
    pub fast_ppt_w: u32,
    #[serde(default)]
    pub tctl_temp: u32,
    #[serde(default)]
    pub ranges: Option<CpuPowerProfileRanges>,
}

pub async fn get_power_profile_info_async() -> Result<PowerProfileInfo, Box<dyn std::error::Error>> {
    let json_str = get_power_profile_raw_async().await?;
    let info: PowerProfileInfo = serde_json::from_str(&json_str)?;
    Ok(info)
}
```

- [ ] **Step 2: Verify `daemon_client.rs` compiles**

Run: `cargo check -p victus-max-gui`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/victus-max-gui/src/daemon_client.rs
git commit -m "feat(gui): add PowerProfileInfo parsing for adaptive Ryzen power limits"
```

---

### Task 4: Multi-Language (i18n) Translations for AMD Ryzen Power Limits

**Files:**
- Modify: `src/victus-max-gui/src/i18n.rs`

- [ ] **Step 1: Add Turkish and English dictionary keys**

In `src/victus-max-gui/src/i18n.rs`:
In `translate_tr`:
```rust
        "ryzen_power_limits_title" => "Güç Limitleri (AMD Ryzen PPT / STAPM)",
        "ryzen_power_limits_desc" => "STAPM sürekli kasa yüzey güç bütçesini, PPT Fast ise anlık turbo tavanını belirler.",
        "stapm_label" => "STAPM / PPT Slow — Sürekli Güç Limiti",
        "stapm_sub" => "Termal doyum sonrası uzun vadeli güç bütçesi (Watt)",
        "fast_ppt_label" => "PPT Fast — Kısa Süreli Turbo Güç Limiti",
        "fast_ppt_sub" => "Anlık yük patlamaları için izin verilen tepe tavan (Watt)",
        "tctl_temp_label" => "Tctl Sıcaklık Limiti (Max CPU Sıcaklığı)",
        "tctl_temp_sub" => "İşlemcinin termal kısmaya girmeden önceki tepe sıcaklık hedefi",
```

In `translate_en`:
```rust
        "ryzen_power_limits_title" => "Power Limits (AMD Ryzen PPT / STAPM)",
        "ryzen_power_limits_desc" => "STAPM manages sustained chassis power budget; PPT Fast sets short-duration boost ceiling.",
        "stapm_label" => "STAPM / PPT Slow — Sustained Power Limit",
        "stapm_sub" => "Long-term sustained power draw ceiling (Watts)",
        "fast_ppt_label" => "PPT Fast — Short Duration Turbo Limit",
        "fast_ppt_sub" => "Peak power ceiling during transient boost bursts (Watts)",
        "tctl_temp_label" => "Tctl Temperature Limit (Max CPU Temperature)",
        "tctl_temp_sub" => "Target peak temperature before thermal throttling begins",
```

- [ ] **Step 2: Verify `i18n.rs` compiles**

Run: `cargo check -p victus-max-gui`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/victus-max-gui/src/i18n.rs
git commit -m "feat(i18n): add Turkish and English translations for AMD Ryzen power limits"
```

---

### Task 5: Adaptive Advanced Power GUI Component

**Files:**
- Modify: `src/victus-max-gui/src/undervolt.rs`

**Interfaces:**
- Produces: Dynamically reconfigures `pwr_group`, `pl1_row`, `pl2_row`, and `tcc_row` based on detected CPU vendor:
  - If AMD:
    - Title: `ryzen_power_limits_title`, Subtitle: `ryzen_power_limits_desc`.
    - Row 1: STAPM, Range set from `ranges.sustained_min` to `ranges.sustained_max`.
    - Row 2: PPT Fast, Range set from `ranges.burst_min` to `ranges.burst_max`.
    - Row 3: Tctl Sıcaklık Limiti, Range set 75.0 to 100.0, label shows `{X}°C`.
    - Live slider values loaded from `PowerProfileInfo`.
    - Reset button resets to CPU's default TDP (e.g. 45W / 65W / 95°C).
  - If Intel:
    - Retains original PL1/PL2/TCC Offset labels and ranges.

- [ ] **Step 1: Update `build_page()` in `src/victus-max-gui/src/undervolt.rs`**

Refactor the widget instantiation so rows, scales, and group references are accessible to the `glib::spawn_future_local` async loader:
- Adapt `pwr_group`, `pl1_row`, `pl2_row`, `tcc_row`.
- When `is_amd` is detected in `glib::spawn_future_local`:
  - Update group title to `i18n::t("ryzen_power_limits_title")`.
  - Update group description to `i18n::t("ryzen_power_limits_desc")`.
  - Update `pl1_row.set_title(i18n::t("stapm_label"))`.
  - Update `pl1_row.set_subtitle(i18n::t("stapm_sub"))`.
  - Update `pl2_row.set_title(i18n::t("fast_ppt_label"))`.
  - Update `pl2_row.set_subtitle(i18n::t("fast_ppt_sub"))`.
  - Update `tcc_row.set_title(i18n::t("tctl_temp_label"))`.
  - Update `tcc_row.set_subtitle(i18n::t("tctl_temp_sub"))`.
  - Set scale ranges according to `info.ranges` (e.g., 25.0 - 75.0 for 7840HS).
  - Set scale values directly from `info.stapm_w`, `info.fast_ppt_w`, `info.tctl_temp`.
  - Update labels to show live values (e.g. "45 W", "65 W", "95°C").
- In Apply button:
  - If AMD:
    - `set_power_limits_sync(stapm_val, fast_val)`
    - Convert `tctl_temp` to TCC offset if daemon requires it (`100 - tctl_val`), or pass directly.

- [ ] **Step 2: Build GUI and test run**

Run: `cargo build -p victus-max-gui`
Expected: Build success with zero errors.

- [ ] **Step 3: Commit**

```bash
git add src/victus-max-gui/src/undervolt.rs
git commit -m "feat(gui): adapt power limit sliders, ranges, and live values for AMD Ryzen"
```

---

### Task 6: Documentation, ADR Record & Log Update

**Files:**
- Create: `docs/victus-max-wiki/decisions/adr-007-adaptive-ryzen-power-limits.md`
- Modify: `docs/victus-max-wiki/interfaces/power-dbus-interface.md`
- Modify: `docs/victus-max-wiki/index.md`
- Modify: `docs/victus-max-wiki/log.md`

- [ ] **Step 1: Create ADR-007**

Document the architectural decision regarding hybrid live-reading via RyzenAdj table dump and CPU TDP tier classification.

- [ ] **Step 2: Update MOC and interfaces**

Update `docs/victus-max-wiki/index.md` and `docs/victus-max-wiki/interfaces/power-dbus-interface.md` with the new JSON payload schema.

- [ ] **Step 3: Update `docs/victus-max-wiki/log.md`**

Add append-only log entry:
`## [2026-10-09] MİMARİ_GÜNCELLEME | AMD Ryzen Dinamik Güç Sınırları ve Canlı Telemetri Entegrasyonu`

- [ ] **Step 4: Commit**

```bash
git add docs/victus-max-wiki/
git commit -m "docs(wiki): add ADR-007 and update wiki for adaptive Ryzen power limits"
```
