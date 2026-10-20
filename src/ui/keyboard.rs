use std::sync::atomic::Ordering;

use egui::{
    Align2, Color32, CornerRadius, FontId, Frame, Margin, Pos2, Rect, RichText, Sense, Stroke, Ui,
    vec2,
};

use crate::mapping::{KeyMappingDef, LayoutPreset, find_key_by_label, get_key_display_info};
use crate::scale::{RootNote, ScaleType};

use super::QwerkeysApp;

enum QwertyKeyDef {
    Playable {
        label: &'static str,
        shift_char: Option<&'static str>,
    },
    Modifier {
        label: &'static str,
        sub: Option<&'static str>,
        units: f32,
        align_right: bool,
    },
    Spacebar(f32),
    ArrowCluster(f32),
}

impl QwerkeysApp {
    pub(super) fn render_qwerty_keyboard(&mut self, ui: &mut Ui) {
        let theme = self.theme_colors;
        let root = self.state.get_root();
        let scale = self.state.get_scale();
        let left_oct = self.state.left_base_octave.load(Ordering::SeqCst);
        let right_oct = self.state.right_base_octave.load(Ordering::SeqCst);
        let pad = self.state.mapping_padding.load(Ordering::SeqCst);
        let preset = self.state.get_layout_preset();

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("KEYBOARD MAPPING")
                        .font(FontId::proportional(12.0))
                        .strong()
                        .color(theme.TEXT_PRIMARY),
                );
            });

            ui.add_space(6.0);

            // Keyboard frame enclosure
            Frame::NONE
                .fill(theme.BG_KEYBOARD)
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    use QwertyKeyDef::*;
                    let rows: [&[QwertyKeyDef]; 5] = [
                        // Row 0: Number Row (14.5 units total)
                        &[
                            Modifier {
                                label: "`",
                                sub: Some("~"),
                                units: 1.0,
                                align_right: false,
                            },
                            Playable {
                                label: "1",
                                shift_char: Some("!"),
                            },
                            Playable {
                                label: "2",
                                shift_char: Some("@"),
                            },
                            Playable {
                                label: "3",
                                shift_char: Some("#"),
                            },
                            Playable {
                                label: "4",
                                shift_char: Some("$"),
                            },
                            Playable {
                                label: "5",
                                shift_char: Some("%"),
                            },
                            Playable {
                                label: "6",
                                shift_char: Some("^"),
                            },
                            Playable {
                                label: "7",
                                shift_char: Some("&"),
                            },
                            Playable {
                                label: "8",
                                shift_char: Some("*"),
                            },
                            Playable {
                                label: "9",
                                shift_char: Some("("),
                            },
                            Playable {
                                label: "0",
                                shift_char: Some(")"),
                            },
                            Playable {
                                label: "-",
                                shift_char: Some("_"),
                            },
                            Playable {
                                label: "=",
                                shift_char: Some("+"),
                            },
                            Modifier {
                                label: "delete",
                                sub: None,
                                units: 1.5,
                                align_right: true,
                            },
                        ],
                        // Row 1: Top Row (14.5 units total)
                        &[
                            Modifier {
                                label: "tab",
                                sub: None,
                                units: 1.5,
                                align_right: false,
                            },
                            Playable {
                                label: "Q",
                                shift_char: None,
                            },
                            Playable {
                                label: "W",
                                shift_char: None,
                            },
                            Playable {
                                label: "E",
                                shift_char: None,
                            },
                            Playable {
                                label: "R",
                                shift_char: None,
                            },
                            Playable {
                                label: "T",
                                shift_char: None,
                            },
                            Playable {
                                label: "Y",
                                shift_char: None,
                            },
                            Playable {
                                label: "U",
                                shift_char: None,
                            },
                            Playable {
                                label: "I",
                                shift_char: None,
                            },
                            Playable {
                                label: "O",
                                shift_char: None,
                            },
                            Playable {
                                label: "P",
                                shift_char: None,
                            },
                            Playable {
                                label: "[",
                                shift_char: Some("{"),
                            },
                            Modifier {
                                label: "]",
                                sub: Some("}"),
                                units: 1.0,
                                align_right: false,
                            },
                            Modifier {
                                label: "\\",
                                sub: Some("|"),
                                units: 1.0,
                                align_right: false,
                            },
                        ],
                        // Row 2: Home Row (14.5 units total)
                        &[
                            Modifier {
                                label: "caps lock",
                                sub: Some("•"),
                                units: 1.8,
                                align_right: false,
                            },
                            Playable {
                                label: "A",
                                shift_char: None,
                            },
                            Playable {
                                label: "S",
                                shift_char: None,
                            },
                            Playable {
                                label: "D",
                                shift_char: None,
                            },
                            Playable {
                                label: "F",
                                shift_char: None,
                            },
                            Playable {
                                label: "G",
                                shift_char: None,
                            },
                            Playable {
                                label: "H",
                                shift_char: None,
                            },
                            Playable {
                                label: "J",
                                shift_char: None,
                            },
                            Playable {
                                label: "K",
                                shift_char: None,
                            },
                            Playable {
                                label: "L",
                                shift_char: None,
                            },
                            Playable {
                                label: ";",
                                shift_char: Some(":"),
                            },
                            Modifier {
                                label: "'",
                                sub: Some("\""),
                                units: 1.0,
                                align_right: false,
                            },
                            Modifier {
                                label: "return",
                                sub: None,
                                units: 1.7,
                                align_right: true,
                            },
                        ],
                        // Row 3: Bottom Row (14.5 units total)
                        &[
                            Modifier {
                                label: "shift",
                                sub: None,
                                units: 2.3,
                                align_right: false,
                            },
                            Playable {
                                label: "Z",
                                shift_char: None,
                            },
                            Playable {
                                label: "X",
                                shift_char: None,
                            },
                            Playable {
                                label: "C",
                                shift_char: None,
                            },
                            Playable {
                                label: "V",
                                shift_char: None,
                            },
                            Playable {
                                label: "B",
                                shift_char: None,
                            },
                            Playable {
                                label: "N",
                                shift_char: None,
                            },
                            Playable {
                                label: "M",
                                shift_char: None,
                            },
                            Playable {
                                label: ",",
                                shift_char: Some("<"),
                            },
                            Playable {
                                label: ".",
                                shift_char: Some(">"),
                            },
                            Modifier {
                                label: "/",
                                sub: Some("?"),
                                units: 1.0,
                                align_right: false,
                            },
                            Modifier {
                                label: "shift",
                                sub: None,
                                units: 2.2,
                                align_right: true,
                            },
                        ],
                        // Row 4: Spacebar & Modifiers (14.5 units total)
                        &[
                            Modifier {
                                label: "fn",
                                sub: Some("🌐"),
                                units: 1.0,
                                align_right: false,
                            },
                            Modifier {
                                label: "control",
                                sub: Some("⌃"),
                                units: 1.0,
                                align_right: false,
                            },
                            Modifier {
                                label: "option",
                                sub: Some("⌥"),
                                units: 1.0,
                                align_right: false,
                            },
                            Modifier {
                                label: "command",
                                sub: Some("⌘"),
                                units: 1.25,
                                align_right: false,
                            },
                            Spacebar(5.0),
                            Modifier {
                                label: "command",
                                sub: Some("⌘"),
                                units: 1.25,
                                align_right: false,
                            },
                            Modifier {
                                label: "option",
                                sub: Some("⌥"),
                                units: 1.0,
                                align_right: false,
                            },
                            ArrowCluster(3.0),
                        ],
                    ];

                    let total_width = ui.available_width();
                    let key_spacing = 4.0;
                    let unit_u = (total_width + key_spacing) / 14.5;
                    let row_h = (unit_u * 0.72).clamp(38.0, 54.0);

                    ui.spacing_mut().item_spacing = vec2(key_spacing, key_spacing);

                    for row in rows {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = vec2(key_spacing, key_spacing);
                            let row_len = row.len();

                            for (idx, key_def) in row.iter().enumerate() {
                                let is_last = idx + 1 == row_len;
                                match key_def {
                                    Playable { label, shift_char } => {
                                        let key_w = if is_last {
                                            ui.available_width().max(1.0)
                                        } else {
                                            1.0 * unit_u - key_spacing
                                        };
                                        if let Some(key) = find_key_by_label(label) {
                                            self.render_playable_keycap(
                                                ui,
                                                key,
                                                *shift_char,
                                                key_w,
                                                row_h,
                                                root,
                                                scale,
                                                left_oct,
                                                right_oct,
                                                preset,
                                                pad,
                                            );
                                        }
                                    }
                                    Modifier {
                                        label,
                                        sub,
                                        units,
                                        align_right,
                                    } => {
                                        let key_w = if is_last {
                                            ui.available_width().max(1.0)
                                        } else {
                                            *units * unit_u - key_spacing
                                        };
                                        self.render_modifier_keycap(
                                            ui,
                                            label,
                                            *sub,
                                            key_w,
                                            row_h,
                                            *align_right,
                                        );
                                    }
                                    Spacebar(units) => {
                                        let key_w = if is_last {
                                            ui.available_width().max(1.0)
                                        } else {
                                            *units * unit_u - key_spacing
                                        };
                                        self.render_spacebar_keycap(ui, key_w, row_h);
                                    }
                                    ArrowCluster(units) => {
                                        let key_w = if is_last {
                                            ui.available_width().max(1.0)
                                        } else {
                                            *units * unit_u - key_spacing
                                        };
                                        self.render_arrow_cluster(ui, key_w, row_h);
                                    }
                                }
                            }
                        });
                    }
                });
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn render_playable_keycap(
        &mut self,
        ui: &mut Ui,
        key: &'static KeyMappingDef,
        shift_char: Option<&'static str>,
        width: f32,
        height: f32,
        root: RootNote,
        scale: ScaleType,
        left_oct: i8,
        right_oct: i8,
        preset: LayoutPreset,
        padding: i8,
    ) {
        let theme = self.theme_colors;
        let (note_str, degree, is_root) =
            get_key_display_info(key, root, scale, left_oct, right_oct, preset, padding);

        let is_triggered = self.state.is_key_active(key.usage);

        let (fill, stroke, text_color, sub_color) = if is_triggered {
            (
                theme.KEY_ACTIVE_BG,
                theme.KEY_ACTIVE_BORDER,
                theme.KEY_ACTIVE_TXT,
                theme.KEY_SUB_TRIGGERED,
            )
        } else if is_root {
            (
                theme.KEY_ROOT_BG,
                theme.KEY_ROOT_BORDER,
                theme.TEXT_PRIMARY,
                theme.SAGE_ACCENT,
            )
        } else {
            (
                theme.KEY_MAPPED_BG,
                theme.KEY_MAPPED_BORDER,
                theme.KEY_MAPPED_TXT,
                theme.KEY_MAPPED_SUB,
            )
        };

        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click_and_drag());

        if response.is_pointer_button_down_on() {
            if !self.clicked_usages.contains(&key.usage) {
                self.clicked_usages.push(key.usage);
                self.midi.handle_key_down(key.usage);
            }
        } else if self.clicked_usages.contains(&key.usage) {
            self.clicked_usages.retain(|&u| u != key.usage);
            self.midi.handle_key_up(key.usage);
        }

        let painter = ui.painter();

        painter.rect(
            rect,
            CornerRadius::same(4),
            fill,
            Stroke::new(if is_triggered || is_root { 1.5 } else { 1.0 }, stroke),
            egui::StrokeKind::Inside,
        );

        // Top key label: shift symbol + physical key label (e.g. "!" / "1" or "Q")
        if let Some(sc) = shift_char {
            painter.text(
                rect.min + vec2(6.0, 3.5),
                Align2::LEFT_TOP,
                sc,
                FontId::proportional(8.0),
                if is_triggered {
                    theme.KEY_LABEL_SHIFT_TRIGGERED
                } else {
                    theme.TEXT_MUTED
                },
            );
            painter.text(
                rect.min + vec2(6.0, 12.0),
                Align2::LEFT_TOP,
                key.label,
                FontId::proportional(9.0),
                if is_triggered {
                    theme.KEY_LABEL_KEY_TRIGGERED
                } else {
                    theme.TEXT_LIGHT_MUTED
                },
            );
        } else {
            painter.text(
                rect.min + vec2(6.0, 4.0),
                Align2::LEFT_TOP,
                key.label,
                FontId::proportional(9.5),
                if is_triggered {
                    theme.KEY_LABEL_KEY_TRIGGERED
                } else {
                    theme.TEXT_MUTED
                },
            );
        }

        // F and J tactile homing bump bar (authentic MacBook detail)
        if key.label == "F" || key.label == "J" {
            let bump_y = rect.min.y + 16.0;
            let bump_x = rect.min.x + 6.0;
            painter.line_segment(
                [Pos2::new(bump_x, bump_y), Pos2::new(bump_x + 6.0, bump_y)],
                Stroke::new(
                    1.2,
                    if is_triggered {
                        Color32::WHITE
                    } else {
                        theme.TEXT_MUTED
                    },
                ),
            );
        }

        // Center note label (e.g. C2, D#3, I(3))
        painter.text(
            rect.center() + vec2(0.0, -1.0),
            Align2::CENTER_CENTER,
            &note_str,
            FontId::proportional(13.0),
            text_color,
        );

        // Bottom-center degree label (e.g. Root, m3rd, 5th)
        painter.text(
            rect.center() + vec2(0.0, 13.0),
            Align2::CENTER_CENTER,
            degree,
            FontId::monospace(8.0),
            sub_color,
        );
    }

    fn render_modifier_keycap(
        &self,
        ui: &mut Ui,
        label: &str,
        sub: Option<&str>,
        width: f32,
        height: f32,
        align_right: bool,
    ) {
        let theme = self.theme_colors;
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let painter = ui.painter();
        let hovered = response.hovered();

        painter.rect(
            rect,
            CornerRadius::same(4),
            if hovered {
                theme.BG_BUTTON_HOVER
            } else {
                theme.KEY_UNMAPPED_BG
            },
            Stroke::new(1.0, theme.BORDER_DARK),
            egui::StrokeKind::Inside,
        );

        let text_color = if hovered {
            theme.TEXT_PRIMARY
        } else {
            theme.TEXT_MUTED
        };

        if label == "caps lock" {
            // Paint caps lock green dot
            painter.circle_filled(rect.min + vec2(8.0, 8.0), 2.0, theme.DOT_LIT);
            painter.text(
                rect.min + vec2(8.0, height - 7.0),
                Align2::LEFT_BOTTOM,
                label,
                FontId::proportional(8.5),
                text_color,
            );
        } else if let Some(sub_text) = sub {
            // Top and bottom dual label (e.g. fn / 🌐, control / ⌃, option / ⌥, ~ / `)
            let icon_color = if hovered {
                theme.TEXT_PRIMARY
            } else {
                theme.TEXT_MUTED
            };

            let top_pos = rect.min + vec2(if align_right { width - 8.0 } else { 8.0 }, 7.0);

            if sub_text == "⌃" {
                // Vector-drawn crisp Caret / Control symbol
                let tip = top_pos + vec2(0.0, -2.5);
                painter.line_segment([tip + vec2(-3.5, 3.5), tip], Stroke::new(1.3, icon_color));
                painter.line_segment([tip, tip + vec2(3.5, 3.5)], Stroke::new(1.3, icon_color));
            } else if sub_text == "⌥" {
                // Vector-drawn crisp Option / Alt symbol
                let o_pos = top_pos;
                // Top-left flat step
                painter.line_segment(
                    [o_pos + vec2(-4.5, -2.5), o_pos + vec2(-2.0, -2.5)],
                    Stroke::new(1.3, icon_color),
                );
                // Diagonal track
                painter.line_segment(
                    [o_pos + vec2(-2.0, -2.5), o_pos + vec2(1.5, 3.5)],
                    Stroke::new(1.3, icon_color),
                );
                // Bottom-right flat step
                painter.line_segment(
                    [o_pos + vec2(1.5, 3.5), o_pos + vec2(4.5, 3.5)],
                    Stroke::new(1.3, icon_color),
                );
                // Top-right branch bar
                painter.line_segment(
                    [o_pos + vec2(1.0, -2.5), o_pos + vec2(4.5, -2.5)],
                    Stroke::new(1.3, icon_color),
                );
            } else {
                painter.text(
                    rect.min + vec2(if align_right { width - 6.0 } else { 6.0 }, 4.5),
                    if align_right {
                        Align2::RIGHT_TOP
                    } else {
                        Align2::LEFT_TOP
                    },
                    sub_text,
                    FontId::proportional(8.5),
                    theme.TEXT_MUTED,
                );
            }

            painter.text(
                rect.min + vec2(if align_right { width - 6.0 } else { 6.0 }, height - 5.0),
                if align_right {
                    Align2::RIGHT_BOTTOM
                } else {
                    Align2::LEFT_BOTTOM
                },
                label,
                FontId::proportional(8.5),
                text_color,
            );
        } else if align_right {
            painter.text(
                rect.min + vec2(width - 7.0, height - 7.0),
                Align2::RIGHT_BOTTOM,
                label,
                FontId::proportional(9.0),
                text_color,
            );
        } else {
            painter.text(
                rect.min + vec2(7.0, height - 7.0),
                Align2::LEFT_BOTTOM,
                label,
                FontId::proportional(9.0),
                text_color,
            );
        }
    }

    fn render_spacebar_keycap(&mut self, ui: &mut Ui, width: f32, height: f32) {
        let theme = self.theme_colors;
        let is_sustain = self.state.sustain_held.load(Ordering::SeqCst);

        let (fill, stroke, text_color) = if is_sustain {
            (theme.KEY_ACTIVE_BG, theme.KEY_ACTIVE_BORDER, Color32::WHITE)
        } else {
            (theme.KEY_UNMAPPED_BG, theme.BORDER_DARK, theme.TEXT_MUTED)
        };

        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click_and_drag());

        if response.is_pointer_button_down_on() {
            if !is_sustain {
                self.midi.handle_key_down(0x2C);
            }
        } else if is_sustain && self.clicked_usages.contains(&0x2C) {
            self.clicked_usages.retain(|&u| u != 0x2C);
            self.midi.handle_key_up(0x2C);
        }

        let painter = ui.painter();
        painter.rect(
            rect,
            CornerRadius::same(4),
            fill,
            Stroke::new(if is_sustain { 1.5 } else { 1.0 }, stroke),
            egui::StrokeKind::Inside,
        );

        let label = if is_sustain {
            "SUSTAIN ON (CC 64) — ALL NOTES HELD"
        } else {
            "space"
        };

        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(10.0),
            text_color,
        );
    }

    fn render_arrow_cluster(&mut self, ui: &mut Ui, width: f32, height: f32) {
        let key_spacing = 4.0;
        let col_w = (width - 2.0 * key_spacing) / 3.0;
        let half_h = (height - key_spacing) / 2.0;

        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let painter = ui.painter();

        // 1. Left Arrow (Bottom half of col 0)
        let left_rect = Rect::from_min_size(
            Pos2::new(rect.min.x, rect.min.y + half_h + key_spacing),
            vec2(col_w, half_h),
        );
        let left_resp = ui.interact(left_rect, ui.id().with("arrow_left"), Sense::click());
        if left_resp.clicked() {
            let mut pad = self.state.mapping_padding.load(Ordering::SeqCst);
            if pad > -7 {
                pad -= 1;
                self.state.set_mapping_padding(pad);
            }
        }
        self.paint_single_arrow(painter, left_rect, "left", left_resp.hovered());

        // 2. Middle column: Up Arrow (top half) and Down Arrow (bottom half)
        let up_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + col_w + key_spacing, rect.min.y),
            vec2(col_w, half_h),
        );
        let up_resp = ui.interact(up_rect, ui.id().with("arrow_up"), Sense::click());
        if up_resp.clicked() {
            let mut base_oct = self.state.base_octave.load(Ordering::SeqCst);
            if base_oct < 7 {
                base_oct += 1;
                self.state.set_base_octave(base_oct);
            }
        }
        self.paint_single_arrow(painter, up_rect, "up", up_resp.hovered());

        let down_rect = Rect::from_min_size(
            Pos2::new(
                rect.min.x + col_w + key_spacing,
                rect.min.y + half_h + key_spacing,
            ),
            vec2(col_w, half_h),
        );
        let down_resp = ui.interact(down_rect, ui.id().with("arrow_down"), Sense::click());
        if down_resp.clicked() {
            let mut base_oct = self.state.base_octave.load(Ordering::SeqCst);
            if base_oct > 0 {
                base_oct -= 1;
                self.state.set_base_octave(base_oct);
            }
        }
        self.paint_single_arrow(painter, down_rect, "down", down_resp.hovered());

        // 3. Right Arrow (Bottom half of col 2)
        let right_rect = Rect::from_min_size(
            Pos2::new(
                rect.min.x + 2.0 * (col_w + key_spacing),
                rect.min.y + half_h + key_spacing,
            ),
            vec2(col_w, half_h),
        );
        let right_resp = ui.interact(right_rect, ui.id().with("arrow_right"), Sense::click());
        if right_resp.clicked() {
            let mut pad = self.state.mapping_padding.load(Ordering::SeqCst);
            if pad < 7 {
                pad += 1;
                self.state.set_mapping_padding(pad);
            }
        }
        self.paint_single_arrow(painter, right_rect, "right", right_resp.hovered());
    }

    fn paint_single_arrow(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        direction: &str,
        hovered: bool,
    ) {
        let theme = self.theme_colors;
        painter.rect(
            rect,
            CornerRadius::same(3),
            if hovered {
                theme.BG_BUTTON_HOVER
            } else {
                theme.KEY_UNMAPPED_BG
            },
            Stroke::new(1.0, theme.BORDER_DARK),
            egui::StrokeKind::Inside,
        );

        let arrow_color = if hovered {
            theme.TEXT_PRIMARY
        } else {
            theme.TEXT_MUTED
        };

        let center = rect.center();
        let size = 3.5;

        match direction {
            "up" => {
                let p1 = center + vec2(0.0, -size);
                let p2 = center + vec2(-size, size);
                let p3 = center + vec2(size, size);
                painter.add(egui::Shape::convex_polygon(
                    vec![p1, p2, p3],
                    arrow_color,
                    Stroke::NONE,
                ));
            }
            "down" => {
                let p1 = center + vec2(0.0, size);
                let p2 = center + vec2(-size, -size);
                let p3 = center + vec2(size, -size);
                painter.add(egui::Shape::convex_polygon(
                    vec![p1, p2, p3],
                    arrow_color,
                    Stroke::NONE,
                ));
            }
            "left" => {
                let p1 = center + vec2(-size, 0.0);
                let p2 = center + vec2(size, -size);
                let p3 = center + vec2(size, size);
                painter.add(egui::Shape::convex_polygon(
                    vec![p1, p2, p3],
                    arrow_color,
                    Stroke::NONE,
                ));
            }
            "right" => {
                let p1 = center + vec2(size, 0.0);
                let p2 = center + vec2(-size, -size);
                let p3 = center + vec2(-size, size);
                painter.add(egui::Shape::convex_polygon(
                    vec![p1, p2, p3],
                    arrow_color,
                    Stroke::NONE,
                ));
            }
            _ => {}
        }
    }
}
