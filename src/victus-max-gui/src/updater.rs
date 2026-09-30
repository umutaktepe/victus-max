use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use crate::i18n;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::process::Stdio;

/* ─────────────────────────────────────────────────────────────
   updater.rs — Victus Max & firmware update checker
   ───────────────────────────────────────────────────────────── */

pub fn build_page(window: &adw::ApplicationWindow) -> gtk::Box {
    let page = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(20)
        .margin_start(24)
        .margin_end(24)
        .margin_top(24)
        .margin_bottom(32)
        .build();

    // ── Header (Matches Settings and Performance tabs) ─────────
    let hdr = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .margin_bottom(4)
        .build();
    hdr.append(&gtk::Label::builder()
        .label(i18n::t("title_updater"))
        .css_classes(["page-title"])
        .halign(gtk::Align::Start)
        .build());
    hdr.append(&gtk::Label::builder()
        .label(i18n::t("updater_desc"))
        .css_classes(["os-section-desc"])
        .halign(gtk::Align::Start)
        .build());
    page.append(&hdr);

    // ── Victus Max update card ────────────────────────────────
    let app_group = adw::PreferencesGroup::builder()
        .title("Victus Max")
        .build();

    let ver_row = adw::ActionRow::builder()
        .title(i18n::t("current_version"))
        .subtitle(i18n::t("last_checked"))
        .build();
    let app_icon = gtk::Image::builder()
        .icon_name("victus-max")
        .pixel_size(24)
        .margin_end(12)
        .build();
    ver_row.add_prefix(&app_icon);
    
    let version_badge = gtk::Label::builder()
        .label(format!("v{} ({})", env!("CARGO_PKG_VERSION"), env!("VICTUS_MAX_GIT_HASH")))
        .css_classes(["os-version-badge"])
        .valign(gtk::Align::Center)
        .build();
    ver_row.add_suffix(&version_badge);
    app_group.add(&ver_row);

    let channel_model = gtk::StringList::new(&[
        &*i18n::t("channel_canary"),
        &*i18n::t("channel_stable"),
    ]);
    let channel_row = adw::ComboRow::builder()
        .title(i18n::t("update_channel_group"))
        .subtitle(i18n::t("channel_sub"))
        .model(&channel_model)
        .build();
    let channel_icon = gtk::Image::builder()
        .icon_name("preferences-system-symbolic")
        .pixel_size(24)
        .margin_end(12)
        .build();
    channel_row.add_prefix(&channel_icon);

    let initial_channel = crate::update_checker::load_update_channel();
    channel_row.set_selected(match initial_channel {
        crate::update_checker::UpdateChannel::Canary => 0,
        crate::update_checker::UpdateChannel::Stable => 1,
    });

    channel_row.connect_selected_notify(move |row| {
        let new_channel = match row.selected() {
            1 => crate::update_checker::UpdateChannel::Stable,
            _ => crate::update_checker::UpdateChannel::Canary,
        };
        let _ = crate::update_checker::save_update_channel(new_channel);
    });
    app_group.add(&channel_row);

    let check_row = adw::ActionRow::builder()
        .title(i18n::t("check_updates"))
        .build();
    let check_icon = gtk::Image::builder()
        .icon_name("view-refresh-symbolic")
        .pixel_size(24)
        .margin_end(12)
        .build();
    check_row.add_prefix(&check_icon);
    
    let check_btn = gtk::Button::builder()
        .label(&*i18n::t("check_updates"))
        .css_classes(["suggested-action", "pill"])
        .valign(gtk::Align::Center)
        .build();
        
    let win_clone = window.clone();
    check_btn.connect_clicked(move |_| {
        show_update_modal(&win_clone, false);
    });
    check_row.add_suffix(&check_btn);
    
    app_group.add(&check_row);

    page.append(&app_group);

    // ── Firmware group ────────────────────────────────────────
    let specs = crate::daemon_client::get_hardware_specs_sync();
    let fw_group = adw::PreferencesGroup::builder()
        .title(i18n::t("firmware_group"))
        .description(i18n::t("firmware_desc"))
        .build();

    let icons = ["computer-symbolic", "cpu-symbolic", "video-display-symbolic"];
    for (i, (device, ver)) in [
        ("HP BIOS",          specs.bios_version.as_str()),
        ("HP EC Firmware",   specs.ec_version.as_str()),
        ("NVIDIA vBIOS",     specs.vbios_version.as_str()),
    ].iter().enumerate() {
        let row = adw::ActionRow::builder().title(*device).subtitle(*ver).build();
        let icon = gtk::Image::builder()
            .icon_name(icons[i])
            .pixel_size(24)
            .margin_end(12)
            .build();
        row.add_prefix(&icon);
        fw_group.add(&row);
    }

    let fwupd_row = adw::ActionRow::builder()
        .title(i18n::t("scan_fwupd"))
        .subtitle(i18n::t("scan_fwupd_sub"))
        .build();
    let fwupd_icon = gtk::Image::builder()
        .icon_name("system-software-update-symbolic")
        .pixel_size(24)
        .margin_end(12)
        .build();
    fwupd_row.add_prefix(&fwupd_icon);
    
    let fwupd_btn = gtk::Button::builder()
        .label(&*i18n::t("scan_fwupd"))
        .css_classes(["suggested-action", "pill"])
        .valign(gtk::Align::Center)
        .build();
        
    let win_clone2 = window.clone();
    fwupd_btn.connect_clicked(move |_| {
        show_update_modal(&win_clone2, true);
    });
    fwupd_row.add_suffix(&fwupd_btn);
    
    fw_group.add(&fwupd_row);

    page.append(&fw_group);

    page
}

fn show_update_modal(window: &adw::ApplicationWindow, is_firmware: bool) {
    if is_firmware {
        show_firmware_update_modal(window);
    } else {
        show_app_update_modal(window);
    }
}

fn show_firmware_update_modal(window: &adw::ApplicationWindow) {
    let dialog = adw::MessageDialog::builder()
        .heading(i18n::t("scan_fwupd"))
        .body(i18n::t("checking_updates_body"))
        .transient_for(window)
        .build();

    dialog.add_response("cancel", i18n::t("cancel"));
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

    let spinner = gtk::Spinner::builder()
        .spinning(true)
        .halign(gtk::Align::Center)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    dialog.set_extra_child(Some(&spinner));
    
    let dialog_clone = dialog.clone();
    glib::spawn_future_local(async move {
        let (tx, rx) = tokio::sync::oneshot::channel();
        crate::daemon_client::get_runtime().spawn(async move {
            let output = tokio::process::Command::new("fwupdmgr").args(["refresh", "--force"]).output().await;
            let _ = tx.send(output);
        });

        if let Ok(output_res) = rx.await {
            let output = output_res;
            spinner.set_spinning(false);
            
            if let Ok(out) = output {
                if out.status.success() {
                    dialog_clone.set_body(i18n::t("no_updates"));
                } else {
                    dialog_clone.set_body(i18n::t("update_failed"));
                }
            } else {
                dialog_clone.set_body(i18n::t("fwupdmgr_missing"));
            }
        }
        
        dialog_clone.add_response("ok", i18n::t("ok_btn"));
        dialog_clone.set_response_appearance("ok", adw::ResponseAppearance::Suggested);
    });

    dialog.present();
}

fn show_app_update_modal(window: &adw::ApplicationWindow) {
    let channel = crate::update_checker::load_update_channel();

    let dialog = gtk::Window::builder()
        .title(i18n::t("title_updater"))
        .transient_for(window)
        .modal(true)
        .default_width(700)
        .default_height(550)
        .hide_on_close(true)
        .build();

    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();

    let title_lbl = gtk::Label::builder()
        .label(i18n::t("checking_updates_title"))
        .css_classes(["title-1"])
        .build();
    vbox.append(&title_lbl);

    let spinner = gtk::Spinner::builder().spinning(true).height_request(40).build();
    vbox.append(&spinner);

    dialog.set_child(Some(&vbox));
    dialog.present();

    let dialog_clone = dialog.clone();
    let vbox_clone = vbox.clone();
    let win_clone = window.clone();

    glib::spawn_future_local(async move {
        let update_res = crate::update_checker::check_updates_async(channel).await;

        spinner.set_spinning(false);
        vbox_clone.remove(&spinner);

        match update_res {
            Ok(res) => {
                if res.is_update_available {
                    title_lbl.set_label(i18n::t("update_available"));

                    if res.fallback_to_canary {
                        let fallback_lbl = gtk::Label::builder()
                            .label(i18n::t("fallback_notice"))
                            .wrap(true)
                            .css_classes(["dim-label", "caption"])
                            .margin_bottom(4)
                            .build();
                        vbox_clone.append(&fallback_lbl);
                    }

                    let version_diff = if res.channel == crate::update_checker::UpdateChannel::Canary || res.fallback_to_canary {
                        format!("{} ➔ {}", res.current_commit, res.target_version)
                    } else {
                        format!("v{} ➔ {}", res.current_version, res.target_version)
                    };

                    let ver_lbl = gtk::Label::builder()
                        .label(&version_diff)
                        .css_classes(["title-2"])
                        .margin_bottom(4)
                        .build();
                    vbox_clone.append(&ver_lbl);

                    let mut meta_parts = Vec::new();
                    if !res.author.is_empty() {
                        meta_parts.push(format!("{}: {}", i18n::t("commit_by"), res.author));
                    }
                    if !res.date.is_empty() {
                        meta_parts.push(format!("{}: {}", i18n::t("commit_date"), res.date));
                    }
                    if !meta_parts.is_empty() {
                        let meta_lbl = gtk::Label::builder()
                            .label(&meta_parts.join("  •  "))
                            .css_classes(["dim-label"])
                            .margin_bottom(6)
                            .build();
                        vbox_clone.append(&meta_lbl);
                    }

                    let notes_lbl = gtk::Label::builder()
                        .label(i18n::t("release_notes"))
                        .halign(gtk::Align::Start)
                        .css_classes(["heading"])
                        .build();
                    vbox_clone.append(&notes_lbl);

                    let body = if res.release_notes.is_empty() {
                        i18n::t("no_release_notes")
                    } else {
                        res.release_notes.as_str()
                    };
                    let buffer = gtk::TextBuffer::builder().text(body).build();
                    let text_view = gtk::TextView::builder()
                        .buffer(&buffer)
                        .editable(false)
                        .wrap_mode(gtk::WrapMode::Word)
                        .build();
                    let scroll = gtk::ScrolledWindow::builder()
                        .child(&text_view)
                        .min_content_height(150)
                        .vexpand(true)
                        .build();
                    vbox_clone.append(&scroll);

                    if !res.html_url.is_empty() {
                        let link_btn = gtk::LinkButton::with_label(&res.html_url, &*i18n::t("view_on_github"));
                        link_btn.set_halign(gtk::Align::Start);
                        link_btn.set_margin_top(4);
                        vbox_clone.append(&link_btn);
                    }

                    let hbox = gtk::Box::builder()
                        .orientation(gtk::Orientation::Horizontal)
                        .spacing(8)
                        .halign(gtk::Align::End)
                        .margin_top(8)
                        .build();

                    let cancel_btn = gtk::Button::builder().label(i18n::t("cancel")).build();
                    let d_cancel = dialog_clone.clone();
                    cancel_btn.connect_clicked(move |_| d_cancel.close());

                    let update_btn = gtk::Button::builder()
                        .label(i18n::t("update"))
                        .css_classes(["suggested-action"])
                        .build();

                    let v_c = vbox_clone.clone();
                    let d_c = dialog_clone.clone();
                    let target_channel = res.channel;
                    update_btn.connect_clicked(move |_| {
                        start_update_process(v_c.clone(), d_c.clone(), target_channel);
                    });

                    hbox.append(&cancel_btn);
                    hbox.append(&update_btn);
                    vbox_clone.append(&hbox);
                } else {
                    title_lbl.set_label(i18n::t("no_updates"));

                    let subtitle_text = format!("v{} ({}) - {}", res.current_version, res.current_commit, res.title);
                    let sub_lbl = gtk::Label::builder()
                        .label(&subtitle_text)
                        .css_classes(["dim-label"])
                        .margin_top(8)
                        .margin_bottom(16)
                        .build();
                    vbox_clone.append(&sub_lbl);

                    if res.fallback_to_canary {
                        let fallback_lbl = gtk::Label::builder()
                            .label(i18n::t("fallback_notice"))
                            .wrap(true)
                            .css_classes(["dim-label", "caption"])
                            .margin_bottom(12)
                            .build();
                        vbox_clone.append(&fallback_lbl);
                    }

                    let close_btn = gtk::Button::builder()
                        .label(i18n::t("close"))
                        .css_classes(["suggested-action", "pill"])
                        .halign(gtk::Align::Center)
                        .build();
                    let d_close = dialog_clone.clone();
                    close_btn.connect_clicked(move |_| d_close.close());
                    vbox_clone.append(&close_btn);
                }
            }
            Err(e) => {
                let err_msg = match e {
                    crate::update_checker::UpdateCheckError::RateLimitExceeded => i18n::t("rate_limit_err"),
                    crate::update_checker::UpdateCheckError::NetworkError(_) => i18n::t("connection_err"),
                    crate::update_checker::UpdateCheckError::InvalidJson(_) => i18n::t("invalid_json_err"),
                    crate::update_checker::UpdateCheckError::NoReleaseFound => i18n::t("version_fetch_err"),
                };
                title_lbl.set_label(err_msg);

                let btn_box = gtk::Box::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .spacing(8)
                    .halign(gtk::Align::Center)
                    .margin_top(16)
                    .build();

                let close_btn = gtk::Button::builder()
                    .label(i18n::t("close"))
                    .css_classes(["pill"])
                    .build();
                let d_close = dialog_clone.clone();
                close_btn.connect_clicked(move |_| d_close.close());
                btn_box.append(&close_btn);

                let win_retry = win_clone.clone();
                let d_retry = dialog_clone.clone();
                let retry_btn = gtk::Button::builder()
                    .label(i18n::t("check_updates"))
                    .css_classes(["suggested-action", "pill"])
                    .build();
                retry_btn.connect_clicked(move |_| {
                    d_retry.close();
                    show_app_update_modal(&win_retry);
                });
                btn_box.append(&retry_btn);

                vbox_clone.append(&btn_box);
            }
        }
    });
}

pub fn resolve_updater_bin() -> String {
    if std::path::Path::new("/usr/libexec/victus-max/victus-max-updater").exists() {
        "/usr/libexec/victus-max/victus-max-updater".to_string()
    } else if std::path::Path::new("/usr/share/victus-max/setup.sh").exists() {
        "/usr/share/victus-max/setup.sh".to_string()
    } else {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let script = format!("{}/../../scripts/victus-max-updater.sh", manifest_dir);
        if std::path::Path::new(&script).exists() {
            script
        } else {
            "victus-max-updater".to_string()
        }
    }
}

pub fn parse_stage_line(line: &str) -> Option<(f64, &str)> {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("[STAGE:PREPARE]") {
        Some((0.15, rest.trim()))
    } else if let Some(rest) = trimmed.strip_prefix("[STAGE:DOWNLOAD]") {
        Some((0.35, rest.trim()))
    } else if let Some(rest) = trimmed.strip_prefix("[STAGE:BUILD]") {
        Some((0.65, rest.trim()))
    } else if let Some(rest) = trimmed.strip_prefix("[STAGE:RESTART]") {
        Some((0.90, rest.trim()))
    } else if let Some(rest) = trimmed.strip_prefix("[STAGE:COMPLETE]") {
        Some((1.0, rest.trim()))
    } else {
        None
    }
}

fn start_update_process(vbox: gtk::Box, dialog: gtk::Window, channel: crate::update_checker::UpdateChannel) {
    // Clear all children
    while let Some(child) = vbox.first_child() {
        vbox.remove(&child);
    }

    let title_lbl = gtk::Label::builder()
        .label(i18n::t("updating"))
        .css_classes(["title-1"])
        .build();
    vbox.append(&title_lbl);

    let status_lbl = gtk::Label::builder()
        .label(i18n::t("checking_updates_body"))
        .css_classes(["dim-label"])
        .margin_top(4)
        .build();
    vbox.append(&status_lbl);

    let progress = gtk::ProgressBar::builder()
        .margin_top(16)
        .margin_bottom(16)
        .fraction(0.05)
        .build();
    vbox.append(&progress);

    let term_toggle = gtk::ToggleButton::builder()
        .icon_name("utilities-terminal-symbolic")
        .halign(gtk::Align::Center)
        .tooltip_text("Terminal Output")
        .build();
    vbox.append(&term_toggle);

    let log_buffer = gtk::TextBuffer::new(None);
    let log_view = gtk::TextView::builder()
        .buffer(&log_buffer)
        .editable(false)
        .monospace(true)
        .build();
    let scroll = gtk::ScrolledWindow::builder()
        .child(&log_view)
        .min_content_height(200)
        .vexpand(true)
        .visible(false)
        .build();

    let scroll_clone = scroll.clone();
    term_toggle.connect_toggled(move |t| {
        scroll_clone.set_visible(t.is_active());
    });

    vbox.append(&scroll);

    let updater_bin = resolve_updater_bin();

    let pbar_clone = progress.clone();
    let status_clone = status_lbl.clone();
    let title_clone = title_lbl.clone();
    let buf_clone = log_buffer.clone();
    let d_c = dialog.clone();
    let v_c = vbox.clone();
    let log_view_for_scroll = log_view.clone();

    glib::spawn_future_local(async move {
        let mut cmd = match tokio::process::Command::new("pkexec")
            .arg(&updater_bin)
            .arg(channel.as_str())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                pbar_clone.set_fraction(1.0);
                title_clone.set_label(&format!("{}: {}", i18n::t("update_failed"), e));
                let close_btn = gtk::Button::builder()
                    .label(i18n::t("close"))
                    .margin_top(12)
                    .css_classes(["suggested-action"])
                    .halign(gtk::Align::End)
                    .build();
                let d_err = d_c.clone();
                close_btn.connect_clicked(move |_| d_err.close());
                v_c.append(&close_btn);
                return;
            }
        };

        let stdout = match cmd.stdout.take() {
            Some(s) => s,
            None => {
                title_clone.set_label(i18n::t("update_failed"));
                return;
            }
        };
        let stderr = match cmd.stderr.take() {
            Some(s) => s,
            None => {
                title_clone.set_label(i18n::t("update_failed"));
                return;
            }
        };

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let tx1 = tx.clone();
        crate::daemon_client::get_runtime().spawn(async move {
            let mut stdout_reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = stdout_reader.next_line().await {
                let _ = tx1.send(line);
            }
        });

        let tx2 = tx.clone();
        crate::daemon_client::get_runtime().spawn(async move {
            let mut stderr_reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = stderr_reader.next_line().await {
                let _ = tx2.send(line);
            }
        });
        drop(tx);

        while let Some(line) = rx.recv().await {
            // Stage checks
            if let Some((frac, msg)) = parse_stage_line(&line) {
                pbar_clone.set_fraction(frac);
                if frac >= 1.0 {
                    status_clone.set_label(i18n::t("update_completed"));
                } else if !msg.is_empty() {
                    status_clone.set_label(msg);
                }
            }

            // Append line to buffer
            let mut iter = buf_clone.end_iter();
            buf_clone.insert(&mut iter, &format!("{}\n", line));

            // Auto-scroll to bottom
            let mark = buf_clone.create_mark(None, &buf_clone.end_iter(), false);
            log_view_for_scroll.scroll_to_mark(&mark, 0.0, true, 0.0, 1.0);
            buf_clone.delete_mark(&mark);
        }

        let status = cmd.wait().await;
        let is_success = status.as_ref().map(|s| s.success()).unwrap_or(false);

        if is_success {
            pbar_clone.set_fraction(1.0);
            title_clone.set_label(i18n::t("update_completed"));
            status_clone.set_label(i18n::t("update_completed"));

            let btn_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(8)
                .halign(gtk::Align::End)
                .margin_top(12)
                .build();

            let close_btn = gtk::Button::builder()
                .label(i18n::t("close"))
                .build();
            let d_fin = d_c.clone();
            close_btn.connect_clicked(move |_| d_fin.close());

            let relaunch_btn = gtk::Button::builder()
                .label(i18n::t("relaunch_app"))
                .css_classes(["suggested-action"])
                .build();
            relaunch_btn.connect_clicked(|_| {
                if let Ok(exe) = std::env::current_exe() {
                    let _ = std::process::Command::new(exe).spawn();
                } else {
                    let _ = std::process::Command::new("victus-max").spawn();
                }
                std::process::exit(0);
            });

            btn_box.append(&close_btn);
            btn_box.append(&relaunch_btn);
            v_c.append(&btn_box);
        } else {
            let err_text = match status {
                Ok(s) => format!("{}", s),
                Err(e) => e.to_string(),
            };
            title_clone.set_label(&format!("{}: {}", i18n::t("update_failed"), err_text));

            let btn_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(8)
                .halign(gtk::Align::End)
                .margin_top(12)
                .build();

            let close_btn = gtk::Button::builder()
                .label(i18n::t("close"))
                .css_classes(["suggested-action"])
                .build();
            let d_fin = d_c.clone();
            close_btn.connect_clicked(move |_| d_fin.close());

            btn_box.append(&close_btn);
            v_c.append(&btn_box);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stage_line() {
        assert_eq!(
            parse_stage_line("[STAGE:PREPARE] Preparing dependencies..."),
            Some((0.15, "Preparing dependencies..."))
        );
        assert_eq!(
            parse_stage_line("[STAGE:DOWNLOAD] Downloading source code..."),
            Some((0.35, "Downloading source code..."))
        );
        assert_eq!(
            parse_stage_line("[STAGE:BUILD] Compiling binaries..."),
            Some((0.65, "Compiling binaries..."))
        );
        assert_eq!(
            parse_stage_line("[STAGE:RESTART] Restarting daemon..."),
            Some((0.90, "Restarting daemon..."))
        );
        assert_eq!(
            parse_stage_line("[STAGE:COMPLETE] Update complete!"),
            Some((1.0, "Update complete!"))
        );
        assert_eq!(
            parse_stage_line("cargo build --release"),
            None
        );
    }

    #[test]
    fn test_resolve_updater_bin() {
        let bin = resolve_updater_bin();
        assert!(!bin.is_empty());
        assert!(bin.contains("victus-max-updater") || bin.contains("setup.sh"));
    }
}
