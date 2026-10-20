pub mod bottom_bar;
pub mod keyboard;
pub mod piano;
pub mod settings;
pub mod setup_bar;
pub mod theme;

use std::sync::Arc;

use egui::{vec2, Align, Frame, Layout, Margin, Stroke};

use crate::midi::MidiEngine;
use crate::state::AppState;

pub use theme::{AppTheme, ThemeColors};

pub struct QwerkeysApp {
    pub(super) state: Arc<AppState>,
    pub(super) midi: Arc<MidiEngine>,
    pub(super) clicked_usages: Vec<u32>,
    pub(super) clicked_piano_notes: Vec<u8>,
    pub(super) aspect_ratio_set: bool,
    pub(super) current_theme: AppTheme,
    pub(super) theme_colors: ThemeColors,
    pub(super) settings_open: bool,
}

#[cfg(target_os = "macos")]
fn try_set_macos_aspect_ratio(width: f64, height: f64) -> bool {
    use std::ffi::{c_char, c_void};

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct NSSize {
        width: f64,
        height: f64,
    }

    #[link(name = "AppKit", kind = "framework")]
    unsafe extern "C" {
        fn objc_getClass(name: *const c_char) -> *mut c_void;
        fn sel_registerName(name: *const c_char) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    }

    unsafe {
        let msg_send = dlsym(-2isize as *mut c_void, b"objc_msgSend\0".as_ptr() as *const c_char);
        if msg_send.is_null() {
            return false;
        }

        let get_ptr: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
            std::mem::transmute(msg_send);
        let get_usize: unsafe extern "C" fn(*mut c_void, *mut c_void) -> usize =
            std::mem::transmute(msg_send);
        let get_obj_at: unsafe extern "C" fn(*mut c_void, *mut c_void, usize) -> *mut c_void =
            std::mem::transmute(msg_send);
        let set_size: unsafe extern "C" fn(*mut c_void, *mut c_void, NSSize) =
            std::mem::transmute(msg_send);

        let cls = objc_getClass(b"NSApplication\0".as_ptr() as *const c_char);
        if cls.is_null() {
            return false;
        }
        let sel_app = sel_registerName(b"sharedApplication\0".as_ptr() as *const c_char);
        let app = get_ptr(cls, sel_app);
        if app.is_null() {
            return false;
        }

        let sel_windows = sel_registerName(b"windows\0".as_ptr() as *const c_char);
        let windows = get_ptr(app, sel_windows);
        if windows.is_null() {
            return false;
        }

        let sel_count = sel_registerName(b"count\0".as_ptr() as *const c_char);
        let count = get_usize(windows, sel_count);
        if count == 0 {
            return false;
        }

        let sel_object_at = sel_registerName(b"objectAtIndex:\0".as_ptr() as *const c_char);
        let sel_set_aspect_ratio = sel_registerName(b"setAspectRatio:\0".as_ptr() as *const c_char);

        let mut applied = false;
        for i in 0..count {
            let win = get_obj_at(windows, sel_object_at, i);
            if !win.is_null() {
                set_size(win, sel_set_aspect_ratio, NSSize { width, height });
                applied = true;
            }
        }
        applied
    }
}

impl QwerkeysApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        state: Arc<AppState>,
        midi: Arc<MidiEngine>,
    ) -> Self {
        state.set_egui_ctx(cc.egui_ctx.clone());

        let current_theme = AppTheme::StudioSlate;
        let theme_colors = current_theme.colors();

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = theme_colors.BG_APP;
        visuals.window_fill = theme_colors.BG_APP;
        visuals.override_text_color = Some(theme_colors.TEXT_SECONDARY);
        cc.egui_ctx.set_visuals(visuals);

        Self {
            state,
            midi,
            clicked_usages: Vec::new(),
            clicked_piano_notes: Vec::new(),
            aspect_ratio_set: false,
            current_theme,
            theme_colors,
            settings_open: false,
        }
    }


    pub(super) fn set_theme(&mut self, theme: AppTheme, ctx: &egui::Context) {
        self.current_theme = theme;
        self.theme_colors = theme.colors();
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = self.theme_colors.BG_APP;
        visuals.window_fill = self.theme_colors.BG_APP;
        visuals.override_text_color = Some(self.theme_colors.TEXT_SECONDARY);
        ctx.set_visuals(visuals);
    }


}

impl eframe::App for QwerkeysApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let theme = self.theme_colors;
        let ctx = ui.ctx().clone();
        self.state.set_egui_ctx(ctx.clone());

        #[cfg(target_os = "macos")]
        if !self.aspect_ratio_set && try_set_macos_aspect_ratio(1160.0, 740.0) {
            self.aspect_ratio_set = true;
        }

        // Maintain aspect ratio if resized on platforms without native aspect ratio locking
        let target_ratio = 1160.0 / 740.0;
        let is_fullscreen = ui.input(|i| i.viewport().fullscreen.unwrap_or(false));
        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        if !is_fullscreen && !is_maximized {
            if let Some(rect) = ui.input(|i| i.viewport().inner_rect) {
                let size = rect.size();
                if size.x > 100.0 && size.y > 100.0 {
                    let cur_ratio = size.x / size.y;
                    if (cur_ratio - target_ratio).abs() > 0.05 {
                        let expected_h = (size.x / target_ratio).round();
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::InnerSize(vec2(size.x, expected_h)));
                    }
                }
            }
        }

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
        let panel_frame = Frame::NONE
            .fill(theme.BG_APP)
            .inner_margin(Margin::ZERO);

        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ui, |ui| {
                let max_content_w = 1240.0;
                let side_padding = 18.0;

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
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
