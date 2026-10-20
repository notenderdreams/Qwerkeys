use std::sync::atomic::Ordering;

use egui::{
    Align, CornerRadius, FontId, Frame, Layout, Margin, Pos2, RichText, Sense, Stroke, Ui, vec2,
};

use crate::iokit::{self, DeviceTargetMode, KeyboardDevice};
use crate::mapping::LayoutPreset;
use crate::scale::{RootNote, ScaleType};

use super::QwerkeysApp;

impl QwerkeysApp {
    pub(super) fn render_setup_bar(&mut self, ui: &mut Ui) {
        let theme = self.theme_colors;
        let avail_w = ui.available_width();
        let sep_space = 14.0;
        let sep_w = 1.0;
        let total_sep = 4.0 * sep_space + 2.0 * sep_w;
        let usable_w = (avail_w - total_sep).max(600.0);

        let dev_w = (usable_w * 0.24).clamp(240.0, 300.0);
        let scale_w = (usable_w * 0.40).clamp(360.0, 480.0);

        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(vec2(dev_w, 102.0), Layout::top_down(Align::Min), |ui| {
                ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                self.render_device_column(ui);
            });

            ui.add_space(sep_space);
            let (sep_rect, _) = ui.allocate_exact_size(vec2(sep_w, 102.0), Sense::hover());
            ui.painter().vline(
                sep_rect.center().x,
                sep_rect.y_range(),
                Stroke::new(1.0, theme.BORDER_FIELD),
            );
            ui.add_space(sep_space);

            ui.allocate_ui_with_layout(vec2(scale_w, 102.0), Layout::top_down(Align::Min), |ui| {
                ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                self.render_scale_column(ui);
            });

            ui.add_space(sep_space);
            let (sep_rect, _) = ui.allocate_exact_size(vec2(sep_w, 102.0), Sense::hover());
            ui.painter().vline(
                sep_rect.center().x,
                sep_rect.y_range(),
                Stroke::new(1.0, theme.BORDER_FIELD),
            );
            ui.add_space(sep_space);

            let out_w = ui.available_width().max(380.0);
            ui.allocate_ui_with_layout(vec2(out_w, 102.0), Layout::top_down(Align::Min), |ui| {
                ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                self.render_output_column(ui);
            });
        });
    }

    /// Column 1: Input device selection & rescan
    fn render_device_column(&mut self, ui: &mut Ui) {
        let theme = self.theme_colors;
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("INPUT DEVICE")
                    .font(FontId::proportional(11.0))
                    .strong()
                    .color(theme.TEXT_HEADING),
            );

            let current_ptr = self.state.selected_device_ptr.load(Ordering::SeqCst);
            let is_dedicated = current_ptr != 0;
            let badge_bg = if is_dedicated {
                theme.SAGE_BADGE_BG
            } else {
                theme.BG_BUTTON
            };
            let badge_txt = if is_dedicated {
                theme.SAGE_BADGE_TXT
            } else {
                theme.TEXT_MUTED
            };

            Frame::NONE
                .fill(badge_bg)
                .stroke(Stroke::new(
                    1.0,
                    if is_dedicated {
                        theme.SAGE_BORDER
                    } else {
                        theme.BORDER_DARK
                    },
                ))
                .corner_radius(CornerRadius::same(3))
                .inner_margin(Margin::symmetric(6, 2))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(if is_dedicated { "DEDICATED" } else { "SHARED" })
                            .font(FontId::monospace(8.0))
                            .color(badge_txt),
                    );
                });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .button(RichText::new("⟳").color(theme.TEXT_MUTED))
                    .on_hover_text("Rescan connected keyboards")
                    .clicked()
                {
                    let fresh = iokit::refresh_keyboards();
                    if let Ok(mut devs) = self.state.devices.lock() {
                        *devs = fresh;
                    }
                }
            });
        });

        ui.add_space(6.0);

        let devices = self.state.devices.lock().unwrap().clone();
        let current_ptr = self.state.selected_device_ptr.load(Ordering::SeqCst);
        let (current_name, current_detail) = if current_ptr == 0 {
            (
                "All Keyboards (Omni)".to_string(),
                "Built-in & External · Shared",
            )
        } else if current_ptr == usize::MAX {
            (
                "All External Keyboards".to_string(),
                "External only · Dedicated",
            )
        } else {
            devices
                .iter()
                .find(|d| d.device_ptr == current_ptr)
                .map(|d| {
                    (
                        d.name.clone(),
                        if d.is_internal {
                            "Built-in · Shared"
                        } else {
                            "External · Dedicated"
                        },
                    )
                })
                .unwrap_or_else(|| ("All Keyboards (Omni)".to_string(), "Shared"))
        };

        // Custom styled card dropdown
        Frame::NONE
            .fill(theme.BG_FIELD)
            .stroke(Stroke::new(1.0, theme.BORDER_FIELD))
            .corner_radius(CornerRadius::same(5))
            .inner_margin(Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    let combo_w = (ui.available_width() - 4.0).max(180.0);
                    egui::ComboBox::from_id_salt("device_picker_box")
                        .width(combo_w)
                        .selected_text(
                            RichText::new(&current_name)
                                .strong()
                                .font(FontId::proportional(11.0))
                                .color(theme.TEXT_PRIMARY),
                        )
                        .show_ui(ui, |ui| {
                            let is_omni = current_ptr == 0;
                            if ui
                                .selectable_label(is_omni, "All Keyboards (Omni)")
                                .clicked()
                            {
                                self.select_all_keyboards();
                            }
                            let is_ext_only = current_ptr == usize::MAX;
                            if ui
                                .selectable_label(is_ext_only, "All External Keyboards")
                                .clicked()
                            {
                                self.select_external_keyboards();
                            }
                            ui.separator();
                            for dev in &devices {
                                let label = format!("{} ({})", dev.name, dev.transport);
                                let is_selected = dev.device_ptr == current_ptr;
                                if ui.selectable_label(is_selected, label).clicked() {
                                    self.switch_keyboard(dev);
                                }
                            }
                        });

                    ui.label(
                        RichText::new(current_detail)
                            .font(FontId::monospace(8.5))
                            .color(theme.TEXT_MUTED),
                    );
                });
            });

        ui.add_space(4.0);
        ui.add(
            egui::Label::new(
                RichText::new("Targeted external keyboards suppress system keystrokes.")
                    .font(FontId::proportional(8.5))
                    .color(theme.TEXT_MUTED),
            )
            .wrap(),
        );
    }

    /// Column 2: Scale root note grid & scale mode selection
    fn render_scale_column(&mut self, ui: &mut Ui) {
        let theme = self.theme_colors;
        let current_root = self.state.get_root();
        let current_scale = self.state.get_scale();

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("SCALE")
                    .font(FontId::proportional(11.0))
                    .strong()
                    .color(theme.TEXT_HEADING),
            );

            Frame::NONE
                .fill(theme.SAGE_BADGE_BG)
                .stroke(Stroke::new(1.0, theme.SAGE_BORDER))
                .corner_radius(CornerRadius::same(3))
                .inner_margin(Margin::symmetric(6, 2))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(
                            format!("{} {}", current_root.name(), current_scale.name())
                                .to_uppercase(),
                        )
                        .font(FontId::monospace(8.5))
                        .strong()
                        .color(theme.SAGE_BADGE_TXT),
                    );
                });
        });

        ui.add_space(6.0);

        ui.horizontal_top(|ui| {
            // Left sub-column: Root Note 6x2 grid
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("ROOT NOTE")
                        .font(FontId::monospace(8.5))
                        .color(theme.TEXT_MUTED),
                );

                let root_rows = [&RootNote::ALL[0..6], &RootNote::ALL[6..12]];

                for row in root_rows {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
                        for &root in row {
                            let is_sel = root == current_root;
                            let btn_text = RichText::new(root.name())
                                .font(FontId::monospace(9.0))
                                .strong()
                                .color(if is_sel {
                                    theme.SAGE_SELECTED_TXT
                                } else {
                                    theme.TEXT_LIGHT_MUTED
                                });

                            let btn = egui::Button::new(btn_text)
                                .fill(if is_sel {
                                    theme.SAGE_SELECTED_BG
                                } else {
                                    theme.BG_BUTTON
                                })
                                .stroke(Stroke::new(
                                    1.0,
                                    if is_sel {
                                        theme.SAGE_SELECTED_BG
                                    } else {
                                        theme.BORDER_GRID
                                    },
                                ))
                                .corner_radius(CornerRadius::same(3))
                                .min_size(vec2(28.0, 20.0));

                            if ui.add(btn).clicked() {
                                self.state.set_root(root);
                            }
                        }
                    });
                }
            });

            ui.add_space(10.0);

            // Right sub-column: Scale Mode selector
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("MODE / SCALE")
                        .font(FontId::monospace(8.5))
                        .color(theme.TEXT_MUTED),
                );

                // Quick buttons for 6 core scales in 2 rows of 3
                let quick_scales: [[ScaleType; 3]; 2] = [
                    [ScaleType::NaturalMinor, ScaleType::Major, ScaleType::Dorian],
                    [
                        ScaleType::MinorPentatonic,
                        ScaleType::HarmonicMinor,
                        ScaleType::Mixolydian,
                    ],
                ];

                for row in quick_scales {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(3.0, 3.0);
                        for st in row {
                            let is_sel = st == current_scale;
                            let label = match st {
                                ScaleType::NaturalMinor => "Minor",
                                ScaleType::Major => "Major",
                                ScaleType::Dorian => "Dorian",
                                ScaleType::MinorPentatonic => "Pent Min",
                                ScaleType::HarmonicMinor => "Harm Min",
                                ScaleType::Mixolydian => "Mixolydian",
                                _ => st.name(),
                            };
                            let btn = egui::Button::new(
                                RichText::new(label).font(FontId::monospace(8.5)).color(
                                    if is_sel {
                                        theme.SAGE_BADGE_TXT
                                    } else {
                                        theme.TEXT_LIGHT_MUTED
                                    },
                                ),
                            )
                            .fill(if is_sel {
                                theme.SAGE_BADGE_BG
                            } else {
                                theme.BG_BUTTON
                            })
                            .stroke(Stroke::new(
                                1.0,
                                if is_sel {
                                    theme.SAGE_BORDER
                                } else {
                                    theme.BORDER_GRID
                                },
                            ))
                            .corner_radius(CornerRadius::same(3))
                            .min_size(vec2(66.0, 20.0));

                            if ui.add(btn).clicked() {
                                self.state.set_scale(st);
                            }
                        }
                    });
                }

                ui.add_space(3.0);

                // Full dropdown for all 13 scales
                egui::ComboBox::from_id_salt("all_scales_select")
                    .width(204.0)
                    .selected_text(
                        RichText::new(current_scale.name())
                            .font(FontId::proportional(10.5))
                            .strong()
                            .color(theme.TEXT_PRIMARY),
                    )
                    .show_ui(ui, |ui| {
                        for st in ScaleType::ALL {
                            if ui
                                .selectable_label(st == current_scale, st.name())
                                .clicked()
                            {
                                self.state.set_scale(st);
                            }
                        }
                    });
            });
        });
    }

    /// Column 3: Output controls (Base octave, mapping padding, velocity, mapping preset)
    fn render_output_column(&mut self, ui: &mut Ui) {
        let theme = self.theme_colors;
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("OUTPUT")
                    .font(FontId::proportional(11.0))
                    .strong()
                    .color(theme.TEXT_HEADING),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let btn = egui::Button::new(
                    RichText::new("⚙ SETTINGS")
                        .font(FontId::monospace(9.0))
                        .strong()
                        .color(if self.settings_open {
                            theme.TEXT_PRIMARY
                        } else {
                            theme.TEXT_MUTED
                        }),
                )
                .fill(if self.settings_open {
                    theme.BG_BUTTON_HOVER
                } else {
                    theme.BG_BUTTON
                })
                .stroke(Stroke::new(
                    1.0,
                    if self.settings_open {
                        theme.SAGE_ACCENT
                    } else {
                        theme.BORDER_DARK
                    },
                ))
                .corner_radius(CornerRadius::same(4));

                if ui.add(btn).clicked() {
                    self.settings_open = !self.settings_open;
                }
            });
        });

        ui.add_space(6.0);

        ui.horizontal_top(|ui| {
            // Left sub-column: Row 1 = BASE and PAD steppers; Row 2 = VEL slider
            ui.vertical(|ui| {
                // Row 1: BASE and PAD side by side on the exact same line
                ui.allocate_ui_with_layout(
                    vec2(ui.available_width(), 22.0),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                        // Base Stepper
                        let base_oct = self.state.base_octave.load(Ordering::SeqCst);
                        let (base_dec, base_inc) = self.render_stepper(
                            ui,
                            "base_oct_stepper",
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

                        // PAD Stepper
                        let pad = self.state.mapping_padding.load(Ordering::SeqCst);
                        let pad_str = if pad > 0 {
                            format!("+{pad}")
                        } else {
                            format!("{pad}")
                        };
                        let (pad_dec, pad_inc) = self.render_stepper(
                            ui,
                            "mapping_pad_stepper",
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
                    },
                );

                ui.add_space(6.0);

                // Row 2: Velocity slider below Base & Pad
                ui.allocate_ui_with_layout(
                    vec2(ui.available_width(), 22.0),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing = vec2(3.5, 0.0);

                        let font_id = FontId::monospace(8.5);
                        let galley = ui
                            .painter()
                            .layout_no_wrap("VEL:".to_string(), font_id, theme.TEXT_MUTED);
                        let (label_rect, _) =
                            ui.allocate_exact_size(vec2(galley.size().x, 22.0), Sense::hover());
                        ui.painter().galley(
                            Pos2::new(
                                label_rect.min.x,
                                label_rect.center().y - galley.size().y * 0.5,
                            ),
                            galley,
                            theme.TEXT_MUTED,
                        );

                        let mut vel = self.state.velocity.load(Ordering::SeqCst);
                        let slider = egui::Slider::new(&mut vel, 10..=127).show_value(true);
                        if ui.add_sized(vec2(175.0, 22.0), slider).changed() {
                            self.state.velocity.store(vel, Ordering::SeqCst);
                        }
                    },
                );
            });

            ui.add_space(28.0);

            // Right sub-column: MAPPING label above preset dropdown
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("MAPPING")
                        .font(FontId::monospace(8.5))
                        .color(theme.TEXT_MUTED),
                );

                ui.add_space(3.0);

                let current_preset = self.state.get_layout_preset();
                egui::ComboBox::from_id_salt("layout_preset_combobox")
                    .width(180.0)
                    .selected_text(
                        RichText::new(current_preset.name())
                            .font(FontId::proportional(10.0))
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
            });
        });
    }

    /// Renders an aligned stepper widget: label on left, framed counter box with minus/value/plus on right.
    fn render_stepper(
        &self,
        ui: &mut Ui,
        id_salt: &str,
        label: &str,
        value_text: &str,
        can_dec: bool,
        can_inc: bool,
    ) -> (bool, bool) {
        let theme = self.theme_colors;
        let row_h = 22.0;
        let mut dec_clicked = false;
        let mut inc_clicked = false;

        ui.push_id(id_salt, |ui| {
            // 1. Label ("BASE:", "PAD:") - measured to exact text width, vertically centered
            let font_id = FontId::monospace(8.5);
            let galley = ui
                .painter()
                .layout_no_wrap(label.to_string(), font_id, theme.TEXT_MUTED);
            let (label_rect, _) =
                ui.allocate_exact_size(vec2(galley.size().x, row_h), Sense::hover());
            ui.painter().galley(
                Pos2::new(
                    label_rect.min.x,
                    label_rect.center().y - galley.size().y * 0.5,
                ),
                galley,
                theme.TEXT_MUTED,
            );

            ui.add_space(3.5);

            // 2. Styled Counter Box [ −  VAL  + ]
            Frame::NONE
                .fill(theme.BG_FIELD)
                .stroke(Stroke::new(1.0, theme.BORDER_FIELD))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(Margin::symmetric(2, 2))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(2.0, 0.0);

                        if ui
                            .button(
                                RichText::new("−")
                                    .color(if can_dec { theme.TEXT_MUTED } else { theme.BORDER_GRID }),
                            )
                            .clicked()
                            && can_dec
                        {
                            dec_clicked = true;
                        }

                        ui.label(
                            RichText::new(value_text)
                                .font(FontId::monospace(10.0))
                                .strong()
                                .color(theme.TEXT_PRIMARY),
                        );

                        if ui
                            .button(
                                RichText::new("+")
                                    .color(if can_inc { theme.TEXT_MUTED } else { theme.BORDER_GRID }),
                            )
                            .clicked()
                            && can_inc
                        {
                            inc_clicked = true;
                        }
                    });
                });
        });

        (dec_clicked, inc_clicked)
    }

    // ------------------------------------------------------------------------
    // DEVICE SWITCHING LOGIC
    // ------------------------------------------------------------------------
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
