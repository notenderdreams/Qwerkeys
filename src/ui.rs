use std::sync::atomic::Ordering;
use std::sync::Arc;
use egui::{
    vec2, Align, Align2, Color32, CornerRadius, FontId, Frame, Layout, Margin, Pos2, Rect,
    RichText, Sense, Stroke, Ui,
};

use crate::iokit::{self, DeviceTargetMode, KeyboardDevice};
use crate::mapping::{find_key_by_label, KeyMappingDef, LayoutPreset};
use crate::midi::MidiEngine;
use crate::scale::{format_midi_note, RootNote, ScaleType};
use crate::state::AppState;

enum QwertyKeyDef {
    Playable(&'static str),
    Modifier(&'static str, f32),
    Spacebar(f32),
    SplitGap(f32),
}

pub struct QwerkeysApp {
    state: Arc<AppState>,
    midi: Arc<MidiEngine>,
    clicked_usages: Vec<u32>,
    clicked_piano_notes: Vec<u8>,
}

impl QwerkeysApp {
    pub fn new(cc: &eframe::CreationContext<'_>, state: Arc<AppState>, midi: Arc<MidiEngine>) -> Self {
        state.set_egui_ctx(cc.egui_ctx.clone());

        // Studio gray aesthetic
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(18, 18, 18);
        visuals.window_fill = Color32::from_rgb(22, 22, 22);
        cc.egui_ctx.set_visuals(visuals);

        Self {
            state,
            midi,
            clicked_usages: Vec::new(),
            clicked_piano_notes: Vec::new(),
        }
    }

    fn render_header(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("QWERKEYS")
                    .font(FontId::proportional(19.0))
                    .strong()
                    .color(Color32::from_rgb(240, 240, 240)),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new("Port: qwerkeys-input")
                        .color(Color32::from_rgb(130, 130, 130))
                        .font(FontId::proportional(11.5)),
                );
            });
        });
    }

    fn render_controls(&self, ui: &mut Ui) {
        Frame::NONE
            .fill(Color32::from_rgb(26, 26, 26))
            .stroke(Stroke::new(1.0, Color32::from_rgb(40, 40, 40)))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Device:").strong().color(Color32::from_rgb(200, 200, 200)));
                    let devices = self.state.devices.lock().unwrap().clone();
                    let current_ptr = self.state.selected_device_ptr.load(Ordering::SeqCst);
                    let current_name = if current_ptr == 0 {
                        "All Keyboards (Omni)".to_string()
                    } else if current_ptr == usize::MAX {
                        "All External Keyboards".to_string()
                    } else {
                        devices
                            .iter()
                            .find(|d| d.device_ptr == current_ptr)
                            .map(|d| format!("{} ({})", d.name, d.transport))
                            .unwrap_or_else(|| "All Keyboards (Omni)".to_string())
                    };

                    egui::ComboBox::from_id_salt("keyboard_select")
                        .width(220.0)
                        .selected_text(RichText::new(&current_name).color(Color32::WHITE))
                        .show_ui(ui, |ui| {
                            let is_all = current_ptr == 0;
                            if ui.selectable_label(is_all, "[*] All Keyboards (Omni)").clicked() {
                                self.select_all_keyboards();
                            }
                            let is_ext_only = current_ptr == usize::MAX;
                            if ui.selectable_label(is_ext_only, "[+] All External Keyboards").clicked() {
                                self.select_external_keyboards();
                            }
                            ui.separator();
                            for dev in &devices {
                                let label = format!(
                                    "[{}] {} ({}) {}",
                                    dev.index,
                                    dev.name,
                                    dev.transport,
                                    if dev.is_internal { "[Built-in]" } else { "[External]" }
                                );
                                let is_selected = dev.device_ptr == current_ptr;
                                if ui.selectable_label(is_selected, label).clicked() {
                                    self.switch_keyboard(dev);
                                }
                            }
                        });

                    if ui.button("⟳").on_hover_text("Scan for newly connected keyboards").clicked() {
                        let fresh = iokit::refresh_keyboards();
                        if let Ok(mut devs) = self.state.devices.lock() {
                            *devs = fresh;
                        }
                    }

                    ui.separator();

                    ui.label(RichText::new("Root:").strong().color(Color32::from_rgb(200, 200, 200)));
                    let current_root = self.state.get_root();
                    egui::ComboBox::from_id_salt("root_select")
                        .width(50.0)
                        .selected_text(RichText::new(current_root.name()).strong().color(Color32::WHITE))
                        .show_ui(ui, |ui| {
                            for root in RootNote::ALL {
                                if ui.selectable_label(root == current_root, root.name()).clicked() {
                                    self.state.set_root(root);
                                }
                            }
                        });

                    if ui.button("<").clicked() {
                        let prev = (current_root.semitone() + 11) % 12;
                        self.state.set_root(RootNote::ALL[prev as usize]);
                    }
                    if ui.button(">").clicked() {
                        let next = (current_root.semitone() + 1) % 12;
                        self.state.set_root(RootNote::ALL[next as usize]);
                    }

                    ui.separator();

                    ui.label(RichText::new("Scale:").strong().color(Color32::from_rgb(200, 200, 200)));
                    let current_scale = self.state.get_scale();
                    egui::ComboBox::from_id_salt("scale_select")
                        .width(170.0)
                        .selected_text(RichText::new(current_scale.name()).strong().color(Color32::WHITE))
                        .show_ui(ui, |ui| {
                            for scale in ScaleType::ALL {
                                if ui.selectable_label(scale == current_scale, scale.name()).clicked() {
                                    self.state.set_scale(scale);
                                }
                            }
                        });
                });

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Left Octave:").color(Color32::from_rgb(160, 160, 160)));
                    let mut left_oct = self.state.left_base_octave.load(Ordering::SeqCst);
                    if ui.button("-").clicked() && left_oct > -1 {
                        left_oct -= 1;
                        self.state.left_base_octave.store(left_oct, Ordering::SeqCst);
                    }
                    ui.label(RichText::new(format!("C{left_oct}")).strong().color(Color32::WHITE));
                    if ui.button("+").clicked() && left_oct < 5 {
                        left_oct += 1;
                        self.state.left_base_octave.store(left_oct, Ordering::SeqCst);
                    }

                    ui.add_space(10.0);

                    ui.label(RichText::new("Right Octave:").color(Color32::from_rgb(160, 160, 160)));
                    let mut right_oct = self.state.right_base_octave.load(Ordering::SeqCst);
                    if ui.button("-").clicked() && right_oct > 0 {
                        right_oct -= 1;
                        self.state.right_base_octave.store(right_oct, Ordering::SeqCst);
                    }
                    ui.label(RichText::new(format!("C{right_oct}")).strong().color(Color32::WHITE));
                    if ui.button("+").clicked() && right_oct < 7 {
                        right_oct += 1;
                        self.state.right_base_octave.store(right_oct, Ordering::SeqCst);
                    }

                    ui.add_space(10.0);

                    ui.label(RichText::new("Velocity:").color(Color32::from_rgb(160, 160, 160)));
                    let mut vel = self.state.velocity.load(Ordering::SeqCst);
                    let slider = egui::Slider::new(&mut vel, 10..=127).show_value(true);
                    if ui.add(slider).changed() {
                        self.state.velocity.store(vel, Ordering::SeqCst);
                    }

                    ui.separator();

                    ui.label(RichText::new("Preset:").strong().color(Color32::from_rgb(200, 200, 200)));
                    let current_preset = self.state.get_layout_preset();
                    egui::ComboBox::from_id_salt("layout_preset_select")
                        .width(180.0)
                        .selected_text(RichText::new(current_preset.name()).strong().color(Color32::WHITE))
                        .show_ui(ui, |ui| {
                            for preset in LayoutPreset::ALL {
                                if ui.selectable_label(preset == current_preset, preset.name()).clicked() {
                                    self.state.set_layout_preset(preset);
                                }
                            }
                        });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(RichText::new("Panic (Esc)").color(Color32::from_rgb(210, 210, 210))).clicked() {
                            self.midi.panic_all_notes();
                        }
                    });
                });
            });
    }

    /// Draws the interactive piano keyboard roll
    fn render_piano(&mut self, ui: &mut Ui) {
        let root = self.state.get_root();
        let scale = self.state.get_scale();
        let left_oct = self.state.left_base_octave.load(Ordering::SeqCst);
        let right_oct = self.state.right_base_octave.load(Ordering::SeqCst);

        let min_oct = left_oct.min(right_oct);
        let max_oct = left_oct.max(right_oct) + 2;
        let num_octaves = (max_oct - min_oct + 1).clamp(1, 7) as usize;
        let total_white_keys = num_octaves * 7 + 1;

        Frame::NONE
            .fill(Color32::from_rgb(24, 24, 24))
            .stroke(Stroke::new(1.0, Color32::from_rgb(40, 40, 40)))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("PIANO KEYBOARD")
                            .font(FontId::proportional(11.0))
                            .strong()
                            .color(Color32::from_rgb(180, 180, 180)),
                    );
                    ui.label(
                        RichText::new(format!(
                            "(Register: C{} – C{})",
                            min_oct,
                            min_oct + num_octaves as i8
                        ))
                        .font(FontId::proportional(10.5))
                        .color(Color32::from_rgb(110, 110, 110)),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let active_count = self.state.active_notes.lock().unwrap().len();
                        if active_count > 0 {
                            ui.label(
                                RichText::new(format!("● {active_count} ACTIVE"))
                                    .color(Color32::WHITE)
                                    .strong()
                                    .font(FontId::proportional(10.5)),
                            );
                        }
                    });
                });

                ui.add_space(4.0);

                let available_w = ui.available_width();
                let wk_h = 68.0;
                let bk_h = 42.0;
                let wk_w = available_w / (total_white_keys as f32);
                let bk_w = (wk_w * 0.62).round();

                let (rect, response) = ui.allocate_exact_size(vec2(available_w, wk_h), Sense::click_and_drag());
                let painter = ui.painter();

                let pointer_pos = ui.input(|i| i.pointer.hover_pos());
                let is_mouse_down = ui.input(|i| i.pointer.primary_down());

                let white_semitones: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];

                // 1. Draw white keys
                let mut hit_white_note: Option<u8> = None;

                for k in 0..total_white_keys {
                    let oct_idx = k / 7;
                    let step_idx = k % 7;
                    let oct = min_oct + oct_idx as i8;
                    let note = ((oct + 1) as i16 * 12 + white_semitones[step_idx] as i16).clamp(0, 127) as u8;

                    let x0 = rect.min.x + (k as f32) * wk_w;
                    let x1 = if k + 1 == total_white_keys { rect.max.x } else { rect.min.x + ((k + 1) as f32) * wk_w };
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
                    let is_root = scale.is_root(root, note);

                    let fill = if is_active {
                        Color32::from_rgb(255, 255, 255)
                    } else if in_scale {
                        Color32::from_rgb(210, 210, 210)
                    } else {
                        Color32::from_rgb(150, 150, 150)
                    };

                    let stroke = Stroke::new(1.0, Color32::from_rgb(46, 46, 46));
                    painter.rect(key_rect, CornerRadius { nw: 0, ne: 0, sw: 3, se: 3 }, fill, stroke, egui::StrokeKind::Inside);

                    if is_root || step_idx == 0 {
                        let note_label = format_midi_note(note);
                        painter.text(
                            key_rect.center_bottom() - vec2(0.0, 6.0),
                            Align2::CENTER_BOTTOM,
                            note_label,
                            FontId::proportional(9.0),
                            if is_active { Color32::BLACK } else { Color32::from_rgb(40, 40, 40) },
                        );
                    } else if in_scale {
                        painter.circle_filled(
                            key_rect.center_bottom() - vec2(0.0, 8.0),
                            2.2,
                            if is_active { Color32::BLACK } else { Color32::from_rgb(90, 90, 90) },
                        );
                    }
                }

                // 2. Draw black keys
                let black_key_defs: [(usize, u8); 5] = [
                    (0, 1),
                    (1, 3),
                    (3, 6),
                    (4, 8),
                    (5, 10),
                ];

                let mut hit_black_note: Option<u8> = None;

                for oct_idx in 0..num_octaves {
                    let oct = min_oct + oct_idx as i8;
                    let base_white_idx = oct_idx * 7;

                    for (left_wk_idx, semitone) in black_key_defs {
                        let wk_index = base_white_idx + left_wk_idx;
                        let note = ((oct + 1) as i16 * 12 + semitone as i16).clamp(0, 127) as u8;

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
                        let is_root = scale.is_root(root, note);

                        let fill = if is_active {
                            Color32::from_rgb(255, 255, 255)
                        } else if in_scale {
                            Color32::from_rgb(36, 36, 36)
                        } else {
                            Color32::from_rgb(18, 18, 18)
                        };

                        let stroke = Stroke::new(1.0, Color32::from_rgb(58, 58, 58));
                        painter.rect(bk_rect, CornerRadius { nw: 0, ne: 0, sw: 2, se: 2 }, fill, stroke, egui::StrokeKind::Inside);

                        if is_root {
                            painter.circle_filled(
                                bk_rect.center_bottom() - vec2(0.0, 6.0),
                                2.3,
                                if is_active { Color32::BLACK } else { Color32::from_rgb(235, 235, 235) },
                            );
                        } else if in_scale {
                            painter.circle_filled(
                                bk_rect.center_bottom() - vec2(0.0, 6.0),
                                1.7,
                                if is_active { Color32::BLACK } else { Color32::from_rgb(130, 130, 130) },
                            );
                        }
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
    }

    /// Draws the physical mechanical QWERTY keyboard with balanced 20 keys per hand and exact QWERTY separation
    fn render_qwerty_keyboard(&mut self, ui: &mut Ui) {
        let root = self.state.get_root();
        let scale = self.state.get_scale();
        let left_oct = self.state.left_base_octave.load(Ordering::SeqCst);
        let right_oct = self.state.right_base_octave.load(Ordering::SeqCst);
        let preset = self.state.get_layout_preset();

        Frame::NONE
            .fill(Color32::from_rgb(25, 25, 25))
            .stroke(Stroke::new(1.0, Color32::from_rgb(40, 40, 40)))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("QWERTY KEYBOARD")
                            .font(FontId::proportional(11.0))
                            .strong()
                            .color(Color32::from_rgb(180, 180, 180)),
                    );

                    ui.label(
                        RichText::new(format!("•  Preset: {}", preset.name()))
                            .font(FontId::proportional(10.5))
                            .color(Color32::from_rgb(120, 120, 120)),
                    );
                });

                ui.add_space(4.0);

                // 5 rows: 15.0u width per row, 5 playable keys per hand, 1.0u gap down 5|6, T|Y, G|H, B|N
                use QwertyKeyDef::*;
                let rows: [&[QwertyKeyDef]; 5] = [
                    // Row 0: Number Row (5 on left, 5 on right)
                    &[
                        Modifier("~", 1.0),
                        Playable("1"), Playable("2"), Playable("3"), Playable("4"), Playable("5"),
                        SplitGap(1.0),
                        Playable("6"), Playable("7"), Playable("8"), Playable("9"), Playable("0"),
                        Modifier("-", 1.0), Modifier("=", 1.0), Modifier("Delete", 1.0),
                    ],
                    // Row 1: Top Row (5 on left, 5 on right)
                    &[
                        Modifier("Tab", 1.5),
                        Playable("Q"), Playable("W"), Playable("E"), Playable("R"), Playable("T"),
                        SplitGap(1.0),
                        Playable("Y"), Playable("U"), Playable("I"), Playable("O"), Playable("P"),
                        Modifier("[", 1.0), Modifier("]", 1.0), Modifier("\\", 0.5),
                    ],
                    // Row 2: Home Row (5 on left, 5 on right)
                    &[
                        Modifier("Caps", 1.75),
                        Playable("A"), Playable("S"), Playable("D"), Playable("F"), Playable("G"),
                        SplitGap(1.0),
                        Playable("H"), Playable("J"), Playable("K"), Playable("L"), Playable(";"),
                        Modifier("'", 1.0), Modifier("Return", 1.25),
                    ],
                    // Row 3: Bottom Row (5 on left, 5 on right)
                    &[
                        Modifier("Shift", 2.0),
                        Playable("Z"), Playable("X"), Playable("C"), Playable("V"), Playable("B"),
                        SplitGap(1.0),
                        Playable("N"), Playable("M"), Playable(","), Playable("."), Playable("/"),
                        Modifier("Shift", 2.0),
                    ],
                    // Row 4: Spacebar Row (Sustain Pedal)
                    &[
                        Modifier("Ctrl", 1.25), Modifier("Opt", 1.25), Modifier("Cmd", 1.25),
                        Spacebar(7.5),
                        Modifier("Cmd", 1.25), Modifier("Opt", 1.25), Modifier("Ctrl", 1.25),
                    ],
                ];

                let total_width = ui.available_width();
                let key_spacing = 3.0;
                let unit_u = (total_width + key_spacing) / 15.0;
                let row_h = 42.0;

                ui.spacing_mut().item_spacing = vec2(key_spacing, key_spacing);

                for row in rows {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(key_spacing, key_spacing);

                        for key_def in row {
                            match key_def {
                                Playable(label) => {
                                    let key_w = 1.0 * unit_u - key_spacing;
                                    if let Some(key) = find_key_by_label(label) {
                                        self.render_playable_keycap(ui, key, key_w, row_h, root, scale, left_oct, right_oct, preset);
                                    }
                                }
                                Modifier(label, units) => {
                                    let key_w = *units * unit_u - key_spacing;
                                    self.render_modifier_keycap(ui, label, key_w, row_h);
                                }
                                Spacebar(units) => {
                                    let key_w = *units * unit_u - key_spacing;
                                    self.render_spacebar_keycap(ui, key_w, row_h);
                                }
                                SplitGap(units) => {
                                    let gap_w = *units * unit_u - key_spacing;
                                    let (rect, _) = ui.allocate_exact_size(vec2(gap_w, row_h), Sense::hover());
                                    // Subtle central divider line marking the clean two-hand split
                                    ui.painter().line_segment(
                                        [rect.center_top(), rect.center_bottom()],
                                        Stroke::new(1.0, Color32::from_rgb(50, 50, 50)),
                                    );
                                }
                            }
                        }
                    });
                }
            });
    }

    #[allow(clippy::too_many_arguments)]
    fn render_playable_keycap(
        &mut self,
        ui: &mut Ui,
        key: &'static KeyMappingDef,
        width: f32,
        height: f32,
        root: RootNote,
        scale: ScaleType,
        left_oct: i8,
        right_oct: i8,
        preset: LayoutPreset,
    ) {
        let (note_str, degree, is_root) = crate::mapping::get_key_display_info(
            key, root, scale, left_oct, right_oct, preset,
        );

        let is_triggered = self.state.is_key_active(key.usage);

        let (fill, stroke, text_color, sub_color) = if is_triggered {
            (
                Color32::from_rgb(250, 250, 250),
                Color32::from_rgb(255, 255, 255),
                Color32::from_rgb(14, 14, 14),
                Color32::from_rgb(50, 50, 50),
            )
        } else if is_root {
            (
                Color32::from_rgb(38, 38, 38),
                Color32::from_rgb(85, 85, 85),
                Color32::from_rgb(245, 245, 245),
                Color32::from_rgb(170, 170, 170),
            )
        } else {
            (
                Color32::from_rgb(30, 30, 30),
                Color32::from_rgb(50, 50, 50),
                Color32::from_rgb(215, 215, 215),
                Color32::from_rgb(115, 115, 115),
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

        painter.text(
            rect.min + vec2(5.0, 4.0),
            Align2::LEFT_TOP,
            key.label,
            FontId::proportional(10.5),
            if is_triggered { Color32::from_rgb(60, 60, 60) } else { Color32::from_rgb(120, 120, 120) },
        );

        painter.text(
            rect.center() + vec2(0.0, -1.0),
            Align2::CENTER_CENTER,
            &note_str,
            FontId::proportional(14.0),
            text_color,
        );

        painter.text(
            rect.center() + vec2(0.0, 13.0),
            Align2::CENTER_CENTER,
            degree,
            FontId::proportional(8.5),
            sub_color,
        );
    }

    fn render_modifier_keycap(&self, ui: &mut Ui, label: &str, width: f32, height: f32) {
        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let painter = ui.painter();

        painter.rect(
            rect,
            CornerRadius::same(4),
            Color32::from_rgb(20, 20, 20),
            Stroke::new(1.0, Color32::from_rgb(36, 36, 36)),
            egui::StrokeKind::Inside,
        );

        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(10.5),
            Color32::from_rgb(85, 85, 85),
        );
    }

    fn render_spacebar_keycap(&mut self, ui: &mut Ui, width: f32, height: f32) {
        let is_sustain = self.state.sustain_held.load(Ordering::SeqCst);

        let (fill, stroke, text_color) = if is_sustain {
            (
                Color32::from_rgb(250, 250, 250),
                Color32::from_rgb(255, 255, 255),
                Color32::from_rgb(14, 14, 14),
            )
        } else {
            (
                Color32::from_rgb(28, 28, 28),
                Stroke::new(1.0, Color32::from_rgb(50, 50, 50)).color,
                Color32::from_rgb(150, 150, 150),
            )
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
            CornerRadius::same(5),
            fill,
            Stroke::new(if is_sustain { 1.5 } else { 1.0 }, stroke),
            egui::StrokeKind::Inside,
        );

        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            if is_sustain {
                "SUSTAIN ON (CC 64)"
            } else {
                "[ SPACEBAR ] SUSTAIN PEDAL"
            },
            FontId::proportional(11.5),
            text_color,
        );
    }

    fn render_bottom_bar(&self, ui: &mut Ui) {
        let event_info = self.state.last_event_info.lock().unwrap().clone();

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("STATUS:")
                    .font(FontId::proportional(11.5))
                    .strong()
                    .color(Color32::from_rgb(140, 140, 140)),
            );

            ui.label(
                RichText::new(&event_info)
                    .font(FontId::proportional(12.5))
                    .strong()
                    .color(Color32::from_rgb(220, 220, 220)),
            );
        });
    }

    fn switch_keyboard(&self, dev: &KeyboardDevice) {
        let is_external = !dev.is_internal;
        self.state.set_selected_device(dev.device_ptr);

        iokit::set_target_mode(DeviceTargetMode::Specific {
            name: dev.name.clone(),
            vid: dev.vendor_id,
            pid: dev.product_id,
        });
        iokit::set_suppression_enabled(is_external);

        if let Ok(mut info) = self.state.last_event_info.lock() {
            *info = format!(
                "Target: {} ({})",
                dev.name,
                if is_external { "Suppressed" } else { "Shared" }
            );
        }
        println!(
            "Target keyboard set to {} (transport: {}, suppressed: {})",
            dev.name, dev.transport, is_external
        );
    }

    fn select_all_keyboards(&self) {
        self.state.set_selected_device(0);
        iokit::set_target_mode(DeviceTargetMode::Omni);
        iokit::set_suppression_enabled(false);

        if let Ok(mut info) = self.state.last_event_info.lock() {
            *info = "Listening to All Keyboards (Omni)".to_string();
        }
        println!("Target keyboard set to All Keyboards (Omni)");
    }

    fn select_external_keyboards(&self) {
        self.state.set_selected_device(usize::MAX);
        iokit::set_target_mode(DeviceTargetMode::ExternalOnly);
        iokit::set_suppression_enabled(true);

        if let Ok(mut info) = self.state.last_event_info.lock() {
            *info = "Listening to All External Keyboards".to_string();
        }
        println!("Target keyboard set to All External Keyboards");
    }
}

impl eframe::App for QwerkeysApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.state.set_egui_ctx(ctx);

        let panel_frame = Frame::NONE
            .fill(Color32::from_rgb(18, 18, 18))
            .inner_margin(Margin {
                left: 10,
                right: 10,
                top: 8,
                bottom: 12,
            });

        egui::CentralPanel::default().frame(panel_frame).show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(5.0, 5.0);

            self.render_header(ui);
            self.render_controls(ui);
            // 1. Draw the Piano (MIDI keyboard)
            self.render_piano(ui);
            // 2. Below the piano, draw the physical mechanical QWERTY keyboard
            self.render_qwerty_keyboard(ui);
            ui.add_space(2.0);
            // 3. Bottom status readout
            self.render_bottom_bar(ui);
        });
    }
}
