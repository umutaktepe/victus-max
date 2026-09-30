use anyhow::Result;
use serde_json::Value;
use std::env;
use std::io::{stdout, Write};
use std::collections::VecDeque;
use std::time::Duration;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::dbus_proxy::{PlatformProxy, FanProxy, PowerProxy, RgbProxy, MuxProxy, SysMonProxy};

pub fn get_ascii_logo(product_name: &str) -> &'static [&'static str] {
    static ASCII_LOGO_OMEN: &[&str] = &[
        "\x1b[38;2;0;162;255m            .::.            \x1b[0m",
        "\x1b[38;2;20;140;255m          .::::::.          \x1b[0m",
        "\x1b[38;2;40;120;255m        .::::::::::.        \x1b[0m",
        "\x1b[38;2;60;100;255m      .::::::::::::::.      \x1b[0m",
        "\x1b[38;2;80;80;255m     .::::::::::::::::.     \x1b[0m",
        "\x1b[38;2;110;60;255m    ::::::::::::::::::::    \x1b[0m",
        "\x1b[38;2;140;40;255m    ::::::::::::::::::::    \x1b[0m",
        "\x1b[38;2;170;20;255m     '::::::::::::::::'     \x1b[0m",
        "\x1b[38;2;200;0;255m      '::::::::::::::'      \x1b[0m",
        "\x1b[38;2;170;20;255m        '::::::::::'        \x1b[0m",
        "\x1b[38;2;140;40;255m          '::::::'          \x1b[0m",
        "\x1b[38;2;0;162;255m            '::'            \x1b[0m",
        "",
        "",
    ];

    static ASCII_LOGO_VICTUS: &[&str] = &[
        "            \x1b[38;2;197;47;33m▄\x1b[38;2;185;37;30m\x1b[48;2;188;23;28m▀\x1b[38;2;182;31;29m\x1b[48;2;187;22;28m▀\x1b[49m\x1b[38;2;192;37;31m▄\x1b[0m            ",
        "          \x1b[38;2;212;58;37m▄\x1b[38;2;208;54;35m\x1b[48;2;198;29;30m▀\x1b[38;2;167;23;26m\x1b[48;2;25;4;4m▀\x1b[38;2;92;12;15m\x1b[48;2;60;8;10m▀\x1b[38;2;94;13;15m\x1b[48;2;65;8;10m▀\x1b[38;2;161;20;24m\x1b[48;2;22;10;5m▀\x1b[38;2;200;43;32m\x1b[48;2;188;28;29m▀\x1b[49m\x1b[38;2;194;46;32m▄\x1b[0m          ",
        "        \x1b[38;2;236;63;40m▄\x1b[38;2;229;62;40m\x1b[48;2;209;30;35m▀\x1b[38;2;209;30;32m\x1b[48;2;142;21;7m▀\x1b[38;2;92;15;6m\x1b[48;2;128;29;4m▀\x1b[38;2;51;17;2m\x1b[48;2;145;44;6m▀\x1b[38;2;83;15;13m\x1b[48;2;84;15;13m▀\x1b[38;2;86;14;13m\x1b[48;2;86;17;14m▀\x1b[38;2;50;29;5m\x1b[48;2;142;58;9m▀\x1b[38;2;91;37;10m\x1b[48;2;130;66;11m▀\x1b[38;2;193;34;29m\x1b[48;2;144;71;15m▀\x1b[38;2;210;58;35m\x1b[48;2;191;35;31m▀\x1b[49m\x1b[38;2;217;71;37m▄\x1b[0m        ",
        "      \x1b[38;2;244;63;41m▄\x1b[38;2;245;64;40m\x1b[48;2;238;36;37m▀\x1b[38;2;218;33;36m\x1b[48;2;159;15;7m▀\x1b[38;2;133;14;6m\x1b[48;2;131;19;3m▀\x1b[38;2;169;33;4m\x1b[48;2;84;20;6m▀\x1b[38;2;163;44;8m\x1b[48;2;132;41;9m▀\x1b[38;2;100;38;7m\x1b[48;2;128;54;12m▀\x1b[38;2;107;25;16m\x1b[48;2;146;45;20m▀\x1b[38;2;109;27;16m\x1b[48;2;148;38;19m▀\x1b[38;2;99;37;7m\x1b[48;2;130;42;10m▀\x1b[38;2;164;73;15m\x1b[48;2;133;57;12m▀\x1b[38;2;170;95;14m\x1b[48;2;85;42;10m▀\x1b[38;2;134;72;13m\x1b[48;2;132;83;14m▀\x1b[38;2;202;49;33m\x1b[48;2;159;91;17m▀\x1b[38;2;233;91;37m\x1b[48;2;233;81;36m▀\x1b[49m\x1b[38;2;241;109;38m▄\x1b[0m      ",
        "    \x1b[38;2;244;58;40m▄\x1b[38;2;245;63;41m\x1b[48;2;219;33;35m▀\x1b[38;2;228;36;37m\x1b[48;2;111;5;5m▀\x1b[38;2;119;8;6m\x1b[48;2;142;6;0m▀\x1b[38;2;88;11;3m\x1b[48;2;86;15;6m▀\x1b[38;2;171;36;9m\x1b[48;2;129;29;7m▀\x1b[38;2;123;37;9m\x1b[48;2;161;52;11m▀\x1b[38;2;61;21;9m\x1b[48;2;83;33;11m▀\x1b[38;2;104;49;11m\x1b[48;2;81;42;11m▀\x1b[38;2;141;45;20m\x1b[48;2;109;32;20m▀\x1b[38;2;143;39;19m\x1b[48;2;112;37;20m▀\x1b[38;2;106;30;8m\x1b[48;2;80;22;8m▀\x1b[38;2;61;27;11m\x1b[48;2;85;32;12m▀\x1b[38;2;123;57;13m\x1b[48;2;162;69;15m▀\x1b[38;2;172;93;18m\x1b[48;2;131;66;13m▀\x1b[38;2;91;61;12m\x1b[48;2;87;49;12m▀\x1b[38;2;117;70;14m\x1b[48;2;140;96;10m▀\x1b[38;2;235;96;37m\x1b[48;2;112;69;12m▀\x1b[38;2;251;126;40m\x1b[48;2;236;103;38m▀\x1b[49m\x1b[38;2;255;139;43m▄\x1b[0m    ",
        "  \x1b[38;2;216;42;33m▄\x1b[38;2;230;52;36m\x1b[48;2;196;29;31m▀\x1b[38;2;208;31;33m\x1b[48;2;22;2;4m▀\x1b[38;2;31;3;5m\x1b[48;2;0;1;0m▀\x1b[38;2;124;18;9m\x1b[48;2;159;21;19m▀\x1b[38;2;172;31;11m\x1b[48;2;237;32;19m▀\x1b[38;2;121;30;9m\x1b[48;2;229;45;24m▀\x1b[38;2;62;21;8m\x1b[48;2;192;52;21m▀\x1b[38;2;78;29;10m\x1b[48;2;148;56;17m▀\x1b[38;2;69;31;13m\x1b[48;2;70;34;15m▀\x1b[38;2;70;40;13m\x1b[48;2;15;15;13m▀\x1b[38;2;105;32;21m\x1b[48;2;115;39;23m▀\x1b[38;2;108;42;22m\x1b[48;2;118;47;22m▀\x1b[38;2;70;22;11m\x1b[48;2;15;4;11m▀\x1b[38;2;72;27;12m\x1b[48;2;68;30;14m▀\x1b[38;2;80;31;11m\x1b[48;2;146;71;20m▀\x1b[38;2;60;35;10m\x1b[48;2;189;86;25m▀\x1b[38;2;117;77;14m\x1b[48;2;229;93;29m▀\x1b[38;2;170;113;19m\x1b[48;2;246;110;29m▀\x1b[38;2;124;70;15m\x1b[48;2;183;40;28m▀\x1b[38;2;34;14;6m\x1b[48;2;0;0;0m▀\x1b[38;2;234;105;37m\x1b[48;2;24;6;4m▀\x1b[38;2;255;136;43m\x1b[48;2;232;95;36m▀\x1b[49m\x1b[38;2;255;120;41m▄\x1b[0m  ",
        "\x1b[38;2;193;30;28m▄\x1b[38;2;205;36;32m\x1b[48;2;167;23;27m▀\x1b[38;2;185;27;30m\x1b[48;2;18;3;4m▀\x1b[38;2;21;3;4m\x1b[48;2;0;0;0m▀\x1b[38;2;0;0;0m\x1b[48;2;4;0;1m▀\x1b[38;2;2;0;0m\x1b[48;2;1;0;0m▀\x1b[38;2;97;15;15m\x1b[48;2;16;5;4m▀\x1b[38;2;223;27;26m\x1b[48;2;191;26;26m▀\x1b[38;2;217;33;28m\x1b[48;2;251;45;30m▀\x1b[38;2;187;25;27m\x1b[48;2;61;21;13m▀\x1b[38;2;164;39;20m\x1b[48;2;43;22;14m▀\x1b[38;2;207;76;22m\x1b[48;2;71;19;16m▀\x1b[38;2;98;52;20m\x1b[48;2;220;61;23m▀\x1b[38;2;108;41;23m\x1b[48;2;228;59;31m▀\x1b[38;2;111;50;24m\x1b[48;2;227;94;29m▀\x1b[38;2;94;54;20m\x1b[48;2;222;67;25m▀\x1b[38;2;206;89;24m\x1b[48;2;75;23;17m▀\x1b[38;2;165;53;22m\x1b[48;2;43;23;15m▀\x1b[38;2;196;26;30m\x1b[48;2;59;25;14m▀\x1b[38;2;221;52;31m\x1b[48;2;250;78;34m▀\x1b[38;2;238;58;34m\x1b[48;2;206;41;33m▀\x1b[38;2;114;13;21m\x1b[48;2;19;5;5m▀\x1b[38;2;2;2;0m\x1b[48;2;1;0;0m▀\x1b[38;2;0;0;0m\x1b[48;2;4;1;1m▀\x1b[38;2;24;4;4m\x1b[48;2;0;0;0m▀\x1b[38;2;231;81;36m\x1b[48;2;20;3;4m▀\x1b[38;2;255;97;40m\x1b[48;2;214;60;34m▀\x1b[49m\x1b[38;2;239;74;38m▄\x1b[0m",
        "\x1b[38;2;193;28;28m▀\x1b[38;2;162;22;26m\x1b[48;2;209;32;30m▀\x1b[38;2;10;2;2m\x1b[48;2;183;27;29m▀\x1b[38;2;0;0;0m\x1b[48;2;14;2;3m▀\x1b[38;2;2;0;0m\x1b[48;2;0;0;0m▀\x1b[38;2;3;1;1m\x1b[48;2;3;1;1m▀\x1b[38;2;0;2;1m\x1b[48;2;139;22;21m▀\x1b[38;2;120;21;19m\x1b[48;2;249;35;34m▀\x1b[38;2;243;35;34m\x1b[48;2;245;34;34m▀\x1b[38;2;193;35;29m\x1b[48;2;254;35;35m▀\x1b[38;2;255;58;31m\x1b[48;2;247;34;34m▀\x1b[38;2;189;53;25m\x1b[48;2;201;34;28m▀\x1b[38;2;109;14;17m\x1b[48;2;3;18;18m▀\x1b[38;2;146;38;24m\x1b[48;2;98;36;24m▀\x1b[38;2;148;60;23m\x1b[48;2;101;43;25m▀\x1b[38;2;107;21;18m\x1b[48;2;1;16;17m▀\x1b[38;2;189;78;29m\x1b[48;2;197;44;28m▀\x1b[38;2;253;76;33m\x1b[48;2;244;45;33m▀\x1b[38;2;188;48;29m\x1b[48;2;250;42;35m▀\x1b[38;2;246;45;37m\x1b[48;2;240;36;34m▀\x1b[38;2;131;24;22m\x1b[48;2;255;41;38m▀\x1b[38;2;0;2;0m\x1b[48;2;157;28;25m▀\x1b[38;2;3;1;1m\x1b[48;2;3;1;1m▀\x1b[38;2;2;0;0m\x1b[48;2;0;0;0m▀\x1b[38;2;0;0;0m\x1b[48;2;14;2;3m▀\x1b[38;2;9;2;2m\x1b[48;2;214;44;35m▀\x1b[38;2;206;45;33m\x1b[48;2;253;70;41m▀\x1b[49m\x1b[38;2;240;69;39m▀\x1b[0m",
        "  \x1b[38;2;222;37;34m▀\x1b[38;2;196;30;32m\x1b[48;2;239;43;37m▀\x1b[38;2;13;2;3m\x1b[48;2;208;32;33m▀\x1b[38;2;1;1;0m\x1b[48;2;13;2;3m▀\x1b[38;2;209;31;31m\x1b[48;2;75;9;9m▀\x1b[38;2;146;19;22m\x1b[48;2;97;8;5m▀\x1b[38;2;153;23;25m\x1b[48;2;118;17;10m▀\x1b[38;2;119;19;21m\x1b[48;2;106;21;10m▀\x1b[38;2;197;27;29m\x1b[48;2;119;25;12m▀\x1b[38;2;62;16;16m\x1b[48;2;32;17;16m▀\x1b[38;2;0;17;18m\x1b[48;2;179;35;30m▀\x1b[38;2;129;38;26m\x1b[48;2;241;44;36m▀\x1b[38;2;133;46;25m\x1b[48;2;240;50;30m▀\x1b[38;2;0;15;17m\x1b[48;2;185;37;27m▀\x1b[38;2;54;14;14m\x1b[48;2;34;15;15m▀\x1b[38;2;189;19;27m\x1b[48;2;114;47;12m▀\x1b[38;2;116;11;20m\x1b[48;2;106;50;10m▀\x1b[38;2;149;13;24m\x1b[48;2;116;58;10m▀\x1b[38;2;145;8;22m\x1b[48;2;96;64;6m▀\x1b[38;2;217;28;32m\x1b[48;2;76;24;10m▀\x1b[38;2;4;2;1m\x1b[48;2;11;0;3m▀\x1b[38;2;11;1;2m\x1b[48;2;204;34;32m▀\x1b[38;2;210;39;34m\x1b[48;2;247;63;41m▀\x1b[49m\x1b[38;2;252;68;42m▀\x1b[0m  ",
        "    \x1b[38;2;252;47;38m▀\x1b[38;2;214;33;34m\x1b[48;2;255;47;38m▀\x1b[38;2;24;2;3m\x1b[48;2;226;35;35m▀\x1b[38;2;123;10;5m\x1b[48;2;11;4;6m▀\x1b[38;2;227;22;0m\x1b[48;2;41;6;4m▀\x1b[38;2;255;39;0m\x1b[48;2;235;31;2m▀\x1b[38;2;250;55;0m\x1b[48;2;255;46;0m▀\x1b[38;2;176;41;12m\x1b[48;2;248;63;0m▀\x1b[38;2;111;15;19m\x1b[48;2;42;20;5m▀\x1b[38;2;116;18;18m\x1b[48;2;67;13;16m▀\x1b[38;2;118;20;19m\x1b[48;2;77;12;17m▀\x1b[38;2;112;9;19m\x1b[48;2;38;31;7m▀\x1b[38;2;178;75;14m\x1b[48;2;247;137;0m▀\x1b[38;2;249;159;0m\x1b[48;2;255;147;0m▀\x1b[38;2;255;174;0m\x1b[48;2;237;147;2m▀\x1b[38;2;228;168;1m\x1b[48;2;47;24;6m▀\x1b[38;2;126;85;7m\x1b[48;2;6;0;5m▀\x1b[38;2;23;12;4m\x1b[48;2;198;29;32m▀\x1b[38;2;195;30;31m\x1b[48;2;231;54;38m▀\x1b[49m\x1b[38;2;242;59;39m▀\x1b[0m    ",
        "      \x1b[38;2;255;47;38m▀\x1b[38;2;199;29;29m\x1b[48;2;201;36;30m▀\x1b[38;2;104;16;20m\x1b[48;2;205;29;32m▀\x1b[38;2;155;20;13m\x1b[48;2;130;14;18m▀\x1b[38;2;250;34;2m\x1b[48;2;117;14;4m▀\x1b[38;2;255;58;1m\x1b[48;2;255;34;10m▀\x1b[38;2;223;68;2m\x1b[48;2;255;75;0m▀\x1b[38;2;119;17;18m\x1b[48;2;183;56;6m▀\x1b[38;2;128;19;21m\x1b[48;2;181;71;6m▀\x1b[38;2;221;110;3m\x1b[48;2;255;119;1m▀\x1b[38;2;255;131;1m\x1b[48;2;255;97;12m▀\x1b[38;2;250;129;3m\x1b[48;2;120;53;6m▀\x1b[38;2;158;53;15m\x1b[48;2;128;8;19m▀\x1b[38;2;120;10;22m\x1b[48;2;223;33;34m▀\x1b[38;2;170;29;25m\x1b[48;2;202;45;32m▀\x1b[49m\x1b[38;2;226;52;36m▀\x1b[0m      ",
        "        \x1b[38;2;187;31;28m▀\x1b[38;2;168;24;27m\x1b[48;2;198;29;28m▀\x1b[38;2;3;2;2m\x1b[48;2;158;21;25m▀\x1b[38;2;156;16;10m\x1b[48;2;22;5;4m▀\x1b[38;2;246;36;14m\x1b[48;2;176;12;19m▀\x1b[38;2;255;90;0m\x1b[48;2;229;49;13m▀\x1b[38;2;255;106;0m\x1b[48;2;229;55;13m▀\x1b[38;2;249;66;16m\x1b[48;2;183;25;20m▀\x1b[38;2;165;47;13m\x1b[48;2;22;7;4m▀\x1b[38;2;2;0;2m\x1b[48;2;155;20;24m▀\x1b[38;2;168;24;26m\x1b[48;2;200;38;30m▀\x1b[49m\x1b[38;2;193;42;30m▀\x1b[0m        ",
        "          \x1b[38;2;199;28;28m▀\x1b[38;2;150;19;24m\x1b[48;2;196;25;28m▀\x1b[38;2;53;7;7m\x1b[48;2;144;18;22m▀\x1b[38;2;192;13;22m\x1b[48;2;101;11;13m▀\x1b[38;2;194;14;22m▀\x1b[38;2;54;8;7m\x1b[48;2;140;18;21m▀\x1b[38;2;144;18;22m\x1b[48;2;196;28;28m▀\x1b[49m\x1b[38;2;197;32;28m▀\x1b[0m          ",
        "            \x1b[38;2;193;24;27m▀\x1b[38;2;144;18;21m\x1b[48;2;183;22;24m▀\x1b[38;2;141;17;21m\x1b[48;2;184;21;25m▀\x1b[49m\x1b[38;2;193;27;28m▀\x1b[0m            ",
    ];

    if product_name.to_lowercase().contains("omen") && !product_name.to_lowercase().contains("victus") {
        ASCII_LOGO_OMEN
    } else {
        ASCII_LOGO_VICTUS
    }
}

pub async fn run_live_dashboard(conn: &zbus::Connection) -> Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, cursor::Hide)?;

    let platform = PlatformProxy::new(conn).await?;
    let fan = FanProxy::new(conn).await?;
    let power = PowerProxy::new(conn).await?;
    let rgb = RgbProxy::new(conn).await?;
    let mux = MuxProxy::new(conn).await.ok();
    let sysmon = SysMonProxy::new(conn).await?;

    let mut input = String::new();
    let mut logs: VecDeque<String> = VecDeque::with_capacity(3);
    logs.push_back(format!("\x1b[1;36m[SYSTEM]\x1b[0m {}", crate::i18n::t("ready")));

    let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
    let hostname = std::fs::read_to_string("/proc/sys/kernel/hostname")
        .unwrap_or_else(|_| "victus-laptop".to_string())
        .trim()
        .to_string();

    let mut last_tick = tokio::time::Instant::now();
    let mut need_redraw = true;

    // Initial state fetch
    let sys_json: Value = serde_json::from_str(&platform.get_hardware_dump_json().await.unwrap_or_default()).unwrap_or(Value::Null);
    let mut fan_json: Value = serde_json::from_str(&fan.get_fan_info().await.unwrap_or_default()).unwrap_or(Value::Null);
    let mut power_json: Value = serde_json::from_str(&power.get_power_profile().await.unwrap_or_default()).unwrap_or(Value::Null);
    let mut rgb_json: Value = serde_json::from_str(&rgb.get_state().await.unwrap_or_default()).unwrap_or(Value::Null);
    let mut state_json: Value = serde_json::from_str(&platform.get_state().await.unwrap_or_default()).unwrap_or(Value::Null);
    let conflicts_json: Value = serde_json::from_str(&platform.check_conflicts().await.unwrap_or_default()).unwrap_or(Value::Null);
    let mut sysmon_json: Value = serde_json::from_str(&sysmon.get_diagnostics().await.unwrap_or_default()).unwrap_or(Value::Null);

    let os_name = get_os_release_name().unwrap_or_else(|| "Linux".to_string());

    let prod_check = sys_json["system"]["product_name"].as_str().unwrap_or("Victus");
    let ascii_logo = get_ascii_logo(prod_check);
    loop {
        if need_redraw {
            need_redraw = false;

            let product_name = sys_json["system"]["product_name"].as_str().unwrap_or("HP Victus Gaming Laptop");
            let board_id = sys_json["system"]["board_id"].as_str().unwrap_or("8BBE");
            let kernel = sys_json["system"]["kernel"].as_str().unwrap_or("Linux");
            let cpu_name = sys_json["system"]["cpu_name"].as_str().unwrap_or("Processor");

            let power_active = power_json["active"].as_str().unwrap_or("balanced");
            let pl1 = power_json["pl1_w"].as_i64().unwrap_or(0);
            let pl2 = power_json["pl2_w"].as_i64().unwrap_or(0);
            let undervolt_mv = power_json["undervolt_mv"].as_i64().unwrap_or(0);
            let gpu_w = power_json["gpu_w"].as_u64().unwrap_or(0);

            let fan_mode = fan_json["mode"].as_str().unwrap_or("auto");
            let fan1_rpm = fan_json["fans"]["1"]["current"].as_u64().unwrap_or(0);
            let fan2_rpm = fan_json["fans"]["2"]["current"].as_u64().unwrap_or(0);

            let battery_limit = state_json["battery_charge_limit"].as_u64().unwrap_or(100);
            let rgb_mode = rgb_json["mode"].as_str().unwrap_or("static");
            let conflict_clean = !conflicts_json["has_conflicts"].as_bool().unwrap_or(false);

            let mux_mode = if let Some(ref m) = mux {
                let info_str = m.get_gpu_info().await.unwrap_or_default();
                let info_json: Value = serde_json::from_str(&info_str).unwrap_or(Value::Null);
                
                let mut display_mode = info_json["mode"].as_str().unwrap_or("Hybrid").to_string();
                // capitalize first letter
                if let Some(r) = display_mode.get_mut(0..1) {
                    r.make_ascii_uppercase();
                }
                display_mode
            } else {
                "Hybrid".to_string()
            };

            let user_host_header = format!("\x1b[1;38;2;0;200;255m{}\x1b[0m@\x1b[1;38;2;180;0;255m{}\x1b[0m", user, hostname);
            let separator = "\x1b[38;2;100;50;255m---------------------------------------\x1b[0m";

            let cpu_temp = sysmon_json["cpu_temp"].as_f64().unwrap_or(0.0).round() as u32;
            let gpu_temp = sysmon_json["gpu_temp"].as_f64().unwrap_or(0.0).round() as u32;
            let info_lines = vec![
                user_host_header,
                separator.to_string(),
                format!("\x1b[1;36m{}\x1b[0m: {} | \x1b[1;36m{}\x1b[0m: {}", crate::i18n::t("os"), os_name, crate::i18n::t("kernel"), kernel),
                format!("\x1b[1;36m{}\x1b[0m: {} ({})", crate::i18n::t("host"), product_name, board_id),
                format!("\x1b[1;36mCPU\x1b[0m: {}", cpu_name),
                format!("\x1b[1;36m{}\x1b[0m: \x1b[1;32m{}\x1b[0m [PL1: {}W / PL2: {}W]", crate::i18n::t("power_profile"), power_active, pl1, pl2),
                format!("\x1b[1;36m{}\x1b[0m: CPU: {}°C | GPU: {}°C | Fan1: {} RPM | Fan2: {} RPM [\x1b[33m{}\x1b[0m]", crate::i18n::t("thermal_fans"), cpu_temp, gpu_temp, fan1_rpm, fan2_rpm, fan_mode),
                format!("\x1b[1;36m{}\x1b[0m: \x1b[1;35m{}\x1b[0m | \x1b[1;31mTGP\x1b[0m: {}W | \x1b[1;36mUV\x1b[0m: {}mV", crate::i18n::t("gpu_mux"), mux_mode, gpu_w, undervolt_mv),
                format!("\x1b[1;36m{}\x1b[0m: {}% {} | \x1b[1;36mRGB\x1b[0m: {}", crate::i18n::t("battery_care"), battery_limit, crate::i18n::t("limit"), rgb_mode),
                format!("\x1b[1;36m{}\x1b[0m: {}", crate::i18n::t("conflicts"), if conflict_clean { format!("\x1b[32m{}\x1b[0m", crate::i18n::t("clean")) } else { format!("\x1b[31m{}\x1b[0m", crate::i18n::t("warning")) }),
                "\x1b[40m   \x1b[41m   \x1b[42m   \x1b[43m   \x1b[44m   \x1b[45m   \x1b[46m   \x1b[47m   \x1b[0m".to_string(),
                "\x1b[100m   \x1b[101m   \x1b[102m   \x1b[103m   \x1b[104m   \x1b[105m   \x1b[106m   \x1b[107m   \x1b[0m".to_string(),
            ];

            let mut row_idx: u16 = 1;
            let max_lines = ascii_logo.len().max(info_lines.len());
            for i in 0..max_lines {
                let logo_part = ascii_logo.get(i).copied().unwrap_or("                          ");
                let info_part = info_lines.get(i).map(|s| s.as_str()).unwrap_or("");
                execute!(out, cursor::MoveTo(2, row_idx))?;
                write!(out, "{}   {}\x1b[K", logo_part, info_part)?;
                row_idx += 1;
            }

            // Compact Cheatsheet
            execute!(out, cursor::MoveTo(0, row_idx))?;
            write!(out, "\x1b[1;30m--------------------------------------------------------------------------------\x1b[0m\x1b[K")?;
            row_idx += 1;

            execute!(out, cursor::MoveTo(2, row_idx))?;
            write!(out, "\x1b[1;36mFan\x1b[0m: \x1b[32mfan auto|ec|max|50\x1b[0m | \x1b[1;36mPower\x1b[0m: \x1b[32mperf perf|bal|eco\x1b[0m | \x1b[1;36mMUX\x1b[0m: \x1b[32mmux hybrid|discrete\x1b[0m\x1b[K")?;
            row_idx += 1;

            execute!(out, cursor::MoveTo(2, row_idx))?;
            write!(out, "\x1b[1;36mRGB\x1b[0m: \x1b[32mrgb red|blue|off\x1b[0m | \x1b[1;36mBat\x1b[0m: \x1b[32mbat 80\x1b[0m | \x1b[1;36mUtils\x1b[0m: \x1b[32muv -50\x1b[0m | \x1b[32mclean\x1b[0m | \x1b[32mdiag\x1b[0m | \x1b[32mexit\x1b[0m\x1b[K")?;
            row_idx += 1;

            // Notification / Log Area
            execute!(out, cursor::MoveTo(0, row_idx))?;
            write!(out, "\x1b[1;30m--------------------------------------------------------------------------------\x1b[0m\x1b[K")?;
            row_idx += 1;

            execute!(out, cursor::MoveTo(0, row_idx))?;
            write!(out, "\x1b[1;33mExecution Log & Notifications:\x1b[0m\x1b[K")?;
            row_idx += 1;

            for log in &logs {
                execute!(out, cursor::MoveTo(2, row_idx))?;
                write!(out, "{}\x1b[K", log)?;
                row_idx += 1;
            }

            execute!(out, cursor::MoveTo(0, row_idx))?;
            write!(out, "\x1b[1;30m--------------------------------------------------------------------------------\x1b[0m\x1b[K")?;
            row_idx += 1;

            execute!(out, cursor::MoveTo(0, row_idx))?;
            write!(out, "\x1b[1;36mvictus-max-cli\x1b[0m \x1b[1;32m>\x1b[0m {}\x1b[K", input)?;
            out.flush()?;
        }

        // Input Polling
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char(c) => {
                        input.push(c);
                        need_redraw = true;
                    }
                    KeyCode::Backspace => {
                        input.pop();
                        need_redraw = true;
                    }
                    KeyCode::Esc => break,
                    KeyCode::Enter => {
                        let cmd_str = input.trim().to_string();
                        input.clear();

                        if cmd_str == "exit" || cmd_str == "quit" {
                            break;
                        }

                        if !cmd_str.is_empty() {
                            let result_log = execute_input_command(&cmd_str, conn).await;
                            if logs.len() >= 3 { logs.pop_front(); }
                            logs.push_back(result_log);

                            // Refresh state immediately after command execution
                            fan_json = serde_json::from_str(&fan.get_fan_info().await.unwrap_or_default()).unwrap_or(Value::Null);
                            power_json = serde_json::from_str(&power.get_power_profile().await.unwrap_or_default()).unwrap_or(Value::Null);
                            rgb_json = serde_json::from_str(&rgb.get_state().await.unwrap_or_default()).unwrap_or(Value::Null);
                            state_json = serde_json::from_str(&platform.get_state().await.unwrap_or_default()).unwrap_or(Value::Null);
                            sysmon_json = serde_json::from_str(&sysmon.get_diagnostics().await.unwrap_or_default()).unwrap_or(Value::Null);
                            need_redraw = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Periodic telemetry refresh every 1s
        if last_tick.elapsed() >= Duration::from_millis(1000) {
            last_tick = tokio::time::Instant::now();
            fan_json = serde_json::from_str(&fan.get_fan_info().await.unwrap_or_default()).unwrap_or(Value::Null);
            power_json = serde_json::from_str(&power.get_power_profile().await.unwrap_or_default()).unwrap_or(Value::Null);
            rgb_json = serde_json::from_str(&rgb.get_state().await.unwrap_or_default()).unwrap_or(Value::Null);
            state_json = serde_json::from_str(&platform.get_state().await.unwrap_or_default()).unwrap_or(Value::Null);
            sysmon_json = serde_json::from_str(&sysmon.get_diagnostics().await.unwrap_or_default()).unwrap_or(Value::Null);
            need_redraw = true;
        }
    }

    disable_raw_mode()?;
    execute!(out, LeaveAlternateScreen, cursor::Show)?;
    Ok(())
}

pub async fn print_victus_fetch(conn: &zbus::Connection) -> Result<()> {
    let platform = PlatformProxy::new(conn).await?;
    let fan = FanProxy::new(conn).await?;
    let power = PowerProxy::new(conn).await?;
    let rgb = RgbProxy::new(conn).await?;
    let mux = MuxProxy::new(conn).await.ok();
    let sysmon = SysMonProxy::new(conn).await?;

    let sys_str = platform.get_hardware_dump_json().await.unwrap_or_default();
    let sys_json: Value = serde_json::from_str(&sys_str).unwrap_or(Value::Null);

    let fan_str = fan.get_fan_info().await.unwrap_or_default();
    let fan_json: Value = serde_json::from_str(&fan_str).unwrap_or(Value::Null);

    let power_str = power.get_power_profile().await.unwrap_or_default();
    let power_json: Value = serde_json::from_str(&power_str).unwrap_or(Value::Null);

    let rgb_str = rgb.get_state().await.unwrap_or_default();
    let rgb_json: Value = serde_json::from_str(&rgb_str).unwrap_or(Value::Null);

    let state_str = platform.get_state().await.unwrap_or_default();
    let state_json: Value = serde_json::from_str(&state_str).unwrap_or(Value::Null);

    let conflicts_str = platform.check_conflicts().await.unwrap_or_default();
    let conflicts_json: Value = serde_json::from_str(&conflicts_str).unwrap_or(Value::Null);

    let sysmon_str = sysmon.get_diagnostics().await.unwrap_or_default();
    let sysmon_json: Value = serde_json::from_str(&sysmon_str).unwrap_or(Value::Null);

    let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
    let hostname = std::fs::read_to_string("/proc/sys/kernel/hostname")
        .unwrap_or_else(|_| "victus-laptop".to_string())
        .trim()
        .to_string();

    let product_name = sys_json["system"]["product_name"].as_str().unwrap_or("HP Victus Gaming Laptop");
    let board_id = sys_json["system"]["board_id"].as_str().unwrap_or("8BBE");
    let kernel = sys_json["system"]["kernel"].as_str().unwrap_or("Linux");
    let cpu_name = sys_json["system"]["cpu_name"].as_str().unwrap_or("Processor");

    let os_name = get_os_release_name().unwrap_or_else(|| "Linux".to_string());

    let power_active = power_json["active"].as_str().unwrap_or("balanced");
    let pl1 = power_json["pl1_w"].as_i64().unwrap_or(0);
    let pl2 = power_json["pl2_w"].as_i64().unwrap_or(0);
    let undervolt_mv = power_json["undervolt_mv"].as_i64().unwrap_or(0);
    let gpu_w = power_json["gpu_w"].as_u64().unwrap_or(0);

    let fan_mode = fan_json["mode"].as_str().unwrap_or("auto");
    let fan1_rpm = fan_json["fans"]["1"]["current"].as_u64().unwrap_or(0);
    let fan2_rpm = fan_json["fans"]["2"]["current"].as_u64().unwrap_or(0);

    let battery_limit = state_json["battery_charge_limit"].as_u64().unwrap_or(100);

    let rgb_mode = rgb_json["mode"].as_str().unwrap_or("static");

    let conflict_clean = !conflicts_json["has_conflicts"].as_bool().unwrap_or(false);

    let mux_mode = if let Some(ref m) = mux {
        let info_str = m.get_gpu_info().await.unwrap_or_default();
        let info_json: Value = serde_json::from_str(&info_str).unwrap_or(Value::Null);
        
        let mut display_mode = info_json["mode"].as_str().unwrap_or("Hybrid").to_string();
        // capitalize first letter
        if let Some(r) = display_mode.get_mut(0..1) {
            r.make_ascii_uppercase();
        }
        display_mode
    } else {
        "Hybrid".to_string()
    };

    let user_host_header = format!("\x1b[1;38;2;0;200;255m{}\x1b[0m@\x1b[1;38;2;180;0;255m{}\x1b[0m", user, hostname);
    let separator = "\x1b[38;2;100;50;255m---------------------------------------\x1b[0m";

    let cpu_temp = sysmon_json["cpu_temp"].as_f64().unwrap_or(0.0).round() as u32;
    let gpu_temp = sysmon_json["gpu_temp"].as_f64().unwrap_or(0.0).round() as u32;
    let info_lines = vec![
        user_host_header,
        separator.to_string(),
        format!("\x1b[1;36m{}\x1b[0m: {} | \x1b[1;36m{}\x1b[0m: {}", crate::i18n::t("os"), os_name, crate::i18n::t("kernel"), kernel),
        format!("\x1b[1;36m{}\x1b[0m: {} ({})", crate::i18n::t("host"), product_name, board_id),
        format!("\x1b[1;36mCPU\x1b[0m: {}", cpu_name),
        format!("\x1b[1;36m{}\x1b[0m: \x1b[1;32m{}\x1b[0m [PL1: {}W / PL2: {}W]", crate::i18n::t("power_profile"), power_active, pl1, pl2),
        format!("\x1b[1;36m{}\x1b[0m: CPU: {}°C | GPU: {}°C | Fan1: {} RPM | Fan2: {} RPM [\x1b[33m{}\x1b[0m]", crate::i18n::t("thermal_fans"), cpu_temp, gpu_temp, fan1_rpm, fan2_rpm, fan_mode),
        format!("\x1b[1;36m{}\x1b[0m: \x1b[1;35m{}\x1b[0m | \x1b[1;31mTGP\x1b[0m: {}W | \x1b[1;36mUV\x1b[0m: {}mV", crate::i18n::t("gpu_mux"), mux_mode, gpu_w, undervolt_mv),
        format!("\x1b[1;36m{}\x1b[0m: {}% {} | \x1b[1;36mRGB\x1b[0m: {}", crate::i18n::t("battery_care"), battery_limit, crate::i18n::t("limit"), rgb_mode),
        format!("\x1b[1;36m{}\x1b[0m: {}", crate::i18n::t("conflicts"), if conflict_clean { format!("\x1b[32m{}\x1b[0m", crate::i18n::t("clean")) } else { format!("\x1b[31m{}\x1b[0m", crate::i18n::t("warning")) }),
        "\x1b[40m   \x1b[41m   \x1b[42m   \x1b[43m   \x1b[44m   \x1b[45m   \x1b[46m   \x1b[47m   \x1b[0m".to_string(),
        "\x1b[100m   \x1b[101m   \x1b[102m   \x1b[103m   \x1b[104m   \x1b[105m   \x1b[106m   \x1b[107m   \x1b[0m".to_string(),
    ];

    let ascii_logo = get_ascii_logo(product_name);
    println!();
    let max_lines = ascii_logo.len().max(info_lines.len());
    for i in 0..max_lines {
        let logo_part = ascii_logo.get(i).copied().unwrap_or("                          ");
        let info_part = info_lines.get(i).map(|s| s.as_str()).unwrap_or("");
        println!("{}   {}", logo_part, info_part);
    }
    println!();

    Ok(())
}

async fn execute_input_command(input: &str, conn: &zbus::Connection) -> String {
    let now = chrono::Local::now().format("%H:%M:%S");
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return String::new();
    }

    let fan = FanProxy::new(conn).await.ok();
    let power = PowerProxy::new(conn).await.ok();
    let platform = PlatformProxy::new(conn).await.ok();
    let rgb = RgbProxy::new(conn).await.ok();
    let mux = MuxProxy::new(conn).await.ok();

    match parts[0].to_lowercase().as_str() {
        "fan" => {
            if parts.len() < 2 {
                return format!("[{}] \x1b[33m{}\x1b[0m", now, crate::i18n::t("usage_fan"));
            }
            let sub = parts[1].to_lowercase();
            if let Some(f) = fan {
                if sub == "auto" || sub == "ec" || sub == "max" || sub == "custom" {
                    match f.set_fan_mode(&sub).await {
                        Ok(_) => format!("[{}] \x1b[1;32m{}\x1b[0m", now, crate::i18n::t("fan_changed").replace("{}", &sub.to_uppercase())),
                        Err(e) => format!("[{}] \x1b[1;31m{} {}\x1b[0m", now, crate::i18n::t("fan_error"), e),
                    }
                } else if let Ok(val) = sub.parse::<u32>() {
                    let rpm = if val <= 100 { val * 60 } else { val };
                    let _ = f.set_fan_mode("custom").await;
                    let r1 = f.set_fan_target(1, rpm).await;
                    let r2 = f.set_fan_target(2, rpm).await;
                    if r1.is_ok() || r2.is_ok() {
                        format!("[{}] \x1b[1;32m{}\x1b[0m", now, crate::i18n::t("fan_set").replacen("{}", &val.min(100).to_string(), 1).replacen("{}", &rpm.to_string(), 1))
                    } else {
                        format!("[{}] \x1b[1;31m{}\x1b[0m", now, crate::i18n::t("fan_set_failed"))
                    }
                } else {
                    format!("[{}] \x1b[1;31m{} {}\x1b[0m", now, crate::i18n::t("fan_invalid"), sub)
                }
            } else {
                format!("[{}] \x1b[1;31m{}\x1b[0m", now, crate::i18n::t("fan_no_service"))
            }
        }
        "perf" | "power" => {
            if parts.len() < 2 {
                return format!("[{}] \x1b[33m{}\x1b[0m", now, crate::i18n::t("usage_perf"));
            }
            let sub = parts[1].to_lowercase();
            if let Some(p) = power {
                if sub == "performance" || sub == "perf" {
                    match p.set_power_profile("performance").await {
                        Ok(_) => format!("[{}] \x1b[1;32m{}\x1b[0m", now, crate::i18n::t("perf_perf")),
                        Err(e) => format!("[{}] \x1b[1;31m{} {}\x1b[0m", now, crate::i18n::t("perf_error"), e),
                    }
                } else if sub == "balanced" || sub == "bal" {
                    match p.set_power_profile("balanced").await {
                        Ok(_) => format!("[{}] \x1b[1;32m{}\x1b[0m", now, crate::i18n::t("perf_bal")),
                        Err(e) => format!("[{}] \x1b[1;31m{} {}\x1b[0m", now, crate::i18n::t("perf_error"), e),
                    }
                } else if sub == "eco" || sub == "quiet" || sub == "power-saver" || sub == "saver" {
                    match p.set_power_profile("power-saver").await {
                        Ok(_) => format!("[{}] \x1b[1;32m{}\x1b[0m", now, crate::i18n::t("perf_eco")),
                        Err(e) => format!("[{}] \x1b[1;31m{} {}\x1b[0m", now, crate::i18n::t("perf_error"), e),
                    }
                } else if let (Ok(pl1), Some(pl2)) = (sub.parse::<i32>(), parts.get(2).and_then(|s| s.parse::<i32>().ok())) {
                    match p.set_power_limits(true, pl1, pl2).await {
                        Ok(_) => format!("[{}] \x1b[1;32m{}\x1b[0m", now, crate::i18n::t("perf_limits").replacen("{}", &pl1.to_string(), 1).replacen("{}", &pl2.to_string(), 1)),
                        Err(e) => format!("[{}] \x1b[1;31mPower limits error: {}\x1b[0m", now, e),
                    }
                } else {
                    format!("[{}] \x1b[1;31mUnknown power profile: {}\x1b[0m", now, sub)
                }
            } else {
                format!("[{}] \x1b[1;31m{}\x1b[0m", now, crate::i18n::t("perf_no_service"))
            }
        }
        "bat" | "battery" => {
            if parts.len() < 2 {
                return format!("[{}] \x1b[33m{}\x1b[0m", now, crate::i18n::t("usage_bat"));
            }
            if let Ok(limit) = parts[1].parse::<u32>() {
                if let Some(p) = platform {
                    match p.set_battery_care(limit).await {
                        Ok(_) => format!("[{}] \x1b[1;32m{}\x1b[0m", now, crate::i18n::t("bat_set").replacen("{}", &limit.clamp(50, 100).to_string(), 1)),
                        Err(e) => format!("[{}] \x1b[1;31mBattery care error: {}\x1b[0m", now, e),
                    }
                } else {
                    format!("[{}] \x1b[1;31m{}\x1b[0m", now, crate::i18n::t("bat_no_service"))
                }
            } else {
                format!("[{}] \x1b[1;31mInvalid battery limit (expected 50-100)\x1b[0m", now)
            }
        }
        "mux" => {
            if parts.len() < 2 {
                if let Some(m) = mux {
                    let info_str = m.get_gpu_info().await.unwrap_or_default();
                    let info_json: Value = serde_json::from_str(&info_str).unwrap_or(Value::Null);
                    let mode = info_json["mode"].as_str().unwrap_or("Unknown").to_string();
                    return format!("[{}] \x1b[36mCurrent GPU MUX Mode: {}\x1b[0m", now, mode);
                }
                return format!("[{}] \x1b[33m{}\x1b[0m", now, crate::i18n::t("usage_mux"));
            }
            let mode = parts[1].to_lowercase();
            if let Some(m) = mux {
                match m.set_gpu_mode(&mode).await {
                    Ok(res) => format!("[{}] \x1b[1;32m{}\x1b[0m", now, crate::i18n::t("mux_set").replacen("{}", &mode, 1).replacen("{}", &res, 1)),
                    Err(e) => format!("[{}] \x1b[1;31m{} {}\x1b[0m", now, crate::i18n::t("mux_error"), e),
                }
            } else {
                format!("[{}] \x1b[1;31m{}\x1b[0m", now, crate::i18n::t("mux_no_service"))
            }
        }
        "uv" | "undervolt" => {
            if parts.len() < 2 {
                return format!("[{}] \x1b[33mUsage: uv <mv_offset> (e.g. uv -50)\x1b[0m", now);
            }
            if let Ok(mv) = parts[1].parse::<i32>() {
                if let Some(p) = power {
                    match p.set_undervolt(mv).await {
                        Ok(_) => format!("[{}] \x1b[1;32m{}\x1b[0m", now, crate::i18n::t("uv_set").replacen("{}", &mv.to_string(), 1)),
                        Err(e) => format!("[{}] \x1b[1;31m{} {}\x1b[0m", now, crate::i18n::t("uv_error"), e),
                    }
                } else {
                    format!("[{}] \x1b[1;31m{}\x1b[0m", now, crate::i18n::t("perf_no_service"))
                }
            } else {
                format!("[{}] \x1b[1;31mInvalid offset (e.g. -50)\x1b[0m", now)
            }
        }
        "rgb" => {
            if parts.len() < 2 {
                return format!("[{}] \x1b[33mUsage: rgb on | off | red | blue | green | white | hex\x1b[0m", now);
            }
            let color_arg = parts[1].to_lowercase();
            if let Some(r) = rgb {
                if color_arg == "off" {
                    match r.set_global(false, 0, "right").await {
                        Ok(_) => format!("[{}] \x1b[1;32m{}\x1b[0m", now, crate::i18n::t("rgb_off")),
                        Err(e) => format!("[{}] \x1b[1;31m{} {}\x1b[0m", now, crate::i18n::t("rgb_error"), e),
                    }
                } else if color_arg == "on" {
                    match r.set_global(true, 100, "right").await {
                        Ok(_) => format!("[{}] \x1b[1;32m{}\x1b[0m", now, crate::i18n::t("rgb_on")),
                        Err(e) => format!("[{}] \x1b[1;31m{} {}\x1b[0m", now, crate::i18n::t("rgb_error"), e),
                    }
                } else {
                    let hex = match color_arg.as_str() {
                        "red" => "FF0000",
                        "green" => "00FF00",
                        "blue" => "0000FF",
                        "white" => "FFFFFF",
                        "yellow" => "FFFF00",
                        "cyan" => "00FFFF",
                        "magenta" | "purple" => "FF00FF",
                        "orange" => "FF8800",
                        other => other.trim_start_matches('#'),
                    };
                    match r.set_color(8, hex).await {
                        Ok(_) => format!("[{}] \x1b[1;32m{}\x1b[0m", now, crate::i18n::t("rgb_color").replacen("{}", hex, 1)),
                        Err(e) => format!("[{}] \x1b[1;31m{} {}\x1b[0m", now, crate::i18n::t("rgb_error"), e),
                    }
                }
            } else {
                format!("[{}] \x1b[1;31m{}\x1b[0m", now, crate::i18n::t("rgb_no_service"))
            }
        }
        "clean" => {
            if let Some(p) = platform {
                match p.clean_memory().await {
                    Ok(_) => format!("[{}] \x1b[1;32m{}\x1b[0m", now, crate::i18n::t("cache_cleared")),
                    Err(e) => format!("[{}] \x1b[1;31mMemory clean error: {}\x1b[0m", now, e),
                }
            } else {
                format!("[{}] \x1b[1;31m{}\x1b[0m", now, crate::i18n::t("bat_no_service"))
            }
        }
        "diag" | "diagnostics" => {
            if let Some(p) = platform {
                let _ = p.run_wmi_diagnostics().await;
                format!("[{}] \x1b[1;32m{}\x1b[0m", now, crate::i18n::t("diag_initiated"))
            } else {
                format!("[{}] \x1b[1;31m{}\x1b[0m", now, crate::i18n::t("bat_no_service"))
            }
        }
        "triage" => {
            if let Some(p) = platform {
                match p.generate_triage_bundle().await {
                    Ok(path) => format!("[{}] \x1b[1;32m{}\x1b[0m", now, crate::i18n::t("diag_bundle").replacen("{}", &path, 1)),
                    Err(e) => format!("[{}] \x1b[1;31mTriage error: {}\x1b[0m", now, e),
                }
            } else {
                format!("[{}] \x1b[1;31m{}\x1b[0m", now, crate::i18n::t("bat_no_service"))
            }
        }
        "help" => {
            format!("[{}] \x1b[1;33mShortcuts:\x1b[0m fan auto|ec|max|50 | perf perf|bal|eco | mux hybrid|discrete | uv -50 | bat 80 | rgb red|off | exit", now)
        }
        _ => {
            // Fallback to clap subcommand parser
            use clap::Parser;
            use crate::Cli;
            use crate::run_command;

            let args = format!("victus-max-cli {}", input);
            let args_vec = match shlex::split(&args) {
                Some(v) => v,
                None => return format!("[{}] \x1b[31mInvalid input quotation\x1b[0m", now),
            };

            match Cli::try_parse_from(args_vec) {
                Ok(parsed_cli) => {
                    if let Some(cmd) = &parsed_cli.command {
                        match run_command(cmd, conn).await {
                            Ok(_) => format!("[{}] \x1b[1;32m{}\x1b[0m", now, crate::i18n::t("executed").replacen("{}", input, 1)),
                            Err(e) => format!("[{}] \x1b[1;31mError executing '{}': {}\x1b[0m", now, input, e),
                        }
                    } else {
                        format!("[{}] \x1b[33mType 'help' to list commands\x1b[0m", now)
                    }
                }
                Err(e) => {
                    format!("[{}] \x1b[31mError: {}\x1b[0m", now, e.render().to_string().replace("\n", " "))
                }
            }
        }
    }
}

fn get_os_release_name() -> Option<String> {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return Some(line.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string());
            }
        }
    }
    None
}
