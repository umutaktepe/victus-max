#[path = "../src/cpu_tier.rs"]
pub mod cpu_tier;

use cpu_tier::{classify_cpu, CpuTier, CpuVendor};

#[test]
fn test_classify_amd_7840hs() {
    let (vendor, tier, ranges) = classify_cpu("AMD Ryzen 7 7840HS w/ Radeon 780M Graphics");
    assert_eq!(vendor, CpuVendor::Amd);
    assert_eq!(tier, CpuTier::AmdStandardHs);
    assert_eq!(ranges.sustained_default, 45);
    assert_eq!(ranges.burst_default, 65);
    assert_eq!(ranges.sustained_min, 25);
    assert_eq!(ranges.sustained_max, 75);
    assert_eq!(ranges.burst_min, 35);
    assert_eq!(ranges.burst_max, 95);
    assert_eq!(ranges.temp_min, 75);
    assert_eq!(ranges.temp_max, 100);
    assert_eq!(ranges.temp_default, 95);
}

#[test]
fn test_classify_amd_u_series() {
    let (vendor, tier, ranges) = classify_cpu("AMD Ryzen 5 5625U with Radeon Graphics");
    assert_eq!(vendor, CpuVendor::Amd);
    assert_eq!(tier, CpuTier::AmdUltraLow);
    assert_eq!(ranges.sustained_default, 15);
    assert_eq!(ranges.sustained_min, 10);
    assert_eq!(ranges.sustained_max, 35);
    assert_eq!(ranges.burst_default, 25);
    assert_eq!(ranges.burst_min, 15);
    assert_eq!(ranges.burst_max, 45);
}

#[test]
fn test_classify_amd_hx_series() {
    let (vendor, tier, ranges) = classify_cpu("AMD Ryzen 9 7945HX with Radeon Graphics");
    assert_eq!(vendor, CpuVendor::Amd);
    assert_eq!(tier, CpuTier::AmdExtremeHx);
    assert_eq!(ranges.sustained_default, 55);
    assert_eq!(ranges.burst_default, 80);
    assert_eq!(ranges.sustained_min, 35);
    assert_eq!(ranges.sustained_max, 100);
}

#[test]
fn test_classify_intel_13700h() {
    let (vendor, tier, ranges) = classify_cpu("13th Gen Intel(R) Core(TM) i7-13700H");
    assert_eq!(vendor, CpuVendor::Intel);
    assert_eq!(tier, CpuTier::IntelStandard);
    assert_eq!(ranges.sustained_default, 45);
    assert_eq!(ranges.burst_default, 95);
}

#[test]
fn test_classify_intel_13900hx() {
    let (vendor, tier, ranges) = classify_cpu("13th Gen Intel(R) Core(TM) i9-13900HX");
    assert_eq!(vendor, CpuVendor::Intel);
    assert_eq!(tier, CpuTier::IntelExtreme);
    assert_eq!(ranges.sustained_default, 55);
    assert_eq!(ranges.burst_default, 115);
}

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
    use cpu_tier::normalize_ryzen_limits;

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

