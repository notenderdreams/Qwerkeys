use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Frame, Layout, Margin, Pos2, Rect, RichText,
    Sense, Stroke, Ui, vec2,
};

use crate::iokit::{self, DeviceTargetMode, KeyboardDevice};
use crate::mapping::{
    KEY_DEFINITIONS, KeyMappingDef, LayoutPreset, calculate_key_notes, find_key_by_label,
    get_key_display_info,
};
use crate::midi::MidiEngine;
use crate::scale::{RootNote, ScaleType};
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppTheme {
    #[default]
    StudioSlate,
    StudioSage,
}

impl AppTheme {
    pub const ALL: [AppTheme; 2] = [AppTheme::StudioSlate, AppTheme::StudioSage];

    pub fn name(&self) -> &'static str {
        match self {
            AppTheme::StudioSlate => "Studio Slate",
            AppTheme::StudioSage => "Studio Sage",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AppTheme::StudioSlate => "Refined dark graphite with slate-blue accents",
            AppTheme::StudioSage => "Warm dark studio with organic sage green accents",
        }
    }

    pub fn colors(&self) -> ThemeColors {
        match self {
            AppTheme::StudioSlate => ThemeColors::studio_slate(),
            AppTheme::StudioSage => ThemeColors::studio_sage(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case, dead_code)]
pub struct ThemeColors {
    pub BG_APP: Color32,
    pub BG_BAR: Color32,
    pub BG_PANEL: Color32,
    pub BG_KEYBOARD: Color32,
    pub BG_FIELD: Color32,
    pub BG_BUTTON: Color32,
    pub BG_BUTTON_HOVER: Color32,

    pub BORDER_DARK: Color32,
    pub BORDER_PANEL: Color32,
    pub BORDER_KEYBOARD: Color32,
    pub BORDER_FIELD: Color32,
    pub BORDER_GRID: Color32,

    pub SAGE_ACCENT: Color32,
    pub SAGE_ACTIVE: Color32,
    pub SAGE_BORDER: Color32,
    pub SAGE_BADGE_BG: Color32,
    pub SAGE_BADGE_TXT: Color32,
    pub SAGE_SELECTED_BG: Color32,
    pub SAGE_SELECTED_TXT: Color32,

    pub TEXT_PRIMARY: Color32,
    pub TEXT_SECONDARY: Color32,
    pub TEXT_MUTED: Color32,
    pub TEXT_LIGHT_MUTED: Color32,
    pub TEXT_HEADING: Color32,

    pub KEY_UNMAPPED_BG: Color32,
    pub KEY_UNMAPPED_BORDER: Color32,
    pub KEY_UNMAPPED_TXT: Color32,

    pub KEY_MAPPED_BG: Color32,
    pub KEY_MAPPED_BORDER: Color32,
    pub KEY_MAPPED_TXT: Color32,
    pub KEY_MAPPED_SUB: Color32,

    pub KEY_ROOT_BG: Color32,
    pub KEY_ROOT_BORDER: Color32,

    pub KEY_ACTIVE_BG: Color32,
    pub KEY_ACTIVE_BORDER: Color32,
    pub KEY_ACTIVE_TXT: Color32,

    pub PIANO_WHITE_BG: Color32,
    pub PIANO_WHITE_BORDER: Color32,
    pub PIANO_WHITE_ROOT: Color32,
    pub PIANO_WHITE_ACTIVE: Color32,
    pub PIANO_WHITE_NOTE: Color32,

    pub PIANO_BLACK_BG: Color32,
    pub PIANO_BLACK_BORDER: Color32,
    pub PIANO_BLACK_ACTIVE: Color32,
    pub PIANO_BLACK_TXT: Color32,

    pub DOT_LIT: Color32,
    pub DOT_UNLIT: Color32,

    pub KEY_SUB_TRIGGERED: Color32,
    pub KEY_LABEL_SHIFT_TRIGGERED: Color32,
    pub KEY_LABEL_KEY_TRIGGERED: Color32,
    pub PIANO_WHITE_MAPPED_ROOT_TXT: Color32,
    pub PIANO_WHITE_MAPPED_TXT: Color32,
    pub PIANO_WHITE_ROOT_TXT: Color32,
    pub PIANO_WHITE_OUT_SCALE_TXT: Color32,
    pub PIANO_WHITE_OUT_SCALE_BG: Color32,
    pub PIANO_BLACK_IN_SCALE_BG: Color32,
    pub PIANO_BLACK_ROOT_BG: Color32,
    pub PIANO_BLACK_IN_SCALE_TXT: Color32,
    pub PIANO_BLACK_OUT_SCALE_TXT: Color32,
}

impl ThemeColors {
    pub fn studio_slate() -> Self {
        Self {
            BG_APP: Color32::from_rgb(0x23, 0x23, 0x23),
            BG_BAR: Color32::from_rgb(0x2B, 0x2B, 0x2B),
            BG_PANEL: Color32::from_rgb(0x30, 0x30, 0x30),
            BG_KEYBOARD: Color32::from_rgb(0x1C, 0x1C, 0x1C),
            BG_FIELD: Color32::from_rgb(0x29, 0x29, 0x29),
            BG_BUTTON: Color32::from_rgb(0x40, 0x40, 0x40),
            BG_BUTTON_HOVER: Color32::from_rgb(0x50, 0x50, 0x50),
            BORDER_DARK: Color32::from_rgb(0x38, 0x38, 0x38),
            BORDER_PANEL: Color32::from_rgb(0x3D, 0x3D, 0x3D),
            BORDER_KEYBOARD: Color32::from_rgb(0x37, 0x37, 0x37),
            BORDER_FIELD: Color32::from_rgb(0x42, 0x42, 0x42),
            BORDER_GRID: Color32::from_rgb(0x48, 0x48, 0x48),
            SAGE_ACCENT: Color32::from_rgb(0x54, 0x5A, 0x99),
            SAGE_ACTIVE: Color32::from_rgb(0x52, 0x58, 0x97),
            SAGE_BORDER: Color32::from_rgb(0x54, 0x5A, 0x99),
            SAGE_BADGE_BG: Color32::from_rgb(0x32, 0x36, 0x54),
            SAGE_BADGE_TXT: Color32::from_rgb(0xBD, 0xCB, 0xEC),
            SAGE_SELECTED_BG: Color32::from_rgb(0x54, 0x5A, 0x99),
            SAGE_SELECTED_TXT: Color32::from_rgb(0xFD, 0xFD, 0xFF),
            TEXT_PRIMARY: Color32::from_rgb(0xFD, 0xFD, 0xFF),
            TEXT_SECONDARY: Color32::from_rgb(0xE6, 0xE6, 0xE6),
            TEXT_MUTED: Color32::from_rgb(0xAA, 0xAA, 0xAA),
            TEXT_LIGHT_MUTED: Color32::from_rgb(0x80, 0x80, 0x80),
            TEXT_HEADING: Color32::from_rgb(0xC9, 0xC9, 0xC9),
            KEY_UNMAPPED_BG: Color32::from_rgb(0x2B, 0x2B, 0x2B),
            KEY_UNMAPPED_BORDER: Color32::from_rgb(0x37, 0x37, 0x37),
            KEY_UNMAPPED_TXT: Color32::from_rgb(0x78, 0x78, 0x78),
            KEY_MAPPED_BG: Color32::from_rgb(0x2A, 0x2D, 0x42),
            KEY_MAPPED_BORDER: Color32::from_rgb(0x47, 0x4E, 0x7A),
            KEY_MAPPED_TXT: Color32::from_rgb(0xD8, 0xDB, 0xF5),
            KEY_MAPPED_SUB: Color32::from_rgb(0x94, 0x99, 0xC7),
            KEY_ROOT_BG: Color32::from_rgb(0x34, 0x3A, 0x58),
            KEY_ROOT_BORDER: Color32::from_rgb(0x6A, 0x73, 0xB0),
            KEY_ACTIVE_BG: Color32::from_rgb(0x54, 0x5A, 0x99),
            KEY_ACTIVE_BORDER: Color32::from_rgb(0x8C, 0x93, 0xD4),
            KEY_ACTIVE_TXT: Color32::from_rgb(0xFD, 0xFD, 0xFF),
            PIANO_WHITE_BG: Color32::from_rgb(0xE6, 0xE6, 0xE6),
            PIANO_WHITE_BORDER: Color32::from_rgb(0xC8, 0xC8, 0xC8),
            PIANO_WHITE_ROOT: Color32::from_rgb(0xC8, 0xCB, 0xE0),
            PIANO_WHITE_ACTIVE: Color32::from_rgb(0x54, 0x5A, 0x99),
            PIANO_WHITE_NOTE: Color32::from_rgb(0x60, 0x60, 0x60),
            PIANO_BLACK_BG: Color32::from_rgb(0x1C, 0x1C, 0x1C),
            PIANO_BLACK_BORDER: Color32::from_rgb(0x12, 0x12, 0x12),
            PIANO_BLACK_ACTIVE: Color32::from_rgb(0x54, 0x5A, 0x99),
            PIANO_BLACK_TXT: Color32::from_rgb(0xAA, 0xAA, 0xAA),
            DOT_LIT: Color32::from_rgb(0x54, 0x5A, 0x99),
            DOT_UNLIT: Color32::from_rgb(0x37, 0x37, 0x37),
            KEY_SUB_TRIGGERED: Color32::from_rgb(0xF0, 0xF2, 0xFD),
            KEY_LABEL_SHIFT_TRIGGERED: Color32::from_rgb(0xD8, 0xDB, 0xF5),
            KEY_LABEL_KEY_TRIGGERED: Color32::from_rgb(0xEB, 0xED, 0xFD),
            PIANO_WHITE_MAPPED_ROOT_TXT: Color32::from_rgb(0x34, 0x3A, 0x58),
            PIANO_WHITE_MAPPED_TXT: Color32::from_rgb(0x47, 0x4E, 0x7A),
            PIANO_WHITE_ROOT_TXT: Color32::from_rgb(0x34, 0x3A, 0x58),
            PIANO_WHITE_OUT_SCALE_TXT: Color32::from_rgb(0x8A, 0x8A, 0x8A),
            PIANO_WHITE_OUT_SCALE_BG: Color32::from_rgb(0xC0, 0xC0, 0xC0),
            PIANO_BLACK_IN_SCALE_BG: Color32::from_rgb(0x24, 0x27, 0x36),
            PIANO_BLACK_ROOT_BG: Color32::from_rgb(0x34, 0x3A, 0x58),
            PIANO_BLACK_IN_SCALE_TXT: Color32::from_rgb(0xC8, 0xCB, 0xE8),
            PIANO_BLACK_OUT_SCALE_TXT: Color32::from_rgb(0x60, 0x60, 0x60),
        }
    }

    pub fn studio_sage() -> Self {
        Self {
            BG_APP: Color32::from_rgb(0x15, 0x15, 0x15),
            BG_BAR: Color32::from_rgb(0x1B, 0x1B, 0x1B),
            BG_PANEL: Color32::from_rgb(0x20, 0x20, 0x20),
            BG_KEYBOARD: Color32::from_rgb(0x18, 0x18, 0x18),
            BG_FIELD: Color32::from_rgb(0x2A, 0x2A, 0x2A),
            BG_BUTTON: Color32::from_rgb(0x22, 0x22, 0x22),
            BG_BUTTON_HOVER: Color32::from_rgb(0x2E, 0x2E, 0x2E),
            BORDER_DARK: Color32::from_rgb(0x34, 0x34, 0x34),
            BORDER_PANEL: Color32::from_rgb(0x38, 0x38, 0x38),
            BORDER_KEYBOARD: Color32::from_rgb(0x3F, 0x3F, 0x3F),
            BORDER_FIELD: Color32::from_rgb(0x45, 0x45, 0x45),
            BORDER_GRID: Color32::from_rgb(0x3B, 0x3B, 0x3B),
            SAGE_ACCENT: Color32::from_rgb(0x93, 0xA8, 0x79),
            SAGE_ACTIVE: Color32::from_rgb(0x74, 0x86, 0x5F),
            SAGE_BORDER: Color32::from_rgb(0x5A, 0x68, 0x4E),
            SAGE_BADGE_BG: Color32::from_rgb(0x39, 0x43, 0x32),
            SAGE_BADGE_TXT: Color32::from_rgb(0xBE, 0xCD, 0xAD),
            SAGE_SELECTED_BG: Color32::from_rgb(0xDF, 0xE5, 0xD8),
            SAGE_SELECTED_TXT: Color32::from_rgb(0x30, 0x38, 0x2A),
            TEXT_PRIMARY: Color32::from_rgb(0xF0, 0xF1, 0xED),
            TEXT_SECONDARY: Color32::from_rgb(0xDE, 0xDF, 0xD9),
            TEXT_MUTED: Color32::from_rgb(0x85, 0x8B, 0x80),
            TEXT_LIGHT_MUTED: Color32::from_rgb(0x99, 0x9F, 0x95),
            TEXT_HEADING: Color32::from_rgb(0xAE, 0xBD, 0x98),
            KEY_UNMAPPED_BG: Color32::from_rgb(0x2B, 0x2B, 0x2B),
            KEY_UNMAPPED_BORDER: Color32::from_rgb(0x46, 0x46, 0x46),
            KEY_UNMAPPED_TXT: Color32::from_rgb(0xB1, 0xB6, 0xAC),
            KEY_MAPPED_BG: Color32::from_rgb(0x33, 0x3C, 0x2E),
            KEY_MAPPED_BORDER: Color32::from_rgb(0x5A, 0x68, 0x4E),
            KEY_MAPPED_TXT: Color32::from_rgb(0xD0, 0xDC, 0xC2),
            KEY_MAPPED_SUB: Color32::from_rgb(0x9F, 0xAF, 0x8D),
            KEY_ROOT_BG: Color32::from_rgb(0x3A, 0x47, 0x34),
            KEY_ROOT_BORDER: Color32::from_rgb(0x8E, 0x9C, 0x7C),
            KEY_ACTIVE_BG: Color32::from_rgb(0x74, 0x86, 0x5F),
            KEY_ACTIVE_BORDER: Color32::from_rgb(0xA3, 0xB5, 0x8A),
            KEY_ACTIVE_TXT: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            PIANO_WHITE_BG: Color32::from_rgb(0xE4, 0xE6, 0xDF),
            PIANO_WHITE_BORDER: Color32::from_rgb(0xD6, 0xD8, 0xD1),
            PIANO_WHITE_ROOT: Color32::from_rgb(0xCD, 0xD6, 0xC2),
            PIANO_WHITE_ACTIVE: Color32::from_rgb(0x93, 0xA8, 0x79),
            PIANO_WHITE_NOTE: Color32::from_rgb(0x73, 0x79, 0x6F),
            PIANO_BLACK_BG: Color32::from_rgb(0x24, 0x27, 0x22),
            PIANO_BLACK_BORDER: Color32::from_rgb(0x0E, 0x0F, 0x0D),
            PIANO_BLACK_ACTIVE: Color32::from_rgb(0x74, 0x86, 0x5F),
            PIANO_BLACK_TXT: Color32::from_rgb(0xBD, 0xC3, 0xB6),
            DOT_LIT: Color32::from_rgb(0x91, 0xA4, 0x77),
            DOT_UNLIT: Color32::from_rgb(0x51, 0x56, 0x4E),
            KEY_SUB_TRIGGERED: Color32::from_rgb(238, 242, 233),
            KEY_LABEL_SHIFT_TRIGGERED: Color32::from_rgb(200, 215, 190),
            KEY_LABEL_KEY_TRIGGERED: Color32::from_rgb(220, 230, 210),
            PIANO_WHITE_MAPPED_ROOT_TXT: Color32::from_rgb(0x35, 0x40, 0x2E),
            PIANO_WHITE_MAPPED_TXT: Color32::from_rgb(0x55, 0x5D, 0x4E),
            PIANO_WHITE_ROOT_TXT: Color32::from_rgb(0x30, 0x38, 0x2A),
            PIANO_WHITE_OUT_SCALE_TXT: Color32::from_rgb(0x90, 0x94, 0x8C),
            PIANO_WHITE_OUT_SCALE_BG: Color32::from_rgb(0xC4, 0xC6, 0xBF),
            PIANO_BLACK_IN_SCALE_BG: Color32::from_rgb(0x28, 0x2D, 0x24),
            PIANO_BLACK_ROOT_BG: Color32::from_rgb(0x42, 0x52, 0x38),
            PIANO_BLACK_IN_SCALE_TXT: Color32::from_rgb(0xB0, 0xB6, 0xA8),
            PIANO_BLACK_OUT_SCALE_TXT: Color32::from_rgb(0x60, 0x66, 0x5C),
        }
    }
}

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

pub struct QwerkeysApp {
    state: Arc<AppState>,
    midi: Arc<MidiEngine>,
    clicked_usages: Vec<u32>,
    clicked_piano_notes: Vec<u8>,
    aspect_ratio_set: bool,
    current_theme: AppTheme,
    theme_colors: ThemeColors,
    settings_open: bool,
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

    // ------------------------------------------------------------------------
    // TOP SETUP BAR
    // ------------------------------------------------------------------------
    // ------------------------------------------------------------------------
    // TOP SETUP BAR (Full window width edge-to-edge)
    // ------------------------------------------------------------------------
    fn render_setup_bar(&mut self, ui: &mut Ui) {
        let theme = self.theme_colors;
        let avail_w = ui.available_width();
        let sep_space = 14.0;
        let sep_w = 1.0;
        let total_sep = 4.0 * sep_space + 2.0 * sep_w;
        let usable_w = (avail_w - total_sep).max(600.0);

        let dev_w = (usable_w * 0.24).clamp(240.0, 300.0);
        let out_w = (usable_w * 0.35).clamp(350.0, 420.0);
        let scale_w = usable_w - dev_w - out_w;

        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                vec2(dev_w, 102.0),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                    self.render_device_column(ui);
                },
            );

            ui.add_space(sep_space);
            let (sep_rect, _) = ui.allocate_exact_size(vec2(sep_w, 102.0), Sense::hover());
            ui.painter().vline(
                sep_rect.center().x,
                sep_rect.y_range(),
                Stroke::new(1.0, theme.BORDER_FIELD),
            );
            ui.add_space(sep_space);

            ui.allocate_ui_with_layout(
                vec2(scale_w, 102.0),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                    self.render_scale_column(ui);
                },
            );

            ui.add_space(sep_space);
            let (sep_rect, _) = ui.allocate_exact_size(vec2(sep_w, 102.0), Sense::hover());
            ui.painter().vline(
                sep_rect.center().x,
                sep_rect.y_range(),
                Stroke::new(1.0, theme.BORDER_FIELD),
            );
            ui.add_space(sep_space);

            ui.allocate_ui_with_layout(
                vec2(out_w, 102.0),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                    self.render_output_column(ui);
                },
            );
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
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("●")
                            .color(if current_ptr != 0 {
                                theme.DOT_LIT
                            } else {
                                theme.DOT_UNLIT
                            })
                            .font(FontId::proportional(10.0)),
                    );

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
                                    .selectable_label(is_omni, "[*] All Keyboards (Omni)")
                                    .clicked()
                                {
                                    self.select_all_keyboards();
                                }
                                let is_ext_only = current_ptr == usize::MAX;
                                if ui
                                    .selectable_label(is_ext_only, "[+] All External Keyboards")
                                    .clicked()
                                {
                                    self.select_external_keyboards();
                                }
                                ui.separator();
                                for dev in &devices {
                                    let label = format!(
                                        "[{}] {} ({}) {}",
                                        dev.index,
                                        dev.name,
                                        dev.transport,
                                        if dev.is_internal {
                                            "[Built-in]"
                                        } else {
                                            "[External]"
                                        }
                                    );
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
                    [ScaleType::MinorPentatonic, ScaleType::HarmonicMinor, ScaleType::Mixolydian],
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
                                RichText::new(label)
                                    .font(FontId::monospace(8.5))
                                    .color(if is_sel {
                                        theme.SAGE_BADGE_TXT
                                    } else {
                                        theme.TEXT_LIGHT_MUTED
                                    }),
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

    /// Column 3: Output controls (dual octaves, layout preset, velocity)
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
                        .color(if self.settings_open { theme.TEXT_PRIMARY } else { theme.TEXT_MUTED }),
                )
                .fill(if self.settings_open { theme.BG_BUTTON_HOVER } else { theme.BG_BUTTON })
                .stroke(Stroke::new(1.0, if self.settings_open { theme.SAGE_ACCENT } else { theme.BORDER_DARK }))
                .corner_radius(CornerRadius::same(4));

                if ui.add(btn).clicked() {
                    self.settings_open = !self.settings_open;
                }
            });
        });

        ui.add_space(6.0);

        // Row 1: Left & Right Octave steppers side by side
        ui.horizontal(|ui| {
            // Left Octave Stepper
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("BASS:")
                        .font(FontId::monospace(8.5))
                        .color(theme.TEXT_MUTED),
                );

                Frame::NONE
                    .fill(theme.BG_FIELD)
                    .stroke(Stroke::new(1.0, theme.BORDER_FIELD))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::symmetric(2, 2))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut left_oct = self.state.left_base_octave.load(Ordering::SeqCst);
                            if ui
                                .button(RichText::new("−").color(theme.TEXT_MUTED))
                                .clicked()
                                && left_oct > -1
                            {
                                left_oct -= 1;
                                self.state
                                    .left_base_octave
                                    .store(left_oct, Ordering::SeqCst);
                            }
                            ui.label(
                                RichText::new(format!("C{left_oct}"))
                                    .font(FontId::monospace(10.0))
                                    .strong()
                                    .color(theme.TEXT_PRIMARY),
                            );
                            if ui
                                .button(RichText::new("+").color(theme.TEXT_MUTED))
                                .clicked()
                                && left_oct < 5
                            {
                                left_oct += 1;
                                self.state
                                    .left_base_octave
                                    .store(left_oct, Ordering::SeqCst);
                            }
                        });
                    });
            });

            ui.add_space(8.0);

            // Right Octave Stepper
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("LEAD:")
                        .font(FontId::monospace(8.5))
                        .color(theme.TEXT_MUTED),
                );

                Frame::NONE
                    .fill(theme.BG_FIELD)
                    .stroke(Stroke::new(1.0, theme.BORDER_FIELD))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::symmetric(2, 2))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut right_oct = self.state.right_base_octave.load(Ordering::SeqCst);
                            if ui
                                .button(RichText::new("−").color(theme.TEXT_MUTED))
                                .clicked()
                                && right_oct > 0
                            {
                                right_oct -= 1;
                                self.state
                                    .right_base_octave
                                    .store(right_oct, Ordering::SeqCst);
                            }
                            ui.label(
                                RichText::new(format!("C{right_oct}"))
                                    .font(FontId::monospace(10.0))
                                    .strong()
                                    .color(theme.TEXT_PRIMARY),
                            );
                            if ui
                                .button(RichText::new("+").color(theme.TEXT_MUTED))
                                .clicked()
                                && right_oct < 7
                            {
                                right_oct += 1;
                                self.state
                                    .right_base_octave
                                    .store(right_oct, Ordering::SeqCst);
                            }
                        });
                    });
            });
        });

        ui.add_space(6.0);

        // Row 2: Layout preset and velocity slider
        ui.horizontal(|ui| {
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

            ui.add_space(6.0);
            ui.label(
                RichText::new("VEL")
                    .font(FontId::monospace(8.5))
                    .color(theme.TEXT_MUTED),
            );
            let mut vel = self.state.velocity.load(Ordering::SeqCst);
            let slider = egui::Slider::new(&mut vel, 10..=127).show_value(true);
            if ui.add(slider).changed() {
                self.state.velocity.store(vel, Ordering::SeqCst);
            }
        });
    }

    // ------------------------------------------------------------------------
    // WORKSPACE: KEYBOARD MAPPING PANEL
    // ------------------------------------------------------------------------
    fn render_qwerty_keyboard(&mut self, ui: &mut Ui) {
        let theme = self.theme_colors;
        let root = self.state.get_root();
        let scale = self.state.get_scale();
        let left_oct = self.state.left_base_octave.load(Ordering::SeqCst);
        let right_oct = self.state.right_base_octave.load(Ordering::SeqCst);
        let preset = self.state.get_layout_preset();

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("KEYBOARD MAPPING")
                        .font(FontId::proportional(12.0))
                        .strong()
                        .color(theme.TEXT_PRIMARY),
                );

                Frame::NONE
                    .fill(theme.SAGE_BADGE_BG)
                    .stroke(Stroke::new(1.0, theme.SAGE_BORDER))
                    .corner_radius(CornerRadius::same(3))
                    .inner_margin(Margin::symmetric(6, 2))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(preset.name().to_uppercase())
                                .font(FontId::monospace(8.0))
                                .color(theme.SAGE_BADGE_TXT),
                        );
                    });
            });

            ui.add_space(6.0);

            // Keyboard frame enclosure
            Frame::NONE
                .fill(theme.BG_KEYBOARD)
                .stroke(Stroke::new(1.0, theme.BORDER_KEYBOARD))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    use QwertyKeyDef::*;
                    let rows: [&[QwertyKeyDef]; 5] = [
                        // Row 0: Number Row (14.5 units total)
                        &[
                            Modifier { label: "`", sub: Some("~"), units: 1.0, align_right: false },
                            Playable { label: "1", shift_char: Some("!") },
                            Playable { label: "2", shift_char: Some("@") },
                            Playable { label: "3", shift_char: Some("#") },
                            Playable { label: "4", shift_char: Some("$") },
                            Playable { label: "5", shift_char: Some("%") },
                            Playable { label: "6", shift_char: Some("^") },
                            Playable { label: "7", shift_char: Some("&") },
                            Playable { label: "8", shift_char: Some("*") },
                            Playable { label: "9", shift_char: Some("(") },
                            Playable { label: "0", shift_char: Some(")") },
                            Modifier { label: "-", sub: Some("_"), units: 1.0, align_right: false },
                            Modifier { label: "=", sub: Some("+"), units: 1.0, align_right: false },
                            Modifier { label: "delete", sub: None, units: 1.5, align_right: true },
                        ],
                        // Row 1: Top Row (14.5 units total)
                        &[
                            Modifier { label: "tab", sub: None, units: 1.5, align_right: false },
                            Playable { label: "Q", shift_char: None },
                            Playable { label: "W", shift_char: None },
                            Playable { label: "E", shift_char: None },
                            Playable { label: "R", shift_char: None },
                            Playable { label: "T", shift_char: None },
                            Playable { label: "Y", shift_char: None },
                            Playable { label: "U", shift_char: None },
                            Playable { label: "I", shift_char: None },
                            Playable { label: "O", shift_char: None },
                            Playable { label: "P", shift_char: None },
                            Modifier { label: "[", sub: Some("{"), units: 1.0, align_right: false },
                            Modifier { label: "]", sub: Some("}"), units: 1.0, align_right: false },
                            Modifier { label: "\\", sub: Some("|"), units: 1.0, align_right: false },
                        ],
                        // Row 2: Home Row (14.5 units total)
                        &[
                            Modifier { label: "caps lock", sub: Some("•"), units: 1.8, align_right: false },
                            Playable { label: "A", shift_char: None },
                            Playable { label: "S", shift_char: None },
                            Playable { label: "D", shift_char: None },
                            Playable { label: "F", shift_char: None },
                            Playable { label: "G", shift_char: None },
                            Playable { label: "H", shift_char: None },
                            Playable { label: "J", shift_char: None },
                            Playable { label: "K", shift_char: None },
                            Playable { label: "L", shift_char: None },
                            Playable { label: ";", shift_char: Some(":") },
                            Modifier { label: "'", sub: Some("\""), units: 1.0, align_right: false },
                            Modifier { label: "return", sub: None, units: 1.7, align_right: true },
                        ],
                        // Row 3: Bottom Row (14.5 units total)
                        &[
                            Modifier { label: "shift", sub: None, units: 2.3, align_right: false },
                            Playable { label: "Z", shift_char: None },
                            Playable { label: "X", shift_char: None },
                            Playable { label: "C", shift_char: None },
                            Playable { label: "V", shift_char: None },
                            Playable { label: "B", shift_char: None },
                            Playable { label: "N", shift_char: None },
                            Playable { label: "M", shift_char: None },
                            Playable { label: ",", shift_char: Some("<") },
                            Playable { label: ".", shift_char: Some(">") },
                            Playable { label: "/", shift_char: Some("?") },
                            Modifier { label: "shift", sub: None, units: 2.2, align_right: true },
                        ],
                        // Row 4: Spacebar & Modifiers (14.5 units total)
                        &[
                            Modifier { label: "fn", sub: Some("🌐"), units: 1.0, align_right: false },
                            Modifier { label: "control", sub: Some("⌃"), units: 1.0, align_right: false },
                            Modifier { label: "option", sub: Some("⌥"), units: 1.0, align_right: false },
                            Modifier { label: "command", sub: Some("⌘"), units: 1.25, align_right: false },
                            Spacebar(5.0),
                            Modifier { label: "command", sub: Some("⌘"), units: 1.25, align_right: false },
                            Modifier { label: "option", sub: Some("⌥"), units: 1.0, align_right: false },
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
                                                ui, key, *shift_char, key_w, row_h, root, scale, left_oct,
                                                right_oct, preset,
                                            );
                                        }
                                    }
                                    Modifier { label, sub, units, align_right } => {
                                        let key_w = if is_last {
                                            ui.available_width().max(1.0)
                                        } else {
                                            *units * unit_u - key_spacing
                                        };
                                        self.render_modifier_keycap(ui, label, *sub, key_w, row_h, *align_right);
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
    ) {
        let theme = self.theme_colors;
        let (note_str, degree, is_root) =
            get_key_display_info(key, root, scale, left_oct, right_oct, preset);

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
                if is_triggered { theme.KEY_LABEL_SHIFT_TRIGGERED } else { theme.TEXT_MUTED },
            );
            painter.text(
                rect.min + vec2(6.0, 12.0),
                Align2::LEFT_TOP,
                key.label,
                FontId::proportional(9.0),
                if is_triggered { theme.KEY_LABEL_KEY_TRIGGERED } else { theme.TEXT_LIGHT_MUTED },
            );
        } else {
            painter.text(
                rect.min + vec2(6.0, 4.0),
                Align2::LEFT_TOP,
                key.label,
                FontId::proportional(9.5),
                if is_triggered { theme.KEY_LABEL_KEY_TRIGGERED } else { theme.TEXT_MUTED },
            );
        }

        // F and J tactile homing bump bar (authentic MacBook detail)
        if key.label == "F" || key.label == "J" {
            let bump_y = rect.min.y + 16.0;
            let bump_x = rect.min.x + 6.0;
            painter.line_segment(
                [Pos2::new(bump_x, bump_y), Pos2::new(bump_x + 6.0, bump_y)],
                Stroke::new(1.2, if is_triggered { Color32::WHITE } else { theme.TEXT_MUTED }),
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
            if hovered { theme.BG_BUTTON_HOVER } else { theme.KEY_UNMAPPED_BG },
            Stroke::new(1.0, theme.BORDER_DARK),
            egui::StrokeKind::Inside,
        );

        let text_color = if hovered { theme.TEXT_PRIMARY } else { theme.TEXT_MUTED };

        if label == "caps lock" {
            // Paint caps lock green dot
            painter.circle_filled(
                rect.min + vec2(8.0, 8.0),
                2.0,
                theme.DOT_LIT,
            );
            painter.text(
                rect.min + vec2(8.0, height - 7.0),
                Align2::LEFT_BOTTOM,
                label,
                FontId::proportional(8.5),
                text_color,
            );
        } else if let Some(sub_text) = sub {
            // Top and bottom dual label (e.g. fn / 🌐, control / ⌃, ~ / `)
            painter.text(
                rect.min + vec2(if align_right { width - 6.0 } else { 6.0 }, 4.5),
                if align_right { Align2::RIGHT_TOP } else { Align2::LEFT_TOP },
                sub_text,
                FontId::proportional(8.5),
                theme.TEXT_MUTED,
            );
            painter.text(
                rect.min + vec2(if align_right { width - 6.0 } else { 6.0 }, height - 5.0),
                if align_right { Align2::RIGHT_BOTTOM } else { Align2::LEFT_BOTTOM },
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
            (
                theme.KEY_ACTIVE_BG,
                theme.KEY_ACTIVE_BORDER,
                Color32::WHITE,
            )
        } else {
            (
                theme.KEY_UNMAPPED_BG,
                theme.BORDER_DARK,
                theme.TEXT_MUTED,
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
            let mut left_oct = self.state.left_base_octave.load(Ordering::SeqCst);
            if left_oct > -1 {
                left_oct -= 1;
                self.state.left_base_octave.store(left_oct, Ordering::SeqCst);
            }
        }
        self.paint_single_arrow(painter, left_rect, "◀", left_resp.hovered());

        // 2. Middle column: Up Arrow (top half) and Down Arrow (bottom half)
        let up_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + col_w + key_spacing, rect.min.y),
            vec2(col_w, half_h),
        );
        let up_resp = ui.interact(up_rect, ui.id().with("arrow_up"), Sense::click());
        if up_resp.clicked() {
            let mut right_oct = self.state.right_base_octave.load(Ordering::SeqCst);
            if right_oct < 7 {
                right_oct += 1;
                self.state.right_base_octave.store(right_oct, Ordering::SeqCst);
            }
        }
        self.paint_single_arrow(painter, up_rect, "▲", up_resp.hovered());

        let down_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + col_w + key_spacing, rect.min.y + half_h + key_spacing),
            vec2(col_w, half_h),
        );
        let down_resp = ui.interact(down_rect, ui.id().with("arrow_down"), Sense::click());
        if down_resp.clicked() {
            let mut right_oct = self.state.right_base_octave.load(Ordering::SeqCst);
            if right_oct > 0 {
                right_oct -= 1;
                self.state.right_base_octave.store(right_oct, Ordering::SeqCst);
            }
        }
        self.paint_single_arrow(painter, down_rect, "▼", down_resp.hovered());

        // 3. Right Arrow (Bottom half of col 2)
        let right_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + 2.0 * (col_w + key_spacing), rect.min.y + half_h + key_spacing),
            vec2(col_w, half_h),
        );
        let right_resp = ui.interact(right_rect, ui.id().with("arrow_right"), Sense::click());
        if right_resp.clicked() {
            let mut left_oct = self.state.left_base_octave.load(Ordering::SeqCst);
            if left_oct < 5 {
                left_oct += 1;
                self.state.left_base_octave.store(left_oct, Ordering::SeqCst);
            }
        }
        self.paint_single_arrow(painter, right_rect, "▶", right_resp.hovered());
    }

    fn paint_single_arrow(&self, painter: &egui::Painter, rect: Rect, glyph: &str, hovered: bool) {
        let theme = self.theme_colors;
        painter.rect(
            rect,
            CornerRadius::same(3),
            if hovered { theme.BG_BUTTON_HOVER } else { theme.KEY_UNMAPPED_BG },
            Stroke::new(1.0, theme.BORDER_DARK),
            egui::StrokeKind::Inside,
        );
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            glyph,
            FontId::proportional(8.0),
            if hovered { theme.TEXT_PRIMARY } else { theme.TEXT_MUTED },
        );
    }

    // ------------------------------------------------------------------------
    // WORKSPACE: NOTE VIEW (PIANO ROLL)
    // ------------------------------------------------------------------------
    fn render_piano(&mut self, ui: &mut Ui) {
        let theme = self.theme_colors;
        let root = self.state.get_root();
        let scale = self.state.get_scale();
        let left_oct = self.state.left_base_octave.load(Ordering::SeqCst);
        let right_oct = self.state.right_base_octave.load(Ordering::SeqCst);
        let preset = self.state.get_layout_preset();

        let min_oct = left_oct.min(right_oct);
        let max_oct = left_oct.max(right_oct) + 2;
        let num_octaves = (max_oct - min_oct + 1).clamp(1, 7) as usize;
        let total_white_keys = num_octaves * 7 + 1;

        // Build reverse lookup: MIDI note -> QWERTY key label(s)
        let mut note_to_key_labels: HashMap<u8, Vec<&'static str>> = HashMap::new();
        for key in KEY_DEFINITIONS {
            let notes = calculate_key_notes(key, root, scale, left_oct, right_oct, preset);
            for n in notes {
                note_to_key_labels.entry(n).or_default().push(key.label);
            }
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("NOTE VIEW (PIANO)")
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

            // Piano keys enclosure
            let available_w = ui.available_width();
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

            // 1. Draw white keys
            let mut hit_white_note: Option<u8> = None;
            let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

            for k in 0..total_white_keys {
                let oct_idx = k / 7;
                let step_idx = k % 7;
                let oct = min_oct + oct_idx as i8;
                let note =
                    ((oct + 1) as i16 * 12 + white_semitones[step_idx] as i16).clamp(0, 127) as u8;

                let x0 = rect.min.x + (k as f32) * wk_w;
                let x1 = if k + 1 == total_white_keys {
                    rect.max.x
                } else {
                    rect.min.x + ((k + 1) as f32) * wk_w
                };
                let key_rect =
                    Rect::from_min_max(Pos2::new(x0, rect.min.y), Pos2::new(x1, rect.min.y + wk_h));

                if let Some(pos) = pointer_pos
                    && key_rect.contains(pos)
                {
                    hit_white_note = Some(note);
                }

                let is_active = self.state.is_note_active(note);
                let in_scale = scale.is_in_scale(root, note);
                let is_root = scale.is_root(root, note);

                let fill = if is_active {
                    theme.PIANO_WHITE_ACTIVE
                } else if is_root {
                    theme.PIANO_WHITE_ROOT
                } else if in_scale {
                    theme.PIANO_WHITE_BG
                } else {
                    theme.PIANO_WHITE_OUT_SCALE_BG
                };

                let stroke = Stroke::new(1.0, theme.PIANO_WHITE_BORDER);
                painter.rect(
                    key_rect,
                    CornerRadius {
                        nw: 0,
                        ne: 0,
                        sw: 3,
                        se: 3,
                    },
                    fill,
                    stroke,
                    egui::StrokeKind::Inside,
                );

                let pitch_idx = (note % 12) as usize;
                let pitch_str = note_names[pitch_idx];
                let note_octave = (note as i8 / 12) - 1;
                let is_c = pitch_idx == 0;

                // Show mapped QWERTY key letter directly (no background box or border)
                if let Some(lbl) = note_to_key_labels.get(&note).and_then(|l| l.first()) {
                    painter.text(
                        key_rect.center_bottom() - vec2(0.0, 20.0),
                        Align2::CENTER_CENTER,
                        *lbl,
                        FontId::monospace(9.5),
                        if is_active {
                            Color32::WHITE
                        } else if is_root {
                            theme.PIANO_WHITE_ROOT_TXT
                        } else {
                            theme.PIANO_WHITE_MAPPED_TXT
                        },
                    );
                }

                // Note name at bottom: C notes and root notes show octave (e.g. C2, C3); others show pitch
                let note_label = if is_c || is_root {
                    format!("{pitch_str}{note_octave}")
                } else {
                    pitch_str.to_string()
                };

                painter.text(
                    key_rect.center_bottom() - vec2(0.0, 6.0),
                    Align2::CENTER_BOTTOM,
                    note_label,
                    FontId::monospace(if is_c || is_root { 9.0 } else { 8.0 }),
                    if is_active {
                        Color32::WHITE
                    } else if is_root {
                        theme.PIANO_WHITE_ROOT_TXT
                    } else if in_scale {
                        theme.PIANO_WHITE_NOTE
                    } else {
                        theme.PIANO_WHITE_OUT_SCALE_TXT
                    },
                );
            }

            // 2. Draw black keys
            let black_key_defs: [(usize, u8); 5] = [(0, 1), (1, 3), (3, 6), (4, 8), (5, 10)];

            let mut hit_black_note: Option<u8> = None;

            for oct_idx in 0..num_octaves {
                let oct = min_oct + oct_idx as i8;
                let base_white_idx = oct_idx * 7;

                for (left_wk_idx, semitone) in black_key_defs {
                    let wk_index = base_white_idx + left_wk_idx;
                    let note = ((oct + 1) as i16 * 12 + semitone as i16).clamp(0, 127) as u8;

                    let bk_x = rect.min.x + (wk_index + 1) as f32 * wk_w - bk_w * 0.5;
                    let bk_rect =
                        Rect::from_min_size(Pos2::new(bk_x, rect.min.y), vec2(bk_w, bk_h));

                    if let Some(pos) = pointer_pos
                        && bk_rect.contains(pos)
                    {
                        hit_black_note = Some(note);
                    }

                    let is_active = self.state.is_note_active(note);
                    let in_scale = scale.is_in_scale(root, note);
                    let is_root = scale.is_root(root, note);

                    let fill = if is_active {
                        theme.PIANO_BLACK_ACTIVE
                    } else if is_root {
                        theme.PIANO_WHITE_ROOT_TXT
                    } else if in_scale {
                        theme.PIANO_BLACK_IN_SCALE_BG
                    } else {
                        theme.PIANO_BLACK_BG
                    };

                    let stroke = Stroke::new(1.0, theme.PIANO_BLACK_BORDER);
                    painter.rect(
                        bk_rect,
                        CornerRadius {
                            nw: 0,
                            ne: 0,
                            sw: 3,
                            se: 3,
                        },
                        fill,
                        stroke,
                        egui::StrokeKind::Inside,
                    );

                    let pitch_idx = (note % 12) as usize;
                    let pitch_str = note_names[pitch_idx];
                    let note_octave = (note as i8 / 12) - 1;

                    // If mapped, show key binding directly on black key (no box or stroke)
                    if let Some(lbl) = note_to_key_labels.get(&note).and_then(|l| l.first()) {
                        painter.text(
                            bk_rect.center_bottom() - vec2(0.0, 18.0),
                            Align2::CENTER_CENTER,
                            *lbl,
                            FontId::monospace(8.5),
                            if is_active {
                                Color32::WHITE
                            } else if in_scale {
                                theme.SAGE_BADGE_TXT
                            } else {
                                theme.TEXT_MUTED
                            },
                        );
                    }

                    // Note name at bottom of black key
                    let note_label = if is_root {
                        format!("{pitch_str}{note_octave}")
                    } else {
                        pitch_str.to_string()
                    };

                    painter.text(
                        bk_rect.center_bottom() - vec2(0.0, 5.0),
                        Align2::CENTER_BOTTOM,
                        &note_label,
                        FontId::monospace(if is_root { 8.0 } else { 7.0 }),
                        if is_active {
                            Color32::WHITE
                        } else if is_root {
                            theme.SAGE_BADGE_TXT
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

            ui.add_space(8.0);

            // Piano footer: Legend & active counter
            ui.horizontal(|ui| {
                // Legend
                ui.horizontal(|ui| {
                    // Root note legend
                    let (rect, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                    ui.painter().rect(
                        rect,
                        CornerRadius::same(2),
                        theme.PIANO_WHITE_ROOT,
                        Stroke::new(1.0, theme.SAGE_BORDER),
                        egui::StrokeKind::Inside,
                    );
                    ui.label(
                        RichText::new("Root note")
                            .font(FontId::monospace(8.5))
                            .color(theme.TEXT_MUTED),
                    );

                    ui.add_space(10.0);

                    // Mapped note legend
                    let (rect, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                    ui.painter().rect(
                        rect,
                        CornerRadius::same(2),
                        theme.KEY_MAPPED_BG,
                        Stroke::new(1.0, theme.KEY_MAPPED_BORDER),
                        egui::StrokeKind::Inside,
                    );
                    ui.label(
                        RichText::new("Mapped note")
                            .font(FontId::monospace(8.5))
                            .color(theme.TEXT_MUTED),
                    );

                    ui.add_space(10.0);

                    // Playing note legend
                    let (rect, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                    ui.painter().rect(
                        rect,
                        CornerRadius::same(2),
                        theme.SAGE_ACCENT,
                        Stroke::NONE,
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

    // ------------------------------------------------------------------------
    // BOTTOM STATUS BAR (Full window width edge-to-edge sticking to bottom)
    // ------------------------------------------------------------------------
    fn render_bottom_bar(&self, ui: &mut Ui) {
        let theme = self.theme_colors;
        let event_info = self.state.last_event_info.lock().unwrap().clone();
        let current_ptr = self.state.selected_device_ptr.load(Ordering::SeqCst);
        let is_suppressed = current_ptr != 0;

        ui.spacing_mut().item_spacing.y = 0.0;
        ui.spacing_mut().interact_size.y = 11.0;

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("STATUS:")
                    .font(FontId::monospace(9.5))
                    .strong()
                    .color(theme.TEXT_MUTED),
            );

            ui.label(
                RichText::new(&event_info)
                    .font(FontId::monospace(10.5))
                    .strong()
                    .color(theme.TEXT_PRIMARY),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(if is_suppressed {
                        "HOOK: Dedicated (Keystrokes Suppressed)"
                    } else {
                        "HOOK: Shared (Non-suppressed)"
                    })
                    .font(FontId::monospace(9.0))
                    .color(if is_suppressed {
                        theme.SAGE_BADGE_TXT
                    } else {
                        theme.TEXT_MUTED
                    }),
                );

                ui.separator();

                ui.label(
                    RichText::new("PORT: qwerkeys-input")
                        .font(FontId::monospace(9.0))
                        .color(theme.TEXT_MUTED),
                );
            });
        });
    }

        fn set_theme(&mut self, theme: AppTheme, ctx: &egui::Context) {
        self.current_theme = theme;
        self.theme_colors = theme.colors();
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = self.theme_colors.BG_APP;
        visuals.window_fill = self.theme_colors.BG_APP;
        visuals.override_text_color = Some(self.theme_colors.TEXT_SECONDARY);
        ctx.set_visuals(visuals);
    }

    fn render_settings_window(&mut self, ctx: &egui::Context) {
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
                let (rect, response) = ui.allocate_exact_size(vec2(avail_w, card_h), Sense::click());
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
