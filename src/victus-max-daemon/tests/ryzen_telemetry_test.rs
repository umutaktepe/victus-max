#[path = "../src/cpu_tier.rs"]
pub mod cpu_tier;

use cpu_tier::parse_ryzenadj_info_output;

#[test]
fn test_parse_ryzenadj_info() {
    let sample = r#"
CPU Family: Phoenix Point
SMU BIOS Interface Version: 18
Version: v0.19.0 
PM Table Version: 4c0008
|        Name         |   Value   |     Parameter      |
|---------------------|-----------|--------------------|
| STAPM LIMIT         |    45.000 | stapm-limit        |
| STAPM VALUE         |    14.231 |                    |
| PPT LIMIT FAST      |    65.000 | fast-limit         |
| PPT VALUE FAST      |    14.300 |                    |
| PPT LIMIT SLOW      |    54.000 | slow-limit         |
| PPT VALUE SLOW      |    14.280 |                    |
| StapmTimeConst      |   275.000 | stapm-time         |
| SlowPPTTimeConst    |     5.000 | slow-time          |
| PPT LIMIT APU       |    35.000 | apu-slow-limit     |
| THM LIMIT CORE      |    95.000 | tctl-temp          |
| THM VALUE CORE      |    52.400 |                    |
"#;
    let metrics = parse_ryzenadj_info_output(sample);
    assert_eq!(metrics.stapm_w, Some(45));
    assert_eq!(metrics.fast_limit_w, Some(65));
    assert_eq!(metrics.slow_limit_w, Some(54));
    assert_eq!(metrics.tctl_temp_c, Some(95));
}

#[test]
fn test_parse_ryzenadj_info_with_nan_and_millwatts() {
    let sample = r#"
| STAPM LIMIT         |       nan | stapm-limit        |
| PPT LIMIT FAST      | 80000.000 | fast-limit         |
| PPT LIMIT SLOW      |    45.400 | slow-limit         |
| THM LIMIT CORE      |       nan | tctl-temp          |
"#;
    let metrics = parse_ryzenadj_info_output(sample);
    assert_eq!(metrics.stapm_w, None);
    // If a version outputs milliwatts (> 1000), normalize to Watts
    assert_eq!(metrics.fast_limit_w, Some(80));
    assert_eq!(metrics.slow_limit_w, Some(45));
    assert_eq!(metrics.tctl_temp_c, None);
}
