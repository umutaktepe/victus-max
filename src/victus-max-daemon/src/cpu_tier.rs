use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

static RYZENADJ_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
static LIVE_METRICS_CACHE: OnceLock<Mutex<Option<(Instant, RyzenLiveMetrics)>>> = OnceLock::new();

fn ryzenadj_lock() -> &'static Mutex<()> {
    RYZENADJ_MUTEX.get_or_init(|| Mutex::new(()))
}

fn metrics_cache() -> &'static Mutex<Option<(Instant, RyzenLiveMetrics)>> {
    LIVE_METRICS_CACHE.get_or_init(|| Mutex::new(None))
}

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
    #[serde(default)]
    pub stapm_fast_linked: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RyzenLiveMetrics {
    pub stapm_w: Option<u32>,
    pub fast_limit_w: Option<u32>,
    pub slow_limit_w: Option<u32>,
    pub tctl_temp_c: Option<u32>,
}

pub fn resolve_ryzenadj_bin() -> Option<String> {
    const COMMON_PATHS: &[&str] = &[
        "/usr/local/bin/ryzenadj",
        "/usr/bin/ryzenadj",
        "/bin/ryzenadj",
        "/usr/local/sbin/ryzenadj",
        "/usr/sbin/ryzenadj",
    ];

    for &path in COMMON_PATHS {
        if Path::new(path).is_file() {
            return Some(path.to_string());
        }
    }

    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join("ryzenadj"))
            .find(|path| path.is_file())
            .map(|p| p.to_string_lossy().into_owned())
    })
}

/// Runs `ryzenadj` with the given arguments while holding a process-wide async mutex
/// so concurrent D-Bus calls can never collide on PCI SMN 0x60/0x64 or MP1 SMU mailboxes.
pub async fn run_ryzenadj(args: &[&str]) -> Option<std::process::Output> {
    let _guard = ryzenadj_lock().lock().await;
    let bin = resolve_ryzenadj_bin().unwrap_or_else(|| "ryzenadj".to_string());

    let is_write = args.iter().any(|a| *a != "-i" && *a != "--info");
    if is_write {
        let mut cache = metrics_cache().lock().await;
        *cache = None;
    }

    let child = tokio::process::Command::new(&bin).args(args).output();
    let res = match tokio::time::timeout(Duration::from_millis(1200), child).await {
        Ok(Ok(output)) => Some(output),
        _ => None,
    };

    // Give the SMU mailbox a brief breather between back-to-back commands
    tokio::time::sleep(Duration::from_millis(40)).await;
    res
}

/// Queries `ryzenadj -i` (serialized via `run_ryzenadj`) with a short 2-second TTL cache.
pub async fn fetch_live_ryzen_metrics() -> Option<RyzenLiveMetrics> {
    {
        let cache = metrics_cache().lock().await;
        if let Some((ts, ref metrics)) = *cache {
            if ts.elapsed() < Duration::from_secs(2) {
                return Some(metrics.clone());
            }
        }
    }

    let output = run_ryzenadj(&["-i"]).await?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed = parse_ryzenadj_info_output(&stdout);
    if parsed.stapm_w.is_some()
        || parsed.slow_limit_w.is_some()
        || parsed.fast_limit_w.is_some()
        || parsed.tctl_temp_c.is_some()
    {
        let mut cache = metrics_cache().lock().await;
        *cache = Some((Instant::now(), parsed.clone()));
        Some(parsed)
    } else {
        None
    }
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
                    let normalize_power_w = |v: f64| -> u32 {
                        if v >= 1000.0 {
                            (v / 1000.0).round() as u32
                        } else {
                            v.round() as u32
                        }
                    };
                    if key.contains("STAPM LIMIT") {
                        metrics.stapm_w = Some(normalize_power_w(val_f));
                    } else if key.contains("PPT LIMIT FAST") {
                        metrics.fast_limit_w = Some(normalize_power_w(val_f));
                    } else if key.contains("PPT LIMIT SLOW") {
                        metrics.slow_limit_w = Some(normalize_power_w(val_f));
                    } else if key.contains("THM LIMIT CORE") {
                        metrics.tctl_temp_c = Some(val_f.round() as u32);
                    }
                }
            }
        }
    }

    metrics
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

pub fn classify_cpu(cpu_name: &str) -> (CpuVendor, CpuTier, CpuPowerProfileRanges) {
    let name_upper = cpu_name.to_uppercase();

    if name_upper.contains("AMD") || name_upper.contains("RYZEN") {
        let is_hx = name_upper
            .split_whitespace()
            .any(|tok| tok.chars().any(|c| c.is_ascii_digit()) && tok.contains("HX"));
        let is_u = name_upper
            .split_whitespace()
            .any(|tok| tok.chars().any(|c| c.is_ascii_digit()) && tok.ends_with('U'));

        let stapm_fast_linked = is_zen4_or_newer_ryzen(cpu_name);

        if is_hx {
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
                    stapm_fast_linked,
                },
            )
        } else if is_u {
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
                    stapm_fast_linked,
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
                    stapm_fast_linked,
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
                    stapm_fast_linked: false,
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
                    stapm_fast_linked: false,
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
                stapm_fast_linked: false,
            },
        )
    }
}
