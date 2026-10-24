use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Layout, ProgressBar, RichText, Sense, Ui, vec2,
};

use super::QwerkeysApp;

impl QwerkeysApp {
    pub(super) fn render_status_bar(&self, ui: &mut Ui) {
        let theme = self.theme_colors;
        let event_info = self.state.last_event_info.lock().unwrap().clone();

        ui.spacing_mut().item_spacing = vec2(8.0, 0.0);

        ui.horizontal_centered(|ui| {
            // Port on far left, vertically centered
            ui.label(
                RichText::new("PORT: qwerkeys-input")
                    .font(FontId::monospace(11.0))
                    .strong()
                    .color(theme.TEXT_SECONDARY),
            );

            ui.separator();

            ui.label(
                RichText::new("STATUS:")
                    .font(FontId::monospace(11.0))
                    .strong()
                    .color(theme.TEXT_SECONDARY),
            );

            ui.label(
                RichText::new(&event_info)
                    .font(FontId::monospace(12.0))
                    .strong()
                    .color(theme.TEXT_PRIMARY),
            );

            // Right side: version & auto-updater controls anchored to far right
            ui.allocate_ui_with_layout(
                vec2(ui.available_width(), ui.available_height()),
                Layout::right_to_left(Align::Center),
                |ui| {
                    let status = {
                        let u = self.updater.lock().unwrap();
                        u.status.clone()
                    };

                    match status {
                        crate::updater::UpdateStatus::Idle
                        | crate::updater::UpdateStatus::UpToDate => {
                            let ver_text = format!("version {}", env!("CARGO_PKG_VERSION"));
                            let resp = ui.add(
                                egui::Label::new(
                                    RichText::new(ver_text)
                                        .font(FontId::monospace(9.0))
                                        .strong()
                                        .color(theme.TEXT_MUTED),
                                )
                                .sense(Sense::click()),
                            );
                            let resp = resp
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .on_hover_text("Click to check for updates");
                            if resp.clicked() {
                                crate::updater::check_for_updates_async(
                                    self.updater.clone(),
                                    ui.ctx().clone(),
                                );
                            }
                        }
                        crate::updater::UpdateStatus::Checking => {
                            ui.label(
                                RichText::new(format!(
                                    "version {} (checking...)",
                                    env!("CARGO_PKG_VERSION")
                                ))
                                .font(FontId::monospace(9.0))
                                .strong()
                                .color(theme.TEXT_MUTED),
                            );
                            ui.spinner();
                        }
                        crate::updater::UpdateStatus::Available {
                            ref version,
                            ref notes,
                            ..
                        } => {
                            let label = format!("Update now (version {version})");
                            let btn = egui::Button::new(
                                RichText::new(label)
                                    .font(FontId::monospace(9.5))
                                    .strong()
                                    .color(theme.BG_APP),
                            )
                            .fill(theme.SAGE_ACCENT)
                            .corner_radius(CornerRadius::same(3));
                            let resp = ui.add(btn);
                            let resp = if !notes.is_empty() {
                                resp.on_hover_text(notes)
                            } else {
                                resp
                            };
                            if resp.clicked() {
                                crate::updater::download_and_install_async(
                                    self.updater.clone(),
                                    ui.ctx().clone(),
                                );
                            }
                        }
                        crate::updater::UpdateStatus::Downloading {
                            ref version,
                            downloaded,
                            total,
                        } => {
                            let progress = if total > 0 {
                                (downloaded as f32 / total as f32).clamp(0.0, 1.0)
                            } else {
                                0.0
                            };
                            let mb_down = downloaded as f64 / 1_048_576.0;
                            let mb_tot = total as f64 / 1_048_576.0;
                            let text = format!(
                                "{:.1}/{:.1} MB ({:.0}%)",
                                mb_down,
                                mb_tot,
                                progress * 100.0
                            );
                            let font = FontId::monospace(9.0);

                            // Slot sized to the WIDEST possible text for this download, so the
                            // slot never changes width. If its width followed the live text,
                            // the cursor advance would push the progress bar around each frame.
                            let widest_text = if total > 0 {
                                format!("{:.1}/{:.1} MB (100%)", mb_tot, mb_tot)
                            } else {
                                format!("{:.1}/{:.1} MB (0%)", mb_down, mb_tot)
                            };
                            let slot_w = ui
                                .painter()
                                .layout_no_wrap(widest_text, font.clone(), Color32::TRANSPARENT)
                                .rect
                                .width();
                            let slot_h = ui
                                .painter()
                                .layout_no_wrap(text.clone(), font.clone(), Color32::TRANSPARENT)
                                .rect
                                .height();

                            // Fixed slot: the advance is exact, the live text is painted without
                            // touching layout, flush right so it lines up with the bar.
                            let (slot, _) =
                                ui.allocate_exact_size(vec2(slot_w, slot_h), Sense::hover());
                            ui.painter().text(
                                slot.right_center(),
                                Align2::RIGHT_CENTER,
                                text,
                                font,
                                theme.TEXT_MUTED,
                            );
                            ui.add_space(6.0);
                            ui.add(
                                ProgressBar::new(progress)
                                    .fill(theme.SAGE_ACCENT)
                                    .desired_width(180.0)
                                    .desired_height(8.0),
                            )
                            .on_hover_text(format!("Downloading version {version}..."));
                        }
                        crate::updater::UpdateStatus::Extracting => {
                            ui.label(
                                RichText::new("Extracting...")
                                    .font(FontId::monospace(9.0))
                                    .strong()
                                    .color(theme.SAGE_ACCENT),
                            );
                            ui.spinner();
                        }
                        crate::updater::UpdateStatus::ReadyToRestart {
                            ref version,
                            ref script_path,
                        } => {
                            let btn = egui::Button::new(
                                RichText::new(format!("Restart to apply (version {version})"))
                                    .font(FontId::monospace(9.5))
                                    .strong()
                                    .color(theme.BG_APP),
                            )
                            .fill(theme.SAGE_ACCENT)
                            .corner_radius(CornerRadius::same(3));
                            if ui.add(btn).clicked()
                                && let Err(e) = crate::updater::relaunch_and_swap(script_path)
                            {
                                let mut u = self.updater.lock().unwrap();
                                u.status = crate::updater::UpdateStatus::Error(e);
                            }
                        }
                        crate::updater::UpdateStatus::DevBuildSkipped { ref version } => {
                            ui.label(
                                RichText::new(format!("version {version} ready (dev build)"))
                                    .font(FontId::monospace(9.0))
                                    .strong()
                                    .color(theme.TEXT_MUTED),
                            );
                        }
                        crate::updater::UpdateStatus::Error(ref err) => {
                            let resp = ui.add(
                                egui::Label::new(
                                    RichText::new("Update failed (retry)")
                                        .font(FontId::monospace(9.0))
                                        .strong()
                                        .color(Color32::from_rgb(235, 87, 87)),
                                )
                                .sense(Sense::click()),
                            );
                            let resp = resp
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .on_hover_text(err);
                            if resp.clicked() {
                                crate::updater::check_for_updates_async(
                                    self.updater.clone(),
                                    ui.ctx().clone(),
                                );
                            }
                        }
                    }
                },
            );
        });
    }
}
