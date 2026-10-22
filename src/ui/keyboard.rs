//! The QWERTY keyboard widget: physical mac layout data, keycap skin and
//! painting primitives, and the egui rendering + MIDI routing.

use std::sync::atomic::Ordering;

use egui::{
    Align2, Color32, CornerRadius, FontId, Frame, Margin, Pos2, Rect, RichText, Sense, Stroke, Ui,
    vec2,
};

use crate::mapping::{KeyMappingDef, find_key_by_label, get_key_display_info};

use super::QwerkeysApp;
use crate::utils::mix;

// Keycap skin + painting primitives
#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy)]
pub struct KeySkin {
    /// enclosure body around the keys
    pub BODY: Color32,
    pub FACE: Color32,
    pub FACE_HOVER: Color32,
    /// piano black-key face
    pub FACE_DARK: Color32,
    pub BORDER: Color32,
    pub BEVEL_MID: Color32,
    pub BEVEL_DEEP: Color32,
    pub TEXT: Color32,
    pub TEXT_SOFT: Color32,
    pub LIGHT: bool,
}

impl KeySkin {
    /// Light aluminum MacBook skin (Studio Daylight)
    pub fn aluminum() -> Self {
        Self {
            BODY: Color32::from_rgb(0xD7, 0xDA, 0xDD),
            FACE: Color32::from_rgb(0xEF, 0xF0, 0xF2),
            FACE_HOVER: Color32::from_rgb(0xE4, 0xE6, 0xE8),
            FACE_DARK: Color32::from_rgb(0x34, 0x37, 0x3E),
            BORDER: Color32::from_rgb(0xAE, 0xB1, 0xB4),
            BEVEL_MID: Color32::from_rgb(0xC6, 0xC9, 0xCC),
            BEVEL_DEEP: Color32::from_rgb(0x9A, 0x9D, 0x9F),
            TEXT: Color32::from_rgb(0x64, 0x67, 0x6C),
            TEXT_SOFT: Color32::from_rgb(0x92, 0x96, 0x9B),
            LIGHT: true,
        }
    }

    /// Dark "blackbook" skin — charcoal caps on a lighter body (dark modes)
    pub fn carbon() -> Self {
        Self {
            BODY: Color32::from_rgb(0x3A, 0x3C, 0x40),
            FACE: Color32::from_rgb(0x1C, 0x1D, 0x1F),
            FACE_HOVER: Color32::from_rgb(0x25, 0x26, 0x29),
            FACE_DARK: Color32::from_rgb(0x12, 0x13, 0x15),
            BORDER: Color32::from_rgb(0x19, 0x1A, 0x1C),
            BEVEL_MID: Color32::from_rgb(0x1B, 0x1C, 0x1E),
            BEVEL_DEEP: Color32::from_rgb(0x0E, 0x0F, 0x10),
            TEXT: Color32::from_rgb(0xC4, 0xC5, 0xC7),
            TEXT_SOFT: Color32::from_rgb(0x8C, 0x8D, 0x90),
            LIGHT: false,
        }
    }

    pub fn from_light(light: bool) -> Self {
        if light {
            Self::aluminum()
        } else {
            Self::carbon()
        }
    }
}

/// Corner profile used by the QWERTY keycaps.
pub const KEYCAP_CORNER: CornerRadius = CornerRadius::same(5);

/// Paints one mac-style keycap face: stacked drop layers simulate the key's
/// 3D depth, a soft top sheen stands in for the CSS inner-glow shadow, and
/// `depressed` (a pressed key) drops the bevel so the cap sits flush.
pub fn draw_mac_keycap(
    skin: &KeySkin,
    painter: &egui::Painter,
    rect: Rect,
    corner: CornerRadius,
    face: Color32,
    border: Color32,
    depressed: bool,
) {
    if !depressed {
        // Box-shadow bevel: `0 1px 0 #cfcfcf, 0 1px 1px #6f6f6f`
        painter.rect(
            Rect::from_min_size(rect.min + vec2(0.0, 1.0), rect.size()),
            corner,
            skin.BEVEL_MID,
            Stroke::NONE,
            egui::StrokeKind::Outside,
        );
        painter.rect(
            Rect::from_min_size(rect.min + vec2(0.0, 2.0), rect.size()),
            corner,
            skin.BEVEL_DEEP,
            Stroke::NONE,
            egui::StrokeKind::Outside,
        );
    }

    painter.rect(
        rect,
        corner,
        face,
        Stroke::new(0.75, border),
        egui::StrokeKind::Inside,
    );

    // Inset glow: `inset 1px 0 25px #dfdfdf`
    // Light aluminum caps get the full glossy sheen; dark keys get a barely-there hint.
    let (strong_a, soft_a) = if skin.LIGHT { (38, 14) } else { (8, 2) };
    let sheen_strong = Color32::from_rgba_unmultiplied(255, 255, 255, strong_a);
    let sheen_soft = Color32::from_rgba_unmultiplied(255, 255, 255, soft_a);
    let inner = rect.shrink2(vec2(1.5, 1.5));
    let wide = inner.width();
    let band_corner = CornerRadius::same(4);
    let band1 = Rect::from_min_max(inner.min, inner.min + vec2(wide, inner.height() * 0.28));
    let band2 = Rect::from_min_max(band1.max, inner.min + vec2(wide, inner.height() * 0.55));
    painter.rect(
        band1,
        band_corner,
        sheen_strong,
        Stroke::NONE,
        egui::StrokeKind::Inside,
    );
    painter.rect(
        band2,
        band_corner,
        sheen_soft,
        Stroke::NONE,
        egui::StrokeKind::Inside,
    );
}

// Physical layout table — one "unit" is a standard keycap width, every row
// of the mac keyboard is 14.5 units wide.
#[derive(Clone, Copy)]
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

/// Rows of the mac keyboard, top to bottom.
static ROWS: [&[QwertyKeyDef]; 5] = [
    // Row 0: Number Row (14.5 units total)
    &[
        QwertyKeyDef::Modifier {
            label: "`",
            sub: Some("~"),
            units: 1.0,
            align_right: false,
        },
        QwertyKeyDef::Playable {
            label: "1",
            shift_char: Some("!"),
        },
        QwertyKeyDef::Playable {
            label: "2",
            shift_char: Some("@"),
        },
        QwertyKeyDef::Playable {
            label: "3",
            shift_char: Some("#"),
        },
        QwertyKeyDef::Playable {
            label: "4",
            shift_char: Some("$"),
        },
        QwertyKeyDef::Playable {
            label: "5",
            shift_char: Some("%"),
        },
        QwertyKeyDef::Playable {
            label: "6",
            shift_char: Some("^"),
        },
        QwertyKeyDef::Playable {
            label: "7",
            shift_char: Some("&"),
        },
        QwertyKeyDef::Playable {
            label: "8",
            shift_char: Some("*"),
        },
        QwertyKeyDef::Playable {
            label: "9",
            shift_char: Some("("),
        },
        QwertyKeyDef::Playable {
            label: "0",
            shift_char: Some(")"),
        },
        QwertyKeyDef::Playable {
            label: "-",
            shift_char: Some("_"),
        },
        QwertyKeyDef::Playable {
            label: "=",
            shift_char: Some("+"),
        },
        QwertyKeyDef::Modifier {
            label: "delete",
            sub: None,
            units: 1.5,
            align_right: true,
        },
    ],
    // Row 1: Top Row (14.5 units total)
    &[
        QwertyKeyDef::Modifier {
            label: "tab",
            sub: None,
            units: 1.5,
            align_right: false,
        },
        QwertyKeyDef::Playable {
            label: "Q",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "W",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "E",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "R",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "T",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "Y",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "U",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "I",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "O",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "P",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "[",
            shift_char: Some("{"),
        },
        QwertyKeyDef::Modifier {
            label: "]",
            sub: Some("}"),
            units: 1.0,
            align_right: false,
        },
        QwertyKeyDef::Modifier {
            label: "\\",
            sub: Some("|"),
            units: 1.0,
            align_right: false,
        },
    ],
    // Row 2: Home Row (14.5 units total)
    &[
        QwertyKeyDef::Modifier {
            label: "caps lock",
            sub: Some("•"),
            units: 1.8,
            align_right: false,
        },
        QwertyKeyDef::Playable {
            label: "A",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "S",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "D",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "F",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "G",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "H",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "J",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "K",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "L",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: ";",
            shift_char: Some(":"),
        },
        QwertyKeyDef::Modifier {
            label: "'",
            sub: Some("\""),
            units: 1.0,
            align_right: false,
        },
        QwertyKeyDef::Modifier {
            label: "return",
            sub: None,
            units: 1.7,
            align_right: true,
        },
    ],
    // Row 3: Bottom Row (14.5 units total)
    &[
        QwertyKeyDef::Modifier {
            label: "shift",
            sub: None,
            units: 2.3,
            align_right: false,
        },
        QwertyKeyDef::Playable {
            label: "Z",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "X",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "C",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "V",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "B",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "N",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: "M",
            shift_char: None,
        },
        QwertyKeyDef::Playable {
            label: ",",
            shift_char: Some("<"),
        },
        QwertyKeyDef::Playable {
            label: ".",
            shift_char: Some(">"),
        },
        QwertyKeyDef::Modifier {
            label: "/",
            sub: Some("?"),
            units: 1.0,
            align_right: false,
        },
        QwertyKeyDef::Modifier {
            label: "shift",
            sub: None,
            units: 2.2,
            align_right: true,
        },
    ],
    // Row 4: Spacebar & Modifiers (14.5 units total)
    &[
        QwertyKeyDef::Modifier {
            label: "fn",
            sub: Some("🌐"),
            units: 1.0,
            align_right: false,
        },
        QwertyKeyDef::Modifier {
            label: "control",
            sub: Some("⌃"),
            units: 1.0,
            align_right: false,
        },
        QwertyKeyDef::Modifier {
            label: "option",
            sub: Some("⌥"),
            units: 1.0,
            align_right: false,
        },
        QwertyKeyDef::Modifier {
            label: "command",
            sub: Some("⌘"),
            units: 1.25,
            align_right: false,
        },
        QwertyKeyDef::Spacebar(5.0),
        QwertyKeyDef::Modifier {
            label: "command",
            sub: Some("⌘"),
            units: 1.25,
            align_right: false,
        },
        QwertyKeyDef::Modifier {
            label: "option",
            sub: Some("⌥"),
            units: 1.0,
            align_right: false,
        },
        QwertyKeyDef::ArrowCluster(3.0),
    ],
];

// ===========================================================================
// Rendering + MIDI routing
// ===========================================================================

impl QwerkeysApp {
    pub(super) fn render_qwerty_keyboard(&mut self, ui: &mut Ui) {
        let theme = self.theme_colors;
        let skin = KeySkin::from_light(self.current_theme.is_light());

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

            // Aluminum keyboard enclosure (mac body)
            Frame::NONE
                .fill(skin.BODY)
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    let total_width = ui.available_width();
                    let key_spacing = 4.0;
                    let unit_u = (total_width + key_spacing) / 14.5;
                    let row_h = (unit_u * 0.72).clamp(38.0, 54.0);

                    ui.spacing_mut().item_spacing = vec2(key_spacing, key_spacing);

                    for row in ROWS {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = vec2(key_spacing, key_spacing);
                            let row_len = row.len();

                            for (idx, key_def) in row.iter().enumerate() {
                                let is_last = idx + 1 == row_len;
                                match key_def {
                                    QwertyKeyDef::Playable { label, shift_char } => {
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
                                            );
                                        }
                                    }
                                    QwertyKeyDef::Modifier {
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
                                    QwertyKeyDef::Spacebar(units) => {
                                        let key_w = if is_last {
                                            ui.available_width().max(1.0)
                                        } else {
                                            *units * unit_u - key_spacing
                                        };
                                        self.render_spacebar_keycap(ui, key_w, row_h);
                                    }
                                    QwertyKeyDef::ArrowCluster(units) => {
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

    fn render_playable_keycap(
        &mut self,
        ui: &mut Ui,
        key: &'static KeyMappingDef,
        shift_char: Option<&'static str>,
        width: f32,
        height: f32,
    ) {
        let theme = self.theme_colors;
        let skin = KeySkin::from_light(self.current_theme.is_light());
        let root = self.state.get_root();
        let scale = self.state.get_scale();
        let left_oct = self.state.left_base_octave.load(Ordering::SeqCst);
        let right_oct = self.state.right_base_octave.load(Ordering::SeqCst);
        let preset = self.state.get_layout_preset();
        let padding = self.state.mapping_padding.load(Ordering::SeqCst);
        let (note_str, degree, is_root) =
            get_key_display_info(key, root, scale, left_oct, right_oct, preset, padding);

        let is_triggered = self.state.is_key_active(key.usage);

        let (fill, border) = if is_triggered {
            // Pressed: accent face, bevel collapsed (depressed cap)
            (theme.KEY_ACTIVE_BG, theme.KEY_ACTIVE_BORDER)
        } else if is_root {
            // Root keys: no accent tint on the cap, just a touch lighter than plain keys
            let root_face = if skin.LIGHT {
                mix(skin.FACE, Color32::WHITE, 0.50)
            } else {
                mix(skin.FACE, Color32::WHITE, 0.14)
            };
            (root_face, skin.BORDER)
        } else {
            (skin.FACE, mix(skin.BORDER, theme.SAGE_ACCENT, 0.22))
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
        draw_mac_keycap(
            &skin,
            painter,
            rect,
            KEYCAP_CORNER,
            fill,
            border,
            is_triggered,
        );

        let (label_color, soft_color) = if is_triggered {
            (
                theme.KEY_LABEL_KEY_TRIGGERED,
                theme.KEY_LABEL_SHIFT_TRIGGERED,
            )
        } else {
            (skin.TEXT, skin.TEXT_SOFT)
        };

        // Top key label: shift symbol + physical key label (e.g. "!" / "1" or "Q")
        if let Some(sc) = shift_char {
            painter.text(
                rect.min + vec2(6.0, 3.5),
                Align2::LEFT_TOP,
                sc,
                FontId::proportional(8.0),
                soft_color,
            );
            painter.text(
                rect.min + vec2(6.0, 12.0),
                Align2::LEFT_TOP,
                key.label,
                FontId::proportional(9.0),
                label_color,
            );
        } else {
            painter.text(
                rect.min + vec2(6.0, 4.0),
                Align2::LEFT_TOP,
                key.label,
                FontId::proportional(9.5),
                label_color,
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
                        skin.TEXT_SOFT
                    },
                ),
            );
        }

        // Center note label (e.g. C2, D#3, I(3))
        let note_color = if is_triggered {
            theme.KEY_ACTIVE_TXT
        } else if skin.LIGHT {
            // readable dark accent on light caps
            mix(
                theme.SAGE_ACCENT,
                Color32::BLACK,
                if is_root { 0.62 } else { 0.45 },
            )
        } else {
            // readable light accent on charcoal caps
            mix(
                theme.SAGE_ACCENT,
                Color32::WHITE,
                if is_root { 0.70 } else { 0.50 },
            )
        };
        painter.text(
            rect.center() + vec2(0.0, -1.0),
            Align2::CENTER_CENTER,
            &note_str,
            FontId::proportional(13.0),
            note_color,
        );

        // Bottom-center degree label (e.g. Root, m3rd, 5th)
        let degree_color = if is_triggered {
            theme.KEY_SUB_TRIGGERED
        } else if skin.LIGHT {
            mix(theme.SAGE_ACCENT, Color32::BLACK, 0.18)
        } else {
            mix(theme.SAGE_ACCENT, Color32::WHITE, 0.62)
        };
        painter.text(
            rect.center() + vec2(0.0, 13.0),
            Align2::CENTER_CENTER,
            degree,
            FontId::monospace(8.0),
            degree_color,
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
        let skin = KeySkin::from_light(self.current_theme.is_light());
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let painter = ui.painter();
        let hovered = response.hovered();

        draw_mac_keycap(
            &skin,
            painter,
            rect,
            KEYCAP_CORNER,
            if hovered { skin.FACE_HOVER } else { skin.FACE },
            skin.BORDER,
            false,
        );

        let text_color = skin.TEXT;
        let icon_color = if hovered { skin.TEXT } else { skin.TEXT_SOFT };

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
                    icon_color,
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
        let skin = KeySkin::from_light(self.current_theme.is_light());
        let is_sustain = self.state.sustain_held.load(Ordering::SeqCst);

        let (fill, stroke) = if is_sustain {
            (theme.KEY_ACTIVE_BG, theme.KEY_ACTIVE_BORDER)
        } else {
            (skin.FACE, skin.BORDER)
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
        draw_mac_keycap(
            &skin,
            painter,
            rect,
            KEYCAP_CORNER,
            fill,
            stroke,
            is_sustain,
        );

        let label = if is_sustain {
            "SUSTAIN ON (CC 64) — ALL NOTES HELD"
        } else {
            "space"
        };

        let text_color = if is_sustain {
            theme.KEY_ACTIVE_TXT
        } else {
            skin.TEXT
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
        let skin = KeySkin::from_light(self.current_theme.is_light());
        draw_mac_keycap(
            &skin,
            painter,
            rect,
            KEYCAP_CORNER,
            if hovered { skin.FACE_HOVER } else { skin.FACE },
            skin.BORDER,
            false,
        );

        let arrow_color = if hovered { skin.TEXT } else { skin.TEXT_SOFT };

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
