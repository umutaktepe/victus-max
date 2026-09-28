#![allow(dead_code)]

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CpuRawTimes {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
}

impl CpuRawTimes {
    pub fn total(&self) -> u64 {
        self.user
            + self.nice
            + self.system
            + self.idle
            + self.iowait
            + self.irq
            + self.softirq
            + self.steal
    }

    pub fn idle_all(&self) -> u64 {
        self.idle + self.iowait
    }
}

pub fn parse_proc_stat(stat_content: &str) -> Option<CpuRawTimes> {
    for line in stat_content.lines() {
        if line.starts_with("cpu ") {
            let mut parts = line.split_whitespace().skip(1);
            let user = parts.next()?.parse().ok()?;
            let nice = parts.next()?.parse().ok()?;
            let system = parts.next()?.parse().ok()?;
            let idle = parts.next()?.parse().ok()?;
            let iowait = parts.next()?.parse().ok()?;
            let irq = parts.next()?.parse().ok()?;
            let softirq = parts.next()?.parse().ok()?;
            let steal = parts.next()?.parse().ok()?;
            return Some(CpuRawTimes {
                user,
                nice,
                system,
                idle,
                iowait,
                irq,
                softirq,
                steal,
            });
        }
    }
    None
}

pub fn read_proc_stat() -> Option<CpuRawTimes> {
    let content = fs::read_to_string("/proc/stat").ok()?;
    parse_proc_stat(&content)
}

pub fn calculate_cpu_usage(prev: &CpuRawTimes, curr: &CpuRawTimes) -> Option<f64> {
    let total_diff = curr.total().saturating_sub(prev.total());
    let idle_diff = curr.idle_all().saturating_sub(prev.idle_all());
    if total_diff == 0 {
        return None;
    }
    let busy_diff = total_diff.saturating_sub(idle_diff);
    let usage = (busy_diff as f64 / total_diff as f64) * 100.0;
    Some(usage.clamp(0.0, 100.0))
}

pub fn is_nvidia_gpu_powered_at(pci_root: &Path) -> bool {
    let Ok(entries) = fs::read_dir(pci_root) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let class = fs::read_to_string(path.join("class")).unwrap_or_default();
        // Class 0x030000 (VGA) or 0x030200 (3D Controller) — ignores audio 0x040300
        if class.trim().starts_with("0x03") {
            let vendor = fs::read_to_string(path.join("vendor")).unwrap_or_default();
            if vendor.trim().eq_ignore_ascii_case("0x10de") {
                let status_path = path.join("power/runtime_status");
                if let Ok(status) = fs::read_to_string(status_path) {
                    if status.trim() == "active" {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn is_nvidia_gpu_powered() -> bool {
    is_nvidia_gpu_powered_at(Path::new("/sys/bus/pci/devices"))
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LoadSnapshot {
    pub cpu_usage_pct: Option<f64>,
    pub gpu_usage_pct: Option<f64>,
    pub gpu_temp_c: Option<f64>,
}

fn read_drm_gpu_busy() -> Option<f64> {
    if let Ok(entries) = glob::glob("/sys/class/drm/card*/device/gpu_busy_percent") {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(&entry) {
                if let Ok(val) = content.trim().parse::<f64>() {
                    return Some(val);
                }
            }
        }
    }
    None
}

fn read_drm_gpu_temp() -> Option<f64> {
    if let Ok(entries) = glob::glob("/sys/class/drm/card*/device/hwmon/hwmon*/temp1_input") {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(&entry) {
                if let Ok(milli) = content.trim().parse::<f64>() {
                    return Some(milli / 1000.0);
                }
            }
        }
    }
    None
}

static PREV_CPU_TIMES: Mutex<Option<CpuRawTimes>> = Mutex::new(None);

pub fn collect_load_snapshot() -> LoadSnapshot {
    let mut snapshot = LoadSnapshot::default();

    // 1. Proactive CPU Load calculation
    if let Some(curr_times) = read_proc_stat() {
        let mut guard = PREV_CPU_TIMES.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ref prev) = *guard {
            snapshot.cpu_usage_pct = calculate_cpu_usage(prev, &curr_times);
        }
        *guard = Some(curr_times);
    }

    // 2. Runtime-PM safe NVIDIA dGPU telemetry
    if is_nvidia_gpu_powered() {
        let output = Command::new("timeout")
            .args([
                "2",
                "nvidia-smi",
                "--query-gpu=temperature.gpu,utilization.gpu",
                "--format=csv,noheader,nounits",
            ])
            .output()
            .or_else(|_| {
                Command::new("nvidia-smi")
                    .args([
                        "--query-gpu=temperature.gpu,utilization.gpu",
                        "--format=csv,noheader,nounits",
                    ])
                    .output()
            });

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Some(line) = stdout.lines().next() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 {
                        snapshot.gpu_temp_c = parts[0].trim().parse::<f64>().ok();
                        snapshot.gpu_usage_pct = parts[1].trim().parse::<f64>().ok();
                    }
                }
            }
        }
    }

    // 3. Fall back to DRM for AMD iGPU/dGPU if NVIDIA metrics are absent
    if snapshot.gpu_usage_pct.is_none() {
        snapshot.gpu_usage_pct = read_drm_gpu_busy();
    }
    if snapshot.gpu_temp_c.is_none() {
        snapshot.gpu_temp_c = read_drm_gpu_temp();
    }

    snapshot
}
