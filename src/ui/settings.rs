use egui::{
    Align, Align2, CornerRadius, FontId, Frame, Layout, Margin, Pos2, Rect, RichText, Sense,
    Stroke, vec2,
};

use super::{AppTheme, QwerkeysApp};

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
        .fixed_size(vec2(460.0, 240.0))
        .frame(
            Frame::NONE
                .fill(theme.BG_PANEL)
                .stroke(Stroke::new(1.0, theme.BORDER_PANEL))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::same(18)),
        )
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 10.0);

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("COLOR THEME")
                        .font(FontId::monospace(9.5))
                        .strong()
                        .color(theme.TEXT_HEADING),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new("Select appearance")
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
                            theme.TEXT_LIGHT_MUTED
                        } else {
                            theme.BORDER_FIELD
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

                // 5. Right-side status badge
                let badge_rect = Rect::from_min_size(
                    Pos2::new(rect.max.x - 68.0, rect.center().y - 9.0),
                    vec2(54.0, 18.0),
                );
                if is_active {
                    painter.rect(
                        badge_rect,
                        CornerRadius::same(3),
                        th_colors.SAGE_BADGE_BG,
                        Stroke::new(1.0, th_colors.SAGE_BORDER),
                        egui::StrokeKind::Inside,
                    );
                    painter.text(
                        badge_rect.center(),
                        Align2::CENTER_CENTER,
                        "ACTIVE",
                        FontId::monospace(8.5),
                        th_colors.SAGE_BADGE_TXT,
                    );
                } else if is_hovered {
                    painter.text(
                        badge_rect.center(),
                        Align2::CENTER_CENTER,
                        "APPLY",
                        FontId::monospace(8.5),
                        theme.TEXT_MUTED,
                    );
                }

                if response.clicked() {
                    self.set_theme(th, ctx);
                }

                ui.add_space(2.0);
            }

            ui.add_space(8.0);

            // Bottom bar: keyboard shortcut hint on left, Done button on right
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("esc to close")
                        .font(FontId::monospace(8.5))
                        .color(theme.TEXT_LIGHT_MUTED),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
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
        });

        self.settings_open = open && !close_requested;
    }
}
