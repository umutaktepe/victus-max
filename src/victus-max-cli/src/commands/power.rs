use clap::Subcommand;
use zbus::Connection;
use anyhow::Result;
use crate::dbus_proxy::{PowerProxy, MuxProxy};
use comfy_table::Table;

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum PowerCommand {
    /// Set power profile (power-saver, balanced, performance)
    SetProfile {
        profile: String,
    },
    /// Set CPU power limits PL1 & PL2 in Watts
    SetLimits {
        #[arg(long)]
        enabled: bool,
        #[arg(long)]
        pl1: i32,
        #[arg(long)]
        pl2: i32,
    },
    /// Apply CPU undervolt offset in mV (e.g. -50)
    Undervolt {
        mv: i32,
    },
    /// Set GPU Mux mode (hybrid, discrete, advanced)
    SetMux {
        mode: String,
    },
    /// Get current power profile and limits
    Info,
    /// Manage hardware power limit presets (list, set, or reset)
    Presets {
        #[arg(help = "Action: list, set, or reset")]
        action: Option<String>,
        #[arg(help = "Target profile: power-saver, balanced, or performance")]
        mode: Option<String>,
    },
    /// Show live CPU power limits and thermal limits (replaces legacy ryzen-limits)
    Limits,
}

pub fn format_presets_table(json: &serde_json::Value) -> (Table, bool) {
    let active_mode = json
        .get("active_mode")
        .and_then(|v| v.as_str())
        .or_else(|| json.get("presets").and_then(|p| p.get("active_mode")).and_then(|v| v.as_str()));

    let manual_override = json
        .get("manual_override")
        .and_then(|v| v.as_bool())
        .or_else(|| json.get("presets").and_then(|p| p.get("manual_override")).and_then(|v| v.as_bool()))
        .unwrap_or(false);

    let presets_obj = json.get("presets").unwrap_or(json);

    let mut table = Table::new();
    table.set_header(vec![
        "Mode",
        "STAPM/PL1 (W)",
        "Slow PPT (W)",
        "Fast PPT/PL2 (W)",
        "Temp Limit (°C)",
    ]);

    let modes = [
        ("power-saver", "power_saver"),
        ("balanced", "balanced"),
        ("performance", "performance"),
    ];

    let norm_active = active_mode.map(|a| a.replace('_', "-").to_lowercase());

    for (canonical_name, snake_name) in modes {
        let mode_limits = presets_obj.get(canonical_name).or_else(|| presets_obj.get(snake_name));

        let stapm = mode_limits
            .and_then(|m| m.get("stapm_w"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let slow = mode_limits
            .and_then(|m| m.get("slow_ppt_w"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let fast = mode_limits
            .and_then(|m| m.get("fast_ppt_w"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let temp = mode_limits
            .and_then(|m| m.get("tctl_temp"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let is_active = norm_active.as_deref() == Some(canonical_name);
        let mode_label = if is_active {
            format!("* {} (Aktif)", canonical_name)
        } else {
            format!("  {}", canonical_name)
        };

        table.add_row(vec![
            mode_label,
            stapm.to_string(),
            slow.to_string(),
            fast.to_string(),
            temp.to_string(),
        ]);
    }

    (table, manual_override)
}

pub fn format_limits_table(json: &serde_json::Value) -> Table {
    let stapm = json
        .get("stapm_w")
        .or_else(|| json.get("pl1_w"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let slow = json
        .get("slow_ppt_w")
        .and_then(|v| v.as_u64())
        .filter(|&v| v > 0)
        .unwrap_or(stapm);
    let fast = json
        .get("fast_ppt_w")
        .or_else(|| json.get("pl2_w"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let tctl = json
        .get("tctl_temp")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let mut table = Table::new();
    table.set_header(vec!["STAPM/PL1", "Slow PPT", "Fast PPT/PL2", "Tctl/TCC Temp"]);
    table.add_row(vec![
        format!("{} W", stapm),
        format!("{} W", slow),
        format!("{} W", fast),
        format!("{} °C", tctl),
    ]);
    table
}

pub async fn handle(cmd: &PowerCommand, conn: &Connection) -> Result<()> {
    let proxy = PowerProxy::new(conn).await?;

    match cmd {
        PowerCommand::SetProfile { profile } => {
            let res = proxy.set_power_profile(profile).await?;
            println!("Response: {}", res);
        }
        PowerCommand::SetLimits { enabled, pl1, pl2 } => {
            let res = proxy.set_power_limits(*enabled, *pl1, *pl2).await?;
            println!("Response: {}", res);
        }
        PowerCommand::Undervolt { mv } => {
            let res = proxy.set_undervolt(*mv).await?;
            println!("Response: {}", res);
        }
        PowerCommand::SetMux { mode } => {
            let mode_lower = mode.to_lowercase();
            if mode_lower != "hybrid" && mode_lower != "discrete" && mode_lower != "advanced" {
                eprintln!("{} {}", crate::i18n::t("mux_invalid"), mode);
                std::process::exit(1);
            }
            let mux_proxy = MuxProxy::new(conn).await?;
            let res = mux_proxy.set_gpu_mode(&mode_lower).await?;
            println!("Response: {}", res);
        }
        PowerCommand::Info => {
            let res = proxy.get_power_profile().await?;
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&res) {
                let mut table = Table::new();
                table.set_header(vec!["Key", "Value"]);
                
                if let Some(obj) = json.as_object() {
                    for (k, v) in obj {
                        table.add_row(vec![k, &v.to_string()]);
                    }
                }
                
                println!("{}", table);
            } else {
                println!("{}", res);
            }
        }
        PowerCommand::Presets { action, mode } => {
            match action.as_deref().map(|s| s.to_lowercase()).as_deref() {
                Some("set") => {
                    if let Some(m) = mode {
                        let res = proxy.set_power_preset_mode(m).await?;
                        println!("Response: {}", res);
                    } else {
                        eprintln!("Error: Mode is required for 'set' action. Target profile: power-saver, balanced, or performance");
                        std::process::exit(1);
                    }
                }
                Some("reset") => {
                    let res = proxy.reset_power_presets().await?;
                    println!("Response: {}", res);
                }
                None | Some("list") => {
                    let res = proxy.get_power_presets().await?;
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&res) {
                        let (table, manual_override) = format_presets_table(&json);
                        println!("{}", table);
                        println!("Manuel Override: {}", if manual_override { "Evet" } else { "Hayır" });
                    } else {
                        println!("{}", res);
                    }
                }
                Some("power-saver") | Some("powersave") | Some("quiet") | Some("balanced") | Some("performance") | Some("perf") => {
                    let m = action.as_deref().unwrap();
                    let res = proxy.set_power_preset_mode(m).await?;
                    println!("Response: {}", res);
                }
                Some(other) => {
                    eprintln!("Unknown action '{}'. Valid actions: list, set, reset", other);
                    std::process::exit(1);
                }
            }
        }
        PowerCommand::Limits => {
            let res = proxy.get_power_limits_info().await?;
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&res) {
                let table = format_limits_table(&json);
                println!("{}", table);
            } else {
                println!("{}", res);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestPowerCli {
        #[command(subcommand)]
        cmd: PowerCommand,
    }

    #[test]
    fn test_power_cli_subcommands_parsing() {
        let cli = TestPowerCli::try_parse_from(["power", "presets"]).unwrap();
        match cli.cmd {
            PowerCommand::Presets { action, mode } => {
                assert_eq!(action, None);
                assert_eq!(mode, None);
            }
            _ => panic!("Expected PowerCommand::Presets"),
        }

        let cli = TestPowerCli::try_parse_from(["power", "presets", "list"]).unwrap();
        match cli.cmd {
            PowerCommand::Presets { action, mode } => {
                assert_eq!(action, Some("list".to_string()));
                assert_eq!(mode, None);
            }
            _ => panic!("Expected PowerCommand::Presets list"),
        }

        let cli = TestPowerCli::try_parse_from(["power", "presets", "set", "performance"]).unwrap();
        match cli.cmd {
            PowerCommand::Presets { action, mode } => {
                assert_eq!(action, Some("set".to_string()));
                assert_eq!(mode, Some("performance".to_string()));
            }
            _ => panic!("Expected PowerCommand::Presets set"),
        }

        let cli = TestPowerCli::try_parse_from(["power", "presets", "reset"]).unwrap();
        match cli.cmd {
            PowerCommand::Presets { action, mode } => {
                assert_eq!(action, Some("reset".to_string()));
                assert_eq!(mode, None);
            }
            _ => panic!("Expected PowerCommand::Presets reset"),
        }

        let cli = TestPowerCli::try_parse_from(["power", "limits"]).unwrap();
        assert!(matches!(cli.cmd, PowerCommand::Limits));
    }

    #[test]
    fn test_format_presets_table_active_mode_and_override() {
        let json_data = serde_json::json!({
            "is_amd": true,
            "presets": {
                "power_saver": { "stapm_w": 25, "slow_ppt_w": 25, "fast_ppt_w": 35, "tctl_temp": 72 },
                "balanced": { "stapm_w": 45, "slow_ppt_w": 45, "fast_ppt_w": 55, "tctl_temp": 84 },
                "performance": { "stapm_w": 65, "slow_ppt_w": 65, "fast_ppt_w": 80, "tctl_temp": 92 }
            },
            "active_mode": "balanced",
            "manual_override": false
        });

        let (table, manual_override) = format_presets_table(&json_data);
        assert!(!manual_override);
        let table_str = table.to_string();

        // Check headers
        assert!(table_str.contains("Mode"));
        assert!(table_str.contains("STAPM/PL1 (W)"));
        assert!(table_str.contains("Slow PPT (W)"));
        assert!(table_str.contains("Fast PPT/PL2 (W)"));
        assert!(table_str.contains("Temp Limit (°C)"));

        // Check balanced is highlighted as active
        assert!(table_str.contains("* balanced (Aktif)"));
        // Check power-saver and performance are present
        assert!(table_str.contains("power-saver"));
        assert!(table_str.contains("performance"));
        // Check wattages
        assert!(table_str.contains("25"));
        assert!(table_str.contains("45"));
        assert!(table_str.contains("65"));
    }

    #[test]
    fn test_format_presets_table_manual_override() {
        let json_data = serde_json::json!({
            "presets": {
                "power_saver": { "stapm_w": 20, "slow_ppt_w": 20, "fast_ppt_w": 25, "tctl_temp": 70 },
                "balanced": { "stapm_w": 35, "slow_ppt_w": 35, "fast_ppt_w": 35, "tctl_temp": 82 },
                "performance": { "stapm_w": 45, "slow_ppt_w": 45, "fast_ppt_w": 45, "tctl_temp": 88 }
            },
            "active_mode": "performance",
            "manual_override": true
        });

        let (table, manual_override) = format_presets_table(&json_data);
        assert!(manual_override);
        let table_str = table.to_string();
        assert!(table_str.contains("* performance (Aktif)"));
    }

    #[test]
    fn test_format_limits_table() {
        let json_data = serde_json::json!({
            "is_amd": true,
            "stapm_w": 45,
            "slow_ppt_w": 45,
            "fast_ppt_w": 65,
            "tctl_temp": 82
        });

        let table = format_limits_table(&json_data);
        let table_str = table.to_string();

        assert!(table_str.contains("STAPM/PL1"));
        assert!(table_str.contains("Slow PPT"));
        assert!(table_str.contains("Fast PPT/PL2"));
        assert!(table_str.contains("Tctl/TCC Temp"));
        assert!(table_str.contains("45 W"));
        assert!(table_str.contains("65 W"));
        assert!(table_str.contains("82 °C"));
    }
}

