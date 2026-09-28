#[path = "../src/fan/better_auto.rs"]
pub mod better_auto;

pub mod omen_space_daemon {
    pub mod fan {
        pub use crate::better_auto;
    }
}

use omen_space_daemon::fan::better_auto::*;

#[test]
fn test_better_auto_preemptive_spike() {
    // When temperature is low (50C, level 1) but usage spikes to 75% (level 6),
    // target level should jump preemptively to level 6.
    let level = compute_better_auto_level(50.0, 75.0, 1, 8);
    assert_eq!(level, 6);

    // Extreme cold (35C) with load spike to 75% should still jump to level 6
    let cold_spike = compute_better_auto_level(35.0, 75.0, 1, 8);
    assert_eq!(cold_spike, 6);

    // Idle temperature and low workload stays at baseline level 1
    let idle_level = compute_better_auto_level(40.0, 10.0, 1, 8);
    assert_eq!(idle_level, 1);
}

#[test]
fn test_better_auto_hysteresis_behavior() {
    // Upward thresholds: [45.0, 54.0, 62.0, 68.0, 73.0, 78.0, 83.0]
    // Downward thresholds: [42.0, 51.0, 59.0, 65.0, 69.5, 74.5, 79.0]

    // Verify upward progression from cold
    assert_eq!(temp_level_from_temperature(40.0, 1), 0);
    assert_eq!(temp_level_from_temperature(50.0, 1), 1);
    assert_eq!(temp_level_from_temperature(55.0, 1), 2);
    assert_eq!(temp_level_from_temperature(63.0, 2), 3);
    assert_eq!(temp_level_from_temperature(70.0, 3), 4);
    assert_eq!(temp_level_from_temperature(75.0, 4), 5);
    assert_eq!(temp_level_from_temperature(79.0, 5), 6);
    assert_eq!(temp_level_from_temperature(84.0, 6), 7);

    // Downward hysteresis hold:
    // When falling from level 6 (UP=78.0, DOWN=74.5)
    // 76.0C is below UP threshold (78.0) but above DOWN threshold (74.5) -> must HOLD level 6
    assert_eq!(temp_level_from_temperature(76.0, 6), 6);

    // Once it drops below 74.5C (e.g. 74.0C), it steps down to level 5
    assert_eq!(temp_level_from_temperature(74.0, 6), 5);

    // When falling from level 3 (UP=62.0, DOWN=59.0)
    // 60.0C is below 62.0 but above 59.0 -> must HOLD level 3
    assert_eq!(temp_level_from_temperature(60.0, 3), 3);

    // Once below 59.0C (e.g. 58.5C), steps down to level 2
    assert_eq!(temp_level_from_temperature(58.5, 3), 2);
}

#[test]
fn test_better_auto_single_step_ramp_down() {
    // When machine was under heavy load at level 7 and load/temperature drops suddenly:
    // Instead of jumping immediately from 7 to 1, it must ramp down by at most 1 level per cycle.
    let cycle1 = compute_better_auto_level(40.0, 5.0, 7, 8);
    assert_eq!(cycle1, 6);

    let cycle2 = compute_better_auto_level(40.0, 5.0, 6, 8);
    assert_eq!(cycle2, 5);

    let cycle3 = compute_better_auto_level(40.0, 5.0, 5, 8);
    assert_eq!(cycle3, 4);

    let cycle4 = compute_better_auto_level(40.0, 5.0, 4, 8);
    assert_eq!(cycle4, 3);

    let cycle5 = compute_better_auto_level(40.0, 5.0, 3, 8);
    assert_eq!(cycle5, 2);

    let cycle6 = compute_better_auto_level(40.0, 5.0, 2, 8);
    assert_eq!(cycle6, 1);

    let cycle7 = compute_better_auto_level(40.0, 5.0, 1, 8);
    assert_eq!(cycle7, 1);
}

#[test]
fn test_better_auto_acoustic_ceiling_in_balanced() {
    // In Balanced mode, acoustic ceiling is level 5.
    // Even if temperature is 80C and load is 90%, it should cap at level 5.
    let level = compute_better_auto_level(80.0, 90.0, 1, 5);
    assert_eq!(level, 5);

    // Sustained high load (85C, 100% load) with ceiling 5 stays at 5
    let level2 = compute_better_auto_level(85.0, 100.0, 5, 5);
    assert_eq!(level2, 5);

    // Stricter acoustic ceiling (e.g. level 4) caps at 4
    let level_strict = compute_better_auto_level(82.0, 95.0, 1, 4);
    assert_eq!(level_strict, 4);
}

#[test]
fn test_better_auto_emergency_thermal_bypass() {
    // In Balanced mode with acoustic ceiling 5, if temperature reaches >= 88.0C,
    // the acoustic ceiling MUST be bypassed and fan level must jump to maximum level 8.
    let level = compute_better_auto_level(88.0, 20.0, 1, 5);
    assert_eq!(level, 8);

    // Extreme temperature (92C) with low load and ceiling 3 still jumps to 8
    let level_extreme = compute_better_auto_level(92.0, 10.0, 1, 3);
    assert_eq!(level_extreme, 8);

    // Temperature just below threshold (87.9C) remains constrained by acoustic ceiling 5
    let level_below = compute_better_auto_level(87.9, 90.0, 1, 5);
    assert_eq!(level_below, 5);
}

#[test]
fn test_better_auto_rpm_interpolation_with_custom_min() {
    let (fan1, fan2) = calculate_target_rpms(1, 2600, 5800, 6100);
    assert_eq!(fan1, 2600);
    assert_eq!(fan2, 2600);

    let (fan1_max, fan2_max) = calculate_target_rpms(8, 2600, 5800, 6100);
    assert_eq!(fan1_max, 5800);
    assert_eq!(fan2_max, 6100);

    // Midpoint check (level 4):
    // Fan 1: step = (5800 - 2600) / 7.0 = 457.1428... -> 2600 + 3 * 457.1428... = 3971 RPM
    // Fan 2: step = (6100 - 2600) / 7.0 = 500.0 -> 2600 + 3 * 500 = 4100 RPM
    let (fan1_mid, fan2_mid) = calculate_target_rpms(4, 2600, 5800, 6100);
    assert_eq!(fan1_mid, 3971);
    assert_eq!(fan2_mid, 4100);

    // Custom minimum RPM (2000 RPM)
    let (fan1_custom, fan2_custom) = calculate_target_rpms(1, 2000, 5800, 6100);
    assert_eq!(fan1_custom, 2000);
    assert_eq!(fan2_custom, 2000);

    // Safe handling when max_rpm <= min_rpm
    let (fan1_safe, fan2_safe) = calculate_target_rpms(5, 3000, 2500, 3000);
    assert_eq!(fan1_safe, 2500);
    assert_eq!(fan2_safe, 3000);
}

#[test]
fn test_better_auto_engine_stateful() {
    let mut engine = BetterAutoEngine::default();
    assert_eq!(engine.current_level, 1);
    assert_eq!(engine.config.min_rpm, 2600);

    // Preemptive spike
    let lvl = engine.update(50.0, 75.0, 8);
    assert_eq!(lvl, 6);
    assert_eq!(engine.current_level, 6);

    let (f1, f2) = engine.calculate_rpms(5800, 6100);
    assert!(f1 > 2600 && f1 < 5800);
    assert!(f2 > 2600 && f2 < 6100);
}
