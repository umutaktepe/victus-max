//! Better Auto fan curve and proactive thermal control engine.
//!
//! Provides an 8-level fan curve responding to both temperature and workload usage,
//! with upward/downward hysteresis, single-step ramp down limiting, context-aware
//! acoustic ceiling capping, and emergency thermal bypass at >= 88°C.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Upward temperature thresholds in °C for transitions between steps.
pub const UP_THRESHOLDS: [f64; 7] = [45.0, 54.0, 62.0, 68.0, 73.0, 78.0, 83.0];

/// Downward temperature thresholds in °C providing hysteresis buffer to prevent hunting.
pub const DOWN_THRESHOLDS: [f64; 7] = [42.0, 51.0, 59.0, 65.0, 69.5, 74.5, 79.0];

/// Workload usage thresholds (CPU / GPU busy %) for proactive fan ramp up.
pub const USAGE_THRESHOLDS: [f64; 7] = [25.0, 35.0, 48.0, 58.0, 66.0, 74.0, 82.0];

/// Default minimum fan RPM in Balanced mode.
pub const DEFAULT_MIN_RPM: u32 = 2600;

/// Emergency thermal bypass temperature in °C. Above this temperature, acoustic
/// ceilings are ignored and fan speed immediately jumps to maximum level 8.
pub const EMERGENCY_TEMP_C: f64 = 88.0;

/// Configuration for Better Auto mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BetterAutoConfig {
    pub min_rpm: u32,
    pub power_profile: String,
}

impl Default for BetterAutoConfig {
    fn default() -> Self {
        Self {
            min_rpm: DEFAULT_MIN_RPM,
            power_profile: "Balanced".to_string(),
        }
    }
}

/// Evaluates a metric against an ascending 7-element threshold array.
/// Returns the number of thresholds met (0 through 7).
pub fn level_from_thresholds(value: f64, thresholds: &[f64; 7]) -> usize {
    let mut level = 0;
    for &th in thresholds {
        if value >= th {
            level += 1;
        } else {
            break;
        }
    }
    level
}

/// Determines the temperature level taking hysteresis into account.
///
/// If temperature is rising or equal compared to `previous_level`, evaluates
/// against `UP_THRESHOLDS`. If falling, steps down through `DOWN_THRESHOLDS`
/// to prevent acoustic hunting and flutter.
pub fn temp_level_from_temperature(temp: f64, previous_level: usize) -> usize {
    let up_level = level_from_thresholds(temp, &UP_THRESHOLDS);
    if up_level >= previous_level {
        up_level
    } else {
        let down_level = level_from_thresholds(temp, &DOWN_THRESHOLDS);
        std::cmp::min(previous_level, down_level)
    }
}

/// Computes the effective Better Auto fan level (1..=8) given temperature,
/// workload usage, previous level, and acoustic ceiling.
///
/// 1. Computes `temp_level` from `temp_level_from_temperature`.
/// 2. Computes `usage_level` from `level_from_thresholds`.
/// 3. Preemptive target: `std::cmp::max(temp_level, usage_level)`.
/// 4. Ramp-down limiter: prevents dropping more than 1 step per cycle (`std::cmp::max(target, previous_level - 1)`).
/// 5. Acoustic ceiling & Emergency bypass:
///    - If `temp_c >= EMERGENCY_TEMP_C` (88.0°C), jumps to 8 despite ceiling.
///    - Otherwise, capped at `acoustic_ceiling.clamp(1, 8)`.
/// 6. Clamps final level to `1..=8`.
pub fn compute_better_auto_level(
    temp_c: f64,
    usage_pct: f64,
    previous_level: usize,
    acoustic_ceiling: usize,
) -> usize {
    let temp_level = temp_level_from_temperature(temp_c, previous_level);
    let usage_level = level_from_thresholds(usage_pct, &USAGE_THRESHOLDS);

    let mut target = std::cmp::max(temp_level, usage_level);

    // Ramp-down limiter: single-step ramp down limiting to protect acoustics
    if target < previous_level {
        target = std::cmp::max(target, previous_level.saturating_sub(1));
    }

    // Acoustic ceiling & Emergency thermal bypass
    if temp_c >= EMERGENCY_TEMP_C {
        target = 8;
    } else {
        let ceiling = acoustic_ceiling.clamp(1, 8);
        target = std::cmp::min(target, ceiling);
    }

    target.clamp(1, 8)
}

/// Calculates target RPMs for both fans given the 1..=8 level and speed bounds.
///
/// Interpolates between `min_rpm` and each fan's `max_rpm` across 7 step intervals:
/// - Level 1 = `min_rpm`
/// - Level 8 = `max_rpm`
///
/// If `max_rpm <= min_rpm`, safely returns `max_rpm`.
pub fn calculate_target_rpms(
    level: usize,
    min_rpm: u32,
    max_fan1: u32,
    max_fan2: u32,
) -> (u32, u32) {
    let calc = |max_rpm: u32| -> u32 {
        if max_rpm <= min_rpm {
            max_rpm
        } else {
            let clamped_level = level.clamp(1, 8);
            let step = (max_rpm - min_rpm) as f64 / 7.0;
            let rpm = min_rpm as f64 + (clamped_level - 1) as f64 * step;
            rpm.round() as u32
        }
    };

    (calc(max_fan1), calc(max_fan2))
}

/// Stateful engine managing Better Auto level updates and target RPM calculations.
#[derive(Debug, Clone)]
pub struct BetterAutoEngine {
    pub current_level: usize,
    pub config: BetterAutoConfig,
}

impl Default for BetterAutoEngine {
    fn default() -> Self {
        Self {
            current_level: 1,
            config: BetterAutoConfig::default(),
        }
    }
}

impl BetterAutoEngine {
    pub fn new(config: BetterAutoConfig) -> Self {
        Self {
            current_level: 1,
            config,
        }
    }

    pub fn update(&mut self, temp_c: f64, usage_pct: f64, acoustic_ceiling: usize) -> usize {
        let level = compute_better_auto_level(temp_c, usage_pct, self.current_level, acoustic_ceiling);
        self.current_level = level;
        level
    }

    pub fn calculate_rpms(&self, max_fan1: u32, max_fan2: u32) -> (u32, u32) {
        calculate_target_rpms(self.current_level, self.config.min_rpm, max_fan1, max_fan2)
    }
}
