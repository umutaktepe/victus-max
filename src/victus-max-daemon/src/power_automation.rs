use log::info;
use zbus::fdo::PropertiesProxy;
use futures::StreamExt;
use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use std::sync::OnceLock;
use std::path::PathBuf;

#[zbus::proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
trait LogindManager {
    #[zbus(signal)]
    fn prepare_for_sleep(&self, start: bool) -> zbus::Result<()>;
}

static AC_ONLINE_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

pub fn normalize_ppd_mode(s: &str) -> &'static str {
    match s.to_lowercase().as_str() {
        "performance" => "performance",
        "power-saver" | "quiet" | "low-power" | "cool" => "power-saver",
        _ => "balanced",
    }
}

#[derive(Clone)]
pub struct PowerAutomationService {
    last_ac_state: Arc<Mutex<Option<bool>>>,
}

impl PowerAutomationService {
    pub fn new() -> Self {
        Self {
            last_ac_state: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_monitor(&self) {
        let last_state = self.last_ac_state.clone();

        // PPD PropertiesChanged listener
        tokio::spawn(async move {
            if let Ok(conn) = zbus::Connection::system().await {
                if let Ok(proxy) = PropertiesProxy::builder(&conn)
                    .destination("net.hadess.PowerProfiles").unwrap()
                    .path("/net/hadess/PowerProfiles").unwrap()
                    .build().await {
                    
                    if let Ok(mut stream) = proxy.receive_properties_changed().await {
                        while let Some(signal) = stream.next().await {
                            if let Ok(args) = signal.args() {
                                if args.interface_name() == "net.hadess.PowerProfiles" {
                                    if let Some(val) = args.changed_properties().get("ActiveProfile") {
                                        if let Ok(profile_str) = <&str>::try_from(val) {
                                            info!("PPD Profile changed to: {}", profile_str);
                                            let norm = normalize_ppd_mode(profile_str);
                                            if let Ok(conn) = zbus::Connection::system().await {
                                                if let Ok(pwr_proxy) = omen_types::PowerProxy::new(&conn).await {
                                                    let _ = pwr_proxy.set_power_preset_mode(norm).await;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        // AC online/offline polling loop
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(4)).await;
                let current_ac = is_ac_power_connected();

                let mut state_lock = last_state.lock().await;
                if let Some(prev) = *state_lock {
                    if prev != current_ac {
                        *state_lock = Some(current_ac);
                        if let Ok(conn) = zbus::Connection::system().await {
                            if let Ok(pwr_proxy) = omen_types::PowerProxy::new(&conn).await {
                                let mut auto_enabled = true;
                                let mut ac_target = "performance".to_string();
                                let mut bat_target = "power-saver".to_string();
                                let mut current_active = "balanced".to_string();

                                if let Ok(presets_json) = pwr_proxy.get_power_presets().await {
                                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&presets_json) {
                                        if let Some(en) = val.get("auto_ac_profile_enabled").and_then(|v| v.as_bool()) {
                                            auto_enabled = en;
                                        }
                                        if let Some(ac) = val.get("ac_power_profile").and_then(|v| v.as_str()) {
                                            ac_target = ac.to_string();
                                        }
                                        if let Some(bat) = val.get("battery_power_profile").and_then(|v| v.as_str()) {
                                            bat_target = bat.to_string();
                                        }
                                        if let Some(act) = val.get("active_mode").and_then(|v| v.as_str()) {
                                            current_active = act.to_string();
                                        }
                                    }
                                }

                                if auto_enabled {
                                    let target = if current_ac { &ac_target } else { &bat_target };
                                    info!(
                                        "{} power connected. Auto switching to preset '{}'...",
                                        if current_ac { "AC" } else { "Battery" },
                                        target
                                    );
                                    let _ = pwr_proxy.set_power_preset_mode(target).await;
                                } else {
                                    info!(
                                        "{} power connected (auto-switch disabled). Re-applying active '{}' preset limits...",
                                        if current_ac { "AC" } else { "Battery" },
                                        current_active
                                    );
                                    let _ = pwr_proxy.set_power_preset_mode(&current_active).await;
                                }
                            }
                        }
                    }
                } else {
                    *state_lock = Some(current_ac);
                }
            }
        });

        // systemd-logind sleep/resume listener
        tokio::spawn(async move {
            if let Ok(conn) = zbus::Connection::system().await {
                if let Ok(logind_proxy) = LogindManagerProxy::new(&conn).await {
                    if let Ok(mut stream) = logind_proxy.receive_prepare_for_sleep().await {
                        while let Some(signal) = stream.next().await {
                            if let Ok(args) = signal.args() {
                                if !args.start() {
                                    info!("System resumed from sleep. Re-enforcing power presets in 1.5s...");
                                    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                                    if let Ok(pwr) = omen_types::PowerProxy::new(&conn).await {
                                        if let Ok(raw_json) = pwr.get_power_presets().await {
                                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw_json) {
                                                if let Some(active) = parsed.get("active_mode").and_then(|v| v.as_str()) {
                                                    let _ = pwr.set_power_preset_mode(active).await;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

fn is_ac_power_connected() -> bool {
    let online_path = AC_ONLINE_PATH.get_or_init(|| {
        if let Ok(entries) = fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("AC") || name.starts_with("ADP") || name.starts_with("ac") {
                        return Some(path.join("online"));
                    }
                }
            }
        }
        None
    });

    if let Some(p) = online_path {
        if let Ok(val) = fs::read_to_string(p) {
            return val.trim() == "1";
        }
    }
    true // default to AC if undetectable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_ppd_mode_performance() {
        assert_eq!(normalize_ppd_mode("performance"), "performance");
        assert_eq!(normalize_ppd_mode("Performance"), "performance");
        assert_eq!(normalize_ppd_mode("PERFORMANCE"), "performance");
    }

    #[test]
    fn test_normalize_ppd_mode_power_saver() {
        assert_eq!(normalize_ppd_mode("power-saver"), "power-saver");
        assert_eq!(normalize_ppd_mode("Power-Saver"), "power-saver");
        assert_eq!(normalize_ppd_mode("quiet"), "power-saver");
        assert_eq!(normalize_ppd_mode("Quiet"), "power-saver");
        assert_eq!(normalize_ppd_mode("low-power"), "power-saver");
        assert_eq!(normalize_ppd_mode("cool"), "power-saver");
    }

    #[test]
    fn test_normalize_ppd_mode_balanced_fallback() {
        assert_eq!(normalize_ppd_mode("balanced"), "balanced");
        assert_eq!(normalize_ppd_mode("Balanced"), "balanced");
        assert_eq!(normalize_ppd_mode("default"), "balanced");
        assert_eq!(normalize_ppd_mode("unknown-profile"), "balanced");
        assert_eq!(normalize_ppd_mode(""), "balanced");
    }
}
