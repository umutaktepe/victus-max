# Victus Max Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Integrate victus-control's proactive "Better Auto" fan algorithm, PM-safe GPU polling, and 10s EC stagger into omen-space's modern Rust ecosystem, establishing Victus Max with configurable minimum fan RPM (default 2600 RPM) and power-profile-aware acoustic ceilings.

**Architecture:** Extend `omen-space-daemon` with a Rust port of Better Auto (`better_auto.rs`), enhanced telemetry in `sysmon`, 10-second EC stagger for dual fan writes, and dynamic minimum RPM configuration. Expose controls over D-Bus (`org.hp.omen.Fan`) and present a first-class "Better Auto" card with min RPM controls in the GTK4/Libadwaita GUI and CLI.

**Tech Stack:** Rust (Tokio, Zbus, Serde), GTK4 / Libadwaita, Linux sysfs / WMI (`hp-wmi`, `hp-omen-extra`), `/proc/stat`, NVIDIA NVML/SMI.

**Spec:** [docs/superpowers/specs/2026-09-29-victus-max-design.md](file:///home/umutaktepe/victus-max/docs/superpowers/specs/2026-09-29-victus-max-design.md)

## Global Constraints

- Never wake a runtime-suspended NVIDIA dGPU: check `/sys/bus/pci/devices/.../power/runtime_status == "active"` before calling `nvidia-smi`.
- Hardware Stagger rule: enforce an asynchronous 10-second wait between Fan 1 and Fan 2 target speed writes to avoid HP EC register lockup.
- BIOS Watchdog rule: reassert target fan RPMs every 90 seconds and manual mode every 80 seconds.
- Minimum Fan RPM: Default to 2600 RPM for Balanced mode, configurable between 2000 and 3500 RPM via persistent config and GUI.
- Do not remove or break existing OMEN/Victus features (RGB, MUX switch, TCC offset, RAPL power limits).

---

### Task 1: Proactive Telemetry & PM-Safe GPU Sensor Module

**Files:**
- Modify: `omen-space/src/omen-space-daemon/src/sysmon/stats.rs`
- Modify: `omen-space/src/omen-space-daemon/src/sysmon/gpu.rs`
- Create: `omen-space/src/omen-space-daemon/src/sysmon/load.rs`
- Test: `omen-space/src/omen-space-daemon/tests/telemetry_test.rs`

**Interfaces:**
- Produces: `pub struct LoadSnapshot { pub cpu_usage_pct: Option<f64>, pub gpu_usage_pct: Option<f64>, pub gpu_temp_c: Option<f64> }`
- Produces: `pub fn collect_load_snapshot() -> LoadSnapshot`
- Produces: `pub fn is_nvidia_gpu_powered() -> bool`

- [ ] **Step 1: Write failing unit test for CPU load delta calculation**

Create `omen-space/src/omen-space-daemon/tests/telemetry_test.rs`:
```rust
use omen_space_daemon::sysmon::load::{calculate_cpu_usage, CpuRawTimes};

#[test]
fn test_cpu_usage_calculation() {
    let t1 = CpuRawTimes { user: 100, nice: 0, system: 50, idle: 800, iowait: 50, irq: 0, softirq: 0, steal: 0 };
    let t2 = CpuRawTimes { user: 150, nice: 0, system: 80, idle: 820, iowait: 50, irq: 0, softirq: 0, steal: 0 };
    // total diff = 1100 - 1000 = 100. idle diff = (820+50) - (800+50) = 20. busy = 80%.
    let usage = calculate_cpu_usage(&t1, &t2);
    assert!(usage.is_some());
    let pct = usage.unwrap();
    assert!((pct - 80.0).abs() < 0.1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test telemetry_test` in `omen-space/src/omen-space-daemon`
Expected: FAIL (module or functions not found)

- [ ] **Step 3: Implement `sysmon/load.rs`**

Create `omen-space/src/omen-space-daemon/src/sysmon/load.rs`:
- Implement `CpuRawTimes` parser from `/proc/stat`.
- Implement `calculate_cpu_usage`.
- Implement `is_nvidia_gpu_powered()` scanning `/sys/bus/pci/devices/*/vendor` for `0x10de` and checking `power/runtime_status == "active"`.
- Implement `collect_load_snapshot()` which runs `nvidia-smi --query-gpu=temperature.gpu,utilization.gpu --format=csv,noheader,nounits` guarded by a 2-second timeout ONLY if powered.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test telemetry_test`
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add src/omen-space-daemon/src/sysmon/load.rs tests/telemetry_test.rs
git commit -m "feat(telemetry): add proactive CPU load and PM-safe GPU polling"
```

---

### Task 2: Better Auto Core Algorithm & Hysteresis Engine

**Files:**
- Create: `omen-space/src/omen-space-daemon/src/fan/better_auto.rs`
- Test: `omen-space/src/omen-space-daemon/tests/better_auto_test.rs`

**Interfaces:**
- Produces: `pub struct BetterAutoConfig { pub min_rpm: u32, pub power_profile: String }`
- Produces: `pub struct BetterAutoEngine`
- Produces: `pub fn compute_better_auto_level(temp_c: f64, usage_pct: f64, previous_level: usize, acoustic_ceiling: usize) -> usize`
- Produces: `pub fn calculate_target_rpms(level: usize, min_rpm: u32, max_fan1: u32, max_fan2: u32) -> (u32, u32)`

- [ ] **Step 1: Write failing unit tests for Better Auto levels and acoustic caps**

Create `omen-space/src/omen-space-daemon/tests/better_auto_test.rs`:
```rust
use omen_space_daemon::fan::better_auto::*;

#[test]
fn test_better_auto_preemptive_spike() {
    // When temperature is low (50C, level 1) but usage spikes to 75% (level 6),
    // target level should jump preemptively to level 6.
    let level = compute_better_auto_level(50.0, 75.0, 1, 8);
    assert_eq!(level, 6);
}

#[test]
fn test_better_auto_acoustic_ceiling_in_balanced() {
    // In Balanced mode, acoustic ceiling is level 5.
    // Even if temperature is 80C and load is 90%, it should cap at level 5.
    let level = compute_better_auto_level(80.0, 90.0, 1, 5);
    assert_eq!(level, 5);
}

#[test]
fn test_better_auto_rpm_interpolation_with_custom_min() {
    let (fan1, fan2) = calculate_target_rpms(1, 2600, 5800, 6100);
    assert_eq!(fan1, 2600);
    assert_eq!(fan2, 2600);

    let (fan1_max, fan2_max) = calculate_target_rpms(8, 2600, 5800, 6100);
    assert_eq!(fan1_max, 5800);
    assert_eq!(fan2_max, 6100);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test better_auto_test`
Expected: FAIL

- [ ] **Step 3: Implement `fan/better_auto.rs`**

Implement:
- `UP_THRESHOLDS = [45.0, 54.0, 62.0, 68.0, 73.0, 78.0, 83.0]`
- `DOWN_THRESHOLDS = [42.0, 51.0, 59.0, 65.0, 69.5, 74.5, 79.0]`
- `USAGE_THRESHOLDS = [25.0, 35.0, 48.0, 58.0, 66.0, 74.0, 82.0]`
- `temp_level_from_temperature` with hysteresis.
- `level_from_thresholds` for workload.
- Single-step downward limiter (`max(target, prev - 1)`).
- `compute_better_auto_level(temp, usage, prev_level, acoustic_ceiling)`.
- `calculate_target_rpms(level, min_rpm, max1, max2)`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test better_auto_test`
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add src/omen-space-daemon/src/fan/better_auto.rs tests/better_auto_test.rs
git commit -m "feat(fan): implement 8-step Better Auto algorithm with hysteresis and acoustic cap"
```

---

### Task 3: Fan Service Integration & Hardware Safety (Stagger & Watchdog)

**Files:**
- Modify: `omen-space/src/omen-space-daemon/src/fan/mod.rs`
- Modify: `omen-space/src/omen-space-daemon/src/config.rs`
- Test: `omen-space/src/omen-space-daemon/tests/fan_service_test.rs`

**Interfaces:**
- Consumes: `LoadSnapshot` from `sysmon::load`
- Consumes: `compute_better_auto_level` from `fan::better_auto`
- Produces: `FanState.min_fan_rpm: u32` (default 2600)
- Produces: `FanState.acoustic_ceiling: usize` (default 5 for Balanced)
- Produces: `BetterAuto` mode handling in `run_monitor_loop`
- Produces: 10s gap write: `write_fan_targets_staggered(&mut state, rpm1, rpm2)`

- [ ] **Step 1: Write unit tests for fan target writes, acoustic ceiling and stagger logic**

Create test verifying state transitions to `"better_auto"` mode and minimum RPM + acoustic ceiling config serialization.

- [ ] **Step 2: Add `min_fan_rpm` and `acoustic_ceiling` to `config.rs`**

Update `AppConfig`:
```rust
#[serde(default = "default_min_fan_rpm")]
pub min_fan_rpm: u32,

#[serde(default = "default_acoustic_ceiling")]
pub acoustic_ceiling: usize,

fn default_min_fan_rpm() -> u32 { 2600 }
fn default_acoustic_ceiling() -> usize { 5 }
```

- [ ] **Step 3: Integrate Better Auto control loop in `fan/mod.rs`**

- In `run_monitor_loop`, add branch for `"better_auto"`.
- Fetch `collect_load_snapshot()`.
- Evaluate `target_level = compute_better_auto_level(temp, usage, current_level, ceiling)`.
- If raw temp exceeds 88°C, bypass acoustic ceiling for hardware protection.
- Apply cooldown grace: hold floor for 30s after level 7+, 15s after level 5+.
- Enforce 90s watchdog reapply and 80s manual mode re-assert (`pwm1_enable = 1`).
- Write `fan1_target` -> wait 10s -> write `fan2_target`.

- [ ] **Step 4: Run tests to verify compilation and execution**

Run: `cargo test` in `omen-space-daemon`
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add src/omen-space-daemon/src/fan/mod.rs src/omen-space-daemon/src/config.rs
git commit -m "feat(fan): integrate Better Auto loop with configurable acoustic ceiling, 10s EC stagger and 90s watchdog"
```

---

### Task 4: Context-Aware Power Profile Synchronization & Ceiling Resolution

**Files:**
- Modify: `omen-space/src/omen-space-daemon/src/power.rs`
- Modify: `omen-space/src/omen-space-daemon/src/fan/mod.rs`

**Interfaces:**
- Consumes: `current_profile` from `PowerService`
- Consumes: `user_ceiling` from `FanState`
- Produces: Dynamic acoustic ceilings in `BetterAutoEngine`:
  - `power-saver` / `eco` -> min(user_ceiling, 3) (~3100 RPM max)
  - `balanced` -> user_ceiling (default Level 5 / ~3900 RPM max, 2600 RPM base)
  - `performance` -> Level 8 cap (100% max, aggressive ramp)

- [ ] **Step 1: Implement acoustic ceiling resolution**

Add helper in `fan/mod.rs`:
```rust
pub fn get_effective_acoustic_ceiling(profile: &str, user_ceiling: usize) -> usize {
    match profile {
        "power-saver" | "quiet" | "eco" => user_ceiling.min(3),
        "balanced" => user_ceiling.clamp(3, 8),
        "performance" => 8,
        _ => user_ceiling.clamp(3, 8),
    }
}
```

- [ ] **Step 2: Connect Power Profile changes to Better Auto loop**

When user or Game Automation switches power profile, reset hysteresis and apply the new acoustic ceiling immediately.

- [ ] **Step 3: Commit changes**

```bash
git add src/omen-space-daemon/src/power.rs src/omen-space-daemon/src/fan/mod.rs
git commit -m "feat(power): synchronize Better Auto acoustic ceilings with user preferences and power profiles"
```

---

### Task 5: D-Bus API Contracts & CLI Tooling

**Files:**
- Modify: `omen-space/src/omen-space-daemon/src/fan/mod.rs`
- Modify: `omen-space/src/omen-cli/src/commands/fan.rs`
- Modify: `omen-space/src/omen-cli/src/dbus_proxy.rs`

**Interfaces:**
- D-Bus `org.hp.omen.Fan`:
  - `SetFanMode(mode: "better_auto") -> String`
  - `GetMinFanRpm() -> u32`
  - `SetMinFanRpm(rpm: u32) -> bool`
  - `GetAcousticCeiling() -> u32`
  - `SetAcousticCeiling(level: u32) -> bool`
- CLI commands:
  - `victus-max fan mode better-auto`
  - `victus-max fan set-min-rpm 2600`
  - `victus-max fan set-ceiling 5`
  - `victus-max fan status`

- [ ] **Step 1: Expose `GetMinFanRpm` and `SetMinFanRpm` in D-Bus `FanService`**

- [ ] **Step 2: Update CLI fan command options**

Add `"better-auto"` to mode choices and add `--min-rpm` argument.

- [ ] **Step 3: Test CLI against mocked D-Bus responses**

Run: `cargo check --workspace`
Expected: PASS

- [ ] **Step 4: Commit changes**

```bash
git add src/omen-space-daemon/src/fan/mod.rs src/omen-cli/src/commands/fan.rs
git commit -m "feat(dbus,cli): add better_auto mode and min_rpm D-Bus methods and CLI commands"
```

---

### Task 6: GTK4 / Libadwaita GUI & Settings Integration

**Files:**
- Modify: `omen-space/src/omen-gui/src/performance_control.rs`
- Modify: `omen-space/src/omen-gui/src/settings.rs`
- Modify: `omen-space/src/omen-gui/src/daemon_client.rs`
- Modify: `omen-space/src/omen-gui/src/i18n.rs`
- Create: `omen-space/src/omen-gui/assets/better_auto.svg`

**Interfaces:**
- Produces: `Better Auto` chip card in FAN MODES section of `performance_control.rs`.
- Produces: Minimum Fan RPM SpinRow (2000 - 3500 RPM, step 100) in settings.
- Produces: Acoustic Ceiling Dropdown/SpinRow (Level 3-8 / ~3100-5800 RPM) in settings.
- Produces: Live sync in GUI for `better_auto` mode.

- [ ] **Step 1: Add i18n keys for Better Auto, Min RPM and Acoustic Ceiling**

In `i18n.rs`:
- `"fan_better_auto": "Better Auto"`
- `"fan_better_auto_sub": "Proactive workload cooling"`
- `"setting_min_fan_rpm": "Minimum Fan Speed (RPM)"`
- `"setting_min_fan_rpm_desc": "Baseline fan RPM for Balanced mode (Default: 2600)"`
- `"setting_acoustic_ceiling": "Acoustic Ceiling (Balanced Mode)"`
- `"setting_acoustic_ceiling_desc": "Maximum fan level allowed before emergency thermal protection (Default: Level 5 / ~3900 RPM)"`

- [ ] **Step 2: Add Better Auto card in `performance_control.rs`**

Insert `build_fan_chip_card` for Better Auto with icon `better_auto.svg`.
Wire toggle event to `daemon_client::set_fan_mode_sync("better_auto".to_string())`.
Update sync timer to reflect `better_auto` active state.

- [ ] **Step 3: Add Min RPM and Acoustic Ceiling controls in `settings.rs`**

Add an `adw::SpinRow` or `gtk::Scale` for Min RPM (2000-3500 RPM).
Add an `adw::ComboRow` or `adw::SpinRow` for Acoustic Ceiling (Level 3 - Level 8).
Connect value change to `daemon_client::set_min_fan_rpm(val)` and `daemon_client::set_acoustic_ceiling(val)`.

- [ ] **Step 4: Run cargo build on GUI crate**

Run: `cargo check -p omen-gui`
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add src/omen-gui/src/performance_control.rs src/omen-gui/src/settings.rs src/omen-gui/src/daemon_client.rs
git commit -m "feat(gui): add Better Auto card, configurable Minimum Fan RPM and Acoustic Ceiling controls"
```

---

### Task 7: Workspace Rebranding & Comprehensive End-to-End Verification

**Files:**
- Modify: `omen-space/data/org.hp.OmenSpace.desktop` -> create Victus Max desktop entry
- Modify: `omen-space/data/omen-space-daemon.service` -> `victus-max-daemon.service`
- Build & Verification scripts

- [ ] **Step 1: Run full workspace test suite**

Run: `cargo test --workspace`
Expected: All tests pass.

- [ ] **Step 2: Live Daemon & Sysfs Verification**

- Start daemon in test mode.
- Set mode to `better_auto`.
- Verify `/sys/devices/platform/hp-wmi/hwmon/hwmon10/pwm1_enable` is `1`.
- Verify `fan1_target` is set to 2600 RPM.
- Verify Fan 2 write occurs after the 10-second stagger window.

- [ ] **Step 3: Commit & Final Tag**

```bash
git add .
git commit -m "feat(victus-max): finalize Victus Max integration with proactive Better Auto"
```
