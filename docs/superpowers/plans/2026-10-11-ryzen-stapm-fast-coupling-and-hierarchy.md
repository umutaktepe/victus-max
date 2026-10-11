# AMD Ryzen STAPM/Fast PPT SMU Coupling & Power Hierarchy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Detect AMD Ryzen SMU generation (`Zen 4/5` coupled `STAPM == Fast PPT` vs `Zen 2/3/3+` 3-stage `STAPM <= Slow PPT <= Fast PPT`) and dynamically adapt both the daemon power controller and the GTK4 Advanced Power UI to enforce hardware-accurate slider coupling, range alignment, and relational clamping.

**Architecture:** Extend `cpu_tier::classify_cpu` (and `CpuPowerProfileRanges` / `GetPowerLimitsInfo`) with a deterministic `stapm_fast_linked: bool` capability flag derived from AMD Ryzen architecture decoding (`Zen 4` Phoenix/Hawk Point/Dragon Range `7x4x`/`8x4x` and `Zen 5` Strix Point `AI 300` vs older `Zen 2/3/3+` `4000/5000/6000/7x3x`). In `victus-max-daemon`, normalize limits according to the CPU's SMU rule (`stapm == fast >= slow` on Zen 4/5; `stapm <= slow <= fast` on Zen 2/3/3+). In `victus-max-gui`, dynamically couple the `STAPM` and `PPT Fast` sliders in lockstep with explanatory Zen 4/5 SMU labels when `stapm_fast_linked` is `true`, and enforce live `STAPM <= PPT Slow <= PPT Fast` relational slider clamping on all Ryzen CPUs.

**Tech Stack:** Rust, Tokio, zbus (D-Bus IPC), GTK4 / Libadwaita, RyzenAdj SMU CLI.

**Spec:** `docs/victus-max-wiki/decisions/adr-007-adaptive-ryzen-power-limits.md`

## Global Constraints

- Follow `AGENTS.md` Karpathy LLM Wiki rules: no numbered filenames, use `[[wikilink]]` syntax in `docs/victus-max-wiki/`, and append a chronological entry to `docs/victus-max-wiki/log.md`.
- Maintain 100% backward compatibility with Intel RAPL (`PL1 <= PL2`) and existing D-Bus method signatures on `org.hp.omen.Power`.
- Prevent infinite signal recursion in GTK4 `connect_value_changed` handlers by guarding programmatic slider updates with a shared `Cell<bool>` re-entrancy lock.
- On `stapm_fast_linked == true` CPUs (e.g. `AMD Ryzen 7 7840HS`), `STAPM` slider maximum range must match `burst_max` (not `sustained_max`) so `STAPM` can always mirror `PPT Fast` up to the full turbo ceiling without clamping.

---

## File Map

- **Modify:** `src/victus-max-daemon/src/cpu_tier.rs`
  - Add `stapm_fast_linked: bool` to `CpuPowerProfileRanges` (or return it alongside `classify_cpu` and add a helper `is_zen4_or_newer_ryzen(cpu_name: &str) -> bool` + `normalize_ryzen_limits(stapm: u32, slow: u32, fast: u32, stapm_fast_linked: bool) -> (u32, u32, u32)`).
- **Modify:** `src/victus-max-daemon/tests/cpu_tier_test.rs`
  - Add unit tests verifying `stapm_fast_linked` detection across `Ryzen 7 7840HS` (Zen 4 -> `true`), `Ryzen 9 8945HS` (Zen 4 -> `true`), `Ryzen AI 9 HX 370` (Zen 5 -> `true`), `Ryzen 7 7735HS` (Zen 3+ -> `false`), `Ryzen 7 5800H` (Zen 3 -> `false`), and testing `normalize_ryzen_limits`.
- **Modify:** `src/victus-max-daemon/src/power.rs`
  - Include `"stapm_fast_linked": ranges.stapm_fast_linked` in `GetPowerProfile()` and `GetPowerLimitsInfo()` JSON payloads.
  - Use `crate::cpu_tier::normalize_ryzen_limits` inside `set_ryzen_power_limits` and `apply_rapl_limits` so the daemon enforces `STAPM == Fast >= Slow` on Zen 4/5 and `STAPM <= Slow <= Fast` on Zen 2/3/3+.
- **Modify:** `src/victus-max-gui/src/daemon_client.rs`
  - Add `#[serde(default)] pub stapm_fast_linked: bool` to `CpuPowerProfileRanges` and `PowerProfileInfo`.
- **Modify:** `src/victus-max-gui/src/i18n.rs`
  - Add Turkish and English localization strings for Zen 4/5 coupled STAPM (`stapm_linked_sub`, `ryzen_power_limits_desc_linked`) and classic 3-stage Ryzen hierarchy (`ryzen_power_limits_desc_hierarchy`).
- **Modify:** `src/victus-max-gui/src/undervolt.rs`
  - Add re-entrancy-safe relational slider synchronization in `build_page()`:
    - When `is_amd && stapm_fast_linked`: moving `PPT Fast` updates `STAPM` to the exact same value (and vice versa), clamps `PPT Slow <= PPT Fast`, sets `STAPM` slider range to `sustained_min..=burst_max`, and updates the `STAPM` subtitle to explain the Zen 4/5 SMU hardware coupling.
    - When `is_amd && !stapm_fast_linked`: moving `STAPM`, `PPT Slow`, or `PPT Fast` enforces `STAPM <= PPT Slow <= PPT Fast` in real time.
    - When `!is_amd` (Intel): moving `PL1` or `PL2` enforces `PL1 <= PL2` in real time.
- **Modify:** `docs/victus-max-wiki/decisions/adr-007-adaptive-ryzen-power-limits.md`, `docs/victus-max-wiki/interfaces/power-dbus-interface.md`, `docs/victus-max-wiki/log.md`
  - Document the SMU generation distinction (`Zen 4/5` coupled `STAPM = Fast PPT` vs `Zen 2/3/3+` `STAPM <= Slow PPT <= Fast PPT`).

---

### Task 1: AMD SMU Generation Detection (`stapm_fast_linked`) & Limit Normalization in `cpu_tier.rs`

**Files:**
- Modify: `src/victus-max-daemon/src/cpu_tier.rs:21-212`
- Test: `src/victus-max-daemon/tests/cpu_tier_test.rs`

**Interfaces:**
- Produces:
  - `CpuPowerProfileRanges::stapm_fast_linked: bool`
  - `pub fn is_zen4_or_newer_ryzen(cpu_name: &str) -> bool`
  - `pub fn normalize_ryzen_limits(stapm: u32, slow: u32, fast: u32, stapm_fast_linked: bool) -> (u32, u32, u32)`

- [ ] **Step 1: Write the failing unit tests in `src/victus-max-daemon/tests/cpu_tier_test.rs`**

```rust
#[test]
fn test_zen4_and_zen5_stapm_fast_linked_detection() {
    // Zen 4 Phoenix / Hawk Point / Dragon Range -> stapm_fast_linked = true
    let (_, _, r_7840hs) = classify_cpu("AMD Ryzen 7 7840HS w/ Radeon 780M Graphics");
    assert!(r_7840hs.stapm_fast_linked);

    let (_, _, r_8845hs) = classify_cpu("AMD Ryzen 7 8845HS w/ Radeon 780M Graphics");
    assert!(r_8845hs.stapm_fast_linked);

    let (_, _, r_7945hx) = classify_cpu("AMD Ryzen 9 7945HX");
    assert!(r_7945hx.stapm_fast_linked);

    // Zen 5 Strix Point -> stapm_fast_linked = true
    let (_, _, r_ai370) = classify_cpu("AMD Ryzen AI 9 HX 370");
    assert!(r_ai370.stapm_fast_linked);

    // Zen 3+ Rembrandt (7735HS / 6800H) and Zen 3 Cezanne (5800H) -> stapm_fast_linked = false
    let (_, _, r_7735hs) = classify_cpu("AMD Ryzen 7 7735HS with Radeon Graphics");
    assert!(!r_7735hs.stapm_fast_linked);

    let (_, _, r_6800h) = classify_cpu("AMD Ryzen 7 6800H with Radeon Graphics");
    assert!(!r_6800h.stapm_fast_linked);

    let (_, _, r_5800h) = classify_cpu("AMD Ryzen 7 5800H with Radeon Graphics");
    assert!(!r_5800h.stapm_fast_linked);
}

#[test]
fn test_normalize_ryzen_limits_coupled_vs_hierarchy() {
    use victus_max_daemon::cpu_tier::normalize_ryzen_limits;

    // On Zen 4/5 (stapm_fast_linked = true):
    // Fast >= Slow, and STAPM is always equal to Fast
    let (stapm, slow, fast) = normalize_ryzen_limits(45, 43, 48, true);
    assert_eq!((stapm, slow, fast), (48, 43, 48));

    // If slow > fast on Zen 4/5, fast (and stapm) are raised to slow
    let (stapm2, slow2, fast2) = normalize_ryzen_limits(40, 52, 48, true);
    assert_eq!((stapm2, slow2, fast2), (52, 52, 52));

    // On Zen 2/3/3+ (stapm_fast_linked = false):
    // Enforces STAPM <= Slow <= Fast while keeping distinct values when valid
    let (stapm3, slow3, fast3) = normalize_ryzen_limits(35, 43, 48, false);
    assert_eq!((stapm3, slow3, fast3), (35, 43, 48));

    // If STAPM > Slow, Slow is raised to STAPM; if Slow > Fast, Fast is raised to Slow
    let (stapm4, slow4, fast4) = normalize_ryzen_limits(50, 43, 48, false);
    assert_eq!((stapm4, slow4, fast4), (50, 50, 50));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cpu_tier_test`
Expected: FAIL (`no field stapm_fast_linked on type CpuPowerProfileRanges` / `unresolved import normalize_ryzen_limits`)

- [ ] **Step 3: Implement `stapm_fast_linked`, `is_zen4_or_newer_ryzen`, and `normalize_ryzen_limits` in `src/victus-max-daemon/src/cpu_tier.rs`**

```rust
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
    #[serde(default)]
    pub stapm_fast_linked: bool,
}

/// Detects whether an AMD Ryzen CPU uses Zen 4 / Zen 5 SMU firmware where
/// `--fast-limit` automatically locks the STAPM ceiling to Fast PPT (`STAPM == Fast PPT`).
///
/// Matches:
/// - "RYZEN AI" (Zen 5 Strix Point / Strix Halo / Krackan)
/// - 4-digit Ryzen 7000/8000/9000 mobile model numbers whose 3rd digit (architecture)
///   is '4', '5', or '6' (e.g. 7840HS, 7640HS, 7945HX, 8845HS, 8945HS).
/// Returns `false` for Zen 2 / Zen 3 / Zen 3+ (e.g. 4800H, 5800H, 6800H, 7735HS, 7530U).
pub fn is_zen4_or_newer_ryzen(cpu_name: &str) -> bool {
    let upper = cpu_name.to_uppercase();
    if upper.contains("RYZEN AI") {
        return true;
    }

    for tok in upper.split_whitespace() {
        let digits: Vec<char> = tok.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.len() == 4 {
            let gen_digit = digits[0];
            let arch_digit = digits[2];
            if ('7'..='9').contains(&gen_digit) && ('4'..='9').contains(&arch_digit) {
                return true;
            }
        }
    }

    false
}

/// Enforces AMD's mobile power limit invariants:
/// - If `stapm_fast_linked` is `true` (Zen 4/5): `slow <= fast` and `stapm == fast`.
/// - If `stapm_fast_linked` is `false` (Zen 2/3/3+): `stapm <= slow <= fast`.
pub fn normalize_ryzen_limits(
    stapm: u32,
    slow: u32,
    fast: u32,
    stapm_fast_linked: bool,
) -> (u32, u32, u32) {
    if stapm_fast_linked {
        let norm_fast = fast.max(slow);
        (norm_fast, slow, norm_fast)
    } else {
        let norm_slow = slow.max(stapm);
        let norm_fast = fast.max(norm_slow);
        (stapm, norm_slow, norm_fast)
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cpu_tier_test`
Expected: PASS (7 passed; 0 failed)

---

### Task 2: Enforce SMU Rules & Expose `stapm_fast_linked` in `power.rs` and `victus-max-gui/src/daemon_client.rs`

**Files:**
- Modify: `src/victus-max-daemon/src/power.rs:530-670, 730-760`
- Modify: `src/victus-max-gui/src/daemon_client.rs:468-515`

**Interfaces:**
- Consumes: `CpuPowerProfileRanges::stapm_fast_linked`, `normalize_ryzen_limits`
- Produces: `"stapm_fast_linked"` field in `GetPowerProfile()` and `GetPowerLimitsInfo()` JSON; `PowerProfileInfo::stapm_fast_linked` in GUI client.

- [ ] **Step 1: Update `power.rs` to normalize limits via `normalize_ryzen_limits` and include `stapm_fast_linked` in JSON responses**

```rust
// Inside apply_rapl_limits(pl1: u32, pl2: u32, slow_opt: Option<u32>):
        let specs = crate::sysmon::get_hardware_specs();
        let (vendor, _, ranges) = crate::cpu_tier::classify_cpu(&specs.cpu_spec);
        let is_amd = vendor == crate::cpu_tier::CpuVendor::Amd;

        if is_amd {
            let raw_slow = slow_opt.unwrap_or(pl1);
            let (stapm_w, slow_w, fast_w) = crate::cpu_tier::normalize_ryzen_limits(
                pl1,
                raw_slow,
                pl2,
                ranges.stapm_fast_linked,
            );
            let arg_stapm = format!("--stapm-limit={}", stapm_w * 1000);
            let arg_slow = format!("--slow-limit={}", slow_w * 1000);
            let arg_fast = format!("--fast-limit={}", fast_w * 1000);
            let _ = crate::cpu_tier::run_ryzenadj(&[&arg_stapm, &arg_slow, &arg_fast]).await;
            info!(
                "AMD RyzenAdj limits set (linked={}): STAPM={}W Slow={}W Fast={}W",
                ranges.stapm_fast_linked, stapm_w, slow_w, fast_w
            );
        }
```

And in `set_ryzen_power_limits`:
```rust
    async fn set_ryzen_power_limits(&self, enabled: bool, stapm: i32, slow: i32, fast: i32) -> String {
        if !(1..=200).contains(&stapm) || !(1..=200).contains(&slow) || !(1..=250).contains(&fast) {
            warn!("SetRyzenPowerLimits: out of safe range stapm={} slow={} fast={}", stapm, slow, fast);
            return "FAIL".to_string();
        }
        let specs = crate::sysmon::get_hardware_specs();
        let (_, _, ranges) = crate::cpu_tier::classify_cpu(&specs.cpu_spec);
        let (norm_stapm, norm_slow, norm_fast) = crate::cpu_tier::normalize_ryzen_limits(
            stapm as u32,
            slow as u32,
            fast as u32,
            ranges.stapm_fast_linked,
        );

        {
            let mut cfg = self.config.lock().await;
            cfg.pl_enabled = enabled;
            cfg.pl1_w = norm_stapm;
            cfg.slow_ppt_w = norm_slow;
            cfg.pl2_w = norm_fast;
            cfg.save();
        }

        if enabled {
            Self::apply_rapl_limits(norm_stapm, norm_fast, Some(norm_slow)).await;
        }

        info!(
            "SetRyzenPowerLimits: enabled={}, STAPM={}W, Slow={}W, Fast={}W (linked={})",
            enabled, norm_stapm, norm_slow, norm_fast, ranges.stapm_fast_linked
        );
        "OK".to_string()
    }
```

And include `"stapm_fast_linked": ranges.stapm_fast_linked` in both `get_power_profile` and `get_power_limits_info`.

- [ ] **Step 2: Add `stapm_fast_linked: bool` to `CpuPowerProfileRanges` and `PowerProfileInfo` in `src/victus-max-gui/src/daemon_client.rs`**

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
    #[serde(default)]
    pub stapm_fast_linked: bool,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PowerProfileInfo {
    #[serde(default)]
    pub is_amd: bool,
    #[serde(default)]
    pub stapm_fast_linked: bool,
    #[serde(default)]
    pub cpu_tier: String,
    // ...
```

- [ ] **Step 3: Verify daemon & GUI types compile and tests pass**

Run: `cargo test --workspace`
Expected: PASS

---

### Task 3: Adaptive GTK4 UI Coupling (`STAPM == Fast PPT` on Zen 4/5 & `STAPM <= Slow <= Fast` Hierarchy)

**Files:**
- Modify: `src/victus-max-gui/src/i18n.rs`
- Modify: `src/victus-max-gui/src/undervolt.rs`

**Interfaces:**
- Consumes: `PowerProfileInfo::stapm_fast_linked`, `CpuPowerProfileRanges::stapm_fast_linked`
- Produces: Synchronized GTK4 sliders in `Advanced Power` with i18n explanations for Zen 4/5 SMU coupling vs Zen 2/3/3+ 3-stage hierarchy.

- [ ] **Step 1: Add localization keys to `src/victus-max-gui/src/i18n.rs` and unit test them**

Add to English & Turkish dictionaries in `src/victus-max-gui/src/i18n.rs`:
- `"stapm_linked_label"`:
  - EN: `"STAPM — Sustained Chassis Power Limit (Linked to PPT Fast)"`
  - TR: `"STAPM — Sürekli Kasa Güç Limiti (PPT Fast ile Bağlı)"`
- `"stapm_linked_sub"`:
  - EN: `"Zen 4/5 SMU firmware automatically locks STAPM ceiling to PPT Fast (STAPM = PPT Fast ≤ Max)"`
  - TR: `"Zen 4/5 SMU yazılımlarında STAPM tavanı otomatik olarak PPT Fast ile eşitlenir (STAPM = PPT Fast)"`
- `"ryzen_power_limits_desc_linked"`:
  - EN: `"AMD Ryzen SMU Power Limits (Zen 4/5: PPT Slow ≤ PPT Fast, STAPM = PPT Fast)"`
  - TR: `"AMD Ryzen SMU Güç Limitleri (Zen 4/5: PPT Slow ≤ PPT Fast, STAPM = PPT Fast)"`
- `"ryzen_power_limits_desc_hierarchy"`:
  - EN: `"AMD Ryzen SMU Power Limits (Rule: STAPM ≤ PPT Slow ≤ PPT Fast)"`
  - TR: `"AMD Ryzen SMU Güç Limitleri (Kural: STAPM ≤ PPT Slow ≤ PPT Fast)"`

- [ ] **Step 2: Implement relational slider coupling in `src/victus-max-gui/src/undervolt.rs`**

In `src/victus-max-gui/src/undervolt.rs`:
1. Track `stapm_fast_linked_flag = Rc::new(RefCell::new(false))` and `syncing_sliders = Rc::new(Cell::new(false))`.
2. In `pl1_scale.connect_value_changed` (`STAPM` / `PL1`):
   - Always update `pl1_lbl`.
   - If `!syncing_sliders.get()`:
     - Set `syncing_sliders.set(true)`.
     - If `*is_amd && *stapm_fast_linked`:
       - Set `pl2_scale` (`PPT Fast`) to `val` and clamp `slow_scale` (`PPT Slow`) so `slow <= val`.
     - Else if `*is_amd` (Zen 2/3/3+):
       - If `slow_scale.value() < val`, set `slow_scale.set_value(val)`.
       - If `pl2_scale.value() < slow_scale.value()`, set `pl2_scale.set_value(slow_scale.value())`.
     - Else (Intel):
       - If `pl2_scale.value() < val`, set `pl2_scale.set_value(val)`.
     - Set `syncing_sliders.set(false)`.
3. In `slow_scale.connect_value_changed` (`PPT Slow`):
   - Always update `slow_lbl`.
   - If `!syncing_sliders.get()`:
     - Set `syncing_sliders.set(true)`.
     - If `*is_amd && *stapm_fast_linked`:
       - If `pl2_scale.value() < val`, set `pl2_scale.set_value(val)` and `pl1_scale.set_value(val)`.
     - Else if `*is_amd`:
       - If `pl1_scale.value() > val`, set `pl1_scale.set_value(val)`.
       - If `pl2_scale.value() < val`, set `pl2_scale.set_value(val)`.
     - Set `syncing_sliders.set(false)`.
4. In `pl2_scale.connect_value_changed` (`PPT Fast` / `PL2`):
   - Always update `pl2_lbl`.
   - If `!syncing_sliders.get()`:
     - Set `syncing_sliders.set(true)`.
     - If `*is_amd && *stapm_fast_linked`:
       - Set `pl1_scale.set_value(val)` (keep `STAPM == PPT Fast` in lockstep).
       - If `slow_scale.value() > val`, set `slow_scale.set_value(val)`.
     - Else if `*is_amd`:
       - If `slow_scale.value() > val`, set `slow_scale.set_value(val)`.
       - If `pl1_scale.value() > slow_scale.value()`, set `pl1_scale.set_value(slow_scale.value())`.
     - Else (Intel):
       - If `pl1_scale.value() > val`, set `pl1_scale.set_value(val)`.
     - Set `syncing_sliders.set(false)`.
5. In `refresh_state` (when loading from `get_power_profile_info_async()`):
   - Set `*stapm_fast_linked_load.borrow_mut() = info.stapm_fast_linked || ranges.stapm_fast_linked;`.
   - If `is_amd && linked`:
     - Set `pl1_s_load.set_range(ranges.sustained_min as f64, ranges.burst_max as f64)` so `STAPM` can reach the exact same ceiling (`95W` on HS / `125W` on HX) as `PPT Fast` without getting clamped at `75W`!
     - Set `pwr_group_clone.set_description(Some(i18n::t("ryzen_power_limits_desc_linked")))`.
     - Set `pl1_row_clone.set_title(i18n::t("stapm_linked_label"))` and `pl1_row_clone.set_subtitle(i18n::t("stapm_linked_sub"))`.
     - In `defaults_load`, set default `(ranges.burst_default, ranges.sustained_default, ranges.burst_default, ranges.temp_default)` so clicking "Reset Defaults" also respects `STAPM == Fast PPT`.
   - If `is_amd && !linked`:
     - Set `pl1_s_load.set_range(ranges.sustained_min as f64, ranges.sustained_max as f64)`.
     - Set `pwr_group_clone.set_description(Some(i18n::t("ryzen_power_limits_desc_hierarchy")))`.
   - Guard initial `set_value` calls with `syncing_sliders.set(true)` / `set(false)` so loading live values (`48 / 43 / 48 / 88`) sets each slider cleanly without intermediate clamping artifacts.

- [ ] **Step 3: Run full test suite**

Run: `cargo test --workspace`
Expected: PASS with 0 warnings

---

### Task 4: Update Living Wiki (`ADR-007`, `power-dbus-interface.md`, `log.md`)

**Files:**
- Modify: `docs/victus-max-wiki/decisions/adr-007-adaptive-ryzen-power-limits.md`
- Modify: `docs/victus-max-wiki/interfaces/power-dbus-interface.md`
- Modify: `docs/victus-max-wiki/log.md`

- [ ] **Step 1: Update `adr-007-adaptive-ryzen-power-limits.md`, `power-dbus-interface.md`, and `log.md`**

Document the `stapm_fast_linked` capability flag, Zen 4/5 SMU `STAPM == Fast PPT` coupling vs Zen 2/3/3+ `STAPM <= Slow PPT <= Fast PPT` hierarchy enforcement, and log the change in `docs/victus-max-wiki/log.md`.
