pub mod bottom_bar;
pub mod keyboard;
pub mod piano;
pub mod settings;
pub mod setup_bar;
pub mod theme;

use std::sync::Arc;
use std::sync::atomic::Ordering;

use egui::{Align, Frame, Layout, Margin, Stroke, vec2};

use crate::midi::MidiEngine;
use crate::state::AppState;

pub use theme::{AppTheme, ThemeColors};

pub struct QwerkeysApp {
    pub(super) state: Arc<AppState>,
    pub(super) midi: Arc<MidiEngine>,
    pub(super) clicked_usages: Vec<u32>,
    pub(super) clicked_piano_notes: Vec<u8>,
    pub(super) current_theme: AppTheme,
    pub(super) theme_colors: ThemeColors,
    pub(super) settings_open: bool,
    pub(super) settings_icon_rot: f32,
}

impl QwerkeysApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        state: Arc<AppState>,
        midi: Arc<MidiEngine>,
    ) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Register the Phosphor icon font (used by the settings gear button)
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);
        state.set_egui_ctx(cc.egui_ctx.clone());

        // Persisted theme (fall back to the default theme if unknown)
        let current_theme = AppTheme::ALL
            .iter()
            .copied()
            .find(|t| *t as usize == state.theme_index.load(Ordering::SeqCst))
            .unwrap_or_default();
        let theme_colors = current_theme.colors();

        let mut visuals = if current_theme.is_light() {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        visuals.panel_fill = theme_colors.BG_APP;
        visuals.window_fill = theme_colors.BG_APP;
        visuals.override_text_color = Some(theme_colors.TEXT_SECONDARY);
        cc.egui_ctx.set_visuals(visuals);

        Self {
            state,
            midi,
            clicked_usages: Vec::new(),
            clicked_piano_notes: Vec::new(),
            current_theme,
            theme_colors,
            settings_open: false,
            settings_icon_rot: 0.0,
        }
    }

    pub(super) fn set_theme(&mut self, theme: AppTheme, ctx: &egui::Context) {
        self.current_theme = theme;
        self.theme_colors = theme.colors();
        let mut visuals = if theme.is_light() {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        visuals.panel_fill = self.theme_colors.BG_APP;
        visuals.window_fill = self.theme_colors.BG_APP;
        visuals.override_text_color = Some(self.theme_colors.TEXT_SECONDARY);
        ctx.set_visuals(visuals);

        // Save-on-change: remember the new theme in state, then snapshot settings
        self.state
            .theme_index
            .store(theme as usize, Ordering::SeqCst);
        self.state.persist();
    }
}

impl eframe::App for QwerkeysApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let theme = self.theme_colors;
        let ctx = ui.ctx().clone();
        self.state.set_egui_ctx(ctx.clone());

        // 1. Edge-to-edge full width setup bar
        egui::Panel::top("top_setup_bar")
            .frame(
                Frame::NONE
                    .fill(theme.BG_BAR)
                    .stroke(Stroke::new(1.0, theme.BORDER_DARK))
                    .inner_margin(Margin {
                        left: 18,
                        right: 18,
                        top: 10,
                        bottom: 10,
                    }),
            )
            .show(ui, |ui| {
                self.render_setup_bar(ui);
            });

        // 2. Edge-to-edge full width bottom status bar sticking to bottom
        egui::Panel::bottom("bottom_status_bar")
            .show_separator_line(false)
            .frame(
                Frame::NONE
                    .fill(theme.BG_BAR)
                    .stroke(Stroke::new(0.5, theme.BORDER_DARK))
                    .inner_margin(Margin {
                        left: 18,
                        right: 18,
                        top: 3,
                        bottom: 3,
                    }),
            )
            .show(ui, |ui| {
                self.render_bottom_bar(ui);
            });

        self.render_settings_window(&ctx);

        // 3. Centered workspace between top bar and bottom status bar
        let panel_frame = Frame::NONE.fill(theme.BG_APP).inner_margin(Margin::ZERO);

        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ui, |ui| {
                let max_content_w = 1240.0;
                let side_padding = 18.0;

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .scroll_bar_visibility(
                        egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                    )
                    .show(ui, |ui| {
                        let avail_w = ui.available_width();
                        let max_w = (avail_w - 2.0 * side_padding).max(600.0);
                        let content_w = max_w.min(max_content_w);
                        let extra_margin = side_padding + ((max_w - content_w) * 0.5).max(0.0);

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if extra_margin > 0.0 {
                                ui.add_space(extra_margin);
                            }
                            ui.allocate_ui_with_layout(
                                vec2(content_w, 0.0),
                                Layout::top_down(Align::Min),
                                |ui| {
                                    ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
                                    self.render_piano(ui);
                                    self.render_qwerty_keyboard(ui);
                                    ui.add_space(2.0);
                                },
                            );
                            if extra_margin > 0.0 {
                                ui.add_space(extra_margin);
                            }
                        });
                    });
            });
    }
}
