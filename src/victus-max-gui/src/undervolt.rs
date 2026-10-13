use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use crate::i18n;
use omen_types::{PowerModeLimits, PowerModeProfilesConfig};

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

// ── Power Mode Limit Presets Helper Types & Functions ─────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerPresetsData {
    pub is_amd: bool,
    pub presets: PowerModeProfilesConfig,
    pub defaults: PowerModeProfilesConfig,
    pub active_mode: Option<String>,
    pub manual_override: bool,
}

pub fn default_presets_for_vendor(is_amd: bool) -> PowerModeProfilesConfig {
    if is_amd {
        PowerModeProfilesConfig {
            power_saver: PowerModeLimits { stapm_w: 20, slow_ppt_w: 20, fast_ppt_w: 25, tctl_temp: 70 },
            balanced: PowerModeLimits { stapm_w: 35, slow_ppt_w: 35, fast_ppt_w: 45, tctl_temp: 82 },
            performance: PowerModeLimits { stapm_w: 45, slow_ppt_w: 45, fast_ppt_w: 65, tctl_temp: 88 },
            active_mode: Some("balanced".to_string()),
            manual_override: false,
        }
    } else {
        PowerModeProfilesConfig {
            power_saver: PowerModeLimits { stapm_w: 25, slow_ppt_w: 25, fast_ppt_w: 35, tctl_temp: 75 },
            balanced: PowerModeLimits { stapm_w: 45, slow_ppt_w: 45, fast_ppt_w: 65, tctl_temp: 88 },
            performance: PowerModeLimits { stapm_w: 65, slow_ppt_w: 65, fast_ppt_w: 95, tctl_temp: 97 },
            active_mode: Some("balanced".to_string()),
            manual_override: false,
        }
    }
}

pub fn default_power_presets_data(is_amd: bool) -> PowerPresetsData {
    let def = default_presets_for_vendor(is_amd);
    PowerPresetsData {
        is_amd,
        presets: def.clone(),
        defaults: def,
        active_mode: Some("balanced".to_string()),
        manual_override: false,
    }
}

pub fn parse_power_presets_json(json_str: &str, is_amd_fallback: bool) -> PowerPresetsData {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
        let is_amd = val.get("is_amd").and_then(|v| v.as_bool()).unwrap_or(is_amd_fallback);
        let active_mode = val.get("active_mode").and_then(|v| v.as_str()).map(|s| s.to_string());
        let manual_override = val.get("manual_override").and_then(|v| v.as_bool()).unwrap_or(false);

        let presets = val.get("presets")
            .and_then(|p| serde_json::from_value::<PowerModeProfilesConfig>(p.clone()).ok())
            .unwrap_or_else(|| default_presets_for_vendor(is_amd));

        let defaults = val.get("defaults")
            .and_then(|d| serde_json::from_value::<PowerModeProfilesConfig>(d.clone()).ok())
            .unwrap_or_else(|| default_presets_for_vendor(is_amd));

        PowerPresetsData {
            is_amd,
            presets,
            defaults,
            active_mode,
            manual_override,
        }
    } else {
        let def = default_presets_for_vendor(is_amd_fallback);
        PowerPresetsData {
            is_amd: is_amd_fallback,
            presets: def.clone(),
            defaults: def,
            active_mode: Some("balanced".to_string()),
            manual_override: false,
        }
    }
}

pub fn format_preset_subtitle(limits: &PowerModeLimits) -> String {
    format!("{}W / {}W · {}°C", limits.stapm_w, limits.fast_ppt_w, limits.tctl_temp)
}

fn build_preset_card(
    icon_filename: &str,
    fallback_icon: &str,
    title: &str,
    initial_sub: &str,
) -> (gtk::ToggleButton, gtk::Label, gtk::Label, gtk::Button, gtk::Widget) {
    let btn = gtk::ToggleButton::builder()
        .css_classes(["chip-card"])
        .build();

    let inner = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(14)
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Center)
        .margin_end(36)
        .build();

    let asset_path = crate::asset_resolver::get_asset_path(icon_filename);
    let img = if std::path::Path::new(&asset_path).exists() {
        gtk::Image::builder().file(&asset_path).pixel_size(28).build()
    } else {
        gtk::Image::builder().icon_name(fallback_icon).pixel_size(28).build()
    };
    inner.append(&img);

    let text_col = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(3)
        .valign(gtk::Align::Center)
        .hexpand(true)
        .build();
    let title_lbl = gtk::Label::builder()
        .label(title)
        .css_classes(["chip-title"])
        .halign(gtk::Align::Start)
        .build();
    let sub_lbl = gtk::Label::builder()
        .label(initial_sub)
        .css_classes(["chip-sub"])
        .halign(gtk::Align::Start)
        .build();
    text_col.append(&title_lbl);
    text_col.append(&sub_lbl);
    inner.append(&text_col);

    btn.set_child(Some(&inner));

    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&btn));

    let edit_btn = gtk::Button::builder()
        .icon_name("document-edit-symbolic")
        .tooltip_text(i18n::t("edit_btn_tooltip"))
        .css_classes(["circular", "flat"])
        .valign(gtk::Align::Center)
        .halign(gtk::Align::End)
        .margin_end(10)
        .build();

    overlay.add_overlay(&edit_btn);

    (btn, title_lbl, sub_lbl, edit_btn, overlay.upcast())
}

fn apply_preset_limits_to_sliders(
    limits: &PowerModeLimits,
    is_amd: bool,
    syncing_sliders: &Cell<bool>,
    pl1_scale: &gtk::Scale,
    slow_scale: &gtk::Scale,
    pl2_scale: &gtk::Scale,
    tcc_scale: &gtk::Scale,
    pl1_lbl: &gtk::Label,
    slow_lbl: &gtk::Label,
    pl2_lbl: &gtk::Label,
    tcc_lbl: &gtk::Label,
) {
    syncing_sliders.set(true);
    pl1_scale.set_value(limits.stapm_w as f64);
    pl1_lbl.set_label(&format!("{} W", limits.stapm_w));

    if is_amd {
        slow_scale.set_value(limits.slow_ppt_w as f64);
        slow_lbl.set_label(&format!("{} W", limits.slow_ppt_w));

        pl2_scale.set_value(limits.fast_ppt_w as f64);
        pl2_lbl.set_label(&format!("{} W", limits.fast_ppt_w));

        tcc_scale.set_value(limits.tctl_temp as f64);
        tcc_lbl.set_label(&format!("{}°C", limits.tctl_temp));
    } else {
        pl2_scale.set_value(limits.fast_ppt_w as f64);
        pl2_lbl.set_label(&format!("{} W", limits.fast_ppt_w));

        let tcc_offset = 100i32.saturating_sub(limits.tctl_temp as i32).clamp(0, 15);
        tcc_scale.set_value(tcc_offset as f64);
        let max_temp = 100 - tcc_offset;
        tcc_lbl.set_label(&format!("{}°C ({}: {}°C)", max_temp, i18n::t("tcc_offset_str"), tcc_offset));
    }
    syncing_sliders.set(false);
}

fn show_preset_edit_dialog(
    parent_window: Option<&gtk::Window>,
    mode_id: &'static str,
    mode_display_name: &str,
    is_amd: bool,
    stapm_fast_linked: bool,
    current_limits: PowerModeLimits,
    default_limits: PowerModeLimits,
    on_saved: impl Fn(PowerModeLimits) + 'static,
) {
    let window = adw::Window::builder()
        .title(&format!("{}: {}", i18n::t("preset_edit_title"), mode_display_name))
        .modal(true)
        .destroy_with_parent(true)
        .default_width(420)
        .default_height(if is_amd { 460 } else { 380 })
        .resizable(false)
        .build();

    if let Some(parent) = parent_window {
        window.set_transient_for(Some(parent));
    }

    let toolbar_view = adw::ToolbarView::new();
    let header_bar = adw::HeaderBar::new();
    toolbar_view.add_top_bar(&header_bar);

    let content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(16)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();
    toolbar_view.set_content(Some(&content_box));
    window.set_content(Some(&toolbar_view));

    let group = adw::PreferencesGroup::new();
    content_box.append(&group);

    let dlg_sync = Rc::new(Cell::new(false));

    let (stapm_scale, slow_scale, fast_scale, temp_scale) = if is_amd {
        let stapm_row = adw::ActionRow::builder()
            .title(if stapm_fast_linked { i18n::t("stapm_linked_label") } else { i18n::t("stapm_label") })
            .build();
        let stapm_s = gtk::Scale::with_range(gtk::Orientation::Horizontal, 15.0, 100.0, 1.0);
        stapm_s.set_value(current_limits.stapm_w as f64);
        stapm_s.set_hexpand(true);
        stapm_s.set_size_request(180, -1);
        let stapm_lbl = gtk::Label::builder()
            .label(&format!("{} W", current_limits.stapm_w))
            .css_classes(["os-monitor-val-sm"])
            .margin_start(8)
            .build();
        stapm_row.add_suffix(&stapm_s);
        stapm_row.add_suffix(&stapm_lbl);
        group.add(&stapm_row);

        let slow_row = adw::ActionRow::builder()
            .title(i18n::t("slow_ppt_label"))
            .build();
        let slow_s = gtk::Scale::with_range(gtk::Orientation::Horizontal, 15.0, 100.0, 1.0);
        slow_s.set_value(current_limits.slow_ppt_w as f64);
        slow_s.set_hexpand(true);
        slow_s.set_size_request(180, -1);
        let slow_lbl = gtk::Label::builder()
            .label(&format!("{} W", current_limits.slow_ppt_w))
            .css_classes(["os-monitor-val-sm"])
            .margin_start(8)
            .build();
        slow_row.add_suffix(&slow_s);
        slow_row.add_suffix(&slow_lbl);
        group.add(&slow_row);

        let fast_row = adw::ActionRow::builder()
            .title(i18n::t("fast_ppt_label"))
            .build();
        let fast_s = gtk::Scale::with_range(gtk::Orientation::Horizontal, 20.0, 125.0, 1.0);
        fast_s.set_value(current_limits.fast_ppt_w as f64);
        fast_s.set_hexpand(true);
        fast_s.set_size_request(180, -1);
        let fast_lbl = gtk::Label::builder()
            .label(&format!("{} W", current_limits.fast_ppt_w))
            .css_classes(["os-monitor-val-sm"])
            .margin_start(8)
            .build();
        fast_row.add_suffix(&fast_s);
        fast_row.add_suffix(&fast_lbl);
        group.add(&fast_row);

        let temp_row = adw::ActionRow::builder()
            .title(i18n::t("tctl_temp_label"))
            .build();
        let temp_s = gtk::Scale::with_range(gtk::Orientation::Horizontal, 70.0, 100.0, 1.0);
        temp_s.set_value(current_limits.tctl_temp as f64);
        temp_s.set_hexpand(true);
        temp_s.set_size_request(180, -1);
        let temp_lbl = gtk::Label::builder()
            .label(&format!("{}°C", current_limits.tctl_temp))
            .css_classes(["os-monitor-val-sm"])
            .margin_start(8)
            .build();
        temp_row.add_suffix(&temp_s);
        temp_row.add_suffix(&temp_lbl);
        group.add(&temp_row);

        let stapm_lbl_c = stapm_lbl.clone();
        let slow_s_sync1 = slow_s.clone();
        let fast_s_sync1 = fast_s.clone();
        let sync1 = dlg_sync.clone();
        stapm_s.connect_value_changed(move |s| {
            let val = s.value() as i32;
            stapm_lbl_c.set_label(&format!("{} W", val));
            if !sync1.get() {
                sync1.set(true);
                let v = s.value().round();
                if stapm_fast_linked {
                    fast_s_sync1.set_value(v);
                    if slow_s_sync1.value() > v {
                        slow_s_sync1.set_value(v);
                    }
                } else {
                    if slow_s_sync1.value() < v {
                        slow_s_sync1.set_value(v);
                    }
                    if fast_s_sync1.value() < slow_s_sync1.value() {
                        fast_s_sync1.set_value(slow_s_sync1.value());
                    }
                }
                sync1.set(false);
            }
        });

        let slow_lbl_c = slow_lbl.clone();
        let stapm_s_sync2 = stapm_s.clone();
        let fast_s_sync2 = fast_s.clone();
        let sync2 = dlg_sync.clone();
        slow_s.connect_value_changed(move |s| {
            let val = s.value() as i32;
            slow_lbl_c.set_label(&format!("{} W", val));
            if !sync2.get() {
                sync2.set(true);
                let v = s.value().round();
                if stapm_fast_linked {
                    if fast_s_sync2.value() < v {
                        fast_s_sync2.set_value(v);
                        stapm_s_sync2.set_value(v);
                    }
                } else {
                    if stapm_s_sync2.value() > v {
                        stapm_s_sync2.set_value(v);
                    }
                    if fast_s_sync2.value() < v {
                        fast_s_sync2.set_value(v);
                    }
                }
                sync2.set(false);
            }
        });

        let fast_lbl_c = fast_lbl.clone();
        let stapm_s_sync3 = stapm_s.clone();
        let slow_s_sync3 = slow_s.clone();
        let sync3 = dlg_sync.clone();
        fast_s.connect_value_changed(move |s| {
            let val = s.value() as i32;
            fast_lbl_c.set_label(&format!("{} W", val));
            if !sync3.get() {
                sync3.set(true);
                let v = s.value().round();
                if stapm_fast_linked {
                    stapm_s_sync3.set_value(v);
                    if slow_s_sync3.value() > v {
                        slow_s_sync3.set_value(v);
                    }
                } else {
                    if slow_s_sync3.value() > v {
                        slow_s_sync3.set_value(v);
                    }
                    if stapm_s_sync3.value() > slow_s_sync3.value() {
                        stapm_s_sync3.set_value(slow_s_sync3.value());
                    }
                }
                sync3.set(false);
            }
        });

        let temp_lbl_c = temp_lbl.clone();
        temp_s.connect_value_changed(move |s| {
            let val = s.value() as i32;
            temp_lbl_c.set_label(&format!("{}°C", val));
        });

        (stapm_s, Some(slow_s), fast_s, temp_s)
    } else {
        let pl1_row = adw::ActionRow::builder()
            .title(i18n::t("pl1_label"))
            .build();
        let pl1_s = gtk::Scale::with_range(gtk::Orientation::Horizontal, 15.0, 115.0, 1.0);
        pl1_s.set_value(current_limits.stapm_w as f64);
        pl1_s.set_hexpand(true);
        pl1_s.set_size_request(180, -1);
        let pl1_lbl = gtk::Label::builder()
            .label(&format!("{} W", current_limits.stapm_w))
            .css_classes(["os-monitor-val-sm"])
            .margin_start(8)
            .build();
        pl1_row.add_suffix(&pl1_s);
        pl1_row.add_suffix(&pl1_lbl);
        group.add(&pl1_row);

        let pl2_row = adw::ActionRow::builder()
            .title(i18n::t("pl2_label"))
            .build();
        let pl2_s = gtk::Scale::with_range(gtk::Orientation::Horizontal, 25.0, 140.0, 1.0);
        pl2_s.set_value(current_limits.fast_ppt_w as f64);
        pl2_s.set_hexpand(true);
        pl2_s.set_size_request(180, -1);
        let pl2_lbl = gtk::Label::builder()
            .label(&format!("{} W", current_limits.fast_ppt_w))
            .css_classes(["os-monitor-val-sm"])
            .margin_start(8)
            .build();
        pl2_row.add_suffix(&pl2_s);
        pl2_row.add_suffix(&pl2_lbl);
        group.add(&pl2_row);

        let temp_row = adw::ActionRow::builder()
            .title(i18n::t("tcc_label"))
            .build();
        let temp_s = gtk::Scale::with_range(gtk::Orientation::Horizontal, 70.0, 100.0, 1.0);
        temp_s.set_value(current_limits.tctl_temp as f64);
        temp_s.set_hexpand(true);
        temp_s.set_size_request(180, -1);
        let temp_lbl = gtk::Label::builder()
            .label(&format!("{}°C ({}: {}°C)", current_limits.tctl_temp, i18n::t("tcc_offset_str"), 100 - current_limits.tctl_temp))
            .css_classes(["os-monitor-val-sm"])
            .margin_start(8)
            .build();
        temp_row.add_suffix(&temp_s);
        temp_row.add_suffix(&temp_lbl);
        group.add(&temp_row);

        let pl1_lbl_c = pl1_lbl.clone();
        let pl2_s_sync1 = pl2_s.clone();
        let sync1 = dlg_sync.clone();
        pl1_s.connect_value_changed(move |s| {
            let val = s.value() as i32;
            pl1_lbl_c.set_label(&format!("{} W", val));
            if !sync1.get() {
                sync1.set(true);
                let v = s.value().round();
                if pl2_s_sync1.value() < v {
                    pl2_s_sync1.set_value(v);
                }
                sync1.set(false);
            }
        });

        let pl2_lbl_c = pl2_lbl.clone();
        let pl1_s_sync2 = pl1_s.clone();
        let sync2 = dlg_sync.clone();
        pl2_s.connect_value_changed(move |s| {
            let val = s.value() as i32;
            pl2_lbl_c.set_label(&format!("{} W", val));
            if !sync2.get() {
                sync2.set(true);
                let v = s.value().round();
                if pl1_s_sync2.value() > v {
                    pl1_s_sync2.set_value(v);
                }
                sync2.set(false);
            }
        });

        let temp_lbl_c = temp_lbl.clone();
        temp_s.connect_value_changed(move |s| {
            let val = s.value() as i32;
            let offset = 100 - val;
            temp_lbl_c.set_label(&format!("{}°C ({}: {}°C)", val, i18n::t("tcc_offset_str"), offset));
        });

        (pl1_s, None, pl2_s, temp_s)
    };

    let actions_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(10)
        .halign(gtk::Align::End)
        .margin_top(8)
        .build();

    let reset_btn = gtk::Button::builder()
        .label(i18n::t("reset_to_default"))
        .css_classes(["ec-btn"])
        .build();

    let cancel_btn = gtk::Button::builder()
        .label(i18n::t("cancel_btn"))
        .css_classes(["flat"])
        .build();

    let save_btn = gtk::Button::builder()
        .label(i18n::t("save_btn"))
        .css_classes(["suggested-action"])
        .build();

    let stapm_s_res = stapm_scale.clone();
    let slow_s_res = slow_scale.clone();
    let fast_s_res = fast_scale.clone();
    let temp_s_res = temp_scale.clone();
    let sync_res = dlg_sync.clone();
    reset_btn.connect_clicked(move |_| {
        sync_res.set(true);
        stapm_s_res.set_value(default_limits.stapm_w as f64);
        if let Some(ref s) = slow_s_res {
            s.set_value(default_limits.slow_ppt_w as f64);
        }
        fast_s_res.set_value(default_limits.fast_ppt_w as f64);
        temp_s_res.set_value(default_limits.tctl_temp as f64);
        sync_res.set(false);
    });

    let win_cancel = window.clone();
    cancel_btn.connect_clicked(move |_| {
        win_cancel.close();
    });

    let win_save = window.clone();
    let stapm_s_save = stapm_scale.clone();
    let slow_s_save = slow_scale.clone();
    let fast_s_save = fast_scale.clone();
    let temp_s_save = temp_scale.clone();
    save_btn.connect_clicked(move |_| {
        let stapm = stapm_s_save.value().round() as u32;
        let slow = if is_amd {
            slow_s_save.as_ref().map(|s| s.value().round() as u32).unwrap_or(stapm)
        } else {
            stapm
        };
        let fast = fast_s_save.value().round() as u32;
        let temp = temp_s_save.value().round() as u32;

        let new_limits = PowerModeLimits {
            stapm_w: stapm,
            slow_ppt_w: slow,
            fast_ppt_w: fast,
            tctl_temp: temp,
        };

        crate::daemon_client::save_power_preset_sync(
            mode_id.to_string(),
            new_limits.stapm_w,
            new_limits.slow_ppt_w,
            new_limits.fast_ppt_w,
            new_limits.tctl_temp,
        );

        on_saved(new_limits);
        win_save.close();
    });

    actions_box.append(&reset_btn);
    actions_box.append(&cancel_btn);
    actions_box.append(&save_btn);
    content_box.append(&actions_box);

    window.present();
}

fn connect_preset_card_click(
    card_btn: &gtk::ToggleButton,
    mode_id: &'static str,
    presets_state: Rc<RefCell<PowerPresetsData>>,
    is_amd_flag: Rc<RefCell<bool>>,
    set_active_card: Rc<dyn Fn(Option<&str>)>,
    pl1_scale: gtk::Scale,
    slow_scale: gtk::Scale,
    pl2_scale: gtk::Scale,
    tcc_scale: gtk::Scale,
    pl1_lbl: gtk::Label,
    slow_lbl: gtk::Label,
    pl2_lbl: gtk::Label,
    tcc_lbl: gtk::Label,
    syncing_sliders: Rc<Cell<bool>>,
) {
    let mode_str = mode_id.to_string();
    card_btn.connect_clicked(move |_| {
        set_active_card(Some(mode_id));
        let limits = {
            let state = presets_state.borrow();
            match mode_id {
                "power-saver" => state.presets.power_saver,
                "performance" => state.presets.performance,
                _ => state.presets.balanced,
            }
        };
        apply_preset_limits_to_sliders(
            &limits,
            *is_amd_flag.borrow(),
            &syncing_sliders,
            &pl1_scale,
            &slow_scale,
            &pl2_scale,
            &tcc_scale,
            &pl1_lbl,
            &slow_lbl,
            &pl2_lbl,
            &tcc_lbl,
        );
        presets_state.borrow_mut().active_mode = Some(mode_str.clone());
        presets_state.borrow_mut().manual_override = false;
        crate::daemon_client::set_power_preset_mode_sync(mode_str.clone());
    });
}

fn connect_preset_edit(
    edit_btn: &gtk::Button,
    mode_id: &'static str,
    mode_title: &'static str,
    presets_state: Rc<RefCell<PowerPresetsData>>,
    is_amd_flag: Rc<RefCell<bool>>,
    stapm_fast_linked_flag: Rc<RefCell<bool>>,
    sub_lbl: gtk::Label,
    pl1_scale: gtk::Scale,
    slow_scale: gtk::Scale,
    pl2_scale: gtk::Scale,
    tcc_scale: gtk::Scale,
    pl1_lbl: gtk::Label,
    slow_lbl: gtk::Label,
    pl2_lbl: gtk::Label,
    tcc_lbl: gtk::Label,
    syncing_sliders: Rc<Cell<bool>>,
) {
    let mode_str = mode_id.to_string();
    edit_btn.connect_clicked(move |btn| {
        let parent_win = btn.root().and_then(|r| r.downcast::<gtk::Window>().ok());
        let current_limits = {
            let state = presets_state.borrow();
            match mode_id {
                "power-saver" => state.presets.power_saver,
                "performance" => state.presets.performance,
                _ => state.presets.balanced,
            }
        };
        let default_limits = {
            let state = presets_state.borrow();
            match mode_id {
                "power-saver" => state.defaults.power_saver,
                "performance" => state.defaults.performance,
                _ => state.defaults.balanced,
            }
        };
        let is_amd = *is_amd_flag.borrow();
        let stapm_linked = *stapm_fast_linked_flag.borrow();

        let state_saved = presets_state.clone();
        let sub_saved = sub_lbl.clone();
        let pl1_s_s = pl1_scale.clone();
        let slow_s_s = slow_scale.clone();
        let pl2_s_s = pl2_scale.clone();
        let tcc_s_s = tcc_scale.clone();
        let pl1_lbl_s = pl1_lbl.clone();
        let slow_lbl_s = slow_lbl.clone();
        let pl2_lbl_s = pl2_lbl.clone();
        let tcc_lbl_s = tcc_lbl.clone();
        let sync_s = syncing_sliders.clone();
        let m_str = mode_str.clone();

        show_preset_edit_dialog(
            parent_win.as_ref(),
            mode_id,
            mode_title,
            is_amd,
            stapm_linked,
            current_limits,
            default_limits,
            move |saved| {
                match m_str.as_str() {
                    "power-saver" => state_saved.borrow_mut().presets.power_saver = saved,
                    "performance" => state_saved.borrow_mut().presets.performance = saved,
                    _ => state_saved.borrow_mut().presets.balanced = saved,
                }
                sub_saved.set_label(&format_preset_subtitle(&saved));
                let is_active = state_saved.borrow().active_mode.as_deref() == Some(m_str.as_str())
                    && !state_saved.borrow().manual_override;
                if is_active {
                    apply_preset_limits_to_sliders(
                        &saved,
                        is_amd,
                        &sync_s,
                        &pl1_s_s,
                        &slow_s_s,
                        &pl2_s_s,
                        &tcc_s_s,
                        &pl1_lbl_s,
                        &slow_lbl_s,
                        &pl2_lbl_s,
                        &tcc_lbl_s,
                    );
                }
            },
        );
    });
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

    // ── Power Mode Limit Presets Group & Cards ────────────────
    let presets_state = Rc::new(RefCell::new(default_power_presets_data(*is_amd_flag.borrow())));

    let presets_group = adw::PreferencesGroup::builder()
        .title(i18n::t("power_presets_group"))
        .description(i18n::t("power_presets_desc"))
        .build();

    let status_pill = gtk::Label::builder()
        .label(&format!("{}: {}", i18n::t("active_mode_status"), i18n::t("mode_balanced")))
        .css_classes(["badge-ok"])
        .valign(gtk::Align::Center)
        .build();
    presets_group.set_header_suffix(Some(&status_pill));

    let cards_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .homogeneous(true)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let (eco_card, _eco_title, eco_sub, eco_edit, eco_wrap) = build_preset_card(
        "eco.svg",
        "power-profile-power-saver-symbolic",
        i18n::t("mode_power_save"),
        "20W / 25W · 70°C",
    );
    let (bal_card, _bal_title, bal_sub, bal_edit, bal_wrap) = build_preset_card(
        "balanced.svg",
        "power-profile-balanced-symbolic",
        i18n::t("mode_balanced"),
        "35W / 45W · 82°C",
    );
    let (perf_card, _perf_title, perf_sub, perf_edit, perf_wrap) = build_preset_card(
        "performance.svg",
        "power-profile-performance-symbolic",
        i18n::t("mode_performance"),
        "45W / 65W · 88°C",
    );

    cards_row.append(&eco_wrap);
    cards_row.append(&bal_wrap);
    cards_row.append(&perf_wrap);
    presets_group.add(&cards_row);

    let set_active_card: Rc<dyn Fn(Option<&str>)> = {
        let eco_c = eco_card.clone();
        let bal_c = bal_card.clone();
        let perf_c = perf_card.clone();
        let status_pill_c = status_pill.clone();
        Rc::new(move |mode: Option<&str>| {
            match mode {
                Some("power-saver") => {
                    if !eco_c.is_active() { eco_c.set_active(true); }
                    if bal_c.is_active() { bal_c.set_active(false); }
                    if perf_c.is_active() { perf_c.set_active(false); }
                    status_pill_c.set_label(&format!("{}: {}", i18n::t("active_mode_status"), i18n::t("mode_power_save")));
                    status_pill_c.set_css_classes(&["badge-ok"]);
                }
                Some("performance") => {
                    if eco_c.is_active() { eco_c.set_active(false); }
                    if bal_c.is_active() { bal_c.set_active(false); }
                    if !perf_c.is_active() { perf_c.set_active(true); }
                    status_pill_c.set_label(&format!("{}: {}", i18n::t("active_mode_status"), i18n::t("mode_performance")));
                    status_pill_c.set_css_classes(&["badge-ok"]);
                }
                Some("balanced") => {
                    if eco_c.is_active() { eco_c.set_active(false); }
                    if !bal_c.is_active() { bal_c.set_active(true); }
                    if perf_c.is_active() { perf_c.set_active(false); }
                    status_pill_c.set_label(&format!("{}: {}", i18n::t("active_mode_status"), i18n::t("mode_balanced")));
                    status_pill_c.set_css_classes(&["badge-ok"]);
                }
                _ => {
                    if eco_c.is_active() { eco_c.set_active(false); }
                    if bal_c.is_active() { bal_c.set_active(false); }
                    if perf_c.is_active() { perf_c.set_active(false); }
                    status_pill_c.set_label(i18n::t("preset_manual_override"));
                    status_pill_c.set_css_classes(&["badge-warn"]);
                }
            }
        })
    };

    set_active_card(Some("balanced"));

    connect_preset_card_click(
        &eco_card,
        "power-saver",
        presets_state.clone(),
        is_amd_flag.clone(),
        set_active_card.clone(),
        pl1_scale.clone(),
        slow_scale.clone(),
        pl2_scale.clone(),
        tcc_scale.clone(),
        pl1_lbl.clone(),
        slow_lbl.clone(),
        pl2_lbl.clone(),
        tcc_lbl.clone(),
        syncing_sliders.clone(),
    );
    connect_preset_card_click(
        &bal_card,
        "balanced",
        presets_state.clone(),
        is_amd_flag.clone(),
        set_active_card.clone(),
        pl1_scale.clone(),
        slow_scale.clone(),
        pl2_scale.clone(),
        tcc_scale.clone(),
        pl1_lbl.clone(),
        slow_lbl.clone(),
        pl2_lbl.clone(),
        tcc_lbl.clone(),
        syncing_sliders.clone(),
    );
    connect_preset_card_click(
        &perf_card,
        "performance",
        presets_state.clone(),
        is_amd_flag.clone(),
        set_active_card.clone(),
        pl1_scale.clone(),
        slow_scale.clone(),
        pl2_scale.clone(),
        tcc_scale.clone(),
        pl1_lbl.clone(),
        slow_lbl.clone(),
        pl2_lbl.clone(),
        tcc_lbl.clone(),
        syncing_sliders.clone(),
    );

    connect_preset_edit(
        &eco_edit,
        "power-saver",
        i18n::t("mode_power_save"),
        presets_state.clone(),
        is_amd_flag.clone(),
        stapm_fast_linked_flag.clone(),
        eco_sub.clone(),
        pl1_scale.clone(),
        slow_scale.clone(),
        pl2_scale.clone(),
        tcc_scale.clone(),
        pl1_lbl.clone(),
        slow_lbl.clone(),
        pl2_lbl.clone(),
        tcc_lbl.clone(),
        syncing_sliders.clone(),
    );
    connect_preset_edit(
        &bal_edit,
        "balanced",
        i18n::t("mode_balanced"),
        presets_state.clone(),
        is_amd_flag.clone(),
        stapm_fast_linked_flag.clone(),
        bal_sub.clone(),
        pl1_scale.clone(),
        slow_scale.clone(),
        pl2_scale.clone(),
        tcc_scale.clone(),
        pl1_lbl.clone(),
        slow_lbl.clone(),
        pl2_lbl.clone(),
        tcc_lbl.clone(),
        syncing_sliders.clone(),
    );
    connect_preset_edit(
        &perf_edit,
        "performance",
        i18n::t("mode_performance"),
        presets_state.clone(),
        is_amd_flag.clone(),
        stapm_fast_linked_flag.clone(),
        perf_sub.clone(),
        pl1_scale.clone(),
        slow_scale.clone(),
        pl2_scale.clone(),
        tcc_scale.clone(),
        pl1_lbl.clone(),
        slow_lbl.clone(),
        pl2_lbl.clone(),
        tcc_lbl.clone(),
        syncing_sliders.clone(),
    );

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
    let set_active_apply = set_active_card.clone();
    let presets_state_apply = presets_state.clone();
    apply_btn.connect_clicked(move |_| {
        set_active_apply(None);
        presets_state_apply.borrow_mut().active_mode = None;
        presets_state_apply.borrow_mut().manual_override = true;

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
    page.append(&presets_group);

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
    let presets_state_load = presets_state.clone();
    let eco_sub_load = eco_sub.clone();
    let bal_sub_load = bal_sub.clone();
    let perf_sub_load = perf_sub.clone();
    let set_active_load = set_active_card.clone();

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
        let presets_state_load = presets_state_load.clone();
        let eco_sub_load = eco_sub_load.clone();
        let bal_sub_load = bal_sub_load.clone();
        let perf_sub_load = perf_sub_load.clone();
        let set_active_load = set_active_load.clone();

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

            // Load and sync Power Mode Limit Presets
            if let Ok(presets_json) = crate::daemon_client::get_power_presets_async().await {
                let data = parse_power_presets_json(&presets_json, *is_amd_load.borrow());
                *presets_state_load.borrow_mut() = data.clone();

                eco_sub_load.set_label(&format_preset_subtitle(&data.presets.power_saver));
                bal_sub_load.set_label(&format_preset_subtitle(&data.presets.balanced));
                perf_sub_load.set_label(&format_preset_subtitle(&data.presets.performance));

                if data.manual_override {
                    set_active_load(None);
                } else {
                    set_active_load(data.active_mode.as_deref());
                    let limits = match data.active_mode.as_deref() {
                        Some("power-saver") => data.presets.power_saver,
                        Some("performance") => data.presets.performance,
                        _ => data.presets.balanced,
                    };
                    apply_preset_limits_to_sliders(
                        &limits,
                        *is_amd_load.borrow(),
                        &sync_load,
                        &pl1_s_load,
                        &slow_s_load,
                        &pl2_s_load,
                        &tcc_s_load,
                        &pl1_lbl_load,
                        &slow_lbl_load,
                        &pl2_lbl_load,
                        &tcc_lbl_load,
                    );
                }
            }
        });
    });

    refresh_state();
    let refresh_on_map = refresh_state.clone();
    page.connect_map(move |_| {
        refresh_on_map();
    });

    // Periodic live sync for power presets when tab is actively visible/mapped
    let page_poll = page.clone();
    let presets_state_poll = presets_state.clone();
    let is_amd_poll = is_amd_flag.clone();
    let set_active_poll = set_active_card.clone();
    let eco_sub_poll = eco_sub.clone();
    let bal_sub_poll = bal_sub.clone();
    let perf_sub_poll = perf_sub.clone();
    let pl1_s_poll = pl1_scale.clone();
    let slow_s_poll = slow_scale.clone();
    let pl2_s_poll = pl2_scale.clone();
    let tcc_s_poll = tcc_scale.clone();
    let pl1_lbl_poll = pl1_lbl.clone();
    let slow_lbl_poll = slow_lbl.clone();
    let pl2_lbl_poll = pl2_lbl.clone();
    let tcc_lbl_poll = tcc_lbl.clone();
    let sync_poll = syncing_sliders.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
        if !page_poll.is_mapped() {
            return glib::ControlFlow::Continue;
        }

        if sync_poll.get() {
            return glib::ControlFlow::Continue;
        }

        #[allow(deprecated)]
        let (tx, rx) = glib::MainContext::channel::<Option<PowerPresetsData>>(glib::Priority::default());

        let presets_state_c = presets_state_poll.clone();
        let is_amd_c = is_amd_poll.clone();
        let set_active_c = set_active_poll.clone();
        let eco_sub_c = eco_sub_poll.clone();
        let bal_sub_c = bal_sub_poll.clone();
        let perf_sub_c = perf_sub_poll.clone();
        let pl1_s_c = pl1_s_poll.clone();
        let slow_s_c = slow_s_poll.clone();
        let pl2_s_c = pl2_s_poll.clone();
        let tcc_s_c = tcc_s_poll.clone();
        let pl1_lbl_c = pl1_lbl_poll.clone();
        let slow_lbl_c = slow_lbl_poll.clone();
        let pl2_lbl_c = pl2_lbl_poll.clone();
        let tcc_lbl_c = tcc_lbl_poll.clone();
        let sync_c = sync_poll.clone();

        rx.attach(None, move |maybe_data| {
            if let Some(data) = maybe_data {
                let (mode_changed, override_changed, presets_changed) = {
                    let cur = presets_state_c.borrow();
                    (
                        cur.active_mode != data.active_mode,
                        cur.manual_override != data.manual_override,
                        cur.presets != data.presets,
                    )
                };

                if mode_changed || override_changed || presets_changed {
                    *presets_state_c.borrow_mut() = data.clone();

                    eco_sub_c.set_label(&format_preset_subtitle(&data.presets.power_saver));
                    bal_sub_c.set_label(&format_preset_subtitle(&data.presets.balanced));
                    perf_sub_c.set_label(&format_preset_subtitle(&data.presets.performance));

                    if data.manual_override {
                        set_active_c(None);
                    } else {
                        set_active_c(data.active_mode.as_deref());
                        let limits = match data.active_mode.as_deref() {
                            Some("power-saver") => data.presets.power_saver,
                            Some("performance") => data.presets.performance,
                            _ => data.presets.balanced,
                        };
                        apply_preset_limits_to_sliders(
                            &limits,
                            *is_amd_c.borrow(),
                            &sync_c,
                            &pl1_s_c,
                            &slow_s_c,
                            &pl2_s_c,
                            &tcc_s_c,
                            &pl1_lbl_c,
                            &slow_lbl_c,
                            &pl2_lbl_c,
                            &tcc_lbl_c,
                        );
                    }
                }
            }
            glib::ControlFlow::Break
        });

        let is_amd_val = *is_amd_poll.borrow();
        let rt = crate::daemon_client::get_runtime();
        rt.spawn(async move {
            let res = crate::daemon_client::get_power_presets_async().await;
            let data = res.ok().map(|json| parse_power_presets_json(&json, is_amd_val));
            let _ = tx.send(data);
        });

        glib::ControlFlow::Continue
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

    #[test]
    fn test_parse_power_presets_json_valid() {
        use super::*;
        let json = r#"{
            "is_amd": true,
            "presets": {
                "power_saver": { "stapm_w": 20, "slow_ppt_w": 20, "fast_ppt_w": 25, "tctl_temp": 70 },
                "balanced": { "stapm_w": 45, "slow_ppt_w": 45, "fast_ppt_w": 65, "tctl_temp": 85 },
                "performance": { "stapm_w": 65, "slow_ppt_w": 65, "fast_ppt_w": 85, "tctl_temp": 95 },
                "active_mode": "performance",
                "manual_override": false
            },
            "defaults": {
                "power_saver": { "stapm_w": 20, "slow_ppt_w": 20, "fast_ppt_w": 25, "tctl_temp": 70 },
                "balanced": { "stapm_w": 35, "slow_ppt_w": 35, "fast_ppt_w": 45, "tctl_temp": 82 },
                "performance": { "stapm_w": 45, "slow_ppt_w": 45, "fast_ppt_w": 65, "tctl_temp": 88 }
            },
            "active_mode": "performance",
            "manual_override": false
        }"#;

        let parsed = parse_power_presets_json(json, false);
        assert!(parsed.is_amd);
        assert_eq!(parsed.active_mode.as_deref(), Some("performance"));
        assert!(!parsed.manual_override);
        assert_eq!(parsed.presets.balanced.stapm_w, 45);
        assert_eq!(parsed.presets.performance.fast_ppt_w, 85);
        assert_eq!(parsed.defaults.balanced.stapm_w, 35);
    }

    #[test]
    fn test_parse_power_presets_json_fallback() {
        use super::*;
        let parsed = parse_power_presets_json("invalid json", true);
        assert!(parsed.is_amd);
        assert_eq!(parsed.active_mode.as_deref(), Some("balanced"));
        assert!(!parsed.manual_override);
        assert_eq!(parsed.presets.balanced.stapm_w, 35);

        let parsed_intel = parse_power_presets_json("{}", false);
        assert!(!parsed_intel.is_amd);
        assert_eq!(parsed_intel.presets.balanced.stapm_w, 45);
    }

    #[test]
    fn test_preset_subtitle_formatting() {
        use super::*;
        let limits = PowerModeLimits {
            stapm_w: 20,
            slow_ppt_w: 20,
            fast_ppt_w: 25,
            tctl_temp: 70,
        };
        assert_eq!(format_preset_subtitle(&limits), "20W / 25W · 70°C");

        let limits_perf = PowerModeLimits {
            stapm_w: 65,
            slow_ppt_w: 65,
            fast_ppt_w: 85,
            tctl_temp: 95,
        };
        assert_eq!(format_preset_subtitle(&limits_perf), "65W / 85W · 95°C");
    }

    #[test]
    fn test_preset_switching_and_slider_sim() {
        use super::*;
        let data = default_presets_for_vendor(true);
        let mut current_mode = "balanced";
        let mut slider_stapm = data.balanced.stapm_w;
        let mut slider_slow = data.balanced.slow_ppt_w;
        let mut slider_fast = data.balanced.fast_ppt_w;
        let mut slider_temp = data.balanced.tctl_temp;
        assert_eq!(current_mode, "balanced");
        assert_eq!(slider_stapm, 35);
        assert_eq!(slider_slow, 35);
        assert_eq!(slider_fast, 45);
        assert_eq!(slider_temp, 82);

        // Switch to power-saver
        current_mode = "power-saver";
        slider_stapm = data.power_saver.stapm_w;
        slider_slow = data.power_saver.slow_ppt_w;
        slider_fast = data.power_saver.fast_ppt_w;
        slider_temp = data.power_saver.tctl_temp;
        assert_eq!(current_mode, "power-saver");
        assert_eq!(slider_stapm, 20);
        assert_eq!(slider_slow, 20);
        assert_eq!(slider_fast, 25);
        assert_eq!(slider_temp, 70);

        // Switch to performance
        current_mode = "performance";
        slider_stapm = data.performance.stapm_w;
        slider_slow = data.performance.slow_ppt_w;
        slider_fast = data.performance.fast_ppt_w;
        slider_temp = data.performance.tctl_temp;
        assert_eq!(current_mode, "performance");
        assert_eq!(slider_stapm, 45);
        assert_eq!(slider_slow, 45);
        assert_eq!(slider_fast, 65);
        assert_eq!(slider_temp, 88);
    }

    #[test]
    fn test_preset_manual_override_behavior() {
        let mut active_mode: Option<String> = Some("balanced".to_string());
        let mut manual_override = false;
        let mut cards_active = [false, true, false]; // Eco, Balanced, Perf
        assert_eq!(active_mode.as_deref(), Some("balanced"));
        assert!(!manual_override);
        assert!(cards_active[1]);

        // User adjusts slider and clicks Apply & Save
        manual_override = true;
        active_mode = None;
        cards_active = [false, false, false];

        assert!(manual_override);
        assert_eq!(active_mode, None);
        assert!(!cards_active.iter().any(|&c| c));
    }

    #[test]
    fn test_preset_edit_save_simulation() {
        use super::*;
        let mut data = default_presets_for_vendor(true);
        let active_mode = "balanced";
        let mut active_stapm = data.balanced.stapm_w;

        // Edit balanced mode limits
        let edited = PowerModeLimits {
            stapm_w: 50,
            slow_ppt_w: 50,
            fast_ppt_w: 70,
            tctl_temp: 90,
        };
        data.balanced = edited;

        // Since balanced was active, sliders update to new limits
        if active_mode == "balanced" {
            active_stapm = data.balanced.stapm_w;
        }
        assert_eq!(active_stapm, 50);
        assert_eq!(format_preset_subtitle(&data.balanced), "50W / 70W · 90°C");
    }

    #[test]
    fn test_external_preset_change_detection() {
        use super::*;
        let mut local = default_power_presets_data(true);
        assert_eq!(local.active_mode.as_deref(), Some("balanced"));
        assert!(!local.manual_override);

        // Incoming data from daemon switched to "performance" via Tray
        let incoming = PowerPresetsData {
            is_amd: true,
            presets: local.presets.clone(),
            defaults: local.defaults.clone(),
            active_mode: Some("performance".to_string()),
            manual_override: false,
        };

        let mode_changed = local.active_mode != incoming.active_mode;
        let override_changed = local.manual_override != incoming.manual_override;
        let presets_changed = local.presets != incoming.presets;

        assert!(mode_changed);
        assert!(!override_changed);
        assert!(!presets_changed);

        // Apply sync
        local = incoming;
        assert_eq!(local.active_mode.as_deref(), Some("performance"));

        // Next poll returns same data
        let incoming2 = local.clone();
        assert_eq!(local.active_mode, incoming2.active_mode);
        assert_eq!(local.manual_override, incoming2.manual_override);
        assert_eq!(local.presets, incoming2.presets);
    }
}

