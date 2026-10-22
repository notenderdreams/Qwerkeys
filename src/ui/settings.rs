use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Frame, Layout, Margin, Pos2, Rect, RichText,
    Sense, Stroke, Ui, vec2,
};

use super::{AppTheme, QwerkeysApp};
use crate::utils::mix;

/// Draws an iOS-style toggle switch, animating toward `checked` (0 = off, 1 = on).
fn render_toggle(
    ui: &mut Ui,
    id: egui::Id,
    rect: Rect,
    checked: bool,
    on_fill: Color32,
    off_fill: Color32,
    border: Color32,
) {
    let painter = ui.painter();
    let t = ui
        .ctx()
        .animate_value_with_time(id, if checked { 1.0 } else { 0.0 }, 0.14);

    let track_fill = mix(off_fill, on_fill, t);
    painter.rect(
        rect,
        CornerRadius::same((rect.height() * 0.5) as u8),
        track_fill,
        Stroke::new(0.8, border),
        egui::StrokeKind::Inside,
    );

    let knob_r = rect.height() * 0.5 - 2.0;
    let knob_x = egui::lerp((rect.min.x + knob_r + 2.0)..=(rect.max.x - knob_r - 2.0), t);
    let knob_pos = Pos2::new(knob_x, rect.center().y);
    painter.circle_filled(knob_pos, knob_r, Color32::WHITE);
    painter.circle_stroke(knob_pos, knob_r, Stroke::new(0.4, border));
}

impl QwerkeysApp {
    pub(super) fn render_settings_window(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }

        let theme = self.theme_colors;
        let mut open = self.settings_open;
        let mut close_requested = false;

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close_requested = true;
        }

        egui::Window::new(
            RichText::new("Settings")
                .font(FontId::proportional(13.0))
                .strong()
                .color(theme.TEXT_PRIMARY),
        )
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .fixed_size(vec2(470.0, 380.0))
        .frame(
            Frame::NONE
                .fill(theme.BG_PANEL)
                .stroke(Stroke::new(1.0, theme.BORDER_PANEL))
                .shadow(egui::Shadow {
                    offset: [0, 8],
                    blur: 24,
                    spread: 0,
                    color: Color32::from_black_alpha(80),
                })
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::same(18)),
        )
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 10.0);

            // ---------------- Appearance section ----------------
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("APPEARANCE")
                        .font(FontId::monospace(9.5))
                        .strong()
                        .color(theme.TEXT_HEADING),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new("Select color theme")
                            .font(FontId::proportional(9.0))
                            .color(theme.TEXT_MUTED),
                    );
                });
            });

            ui.add_space(2.0);

            let avail_w = ui.available_width();
            let card_h = 56.0;

            for th in AppTheme::ALL {
                let is_active = self.current_theme == th;
                let (rect, response) =
                    ui.allocate_exact_size(vec2(avail_w, card_h), Sense::click());
                let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
                let is_hovered = response.hovered();

                let bg_color = if is_active {
                    theme.BG_BUTTON_HOVER
                } else if is_hovered {
                    theme.BG_FIELD
                } else {
                    theme.BG_BUTTON
                };

                let border_stroke = if is_active {
                    Stroke::new(1.5, theme.SAGE_ACCENT)
                } else if is_hovered {
                    Stroke::new(1.0, theme.BORDER_FIELD)
                } else {
                    Stroke::new(1.0, theme.BORDER_DARK)
                };

                let painter = ui.painter();

                // 1. Card container background & border
                painter.rect(
                    rect,
                    CornerRadius::same(6),
                    bg_color,
                    border_stroke,
                    egui::StrokeKind::Inside,
                );

                // 2. Radio selection indicator
                let radio_center = Pos2::new(rect.min.x + 20.0, rect.center().y);
                painter.circle_stroke(
                    radio_center,
                    6.5,
                    Stroke::new(
                        1.2,
                        if is_active {
                            theme.SAGE_ACCENT
                        } else if is_hovered {
                            theme.TEXT_MUTED
                        } else {
                            // clearly visible on both light and dark card fills
                            mix(theme.BORDER_FIELD, theme.TEXT_PRIMARY, 0.45)
                        },
                    ),
                );
                if is_active {
                    painter.circle_filled(radio_center, 3.5, theme.SAGE_ACCENT);
                }

                // 3. Theme labels
                let text_x = rect.min.x + 36.0;
                painter.text(
                    Pos2::new(text_x, rect.center().y - 8.0),
                    Align2::LEFT_CENTER,
                    th.name(),
                    FontId::proportional(12.5),
                    if is_active || is_hovered {
                        theme.TEXT_PRIMARY
                    } else {
                        theme.TEXT_SECONDARY
                    },
                );
                painter.text(
                    Pos2::new(text_x, rect.center().y + 8.5),
                    Align2::LEFT_CENTER,
                    th.description(),
                    FontId::proportional(9.5),
                    theme.TEXT_MUTED,
                );

                // 4. Color swatches preview
                let th_colors = th.colors();
                let swatches = [
                    th_colors.SAGE_ACCENT,
                    th_colors.KEY_MAPPED_BG,
                    th_colors.PIANO_WHITE_BG,
                    th_colors.BG_APP,
                ];
                let swatch_start_x = rect.max.x - 128.0;
                let swatch_y = rect.center().y - 5.0;
                for (idx, color) in swatches.iter().enumerate() {
                    let chip_rect = Rect::from_min_size(
                        Pos2::new(swatch_start_x + idx as f32 * 13.0, swatch_y),
                        vec2(10.0, 10.0),
                    );
                    painter.rect(
                        chip_rect,
                        CornerRadius::same(2),
                        *color,
                        Stroke::new(0.5, theme.BORDER_PANEL),
                        egui::StrokeKind::Inside,
                    );
                }

                if response.clicked() {
                    self.set_theme(th, ctx);
                }

                ui.add_space(2.0);
            }

            // ---------------- Display section ----------------
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("DISPLAY")
                        .font(FontId::monospace(9.5))
                        .strong()
                        .color(theme.TEXT_HEADING),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new("Piano & keys")
                            .font(FontId::proportional(9.0))
                            .color(theme.TEXT_MUTED),
                    );
                });
            });

            let avail_w = ui.available_width();
            let row_h = 54.0;
            let (row_rect, row_resp) = ui.allocate_exact_size(vec2(avail_w, row_h), Sense::click());
            let row_resp = row_resp.on_hover_cursor(egui::CursorIcon::PointingHand);
            let row_hovered = row_resp.hovered();

            let toggle_on = self.state.get_show_unmapped_note_names();
            if row_resp.clicked() {
                self.state.set_show_unmapped_note_names(!toggle_on);
            }

            let painter = ui.painter();
            painter.rect(
                row_rect,
                CornerRadius::same(6),
                if row_hovered {
                    theme.BG_FIELD
                } else {
                    theme.BG_BUTTON
                },
                Stroke::new(if row_hovered { 1.0 } else { 0.8 }, theme.BORDER_DARK),
                egui::StrokeKind::Inside,
            );

            painter.text(
                Pos2::new(row_rect.min.x + 14.0, row_rect.center().y - 8.0),
                Align2::LEFT_CENTER,
                "Note names on unmapped keys",
                FontId::proportional(12.5),
                if row_hovered {
                    theme.TEXT_PRIMARY
                } else {
                    theme.TEXT_SECONDARY
                },
            );
            painter.text(
                Pos2::new(row_rect.min.x + 14.0, row_rect.center().y + 8.5),
                Align2::LEFT_CENTER,
                "Show pitch labels below keys outside the scale",
                FontId::proportional(9.5),
                theme.TEXT_MUTED,
            );

            let switch_w = 32.0;
            let switch_h = 18.0;
            let switch_rect = Rect::from_center_size(
                Pos2::new(row_rect.max.x - 36.0 - switch_w * 0.5, row_rect.center().y),
                vec2(switch_w, switch_h),
            );
            render_toggle(
                ui,
                ui.id().with("toggle_unmapped_note_names"),
                switch_rect,
                toggle_on,
                theme.SAGE_ACCENT,
                theme.BORDER_DARK,
                theme.BORDER_FIELD,
            );

            ui.add_space(8.0);

            // Bottom bar: Done button
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() - 60.0);
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("Done")
                                .font(FontId::proportional(11.0))
                                .strong()
                                .color(theme.TEXT_PRIMARY),
                        )
                        .fill(theme.BG_BUTTON)
                        .stroke(Stroke::new(1.0, theme.BORDER_DARK))
                        .corner_radius(CornerRadius::same(4))
                        .min_size(vec2(60.0, 24.0)),
                    )
                    .clicked()
                {
                    close_requested = true;
                }
            });
        });

        self.settings_open = open && !close_requested;

        // Any exit path (icon, Done button, Escape, window X) unwinds the gear spin
        if !self.settings_open {
            self.settings_icon_rot = 0.0;
        }
    }
}
