use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FanPreset {
    pub name: String,
    pub points: Vec<(f64, f64)>,
}

fn get_presets_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        let p_victus = PathBuf::from(&home).join(".config/victus-max/fan_presets.json");
        if p_victus.exists() {
            return p_victus;
        }
        let p_omen = PathBuf::from(&home).join(".config/omenspace/fan_presets.json");
        if p_omen.exists() {
            return p_omen;
        }
        p_victus
    } else {
        PathBuf::from("/tmp/fan_presets.json")
    }
}

pub fn load_presets() -> Vec<FanPreset> {
    let path = get_presets_path();
    if let Ok(content) = fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
    } else {
        let default_quiet = FanPreset {
            name: "Quiet".to_string(),
            points: vec![(40.0, 0.0), (55.0, 15.0), (70.0, 35.0), (85.0, 60.0), (100.0, 100.0)],
        };
        let defaults = vec![default_quiet];
        // Don't auto-save here to avoid permission issues if dir doesn't exist,
        // it will be saved if the user adds/removes presets.
        defaults
    }
}

pub fn save_presets(presets: &[FanPreset]) {
    let path = get_presets_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(presets) {
        let _ = fs::write(path, json);
    }
}

pub fn delete_preset(name: &str) {
    let mut presets = load_presets();
    let original_len = presets.len();
    presets.retain(|p| p.name != name);
    if presets.len() < original_len {
        save_presets(&presets);
    }
}

/// Compares two fan curves with an epsilon tolerance.
/// Returns true if both curves have the same point count and all
/// (temp, speed) pairs match within 0.5.
pub fn matches_curve(a: &[(f64, f64)], b: &[(f64, f64)]) -> bool {
    if a.is_empty() || a.len() != b.len() {
        return false;
    }
    for ((x1, y1), (x2, y2)) in a.iter().zip(b.iter()) {
        if (x1 - x2).abs() > 0.5 || (y1 - y2).abs() > 0.5 {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_curve_exact() {
        let pts1 = vec![(40.0, 0.0), (55.0, 15.0), (70.0, 35.0), (85.0, 60.0), (100.0, 100.0)];
        let pts2 = vec![(40.0, 0.0), (55.0, 15.0), (70.0, 35.0), (85.0, 60.0), (100.0, 100.0)];
        assert!(matches_curve(&pts1, &pts2));
    }

    #[test]
    fn test_matches_curve_within_epsilon() {
        let pts1 = vec![(40.0, 0.0), (55.0, 15.0), (70.0, 35.0), (85.0, 60.0), (100.0, 100.0)];
        let pts2 = vec![(40.1, 0.2), (54.9, 15.1), (70.0, 35.2), (85.3, 59.8), (100.0, 100.0)];
        assert!(matches_curve(&pts1, &pts2));
    }

    #[test]
    fn test_matches_curve_different_length() {
        let pts1 = vec![(40.0, 0.0), (55.0, 15.0)];
        let pts2 = vec![(40.0, 0.0), (55.0, 15.0), (70.0, 35.0)];
        assert!(!matches_curve(&pts1, &pts2));
    }

    #[test]
    fn test_matches_curve_empty() {
        let pts1: Vec<(f64, f64)> = vec![];
        let pts2: Vec<(f64, f64)> = vec![];
        assert!(!matches_curve(&pts1, &pts2));
    }

    #[test]
    fn test_matches_curve_mismatch() {
        let pts1 = vec![(40.0, 0.0), (55.0, 15.0), (70.0, 35.0), (85.0, 60.0), (100.0, 100.0)];
        let pts2 = vec![(40.0, 20.0), (55.0, 35.0), (70.0, 60.0), (85.0, 82.0), (100.0, 100.0)];
        assert!(!matches_curve(&pts1, &pts2));
    }

    #[test]
    fn test_quiet_preset_default() {
        let presets = load_presets();
        assert!(!presets.is_empty());
        let quiet = presets.iter().find(|p| p.name == "Quiet");
        assert!(quiet.is_some());
        let q = quiet.unwrap();
        assert_eq!(q.points.len(), 5);
        assert_eq!(q.points[0], (40.0, 0.0));
    }
}
