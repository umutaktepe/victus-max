use clap::Subcommand;
use zbus::Connection;
use anyhow::Result;
use crate::dbus_proxy::FanProxy;
use comfy_table::Table;

#[derive(Subcommand, Debug, Clone)]
pub enum FanCommand {
    /// Set fan mode (auto, max, custom, better-auto)
    SetMode {
        mode: String,
    },
    /// Set target RPM for a specific fan
    SetTarget {
        fan_id: u32,
        rpm: u32,
    },
    /// Get current fan mode
    Mode,
    /// Get detailed fans info
    Info,
    /// Set minimum fan RPM for Balanced / Better Auto mode (2000-3500)
    SetMinRpm {
        rpm: u32,
    },
    /// Get minimum fan RPM
    GetMinRpm,
    /// Set acoustic ceiling level for Balanced / Better Auto mode (3-8)
    SetCeiling {
        level: u32,
    },
    /// Get acoustic ceiling level
    GetCeiling,
}

pub async fn handle(cmd: &FanCommand, conn: &Connection) -> Result<()> {
    let proxy = FanProxy::new(conn).await?;

    match cmd {
        FanCommand::SetMode { mode } => {
            let normalized_mode = mode.replace('-', "_");
            let res = proxy.set_fan_mode(&normalized_mode).await?;
            println!("Response: {}", res);
        }
        FanCommand::SetTarget { fan_id, rpm } => {
            let res = proxy.set_fan_target(*fan_id, *rpm).await?;
            println!("Response: {}", res);
        }
        FanCommand::Mode => {
            let res = proxy.get_fan_mode().await?;
            println!("Fan Mode: {}", res);
        }
        FanCommand::Info => {
            let res = proxy.get_fan_info().await?;
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&res) {
                let mut table = Table::new();
                table.set_header(vec!["Property", "Value"]);
                
                if let Some(mode) = json.get("mode").and_then(|v| v.as_str()) {
                    table.add_row(vec!["Global Mode".to_string(), mode.to_string()]);
                }

                if let Some(min_rpm) = json.get("min_fan_rpm").and_then(|v| v.as_u64()) {
                    table.add_row(vec!["Minimum RPM".to_string(), format!("{} RPM", min_rpm)]);
                }

                if let Some(ceiling) = json.get("acoustic_ceiling").and_then(|v| v.as_u64()) {
                    table.add_row(vec!["Acoustic Ceiling".to_string(), format!("Level {}", ceiling)]);
                }

                if let Some(fans) = json.get("fans").and_then(|f| f.as_object()) {
                    for (fan_id, details) in fans {
                        table.add_row(vec![
                            format!("Fan {}", fan_id),
                            format!(
                                "Current RPM: {} | Target: {} | Max: {}",
                                details.get("current").and_then(|v| v.as_u64()).unwrap_or(0),
                                details.get("target").and_then(|v| v.as_u64()).unwrap_or(0),
                                details.get("max").and_then(|v| v.as_u64()).unwrap_or(0)
                            )
                        ]);
                    }
                }
                
                println!("{}", table);
            } else {
                println!("{}", res);
            }
        }
        FanCommand::SetMinRpm { rpm } => {
            let res = proxy.set_min_fan_rpm(*rpm).await?;
            if res {
                let clamped = (*rpm).clamp(2000, 3500);
                if clamped != *rpm {
                    println!("Minimum Fan RPM set to {} RPM (clamped to 2000-3500)", clamped);
                } else {
                    println!("Minimum Fan RPM set to {} RPM", clamped);
                }
            } else {
                eprintln!("Failed to set minimum fan RPM");
            }
        }
        FanCommand::GetMinRpm => {
            let rpm = proxy.get_min_fan_rpm().await?;
            println!("Minimum Fan RPM: {} RPM", rpm);
        }
        FanCommand::SetCeiling { level } => {
            let res = proxy.set_acoustic_ceiling(*level).await?;
            if res {
                let clamped = (*level).clamp(3, 8);
                if clamped != *level {
                    println!("Acoustic Ceiling set to Level {} (clamped to 3-8)", clamped);
                } else {
                    println!("Acoustic Ceiling set to Level {}", clamped);
                }
            } else {
                eprintln!("Failed to set acoustic ceiling");
            }
        }
        FanCommand::GetCeiling => {
            let level = proxy.get_acoustic_ceiling().await?;
            println!("Acoustic Ceiling: Level {}", level);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    struct TestCli {
        #[command(subcommand)]
        cmd: FanCommand,
    }

    #[test]
    fn test_fan_cli_subcommands_parsing() {
        let cli = TestCli::try_parse_from(["fan", "set-min-rpm", "2800"]).unwrap();
        match cli.cmd {
            FanCommand::SetMinRpm { rpm } => assert_eq!(rpm, 2800),
            _ => panic!("Expected SetMinRpm"),
        }

        let cli = TestCli::try_parse_from(["fan", "get-min-rpm"]).unwrap();
        assert!(matches!(cli.cmd, FanCommand::GetMinRpm));

        let cli = TestCli::try_parse_from(["fan", "set-ceiling", "6"]).unwrap();
        match cli.cmd {
            FanCommand::SetCeiling { level } => assert_eq!(level, 6),
            _ => panic!("Expected SetCeiling"),
        }

        let cli = TestCli::try_parse_from(["fan", "get-ceiling"]).unwrap();
        assert!(matches!(cli.cmd, FanCommand::GetCeiling));

        let cli = TestCli::try_parse_from(["fan", "set-mode", "better-auto"]).unwrap();
        match cli.cmd {
            FanCommand::SetMode { mode } => {
                assert_eq!(mode, "better-auto");
                let normalized = mode.replace('-', "_");
                assert_eq!(normalized, "better_auto");
            }
            _ => panic!("Expected SetMode"),
        }
    }
}
