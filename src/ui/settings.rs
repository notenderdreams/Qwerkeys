//! Settings window: a sidebar (General / Appearance / Mapping / Music) plus
//! a content page holding all controls for the selected section.

use std::sync::atomic::Ordering;

use egui::{
    Align, Align2, Color32, CornerRadius, CursorIcon, FontId, Frame, Layout, Margin, Pos2, Rect,
    RichText, Sense, Stroke, Ui, vec2,
};
use egui_phosphor::regular::{ARROWS_CLOCKWISE, ERASER, GEAR_SIX, PALETTE, PIANO_KEYS};

use super::{AppTheme, QwerkeysApp, ThemeColors};
use crate::mapping::LayoutPreset;
use crate::utils::mix;

/// Sidebar navigaton column width.
const SIDEBAR_W: f32 = 150.0;
/// Height of one sidebar navigation row.
const NAV_ROW_H: f32 = 30.0;
/// Space reserved under the scrollable content column for the Done row.
const DONE_ROW_H: f32 = 34.0;

/// One page of the settings window, selected via the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SettingsSection {
    #[default]
    General,
    Appearance,
    Mapping,
}

impl SettingsSection {
    pub(super) const ALL: [SettingsSection; 3] = [
        SettingsSection::General,
        SettingsSection::Appearance,
        SettingsSection::Mapping,
    ];

    fn icon(self) -> &'static str {
        match self {
            SettingsSection::General => GEAR_SIX,
            SettingsSection::Appearance => PALETTE,
            SettingsSection::Mapping => PIANO_KEYS,
        }
    }

    fn label(self) -> &'static str {
        match self {
            SettingsSection::General => "General",
            SettingsSection::Appearance => "Appearance",
            SettingsSection::Mapping => "Mapping",
        }
    }
}

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

/// Clickable card row: title + description on the left, a toggle switch on
/// the right. Returns whether the card was clicked (callers flip the value).
fn toggle_row(
    ui: &mut Ui,
    theme: &ThemeColors,
    id: egui::Id,
    title: &str,
    subtitle: &str,
    checked: bool,
) -> bool {
    let row_w = ui.available_width();
    let row_h = 54.0;
    let (row_rect, row_resp) = ui.allocate_exact_size(vec2(row_w, row_h), Sense::click());
    let row_resp = row_resp.on_hover_cursor(CursorIcon::PointingHand);
    let row_hovered = row_resp.hovered();
    let clicked = row_resp.clicked();

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
        title,
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
        subtitle,
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
        id.with("switch"),
        switch_rect,
        checked,
        theme.SAGE_ACCENT,
        theme.BORDER_DARK,
        theme.BORDER_FIELD,
    );

    clicked
}

/// Section header row: monospace kicker on the left, small hint on the right.
fn section_header(ui: &mut Ui, theme: &ThemeColors, title: &str, hint: &str) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(title)
                .font(FontId::monospace(9.5))
                .strong()
                .color(theme.TEXT_HEADING),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(hint)
                    .font(FontId::proportional(9.0))
                    .color(theme.TEXT_MUTED),
            );
        });
    });
}

/// Small muted label above a control group.
fn group_label(ui: &mut Ui, theme: &ThemeColors, text: &str) {
    ui.label(
        RichText::new(text)
            .font(FontId::monospace(8.5))
            .color(theme.TEXT_MUTED),
    );
}

impl QwerkeysApp {
    pub(super) fn render_settings_window(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }

        let mut close_requested = false;

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close_requested = true;
        }

        egui::Window::new("")
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .fixed_size(vec2(600.0, 430.0))
            .frame(
                Frame::NONE
                    .fill(self.theme_colors.BG_PANEL)
                    .stroke(Stroke::new(1.0, self.theme_colors.BORDER_PANEL))
                    .shadow(egui::Shadow {
                        offset: [0, 8],
                        blur: 24,
                        spread: 0,
                        color: egui::Color32::from_black_alpha(80),
                    })
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::same(18)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                let body_h = ui.available_height();

                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                    self.render_settings_sidebar(ui, body_h);
                    ui.add_space(14.0);
                    self.render_settings_content(ui, body_h, &mut close_requested);
                });
            });

        self.settings_open = self.settings_open && !close_requested;
        if !self.settings_open {
            self.settings_icon_rot = 0.0;
        }
    }

    // ------------------------------------------------------------------ //
    // SHELL: sidebar + content column                                    //
    // ------------------------------------------------------------------ //

    /// Sidebar navigation card: SETTINGS header + one row per [`SettingsSection`].
    fn render_settings_sidebar(&mut self, ui: &mut Ui, height: f32) {
        let theme = self.theme_colors;

        let (card_rect, _) = ui.allocate_exact_size(vec2(SIDEBAR_W, height), Sense::hover());
        let painter = ui.painter_at(card_rect);
        painter.rect(
            card_rect,
            CornerRadius::same(8),
            theme.BG_KEYBOARD,
            Stroke::new(1.0, theme.BORDER_DARK),
            egui::StrokeKind::Inside,
        );

        let pad_x = 10.0;
        let mut y = card_rect.min.y + 10.0;
        let header_h = 28.0;
        let header_rect = Rect::from_min_size(
            Pos2::new(card_rect.min.x + pad_x, y),
            vec2(SIDEBAR_W - 2.0 * pad_x, header_h),
        );
        y += header_h + 6.0;

        // Header: SETTINGS title
        painter.text(
            Pos2::new(header_rect.left() + 2.0, header_rect.center().y),
            Align2::LEFT_CENTER,
            "SETTINGS",
            FontId::monospace(9.5),
            theme.TEXT_HEADING,
        );

        for (idx, section) in SettingsSection::ALL.iter().enumerate() {
            let row_rect = Rect::from_min_size(
                Pos2::new(card_rect.min.x + pad_x, y),
                vec2(SIDEBAR_W - 2.0 * pad_x, NAV_ROW_H),
            );
            y += NAV_ROW_H + 4.0;

            let response = ui
                .interact(
                    row_rect,
                    ui.id().with(("settings_section_row", idx)),
                    Sense::click(),
                )
                .on_hover_cursor(CursorIcon::PointingHand);
            let hovered = response.hovered();
            let is_selected = self.settings_section == *section;

            // Row background
            painter.rect(
                row_rect,
                CornerRadius::same(6),
                if is_selected {
                    theme.BG_FIELD
                } else if hovered {
                    theme.BG_BUTTON
                } else {
                    Color32::TRANSPARENT
                },
                if is_selected {
                    Stroke::new(1.0, theme.BORDER_FIELD)
                } else {
                    Stroke::NONE
                },
                egui::StrokeKind::Inside,
            );

            let icon_color = if is_selected {
                theme.SAGE_ACCENT
            } else if hovered {
                theme.TEXT_PRIMARY
            } else {
                theme.TEXT_MUTED
            };
            painter.text(
                Pos2::new(row_rect.min.x + 12.0, row_rect.center().y),
                Align2::LEFT_CENTER,
                section.icon(),
                FontId::proportional(13.0),
                icon_color,
            );

            let label_color = if is_selected || hovered {
                theme.TEXT_PRIMARY
            } else {
                theme.TEXT_SECONDARY
            };
            painter.text(
                Pos2::new(row_rect.min.x + 32.0, row_rect.center().y),
                Align2::LEFT_CENTER,
                section.label(),
                FontId::proportional(12.0),
                label_color,
            );

            if response.clicked() {
                self.settings_section = *section;
            }
        }
    }

    /// Content column: scrollable section page + Done button pinned bottom.
    fn render_settings_content(&mut self, ui: &mut Ui, body_h: f32, close_requested: &mut bool) {
        let theme = self.theme_colors;

        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            egui::ScrollArea::vertical()
                .id_salt("settings_content_scroll")
                .auto_shrink([false, false])
                .max_height((body_h - DONE_ROW_H).max(220.0))
                .show(ui, |ui| match self.settings_section {
                    SettingsSection::General => self.render_general_section(ui),
                    SettingsSection::Appearance => self.render_appearance_section(ui),
                    SettingsSection::Mapping => self.render_mapping_section(ui),
                });

            ui.add_space(8.0);

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
                    *close_requested = true;
                }
            });
        });
    }

    // ------------------------------------------------------------------ //
    // GENERAL                                                            //
    // ------------------------------------------------------------------ //

    fn render_general_section(&mut self, ui: &mut Ui) {
        let theme = self.theme_colors;

        section_header(ui, &theme, "GENERAL", "App behavior");
        ui.add_space(4.0);

        let auto_updates = self.state.auto_check_updates.load(Ordering::SeqCst);
        if toggle_row(
            ui,
            &theme,
            ui.id().with("toggle_auto_check_updates"),
            "Check for updates automatically",
            "Look for new releases when Qwerkeys starts",
            auto_updates,
        ) {
            self.state.set_auto_check_updates(!auto_updates);
        }

        ui.add_space(12.0);
        section_header(ui, &theme, "SETUP", "Permissions & onboarding");
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(format!("Qwerkeys v{}", env!("CARGO_PKG_VERSION")))
                        .font(FontId::proportional(12.5))
                        .strong()
                        .color(theme.TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new("Revisit the permission setup screen")
                        .font(FontId::proportional(9.5))
                        .color(theme.TEXT_MUTED),
                );
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(format!("{ARROWS_CLOCKWISE} Run setup again"))
                                .font(FontId::proportional(10.5))
                                .color(theme.TEXT_PRIMARY),
                        )
                        .fill(theme.BG_BUTTON)
                        .stroke(Stroke::new(1.0, theme.BORDER_FIELD))
                        .corner_radius(CornerRadius::same(4))
                        .min_size(vec2(130.0, 24.0)),
                    )
                    .on_hover_text("Shows the macOS permission screen again on app restart")
                    .clicked()
                {
                    self.replay_onboarding();
                }
            });
        });
    }

    // ------------------------------------------------------------------ //
    // APPEARANCE                                                         //
    // ------------------------------------------------------------------ //

    fn render_appearance_section(&mut self, ui: &mut Ui) {
        let theme = self.theme_colors;

        section_header(ui, &theme, "APPEARANCE", "Select color theme");
        ui.add_space(2.0);

        let avail_w = ui.available_width();
        let card_h = 56.0;

        for th in AppTheme::ALL {
            let is_active = self.current_theme == th;
            let (rect, response) = ui.allocate_exact_size(vec2(avail_w, card_h), Sense::click());
            let response = response.on_hover_cursor(CursorIcon::PointingHand);
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
                self.set_theme(th, ui.ctx());
            }

            ui.add_space(2.0);
        }

        ui.add_space(12.0);
        section_header(ui, &theme, "DISPLAY", "Piano & keys");
        ui.add_space(2.0);

        let show_unmapped = self.state.get_show_unmapped_note_names();
        if toggle_row(
            ui,
            &theme,
            ui.id().with("toggle_unmapped_note_names"),
            "Note names on unmapped keys",
            "Show pitch labels below keys outside the scale",
            show_unmapped,
        ) {
            self.state.set_show_unmapped_note_names(!show_unmapped);
        }
    }

    // ------------------------------------------------------------------ //
    // MAPPING                                                            //
    // ------------------------------------------------------------------ //

    fn render_mapping_section(&mut self, ui: &mut Ui) {
        let theme = self.theme_colors;

        section_header(ui, &theme, "MAPPING", "Layout & octave");
        ui.add_space(4.0);

        group_label(ui, &theme, "MAPPING PRESET");
        let current_preset = self.state.get_layout_preset();
        egui::ComboBox::from_id_salt("settings_layout_preset")
            .width(ui.available_width().min(300.0))
            .selected_text(
                RichText::new(current_preset.name())
                    .font(FontId::proportional(10.5))
                    .strong()
                    .color(theme.TEXT_PRIMARY),
            )
            .show_ui(ui, |ui| {
                for p in LayoutPreset::ALL {
                    if ui.selectable_label(p == current_preset, p.name()).clicked() {
                        self.state.set_layout_preset(p);
                    }
                }
            });

        if current_preset.is_custom() {
            ui.add_space(6.0);

            let assigned = self.state.get_custom_layout().len();
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{assigned} keys have custom notes"))
                        .font(FontId::proportional(11.0))
                        .color(theme.TEXT_SECONDARY),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(format!("{ERASER} Clear custom mapping"))
                                    .font(FontId::proportional(10.5))
                                    .color(theme.TEXT_PRIMARY),
                            )
                            .fill(theme.BG_BUTTON)
                            .stroke(Stroke::new(1.0, theme.BORDER_FIELD))
                            .corner_radius(CornerRadius::same(4))
                            .min_size(vec2(160.0, 24.0)),
                        )
                        .on_hover_text("Remove every custom note assignment (cannot be undone)")
                        .clicked()
                    {
                        self.state.clear_custom_layout();
                    }
                });
            });

            ui.add_space(4.0);
            ui.label(
                RichText::new(
                    "Pick a note on the piano roll, then click keys on the \
                     keyboard to assign it. Unassigned keys stay silent.",
                )
                .font(FontId::proportional(9.0))
                .color(theme.TEXT_MUTED),
            );
        }

        ui.add_space(14.0);
        section_header(ui, &theme, "OCTAVES", "Base octave & padding");
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

            // Base octave stepper
            let base_oct = self.state.base_octave.load(Ordering::SeqCst);
            let (base_dec, base_inc) = self.render_stepper(
                ui,
                "settings_base_oct_stepper",
                "BASE:",
                &format!("C{base_oct}"),
                base_oct > 0,
                base_oct < 7,
            );
            if base_dec {
                self.state.set_base_octave(base_oct - 1);
            }
            if base_inc {
                self.state.set_base_octave(base_oct + 1);
            }

            ui.add_space(16.0);

            // Mapping padding stepper
            let pad = self.state.mapping_padding.load(Ordering::SeqCst);
            let pad_str = if pad > 0 {
                format!("+{pad}")
            } else {
                format!("{pad}")
            };
            let (pad_dec, pad_inc) = self.render_stepper(
                ui,
                "settings_mapping_pad_stepper",
                "PAD:",
                &pad_str,
                pad > -7,
                pad < 7,
            );
            if pad_dec {
                self.state.set_mapping_padding(pad - 1);
            }
            if pad_inc {
                self.state.set_mapping_padding(pad + 1);
            }
        });

        ui.add_space(6.0);
        ui.label(
            RichText::new(
                "BASE sets the octave everything starts at; PAD shifts mapping \
                 degrees sideways. Fine control stays in the setup bar.",
            )
            .font(FontId::proportional(9.0))
            .color(theme.TEXT_MUTED),
        );
    }

    // ------------------------------------------------------------------ //
    // ONBOARDING REPLAY                                                  //
    // ------------------------------------------------------------------ //

    /// Close settings and re-open the permission/onboarding page.
    fn replay_onboarding(&mut self) {
        self.settings_open = false;
        self.onboarding_done = false;
        self.onboarding_requested = [false; 2];
        self.state
            .onboarding_completed
            .store(false, Ordering::SeqCst);
        self.state.persist();
    }
}
