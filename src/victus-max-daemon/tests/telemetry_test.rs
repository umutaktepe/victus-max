#[path = "../src/sysmon/load.rs"]
pub mod load;

pub mod omen_space_daemon {
    pub mod sysmon {
        pub use crate::load;
    }
}

use omen_space_daemon::sysmon::load::{
    calculate_cpu_usage, collect_load_snapshot, is_nvidia_gpu_powered,
    is_nvidia_gpu_powered_at, parse_proc_stat, CpuRawTimes, LoadSnapshot,
};
use std::fs;

#[test]
fn test_cpu_usage_calculation() {
    let t1 = CpuRawTimes {
        user: 100,
        nice: 0,
        system: 50,
        idle: 800,
        iowait: 50,
        irq: 0,
        softirq: 0,
        steal: 0,
    };
    let t2 = CpuRawTimes {
        user: 150,
        nice: 0,
        system: 80,
        idle: 820,
        iowait: 50,
        irq: 0,
        softirq: 0,
        steal: 0,
    };
    // total diff = 1100 - 1000 = 100. idle diff = (820+50) - (800+50) = 20. busy = 80%.
    let usage = calculate_cpu_usage(&t1, &t2);
    assert!(usage.is_some());
    let pct = usage.unwrap();
    assert!((pct - 80.0).abs() < 0.1, "Expected ~80.0%, got {}", pct);
}

#[test]
fn test_cpu_usage_zero_diff() {
    let t1 = CpuRawTimes {
        user: 100,
        nice: 0,
        system: 50,
        idle: 800,
        iowait: 50,
        irq: 0,
        softirq: 0,
        steal: 0,
    };
    let usage = calculate_cpu_usage(&t1, &t1);
    assert!(usage.is_none(), "Zero delta should return None");
}

#[test]
fn test_cpu_usage_extremes() {
    let t1 = CpuRawTimes {
        user: 100,
        nice: 0,
        system: 50,
        idle: 800,
        iowait: 50,
        irq: 0,
        softirq: 0,
        steal: 0,
    };
    // 100% idle delta
    let t_idle = CpuRawTimes {
        user: 100,
        nice: 0,
        system: 50,
        idle: 900,
        iowait: 50,
        irq: 0,
        softirq: 0,
        steal: 0,
    };
    let usage_idle = calculate_cpu_usage(&t1, &t_idle);
    assert_eq!(usage_idle, Some(0.0));

    // 100% busy delta
    let t_busy = CpuRawTimes {
        user: 200,
        nice: 0,
        system: 50,
        idle: 800,
        iowait: 50,
        irq: 0,
        softirq: 0,
        steal: 0,
    };
    let usage_busy = calculate_cpu_usage(&t1, &t_busy);
    assert_eq!(usage_busy, Some(100.0));
}

#[test]
fn test_parse_proc_stat() {
    let sample = "cpu  10132153 290696 2307184 213919248 111817 896333 133742 12345 0 0\n\
                  cpu0 1234 56 78 9012 34 56 78 0 0 0\n";
    let parsed = parse_proc_stat(sample);
    assert!(parsed.is_some());
    let times = parsed.unwrap();
    assert_eq!(times.user, 10132153);
    assert_eq!(times.nice, 290696);
    assert_eq!(times.system, 2307184);
    assert_eq!(times.idle, 213919248);
    assert_eq!(times.iowait, 111817);
    assert_eq!(times.irq, 896333);
    assert_eq!(times.softirq, 133742);
    assert_eq!(times.steal, 12345);
    assert_eq!(times.idle_all(), 213919248 + 111817);
}

#[test]
fn test_nvidia_runtime_pm_check_mock() {
    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("omen_test_pci_{}", unique_id));
    fs::create_dir_all(&root).unwrap();

    // 1. Active NVIDIA VGA controller (0x10de, 0x030000, active)
    let dgpu_path = root.join("0000:01:00.0");
    fs::create_dir_all(dgpu_path.join("power")).unwrap();
    fs::write(dgpu_path.join("vendor"), "0x10de\n").unwrap();
    fs::write(dgpu_path.join("class"), "0x030000\n").unwrap();
    fs::write(dgpu_path.join("power/runtime_status"), "active\n").unwrap();

    assert!(is_nvidia_gpu_powered_at(&root));

    // 2. Suspended NVIDIA GPU (runtime_status == "suspended")
    fs::write(dgpu_path.join("power/runtime_status"), "suspended\n").unwrap();
    assert!(!is_nvidia_gpu_powered_at(&root));

    // 3. Audio controller (vendor 0x10de, class 0x040300) should be ignored
    fs::write(dgpu_path.join("class"), "0x040300\n").unwrap();
    fs::write(dgpu_path.join("power/runtime_status"), "active\n").unwrap();
    assert!(!is_nvidia_gpu_powered_at(&root));

    // 4. Intel VGA (vendor 0x8086, class 0x030000)
    fs::write(dgpu_path.join("class"), "0x030000\n").unwrap();
    fs::write(dgpu_path.join("vendor"), "0x8086\n").unwrap();
    assert!(!is_nvidia_gpu_powered_at(&root));

    // Clean up
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_load_snapshot_structure() {
    let snapshot = LoadSnapshot::default();
    assert_eq!(snapshot.cpu_usage_pct, None);
    assert_eq!(snapshot.gpu_usage_pct, None);
    assert_eq!(snapshot.gpu_temp_c, None);
}

#[test]
fn test_collect_load_snapshot_smoke() {
    // Should run without crashing and return a LoadSnapshot
    let snapshot = collect_load_snapshot();
    println!("Collected snapshot: {:?}", snapshot);
}

#[test]
fn test_is_nvidia_gpu_powered_live() {
    // Should execute safely without panicking
    let is_powered = is_nvidia_gpu_powered();
    println!("NVIDIA GPU powered live state: {}", is_powered);
}

