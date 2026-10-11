use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use crate::i18n;

/* ─────────────────────────────────────────────────────────────
   undervolt.rs — Intel 13th Gen Undervolt & Power Limits
   ───────────────────────────────────────────────────────────── */

fn is_cpu_locked(cpu: &str) -> bool {
    let name_part = cpu.split('·').next().unwrap_or(cpu).to_uppercase();
    if name_part.contains("AMD") { return false; } // AMD handled by ryzenadj in daemon
    if !name_part.contains("INTEL") { return false; }
    
    // Check for 12, 13, 14th gen
    if name_part.contains("-12") || name_part.contains("-13") || name_part.contains("-14") 
        || name_part.contains(" 12") || name_part.contains(" 13") || name_part.contains(" 14") {
        if name_part.contains("HK") || name_part.contains("HX") {
            return false;
        }
        if name_part.contains("H") || name_part.contains("P") || name_part.contains("U") {
            return true;
        }
    }
    // Check for Core Ultra
    if name_part.contains("ULTRA") {
        return true;
    }
    false
}

pub fn build_page() -> gtk::Box {
    let page = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(16)
        .build();

    // ── Header ───────────────────────────────────────────────
    let hdr = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .margin_bottom(2)
        .build();
    hdr.append(&gtk::Label::builder()
        .label(i18n::t("title_undervolt"))
        .css_classes(["page-title"])
        .halign(gtk::Align::Start)
        .build());
    hdr.append(&gtk::Label::builder()
        .label(i18n::t("uv_desc"))
        .css_classes(["os-section-desc"])
        .halign(gtk::Align::Start)
        .build());
    page.append(&hdr);

    // ── Protection / Warning Banner ───────────────────────────
    let warn_card = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .css_classes(["os-card"])
        .build();

    let warn_icon = gtk::Image::builder()
        .icon_name("dialog-warning-symbolic")
        .pixel_size(24)
        .valign(gtk::Align::Center)
        .build();
    warn_card.append(&warn_icon);

    let warn_text_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();
    let warn_title = gtk::Label::builder()
        .label(i18n::t("uv_lock_title"))
        .css_classes(["chip-title"])
        .halign(gtk::Align::Start)
        .build();
    warn_text_box.append(&warn_title);
    
    let warn_desc = gtk::Label::builder()
        .label(i18n::t("uv_lock_desc"))
        .css_classes(["os-section-desc"])
        .halign(gtk::Align::Start)
        .wrap(true)
        .build();
    warn_text_box.append(&warn_desc);
    
    warn_card.append(&warn_text_box);

    let status_badge = gtk::Label::builder()
        .label(i18n::t("msr_protected"))
        .css_classes(["badge-warn"])
        .valign(gtk::Align::Center)
        .build();
    warn_card.append(&status_badge);

    page.append(&warn_card);

    // ── Voltage Offsets Section ───────────────────────────────
    let volt_group = adw::PreferencesGroup::builder()
        .title(i18n::t("voltage_offsets_group"))
        .description(i18n::t("voltage_offsets_desc"))
        .build();

    // 1. Core Voltage Offset — initialize then load from daemon
    let core_val = Rc::new(RefCell::new(0i32));
    let core_row = adw::ActionRow::builder()
        .title(i18n::t("core_offset_title"))
        .subtitle(i18n::t("core_offset_sub"))
        .build();
    let core_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, -150.0, 0.0, 5.0);
    core_scale.set_value(0.0);
    core_scale.set_hexpand(true);
    core_scale.set_size_request(200, -1);
    core_scale.set_valign(gtk::Align::Center);
    let core_lbl = gtk::Label::builder()
        .label("0 mV")
        .css_classes(["os-monitor-val-sm"])
        .valign(gtk::Align::Center)
        .margin_start(8)
        .build();
    let core_lbl_clone = core_lbl.clone();
    let core_val_clone = core_val.clone();
    core_scale.connect_value_changed(move |s| {
        let val = s.value() as i32;
        *core_val_clone.borrow_mut() = val;
        core_lbl_clone.set_label(&format!("{} mV", val));
    });
    core_row.add_suffix(&core_scale);
    core_row.add_suffix(&core_lbl);
    volt_group.add(&core_row);

    // 2. CPU Cache (Ring) Offset
    let cache_row = adw::ActionRow::builder()
        .title(i18n::t("cache_offset_title"))
        .subtitle(i18n::t("cache_offset_sub"))
        .build();
    let cache_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, -150.0, 0.0, 5.0);
    cache_scale.set_value(0.0);
    cache_scale.set_hexpand(true);
    cache_scale.set_size_request(200, -1);
    cache_scale.set_valign(gtk::Align::Center);
    let cache_lbl = gtk::Label::builder()
        .label("0 mV")
        .css_classes(["os-monitor-val-sm"])
        .valign(gtk::Align::Center)
        .margin_start(8)
        .build();
    let cache_lbl_clone = cache_lbl.clone();
    cache_scale.connect_value_changed(move |s| {
        let val = s.value() as i32;
        cache_lbl_clone.set_label(&format!("{} mV", val));
    });
    cache_row.add_suffix(&cache_scale);
    cache_row.add_suffix(&cache_lbl);
    volt_group.add(&cache_row);

    page.append(&volt_group);

    // Shared runtime flags & defaults for callbacks
    let is_amd_flag = Rc::new(RefCell::new(false));
    let stapm_fast_linked_flag = Rc::new(RefCell::new(false));
    let syncing_sliders = Rc::new(Cell::new(false));
    let defaults_state = Rc::new(RefCell::new((45.0f64, 45.0f64, 95.0f64, 0.0f64)));

    // ── Power Limits (Intel RAPL / AMD Ryzen PPT & STAPM) ──────
    let pwr_group = adw::PreferencesGroup::builder()
        .title(i18n::t("power_limits_title"))
        .description(i18n::t("power_limits_desc"))
        .build();

    // PL1 / STAPM (Sürekli Güç Limiti)
    let pl1_row = adw::ActionRow::builder()
        .title(i18n::t("pl1_label"))
        .subtitle(i18n::t("pl1_sub"))
        .build();
    let pl1_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 25.0, 75.0, 1.0);
    pl1_scale.set_value(45.0);
    pl1_scale.set_hexpand(true);
    pl1_scale.set_size_request(200, -1);
    pl1_scale.set_valign(gtk::Align::Center);
    let pl1_lbl = gtk::Label::builder()
        .label("45 W")
        .css_classes(["os-monitor-val-sm"])
        .valign(gtk::Align::Center)
        .margin_start(8)
        .build();
    pl1_row.add_suffix(&pl1_scale);
    pl1_row.add_suffix(&pl1_lbl);
    pwr_group.add(&pl1_row);

    // PPT Slow (AMD Ryzen Uzun Süreli Paket Güç Limiti — Intel'de gizli)
    let slow_row = adw::ActionRow::builder()
        .title(i18n::t("slow_ppt_label"))
        .subtitle(i18n::t("slow_ppt_sub"))
        .visible(false)
        .build();
    let slow_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 25.0, 75.0, 1.0);
    slow_scale.set_value(45.0);
    slow_scale.set_hexpand(true);
    slow_scale.set_size_request(200, -1);
    slow_scale.set_valign(gtk::Align::Center);
    let slow_lbl = gtk::Label::builder()
        .label("45 W")
        .css_classes(["os-monitor-val-sm"])
        .valign(gtk::Align::Center)
        .margin_start(8)
        .build();
    slow_row.add_suffix(&slow_scale);
    slow_row.add_suffix(&slow_lbl);
    pwr_group.add(&slow_row);

    // PL2 / PPT Fast (Kısa Süreli Turbo Güç Limiti)
    let pl2_row = adw::ActionRow::builder()
        .title(i18n::t("pl2_label"))
        .subtitle(i18n::t("pl2_sub"))
        .build();
    let pl2_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 45.0, 115.0, 1.0);
    pl2_scale.set_value(95.0);
    pl2_scale.set_hexpand(true);
    pl2_scale.set_size_request(200, -1);
    pl2_scale.set_valign(gtk::Align::Center);
    let pl2_lbl = gtk::Label::builder()
        .label("95 W")
        .css_classes(["os-monitor-val-sm"])
        .valign(gtk::Align::Center)
        .margin_start(8)
        .build();
    pl2_row.add_suffix(&pl2_scale);
    pl2_row.add_suffix(&pl2_lbl);
    pwr_group.add(&pl2_row);

    // Relational slider synchronization across PL1/STAPM, PPT Slow, and PL2/PPT Fast
    let pl1_lbl_c = pl1_lbl.clone();
    let slow_s_sync1 = slow_scale.clone();
    let pl2_s_sync1 = pl2_scale.clone();
    let is_amd_c1 = is_amd_flag.clone();
    let stapm_linked_c1 = stapm_fast_linked_flag.clone();
    let sync_c1 = syncing_sliders.clone();
    pl1_scale.connect_value_changed(move |s| {
        let val = s.value() as i32;
        pl1_lbl_c.set_label(&format!("{} W", val));

        if !sync_c1.get() {
            sync_c1.set(true);
            let v = s.value().round();
            if *is_amd_c1.borrow() && *stapm_linked_c1.borrow() {
                pl2_s_sync1.set_value(v);
                if slow_s_sync1.value() > v {
                    slow_s_sync1.set_value(v);
                }
            } else if *is_amd_c1.borrow() {
                if slow_s_sync1.value() < v {
                    slow_s_sync1.set_value(v);
                }
                if pl2_s_sync1.value() < slow_s_sync1.value() {
                    pl2_s_sync1.set_value(slow_s_sync1.value());
                }
            } else {
                if pl2_s_sync1.value() < v {
                    pl2_s_sync1.set_value(v);
                }
            }
            sync_c1.set(false);
        }
    });

    let slow_lbl_c = slow_lbl.clone();
    let pl1_s_sync2 = pl1_scale.clone();
    let pl2_s_sync2 = pl2_scale.clone();
    let is_amd_c2 = is_amd_flag.clone();
    let stapm_linked_c2 = stapm_fast_linked_flag.clone();
    let sync_c2 = syncing_sliders.clone();
    slow_scale.connect_value_changed(move |s| {
        let val = s.value() as i32;
        slow_lbl_c.set_label(&format!("{} W", val));

        if !sync_c2.get() {
            sync_c2.set(true);
            let v = s.value().round();
            if *is_amd_c2.borrow() && *stapm_linked_c2.borrow() {
                if pl2_s_sync2.value() < v {
                    pl2_s_sync2.set_value(v);
                    pl1_s_sync2.set_value(v);
                }
            } else if *is_amd_c2.borrow() {
                if pl1_s_sync2.value() > v {
                    pl1_s_sync2.set_value(v);
                }
                if pl2_s_sync2.value() < v {
                    pl2_s_sync2.set_value(v);
                }
            }
            sync_c2.set(false);
        }
    });

    let pl2_lbl_c = pl2_lbl.clone();
    let pl1_s_sync3 = pl1_scale.clone();
    let slow_s_sync3 = slow_scale.clone();
    let is_amd_c3 = is_amd_flag.clone();
    let stapm_linked_c3 = stapm_fast_linked_flag.clone();
    let sync_c3 = syncing_sliders.clone();
    pl2_scale.connect_value_changed(move |s| {
        let val = s.value() as i32;
        pl2_lbl_c.set_label(&format!("{} W", val));

        if !sync_c3.get() {
            sync_c3.set(true);
            let v = s.value().round();
            if *is_amd_c3.borrow() && *stapm_linked_c3.borrow() {
                pl1_s_sync3.set_value(v);
                if slow_s_sync3.value() > v {
                    slow_s_sync3.set_value(v);
                }
            } else if *is_amd_c3.borrow() {
                if slow_s_sync3.value() > v {
                    slow_s_sync3.set_value(v);
                }
                if pl1_s_sync3.value() > slow_s_sync3.value() {
                    pl1_s_sync3.set_value(slow_s_sync3.value());
                }
            } else {
                if pl1_s_sync3.value() > v {
                    pl1_s_sync3.set_value(v);
                }
            }
            sync_c3.set(false);
        }
    });

    // TCC Offset / AMD Tctl Temperature Limit
    let tcc_row = adw::ActionRow::builder()
        .title(i18n::t("tcc_label"))
        .subtitle(i18n::t("tcc_sub"))
        .build();
    let tcc_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 15.0, 1.0);
    tcc_scale.set_value(3.0);
    tcc_scale.set_hexpand(true);
    tcc_scale.set_size_request(200, -1);
    tcc_scale.set_valign(gtk::Align::Center);
    let tcc_lbl = gtk::Label::builder()
        .label(format!("97°C ({}: 3°C)", i18n::t("tcc_offset_str")))
        .css_classes(["os-monitor-val-sm"])
        .valign(gtk::Align::Center)
        .margin_start(8)
        .build();
    let tcc_lbl_clone = tcc_lbl.clone();
    let is_amd_tcc = is_amd_flag.clone();
    tcc_scale.connect_value_changed(move |s| {
        let val = s.value() as i32;
        if *is_amd_tcc.borrow() {
            tcc_lbl_clone.set_label(&format!("{}°C", val));
        } else {
            let max_temp = 100 - val;
            tcc_lbl_clone.set_label(&format!("{}°C ({}: {}°C)", max_temp, i18n::t("tcc_offset_str"), val));
        }
    });
    tcc_row.add_suffix(&tcc_scale);
    tcc_row.add_suffix(&tcc_lbl);
    pwr_group.add(&tcc_row);

    page.append(&pwr_group);

    // ── Action Buttons Row ────────────────────────────────────
    let actions_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::End)
        .margin_top(4)
        .margin_bottom(12)
        .build();

    let reset_btn = gtk::Button::builder()
        .label(i18n::t("reset_defaults"))
        .css_classes(["ec-btn"])
        .build();
    let core_s_c = core_scale.clone();
    let cache_s_c = cache_scale.clone();
    let pl1_s_c = pl1_scale.clone();
    let slow_s_c = slow_scale.clone();
    let pl2_s_c = pl2_scale.clone();
    let tcc_s_c = tcc_scale.clone();
    let defaults_reset = defaults_state.clone();
    let sync_reset = syncing_sliders.clone();
    reset_btn.connect_clicked(move |_| {
        let (def_pl1, def_slow, def_pl2, def_tcc) = *defaults_reset.borrow();
        sync_reset.set(true);
        core_s_c.set_value(0.0);
        cache_s_c.set_value(0.0);
        pl1_s_c.set_value(def_pl1);
        slow_s_c.set_value(def_slow);
        pl2_s_c.set_value(def_pl2);
        tcc_s_c.set_value(def_tcc);
        sync_reset.set(false);
    });

    let apply_btn = gtk::Button::builder()
        .label(i18n::t("apply_save"))
        .css_classes(["suggested-action"])
        .build();

    let core_val_c = core_val.clone();
    let pl1_s_apply = pl1_scale.clone();
    let slow_s_apply = slow_scale.clone();
    let pl2_s_apply = pl2_scale.clone();
    let cache_s_apply = cache_scale.clone();
    let tcc_s_apply = tcc_scale.clone();
    let is_amd_apply = is_amd_flag.clone();
    apply_btn.connect_clicked(move |_| {
        let core_mv = *core_val_c.borrow();
        let cache_mv = cache_s_apply.value() as i32;
        let pl1 = pl1_s_apply.value() as i32;
        let slow = slow_s_apply.value() as i32;
        let pl2 = pl2_s_apply.value() as i32;
        let tcc = tcc_s_apply.value() as i32;
        crate::daemon_client::apply_all_tuning_sync(
            *is_amd_apply.borrow(),
            core_mv,
            cache_mv,
            pl1,
            slow,
            pl2,
            tcc,
        );
    });

    actions_box.append(&reset_btn);
    actions_box.append(&apply_btn);

    page.append(&actions_box);

    // ── Load current state from daemon (async on startup + tab switch) ────────
    let core_s_load = core_scale.clone();
    let cache_s_load = cache_scale.clone();
    let pl1_s_load = pl1_scale.clone();
    let slow_s_load = slow_scale.clone();
    let pl2_s_load = pl2_scale.clone();
    let tcc_s_load = tcc_scale.clone();
    let pl1_lbl_load = pl1_lbl.clone();
    let slow_lbl_load = slow_lbl.clone();
    let pl2_lbl_load = pl2_lbl.clone();
    let tcc_lbl_load = tcc_lbl.clone();
    let badge_load = status_badge.clone();
    
    let cache_row_clone = cache_row.clone();
    let core_row_clone = core_row.clone();
    let core_scale_range = core_scale.clone();
    
    let pwr_group_clone = pwr_group.clone();
    let pl1_row_clone = pl1_row.clone();
    let slow_row_clone = slow_row.clone();
    let pl2_row_clone = pl2_row.clone();
    let tcc_row_clone = tcc_row.clone();

    let warn_title_clone = warn_title.clone();
    let warn_desc_clone = warn_desc.clone();
    let volt_group_clone = volt_group.clone();
    let is_amd_load = is_amd_flag.clone();
    let defaults_load = defaults_state.clone();
    let linked_load = stapm_fast_linked_flag.clone();
    let sync_load = syncing_sliders.clone();

    let refresh_state = std::rc::Rc::new(move || {
        let core_s_load = core_s_load.clone();
        let cache_s_load = cache_s_load.clone();
        let pl1_s_load = pl1_s_load.clone();
        let slow_s_load = slow_s_load.clone();
        let pl2_s_load = pl2_s_load.clone();
        let tcc_s_load = tcc_s_load.clone();
        let pl1_lbl_load = pl1_lbl_load.clone();
        let slow_lbl_load = slow_lbl_load.clone();
        let pl2_lbl_load = pl2_lbl_load.clone();
        let tcc_lbl_load = tcc_lbl_load.clone();
        let badge_load = badge_load.clone();
        let cache_row_clone = cache_row_clone.clone();
        let core_row_clone = core_row_clone.clone();
        let core_scale_range = core_scale_range.clone();
        let pwr_group_clone = pwr_group_clone.clone();
        let pl1_row_clone = pl1_row_clone.clone();
        let slow_row_clone = slow_row_clone.clone();
        let pl2_row_clone = pl2_row_clone.clone();
        let tcc_row_clone = tcc_row_clone.clone();
        let warn_title_clone = warn_title_clone.clone();
        let warn_desc_clone = warn_desc_clone.clone();
        let volt_group_clone = volt_group_clone.clone();
        let is_amd_load = is_amd_load.clone();
        let defaults_load = defaults_load.clone();
        let linked_load = linked_load.clone();
        let sync_load = sync_load.clone();

        glib::spawn_future_local(async move {
            // First check CPU lock & vendor from hardware specs
            let mut locked = false;
            let mut is_amd = false;
            if let Ok(hw_json) = crate::daemon_client::get_hardware_specs_async().await {
                if let Ok(hw) = serde_json::from_str::<crate::daemon_client::HardwareSpecs>(&hw_json) {
                    let cpu_upper = hw.cpu_spec.to_uppercase();
                    is_amd = cpu_upper.contains("AMD") || cpu_upper.contains("RYZEN");
                    if is_cpu_locked(&hw.cpu_spec) {
                        locked = true;
                        warn_title_clone.set_label(i18n::t("uv_unsupported_title"));
                        warn_desc_clone.set_label(&i18n::t("uv_unsupported_desc").replace("{}", &hw.cpu_spec));
                        badge_load.set_label("Locked");
                        badge_load.set_css_classes(&["badge-err"]);
                        
                        volt_group_clone.set_sensitive(false);
                    }
                }
            }

            let mut uv_tcc_offset: Option<f64> = None;
            let mut uv_tctl_temp: Option<f64> = None;

            if let Ok(json) = crate::daemon_client::get_undervolt_state_async().await {
                if let Ok(state) = serde_json::from_str::<serde_json::Value>(&json) {
                    if let Some(amd_flag) = state.get("is_amd").and_then(|v| v.as_bool()) {
                        is_amd = amd_flag;
                    }

                    if is_amd {
                        warn_title_clone.set_label("AMD Curve Optimizer & PBO");
                        if let Some(w) = state.get("warning").and_then(|v| v.as_str()) {
                            warn_desc_clone.set_label(w);
                            badge_load.set_label("RyzenAdj Missing");
                            badge_load.set_css_classes(&["badge-err"]);
                        } else {
                            warn_desc_clone.set_label("RyzenAdj is managing Curve Optimizer and Power Limits.");
                            badge_load.set_label("RyzenAdj OK");
                            badge_load.set_css_classes(&["badge-ok"]);
                        }
                        core_scale_range.set_range(-30.0, 30.0);
                        cache_row_clone.set_visible(false);
                        core_row_clone.set_title("Curve Optimizer (All Core)");
                        core_row_clone.set_subtitle("Negative values = Undervolt. Range: -30 to +30.");
                    } else if !locked {
                        badge_load.set_label("MSR OK");
                        badge_load.set_css_classes(&["badge-ok"]);
                    }

                    if let Some(core) = state.get("offsets").and_then(|v| v.get("core")).and_then(|v| v.as_f64()) {
                        core_s_load.set_value(core);
                    }
                    if let Some(core) = state.get("core").and_then(|v| v.as_f64()) {
                        core_s_load.set_value(core);
                    }
                    
                    if let Some(cache) = state.get("offsets").and_then(|v| v.get("cache")).and_then(|v| v.as_f64()) {
                        cache_s_load.set_value(cache);
                    }
                    if let Some(cache) = state.get("cache").and_then(|v| v.as_f64()) {
                        cache_s_load.set_value(cache);
                    }

                    uv_tcc_offset = state.get("tcc_offset").and_then(|v| v.as_f64());
                    uv_tctl_temp = state.get("tctl_temp").and_then(|v| v.as_f64());
                }
            }
            
            // Load Power Limits & adaptive ranges from Power service
            if let Ok(info) = crate::daemon_client::get_power_profile_info_async().await {
                if info.is_amd {
                    is_amd = true;
                }
                *is_amd_load.borrow_mut() = is_amd;

                let stapm_fast_linked = is_amd
                    && (info.stapm_fast_linked
                        || info
                            .ranges
                            .as_ref()
                            .map(|r| r.stapm_fast_linked)
                            .unwrap_or(false));
                *linked_load.borrow_mut() = stapm_fast_linked;

                let ranges = info.ranges.unwrap_or(crate::daemon_client::CpuPowerProfileRanges {
                    sustained_min: 25,
                    sustained_max: 75,
                    sustained_default: 45,
                    burst_min: if is_amd { 35 } else { 45 },
                    burst_max: if is_amd { 95 } else { 115 },
                    burst_default: if is_amd { 65 } else { 95 },
                    temp_min: if is_amd { 75 } else { 85 },
                    temp_max: 100,
                    temp_default: if is_amd { 95 } else { 97 },
                    ..Default::default()
                });

                sync_load.set(true);

                if is_amd {
                    pwr_group_clone.set_title(i18n::t("ryzen_power_limits_title"));

                    if stapm_fast_linked {
                        pl1_s_load.set_range(ranges.sustained_min as f64, ranges.burst_max as f64);
                        pwr_group_clone.set_description(Some(i18n::t("ryzen_power_limits_desc_linked")));
                        pl1_row_clone.set_title(i18n::t("stapm_linked_label"));
                        pl1_row_clone.set_subtitle(&format!(
                            "{} ({}W | {}W - {}W)",
                            i18n::t("stapm_linked_sub"),
                            ranges.burst_default,
                            ranges.sustained_min,
                            ranges.burst_max
                        ));
                        *defaults_load.borrow_mut() = (
                            ranges.burst_default as f64,
                            ranges.sustained_default as f64,
                            ranges.burst_default as f64,
                            ranges.temp_default as f64,
                        );
                    } else {
                        pl1_s_load.set_range(ranges.sustained_min as f64, ranges.sustained_max as f64);
                        pwr_group_clone.set_description(Some(i18n::t("ryzen_power_limits_desc_hierarchy")));
                        pl1_row_clone.set_title(i18n::t("stapm_label"));
                        pl1_row_clone.set_subtitle(&format!(
                            "{} ({}W | {}W - {}W)",
                            i18n::t("stapm_sub"),
                            ranges.sustained_default,
                            ranges.sustained_min,
                            ranges.sustained_max
                        ));
                        *defaults_load.borrow_mut() = (
                            ranges.sustained_default as f64,
                            ranges.sustained_default as f64,
                            ranges.burst_default as f64,
                            ranges.temp_default as f64,
                        );
                    }

                    slow_s_load.set_range(ranges.sustained_min as f64, ranges.sustained_max as f64);
                    pl2_s_load.set_range(ranges.burst_min as f64, ranges.burst_max as f64);

                    slow_row_clone.set_visible(true);
                    slow_row_clone.set_title(i18n::t("slow_ppt_label"));
                    slow_row_clone.set_subtitle(&format!(
                        "{} ({}W | {}W - {}W)",
                        i18n::t("slow_ppt_sub"),
                        ranges.sustained_default,
                        ranges.sustained_min,
                        ranges.sustained_max
                    ));

                    pl2_row_clone.set_title(i18n::t("fast_ppt_label"));
                    pl2_row_clone.set_subtitle(&format!(
                        "{} ({}W | {}W - {}W)",
                        i18n::t("fast_ppt_sub"),
                        ranges.burst_default,
                        ranges.burst_min,
                        ranges.burst_max
                    ));

                    tcc_row_clone.set_title(i18n::t("tctl_temp_label"));
                    tcc_row_clone.set_subtitle(i18n::t("tctl_temp_sub"));
                    tcc_s_load.set_range(ranges.temp_min as f64, ranges.temp_max as f64);

                    let target_fast = if info.fast_ppt_w > 0 {
                        info.fast_ppt_w
                    } else if info.pl2_w > 0 {
                        info.pl2_w
                    } else {
                        ranges.burst_default
                    };
                    let target_stapm = if stapm_fast_linked {
                        target_fast
                    } else if info.stapm_w > 0 {
                        info.stapm_w
                    } else if info.pl1_w > 0 {
                        info.pl1_w
                    } else {
                        ranges.sustained_default
                    };
                    let target_slow = if info.slow_ppt_w > 0 {
                        info.slow_ppt_w
                    } else if stapm_fast_linked {
                        ranges.sustained_default.min(target_fast)
                    } else {
                        target_stapm
                    };
                    let target_tctl = if info.tctl_temp > 0 {
                        info.tctl_temp as f64
                    } else if let Some(t) = uv_tctl_temp {
                        t
                    } else {
                        ranges.temp_default as f64
                    };

                    pl1_s_load.set_value(target_stapm as f64);
                    slow_s_load.set_value(target_slow as f64);
                    pl2_s_load.set_value(target_fast as f64);
                    tcc_s_load.set_value(target_tctl);

                    pl1_lbl_load.set_label(&format!("{} W", pl1_s_load.value() as i32));
                    slow_lbl_load.set_label(&format!("{} W", slow_s_load.value() as i32));
                    pl2_lbl_load.set_label(&format!("{} W", pl2_s_load.value() as i32));
                    tcc_lbl_load.set_label(&format!("{}°C", tcc_s_load.value() as i32));
                } else {
                    pl1_s_load.set_range(ranges.sustained_min as f64, ranges.sustained_max as f64);
                    slow_s_load.set_range(ranges.sustained_min as f64, ranges.sustained_max as f64);
                    pl2_s_load.set_range(ranges.burst_min as f64, ranges.burst_max as f64);

                    slow_row_clone.set_visible(false);
                    *defaults_load.borrow_mut() = (
                        ranges.sustained_default as f64,
                        ranges.sustained_default as f64,
                        ranges.burst_default as f64,
                        0.0,
                    );

                    let target_pl1 = if info.pl1_w > 0 { info.pl1_w } else { ranges.sustained_default };
                    let target_pl2 = if info.pl2_w > 0 { info.pl2_w } else { ranges.burst_default };
                    pl1_s_load.set_value(target_pl1 as f64);
                    pl2_s_load.set_value(target_pl2 as f64);
                    if let Some(tcc) = uv_tcc_offset {
                        tcc_s_load.set_value(tcc);
                    }
                    pl1_lbl_load.set_label(&format!("{} W", pl1_s_load.value() as i32));
                    pl2_lbl_load.set_label(&format!("{} W", pl2_s_load.value() as i32));
                }

                sync_load.set(false);
            } else {
                // Fallback if power service call fails
                *is_amd_load.borrow_mut() = is_amd;
                *linked_load.borrow_mut() = false;
                sync_load.set(true);
                if is_amd {
                    pwr_group_clone.set_title(i18n::t("ryzen_power_limits_title"));
                    pwr_group_clone.set_description(Some(i18n::t("ryzen_power_limits_desc_hierarchy")));
                    pl1_row_clone.set_title(i18n::t("stapm_label"));
                    slow_row_clone.set_visible(true);
                    slow_row_clone.set_title(i18n::t("slow_ppt_label"));
                    pl2_row_clone.set_title(i18n::t("fast_ppt_label"));
                    tcc_row_clone.set_title(i18n::t("tctl_temp_label"));
                    tcc_row_clone.set_subtitle(i18n::t("tctl_temp_sub"));
                    tcc_s_load.set_range(75.0, 100.0);
                    let target_tctl = uv_tctl_temp.unwrap_or(95.0);
                    tcc_s_load.set_value(target_tctl);
                    tcc_lbl_load.set_label(&format!("{}°C", tcc_s_load.value() as i32));
                    *defaults_load.borrow_mut() = (45.0, 45.0, 65.0, 95.0);
                } else if let Some(tcc) = uv_tcc_offset {
                    tcc_s_load.set_value(tcc);
                }
                sync_load.set(false);
            }
        });
    });

    refresh_state();
    let refresh_on_map = refresh_state.clone();
    page.connect_map(move |_| {
        refresh_on_map();
    });

    page
}

#[cfg(test)]
mod tests {
    struct SlidersSim {
        is_amd: bool,
        stapm_fast_linked: bool,
        pl1: f64,
        slow: f64,
        pl2: f64,
    }

    impl SlidersSim {
        fn new(is_amd: bool, stapm_fast_linked: bool, pl1: f64, slow: f64, pl2: f64) -> Self {
            Self {
                is_amd,
                stapm_fast_linked,
                pl1,
                slow,
                pl2,
            }
        }

        fn on_pl1_changed(&mut self, val: f64) {
            let v = val.round();
            self.pl1 = v;
            if self.is_amd && self.stapm_fast_linked {
                self.pl2 = v;
                if self.slow > v {
                    self.slow = v;
                }
            } else if self.is_amd {
                if self.slow < v {
                    self.slow = v;
                }
                if self.pl2 < self.slow {
                    self.pl2 = self.slow;
                }
            } else {
                if self.pl2 < v {
                    self.pl2 = v;
                }
            }
        }

        fn on_slow_changed(&mut self, val: f64) {
            let v = val.round();
            self.slow = v;
            if self.is_amd && self.stapm_fast_linked {
                if self.pl2 < v {
                    self.pl2 = v;
                    self.pl1 = v;
                }
            } else if self.is_amd {
                if self.pl1 > v {
                    self.pl1 = v;
                }
                if self.pl2 < v {
                    self.pl2 = v;
                }
            }
        }

        fn on_pl2_changed(&mut self, val: f64) {
            let v = val.round();
            self.pl2 = v;
            if self.is_amd && self.stapm_fast_linked {
                self.pl1 = v;
                if self.slow > v {
                    self.slow = v;
                }
            } else if self.is_amd {
                if self.slow > v {
                    self.slow = v;
                }
                if self.pl1 > self.slow {
                    self.pl1 = self.slow;
                }
            } else {
                if self.pl1 > v {
                    self.pl1 = v;
                }
            }
        }
    }

    #[test]
    fn test_zen4_5_stapm_fast_coupling() {
        let mut sim = SlidersSim::new(true, true, 65.0, 45.0, 65.0);

        // STAPM changed pushes Fast PPT to match, Slow PPT stays if <= v
        sim.on_pl1_changed(80.0);
        assert_eq!(sim.pl1, 80.0);
        assert_eq!(sim.pl2, 80.0);
        assert_eq!(sim.slow, 45.0);

        // PPT Slow changed above Fast/STAPM pushes both to match
        sim.on_slow_changed(85.0);
        assert_eq!(sim.slow, 85.0);
        assert_eq!(sim.pl2, 85.0);
        assert_eq!(sim.pl1, 85.0);

        // Fast PPT changed down pulls STAPM to match, and clamps Slow PPT
        sim.on_pl2_changed(50.0);
        assert_eq!(sim.pl2, 50.0);
        assert_eq!(sim.pl1, 50.0);
        assert_eq!(sim.slow, 50.0);

        // STAPM changed down pulls Fast PPT to match and clamps Slow PPT
        sim.on_pl1_changed(35.0);
        assert_eq!(sim.pl1, 35.0);
        assert_eq!(sim.pl2, 35.0);
        assert_eq!(sim.slow, 35.0);
    }

    #[test]
    fn test_zen2_3_hierarchy() {
        let mut sim = SlidersSim::new(true, false, 35.0, 45.0, 65.0);

        // Moving STAPM up pushes Slow PPT up, which in turn pushes Fast PPT if exceeded
        sim.on_pl1_changed(50.0);
        assert_eq!(sim.pl1, 50.0);
        assert_eq!(sim.slow, 50.0);
        assert_eq!(sim.pl2, 65.0);

        sim.on_pl1_changed(75.0);
        assert_eq!(sim.pl1, 75.0);
        assert_eq!(sim.slow, 75.0);
        assert_eq!(sim.pl2, 75.0);

        // Moving Fast PPT down pulls Slow PPT down, which pulls STAPM down
        sim.on_pl2_changed(60.0);
        assert_eq!(sim.pl2, 60.0);
        assert_eq!(sim.slow, 60.0);
        assert_eq!(sim.pl1, 60.0);

        // Moving Slow PPT down pulls STAPM down
        sim.on_slow_changed(40.0);
        assert_eq!(sim.slow, 40.0);
        assert_eq!(sim.pl1, 40.0);
        assert_eq!(sim.pl2, 60.0);

        // Moving Slow PPT up pushes Fast PPT up
        sim.on_slow_changed(70.0);
        assert_eq!(sim.slow, 70.0);
        assert_eq!(sim.pl1, 40.0);
        assert_eq!(sim.pl2, 70.0);
    }

    #[test]
    fn test_intel_pl1_pl2_hierarchy() {
        let mut sim = SlidersSim::new(false, false, 45.0, 45.0, 95.0);

        // Moving PL1 above PL2 pushes PL2 up
        sim.on_pl1_changed(105.0);
        assert_eq!(sim.pl1, 105.0);
        assert_eq!(sim.pl2, 105.0);

        // Moving PL2 below PL1 pulls PL1 down
        sim.on_pl2_changed(60.0);
        assert_eq!(sim.pl2, 60.0);
        assert_eq!(sim.pl1, 60.0);
    }
}
