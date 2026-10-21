use std::collections::HashMap;
use std::sync::atomic::Ordering;

use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Frame, Layout, Margin, Pos2, Rect, RichText,
    Sense, Stroke, Ui, vec2,
};

use crate::mapping::{KEY_DEFINITIONS, calculate_key_notes};

use super::QwerkeysApp;
use super::keyboard::{KeySkin, draw_mac_keycap, mix};

impl QwerkeysApp {
    pub(super) fn render_piano(&mut self, ui: &mut Ui) {
        let theme = self.theme_colors;
        let skin = KeySkin::from_light(self.current_theme.is_light());
        let root = self.state.get_root();
        let scale = self.state.get_scale();
        let left_oct = self.state.left_base_octave.load(Ordering::SeqCst);
        let right_oct = self.state.right_base_octave.load(Ordering::SeqCst);
        let pad = self.state.mapping_padding.load(Ordering::SeqCst);
        let preset = self.state.get_layout_preset();

        let min_oct = left_oct.min(right_oct);
        let max_oct = left_oct.max(right_oct) + 2;
        let num_octaves = (max_oct - min_oct + 1).clamp(1, 7) as usize;
        let total_white_keys = num_octaves * 7 + 1;

        // Build reverse lookup: MIDI note -> QWERTY key label(s)
        let mut note_to_key_labels: HashMap<u8, Vec<&'static str>> = HashMap::new();
        for key in KEY_DEFINITIONS {
            let notes = calculate_key_notes(key, root, scale, left_oct, right_oct, preset, pad);
            for n in notes {
                note_to_key_labels.entry(n).or_default().push(key.label);
            }
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("NOTE VIEW")
                        .font(FontId::proportional(12.0))
                        .strong()
                        .color(theme.TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new(format!(
                        "(Register: C{} – C{})",
                        min_oct,
                        min_oct + num_octaves as i8
                    ))
                    .font(FontId::monospace(9.5))
                    .color(theme.TEXT_MUTED),
                );
            });

            ui.add_space(6.0);

            // Piano enclosure (dark body behind the keybed)
            Frame::NONE
                .fill(skin.BODY)
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    let available_w = ui.available_width().max(1.0);
                    let wk_w = available_w / (total_white_keys as f32);
                    let bk_w = (wk_w * 0.60).round();
                    let wk_h = (wk_w * 4.4).clamp(95.0, 140.0);
                    let bk_h = (wk_h * 0.62).round();

                    let (rect, response) =
                        ui.allocate_exact_size(vec2(available_w, wk_h), Sense::click_and_drag());
                    let painter = ui.painter();

                    let pointer_pos = ui.input(|i| i.pointer.hover_pos());
                    let is_mouse_down = ui.input(|i| i.pointer.primary_down());

                    let white_semitones: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];

                    // Piano key profile: square top, rounded bottom lip
                    let piano_corner = CornerRadius {
                        nw: 0,
                        ne: 0,
                        sw: 3,
                        se: 3,
                    };

                    // 1. Draw white keys
                    let mut hit_white_note: Option<u8> = None;
                    let note_names = [
                        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
                    ];

                    for k in 0..total_white_keys {
                        let oct_idx = k / 7;
                        let step_idx = k % 7;
                        let oct = min_oct + oct_idx as i8;
                        let note = ((oct + 1) as i16 * 12
                            + white_semitones[step_idx] as i16)
                            .clamp(0, 127) as u8;

                        let x0 = rect.min.x + (k as f32) * wk_w;
                        let x1 = if k + 1 == total_white_keys {
                            rect.max.x
                        } else {
                            rect.min.x + ((k + 1) as f32) * wk_w
                        };
                        let key_rect = Rect::from_min_max(
                            Pos2::new(x0, rect.min.y),
                            Pos2::new(x1, rect.min.y + wk_h),
                        );

                        if let Some(pos) = pointer_pos
                            && key_rect.contains(pos)
                        {
                            hit_white_note = Some(note);
                        }

                        let is_active = self.state.is_note_active(note);
                        let in_scale = scale.is_in_scale(root, note);

                        // Mac-style faces — state is encoded as clear brightness steps:
                        // mapped (lightest) > unmapped/darker > accent only for pressed
                        let (fill, border) = if is_active {
                            (theme.PIANO_WHITE_ACTIVE, theme.KEY_ACTIVE_BORDER)
                        } else if in_scale {
                            // Mapped: clearly lighter neutral face, plain border
                            (
                                mix(skin.FACE, Color32::WHITE, if skin.LIGHT { 0.45 } else { 0.35 }),
                                skin.BORDER,
                            )
                        } else {
                            // Unmapped: darker than the caps, neutral border
                            (
                                mix(skin.FACE, Color32::BLACK, if skin.LIGHT { 0.08 } else { 0.14 }),
                                skin.BORDER,
                            )
                        };

                        // Depressed (playing) keys lose their bevel and sit flush
                        draw_mac_keycap(&skin,
            painter,
                            key_rect,
                            piano_corner,
                            fill,
                            border,
                            is_active,
                        );

                        let pitch_idx = (note % 12) as usize;
                        let pitch_str = note_names[pitch_idx];
                        let note_octave = (note as i8 / 12) - 1;
                        let is_c = pitch_idx == 0;

                        // Show mapped QWERTY key letter
                        if let Some(lbl) = note_to_key_labels.get(&note).and_then(|l| l.first()) {
                            let label_pos = key_rect.center_bottom() - vec2(0.0, 26.0);
                            let txt_color = if is_active {
                                Color32::WHITE
                            } else if in_scale {
                                if skin.LIGHT {
                                    theme.PIANO_WHITE_MAPPED_TXT
                                } else {
                                    mix(theme.SAGE_ACCENT, Color32::WHITE, 0.45)
                                }
                            } else {
                                skin.TEXT_SOFT
                            };
                            painter.text(
                                label_pos,
                                Align2::CENTER_CENTER,
                                *lbl,
                                FontId::monospace(9.5),
                                txt_color,
                            );
                        }

                        // Note name at bottom: C notes show octave (e.g. C2, C3); others show pitch
                        let note_label = if is_c {
                            format!("{pitch_str}{note_octave}")
                        } else {
                            pitch_str.to_string()
                        };

                        painter.text(
                            key_rect.center_bottom() - vec2(0.0, 6.0),
                            Align2::CENTER_BOTTOM,
                            note_label,
                            FontId::monospace(if is_c { 9.0 } else { 8.0 }),
                            if is_active {
                                Color32::WHITE
                            } else if skin.LIGHT {
                                mix(theme.SAGE_ACCENT, Color32::BLACK, 0.45)
                            } else {
                                mix(theme.SAGE_ACCENT, Color32::WHITE, 0.50)
                            },
                        );
                    }

                    // 2. Draw black keys
                    let black_key_defs: [(usize, u8); 5] =
                        [(0, 1), (1, 3), (3, 6), (4, 8), (5, 10)];

                    let mut hit_black_note: Option<u8> = None;

                    for oct_idx in 0..num_octaves {
                        let oct = min_oct + oct_idx as i8;
                        let base_white_idx = oct_idx * 7;

                        for (left_wk_idx, semitone) in black_key_defs {
                            let wk_index = base_white_idx + left_wk_idx;
                            let note =
                                ((oct + 1) as i16 * 12 + semitone as i16).clamp(0, 127) as u8;

                            let bk_x = rect.min.x + (wk_index + 1) as f32 * wk_w - bk_w * 0.5;
                            let bk_rect = Rect::from_min_size(
                                Pos2::new(bk_x, rect.min.y),
                                vec2(bk_w, bk_h),
                            );

                            if let Some(pos) = pointer_pos
                                && bk_rect.contains(pos)
                            {
                                hit_black_note = Some(note);
                            }

                            let is_active = self.state.is_note_active(note);
                            let in_scale = scale.is_in_scale(root, note);

                            // Glossy dark mac keys — root plays no special role
                            let fill = if is_active {
                                theme.PIANO_BLACK_ACTIVE
                            } else if in_scale {
                                // Mapped: lighter neutral, no accent tint
                                mix(skin.FACE_DARK, Color32::WHITE, 0.12)
                            } else {
                                skin.FACE_DARK
                            };

                            painter.rect(
                                bk_rect,
                                piano_corner,
                                fill,
                                Stroke::new(0.75, Color32::from_rgb(0x0E, 0x0F, 0x11)),
                                egui::StrokeKind::Inside,
                            );

                            if !is_active {
                                // Glossy top sheen on the dark cap (lost when playing)
                                let sheen = Color32::from_rgba_unmultiplied(255, 255, 255, 22);
                                let inner = bk_rect.shrink2(vec2(1.0, 1.5));
                                let band = Rect::from_min_max(
                                    inner.min,
                                    inner.min + vec2(inner.width(), inner.height() * 0.35),
                                );
                                painter.rect(
                                    band,
                                    CornerRadius {
                                        nw: 0,
                                        ne: 0,
                                        sw: 2,
                                        se: 2,
                                    },
                                    sheen,
                                    Stroke::NONE,
                                    egui::StrokeKind::Inside,
                                );
                            }

                            let pitch_idx = (note % 12) as usize;
                            let pitch_str = note_names[pitch_idx];

                            // If mapped, show key binding letter on black key
                            if let Some(lbl) =
                                note_to_key_labels.get(&note).and_then(|l| l.first())
                            {
                                let label_pos = bk_rect.center_bottom() - vec2(0.0, 22.0);
                                let txt_color = if is_active {
                                    Color32::WHITE
                                } else {
                                    theme.PIANO_BLACK_IN_SCALE_TXT
                                };
                                painter.text(
                                    label_pos,
                                    Align2::CENTER_CENTER,
                                    *lbl,
                                    FontId::monospace(8.5),
                                    txt_color,
                                );
                            }

                            // Note name at bottom of black key
                            painter.text(
                                bk_rect.center_bottom() - vec2(0.0, 5.0),
                                Align2::CENTER_BOTTOM,
                                pitch_str,
                                FontId::monospace(7.0),
                                if is_active {
                                    Color32::WHITE
                                } else if in_scale {
                                    theme.PIANO_BLACK_IN_SCALE_TXT
                                } else {
                                    theme.PIANO_BLACK_OUT_SCALE_TXT
                                },
                            );
                        }
                    }

                    // Handle direct piano mouse clicking
                    if response.hovered() && is_mouse_down {
                        let clicked = hit_black_note.or(hit_white_note);
                        if let Some(note) = clicked
                            && !self.clicked_piano_notes.contains(&note)
                        {
                            self.clicked_piano_notes.push(note);
                            self.midi.trigger_note_on(note);
                        }
                    } else if !self.clicked_piano_notes.is_empty() {
                        for note in self.clicked_piano_notes.drain(..) {
                            self.midi.trigger_note_off(note);
                        }
                    }
                });

            ui.add_space(8.0);

            // Piano footer: Legend & active counter
            ui.horizontal(|ui| {
                // Legend
                ui.horizontal(|ui| {
                    // Mapped note legend
                    let (rect, _) = ui.allocate_exact_size(vec2(9.0, 9.0), Sense::hover());
                    ui.painter().rect(
                        rect,
                        CornerRadius::same(2),
                        mix(skin.FACE, Color32::WHITE, if skin.LIGHT { 0.45 } else { 0.35 }),
                        Stroke::new(1.0, skin.BORDER),
                        egui::StrokeKind::Inside,
                    );
                    ui.label(
                        RichText::new("Mapped note")
                            .font(FontId::monospace(8.5))
                            .color(theme.TEXT_MUTED),
                    );

                    ui.add_space(10.0);

                    // Playing note legend
                    let (rect, _) = ui.allocate_exact_size(vec2(9.0, 9.0), Sense::hover());
                    ui.painter().rect(
                        rect,
                        CornerRadius::same(2),
                        theme.PIANO_WHITE_ACTIVE,
                        Stroke::new(1.0, theme.KEY_ACTIVE_BORDER),
                        egui::StrokeKind::Inside,
                    );
                    ui.label(
                        RichText::new("Playing")
                            .font(FontId::monospace(8.5))
                            .color(theme.TEXT_MUTED),
                    );
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let active_count = self.state.active_notes.lock().unwrap().len();
                    ui.label(
                        RichText::new(format!(
                            "{active_count} active {}",
                            if active_count == 1 { "voice" } else { "voices" }
                        ))
                        .font(FontId::monospace(8.5))
                        .color(theme.TEXT_MUTED),
                    );
                });
            });
        });
    }
}
