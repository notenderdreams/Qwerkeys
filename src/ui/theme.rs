use egui::Color32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppTheme {
    StudioDaylight,
    #[default]
    StudioSlate,
    StudioSage,
}

impl AppTheme {
    pub const ALL: [AppTheme; 3] = [
        AppTheme::StudioSlate,
        AppTheme::StudioSage,
        AppTheme::StudioDaylight,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            AppTheme::StudioDaylight => "Studio Daylight",
            AppTheme::StudioSlate => "Studio Slate",
            AppTheme::StudioSage => "Studio Sage",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AppTheme::StudioDaylight => "Bright silver aluminum with slate-blue accents",
            AppTheme::StudioSlate => "Refined dark graphite with slate-blue accents",
            AppTheme::StudioSage => "Warm dark studio with organic sage green accents",
        }
    }

    /// Light themes need egui's light base visuals (menus, sliders, etc.)
    pub fn is_light(&self) -> bool {
        matches!(self, AppTheme::StudioDaylight)
    }
}

impl AppTheme {
    pub fn colors(&self) -> ThemeColors {
        match self {
            AppTheme::StudioDaylight => ThemeColors::studio_daylight(),
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
    pub LOGO: Color32,

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
    pub fn studio_daylight() -> Self {
        Self {
            BG_APP: Color32::from_rgb(0xF2, 0xF3, 0xF5),
            BG_BAR: Color32::from_rgb(0xE9, 0xEB, 0xED),
            BG_PANEL: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            BG_KEYBOARD: Color32::from_rgb(0xE1, 0xE3, 0xE6),
            BG_FIELD: Color32::from_rgb(0xF0, 0xF1, 0xF3),
            BG_BUTTON: Color32::from_rgb(0xDF, 0xE1, 0xE4),
            BG_BUTTON_HOVER: Color32::from_rgb(0xCD, 0xD0, 0xD4),
            BORDER_DARK: Color32::from_rgb(0xC2, 0xC5, 0xC9),
            BORDER_PANEL: Color32::from_rgb(0xDC, 0xDF, 0xE3),
            BORDER_KEYBOARD: Color32::from_rgb(0xB8, 0xBB, 0xC0),
            BORDER_FIELD: Color32::from_rgb(0xC6, 0xCA, 0xCE),
            BORDER_GRID: Color32::from_rgb(0xB8, 0xBC, 0xC2),
            SAGE_ACCENT: Color32::from_rgb(0x54, 0x5A, 0x99),
            SAGE_ACTIVE: Color32::from_rgb(0x4A, 0x50, 0x90),
            SAGE_BORDER: Color32::from_rgb(0x54, 0x5A, 0x99),
            SAGE_BADGE_BG: Color32::from_rgb(0xE7, 0xEA, 0xF8),
            SAGE_BADGE_TXT: Color32::from_rgb(0x3B, 0x42, 0x70),
            SAGE_SELECTED_BG: Color32::from_rgb(0x54, 0x5A, 0x99),
            SAGE_SELECTED_TXT: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            TEXT_PRIMARY: Color32::from_rgb(0x1E, 0x1F, 0x23),
            TEXT_SECONDARY: Color32::from_rgb(0x40, 0x43, 0x4A),
            TEXT_MUTED: Color32::from_rgb(0x7A, 0x7E, 0x85),
            TEXT_LIGHT_MUTED: Color32::from_rgb(0x9D, 0xA0, 0xA7),
            TEXT_HEADING: Color32::from_rgb(0x43, 0x46, 0x4E),
            LOGO: Color32::from_rgb(0x3E, 0x42, 0x4A),
            KEY_UNMAPPED_BG: Color32::from_rgb(0xDA, 0xDD, 0xDF),
            KEY_UNMAPPED_BORDER: Color32::from_rgb(0xC3, 0xC6, 0xC9),
            KEY_UNMAPPED_TXT: Color32::from_rgb(0x8F, 0x92, 0x98),
            KEY_MAPPED_BG: Color32::from_rgb(0xFA, 0xFA, 0xFC),
            KEY_MAPPED_BORDER: Color32::from_rgb(0xBC, 0xC3, 0xE2),
            KEY_MAPPED_TXT: Color32::from_rgb(0x3E, 0x45, 0x70),
            KEY_MAPPED_SUB: Color32::from_rgb(0x76, 0x7E, 0xA8),
            KEY_ROOT_BG: Color32::from_rgb(0xE2, 0xE7, 0xFA),
            KEY_ROOT_BORDER: Color32::from_rgb(0x54, 0x5A, 0x99),
            KEY_ACTIVE_BG: Color32::from_rgb(0x54, 0x5A, 0x99),
            KEY_ACTIVE_BORDER: Color32::from_rgb(0x33, 0x39, 0x5E),
            KEY_ACTIVE_TXT: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            PIANO_WHITE_BG: Color32::from_rgb(0xF7, 0xF8, 0xFA),
            PIANO_WHITE_BORDER: Color32::from_rgb(0xC9, 0xCC, 0xD0),
            PIANO_WHITE_ROOT: Color32::from_rgb(0xC5, 0xCE, 0xF0),
            PIANO_WHITE_ACTIVE: Color32::from_rgb(0x54, 0x5A, 0x99),
            PIANO_WHITE_NOTE: Color32::from_rgb(0x3B, 0x40, 0x4E),
            PIANO_BLACK_BG: Color32::from_rgb(0x34, 0x37, 0x3E),
            PIANO_BLACK_BORDER: Color32::from_rgb(0x22, 0x25, 0x2A),
            PIANO_BLACK_ACTIVE: Color32::from_rgb(0x54, 0x5A, 0x99),
            PIANO_BLACK_TXT: Color32::from_rgb(0xB2, 0xB2, 0xB2),
            DOT_LIT: Color32::from_rgb(0x54, 0x5A, 0x99),
            DOT_UNLIT: Color32::from_rgb(0xC9, 0xCC, 0xD0),
            KEY_SUB_TRIGGERED: Color32::from_rgb(0xF2, 0xF4, 0xFE),
            KEY_LABEL_SHIFT_TRIGGERED: Color32::from_rgb(0xE4, 0xE7, 0xF8),
            KEY_LABEL_KEY_TRIGGERED: Color32::from_rgb(0xEE, 0xF0, 0xFB),
            PIANO_WHITE_MAPPED_ROOT_TXT: Color32::from_rgb(0x2A, 0x30, 0x57),
            PIANO_WHITE_MAPPED_TXT: Color32::from_rgb(0x3E, 0x45, 0x70),
            PIANO_WHITE_ROOT_TXT: Color32::from_rgb(0x1E, 0x24, 0x4A),
            PIANO_WHITE_OUT_SCALE_TXT: Color32::from_rgb(0x9B, 0x9E, 0xA6),
            PIANO_WHITE_OUT_SCALE_BG: Color32::from_rgb(0xD9, 0xDC, 0xE0),
            PIANO_BLACK_IN_SCALE_BG: Color32::from_rgb(0x2E, 0x35, 0x50),
            PIANO_BLACK_ROOT_BG: Color32::from_rgb(0x45, 0x4E, 0x7E),
            PIANO_BLACK_IN_SCALE_TXT: Color32::from_rgb(0xCF, 0xD5, 0xF2),
            PIANO_BLACK_OUT_SCALE_TXT: Color32::from_rgb(0x4A, 0x4E, 0x5F),
        }
    }

    pub fn studio_slate() -> Self {
        Self {
            BG_APP: Color32::from_rgb(0x23, 0x23, 0x23),
            BG_BAR: Color32::from_rgb(0x34, 0x34, 0x36),
            BG_PANEL: Color32::from_rgb(0x30, 0x30, 0x30),
            BG_KEYBOARD: Color32::from_rgb(0x1C, 0x1C, 0x1C),
            BG_FIELD: Color32::from_rgb(0x29, 0x29, 0x29),
            BG_BUTTON: Color32::from_rgb(0x40, 0x40, 0x40),
            BG_BUTTON_HOVER: Color32::from_rgb(0x50, 0x50, 0x50),
            BORDER_DARK: Color32::from_rgb(0x38, 0x38, 0x38),
            BORDER_PANEL: Color32::from_rgb(0x3D, 0x3D, 0x3D),
            BORDER_KEYBOARD: Color32::from_rgb(0x35, 0x35, 0x35),
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
            LOGO: Color32::from_rgb(0xFD, 0xFD, 0xFF),
            KEY_UNMAPPED_BG: Color32::from_rgb(0x22, 0x22, 0x22),
            KEY_UNMAPPED_BORDER: Color32::from_rgb(0x30, 0x30, 0x30),
            KEY_UNMAPPED_TXT: Color32::from_rgb(0x6E, 0x6E, 0x6E),
            KEY_MAPPED_BG: Color32::from_rgb(0x22, 0x25, 0x33),
            KEY_MAPPED_BORDER: Color32::from_rgb(0x38, 0x3D, 0x56),
            KEY_MAPPED_TXT: Color32::from_rgb(0xB8, 0xBE, 0xD4),
            KEY_MAPPED_SUB: Color32::from_rgb(0x75, 0x7B, 0x9E),
            KEY_ROOT_BG: Color32::from_rgb(0x3E, 0x46, 0x6E),
            KEY_ROOT_BORDER: Color32::from_rgb(0x78, 0x82, 0xC8),
            KEY_ACTIVE_BG: Color32::from_rgb(0x54, 0x5A, 0x99),
            KEY_ACTIVE_BORDER: Color32::from_rgb(0x8C, 0x93, 0xD4),
            KEY_ACTIVE_TXT: Color32::from_rgb(0xFD, 0xFD, 0xFF),
            PIANO_WHITE_BG: Color32::from_rgb(0xF5, 0xF6, 0xFA),
            PIANO_WHITE_BORDER: Color32::from_rgb(0x40, 0x44, 0x54),
            PIANO_WHITE_ROOT: Color32::from_rgb(0xC4, 0xCE, 0xEE),
            PIANO_WHITE_ACTIVE: Color32::from_rgb(0x54, 0x5A, 0x99),
            PIANO_WHITE_NOTE: Color32::from_rgb(0x38, 0x3C, 0x4E),
            PIANO_BLACK_BG: Color32::from_rgb(0x13, 0x14, 0x18),
            PIANO_BLACK_BORDER: Color32::from_rgb(0x0C, 0x0D, 0x10),
            PIANO_BLACK_ACTIVE: Color32::from_rgb(0x54, 0x5A, 0x99),
            PIANO_BLACK_TXT: Color32::from_rgb(0xAA, 0xAA, 0xAA),
            DOT_LIT: Color32::from_rgb(0x54, 0x5A, 0x99),
            DOT_UNLIT: Color32::from_rgb(0x37, 0x37, 0x37),
            KEY_SUB_TRIGGERED: Color32::from_rgb(0xF0, 0xF2, 0xFD),
            KEY_LABEL_SHIFT_TRIGGERED: Color32::from_rgb(0xD8, 0xDB, 0xF5),
            KEY_LABEL_KEY_TRIGGERED: Color32::from_rgb(0xEB, 0xED, 0xFD),
            PIANO_WHITE_MAPPED_ROOT_TXT: Color32::from_rgb(0x34, 0x3A, 0x58),
            PIANO_WHITE_MAPPED_TXT: Color32::from_rgb(0x47, 0x4E, 0x7A),
            PIANO_WHITE_ROOT_TXT: Color32::from_rgb(0x1E, 0x24, 0x4A),
            PIANO_WHITE_OUT_SCALE_TXT: Color32::from_rgb(0x84, 0x88, 0x98),
            PIANO_WHITE_OUT_SCALE_BG: Color32::from_rgb(0x54, 0x58, 0x68),
            PIANO_BLACK_IN_SCALE_BG: Color32::from_rgb(0x26, 0x2B, 0x3E),
            PIANO_BLACK_ROOT_BG: Color32::from_rgb(0x3E, 0x48, 0x76),
            PIANO_BLACK_IN_SCALE_TXT: Color32::from_rgb(0xC4, 0xCE, 0xEE),
            PIANO_BLACK_OUT_SCALE_TXT: Color32::from_rgb(0x4A, 0x4E, 0x5C),
        }
    }

    pub fn studio_sage() -> Self {
        Self {
            BG_APP: Color32::from_rgb(0x15, 0x15, 0x15),
            BG_BAR: Color32::from_rgb(0x24, 0x26, 0x24),
            BG_PANEL: Color32::from_rgb(0x20, 0x20, 0x20),
            BG_KEYBOARD: Color32::from_rgb(0x18, 0x18, 0x18),
            BG_FIELD: Color32::from_rgb(0x2A, 0x2A, 0x2A),
            BG_BUTTON: Color32::from_rgb(0x22, 0x22, 0x22),
            BG_BUTTON_HOVER: Color32::from_rgb(0x2E, 0x2E, 0x2E),
            BORDER_DARK: Color32::from_rgb(0x34, 0x34, 0x34),
            BORDER_PANEL: Color32::from_rgb(0x38, 0x38, 0x38),
            BORDER_KEYBOARD: Color32::from_rgb(0x2C, 0x2C, 0x2C),
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
            LOGO: Color32::from_rgb(0xF0, 0xF1, 0xED),
            KEY_UNMAPPED_BG: Color32::from_rgb(0x20, 0x20, 0x20),
            KEY_UNMAPPED_BORDER: Color32::from_rgb(0x35, 0x35, 0x35),
            KEY_UNMAPPED_TXT: Color32::from_rgb(0x80, 0x85, 0x7C),
            KEY_MAPPED_BG: Color32::from_rgb(0x24, 0x2B, 0x22),
            KEY_MAPPED_BORDER: Color32::from_rgb(0x3E, 0x48, 0x38),
            KEY_MAPPED_TXT: Color32::from_rgb(0xB8, 0xC4, 0xAE),
            KEY_MAPPED_SUB: Color32::from_rgb(0x7B, 0x8A, 0x70),
            KEY_ROOT_BG: Color32::from_rgb(0x3E, 0x50, 0x36),
            KEY_ROOT_BORDER: Color32::from_rgb(0x96, 0xA8, 0x84),
            KEY_ACTIVE_BG: Color32::from_rgb(0x74, 0x86, 0x5F),
            KEY_ACTIVE_BORDER: Color32::from_rgb(0xA3, 0xB5, 0x8A),
            KEY_ACTIVE_TXT: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            PIANO_WHITE_BG: Color32::from_rgb(0xF2, 0xF5, 0xEE),
            PIANO_WHITE_BORDER: Color32::from_rgb(0x48, 0x50, 0x44),
            PIANO_WHITE_ROOT: Color32::from_rgb(0xC8, 0xD8, 0xB8),
            PIANO_WHITE_ACTIVE: Color32::from_rgb(0x93, 0xA8, 0x79),
            PIANO_WHITE_NOTE: Color32::from_rgb(0x38, 0x42, 0x34),
            PIANO_BLACK_BG: Color32::from_rgb(0x14, 0x16, 0x13),
            PIANO_BLACK_BORDER: Color32::from_rgb(0x0C, 0x0E, 0x0B),
            PIANO_BLACK_ACTIVE: Color32::from_rgb(0x74, 0x86, 0x5F),
            PIANO_BLACK_TXT: Color32::from_rgb(0xBD, 0xC3, 0xB6),
            DOT_LIT: Color32::from_rgb(0x91, 0xA4, 0x77),
            DOT_UNLIT: Color32::from_rgb(0x51, 0x56, 0x4E),
            KEY_SUB_TRIGGERED: Color32::from_rgb(238, 242, 233),
            KEY_LABEL_SHIFT_TRIGGERED: Color32::from_rgb(200, 215, 190),
            KEY_LABEL_KEY_TRIGGERED: Color32::from_rgb(220, 230, 210),
            PIANO_WHITE_MAPPED_ROOT_TXT: Color32::from_rgb(0x35, 0x40, 0x2E),
            PIANO_WHITE_MAPPED_TXT: Color32::from_rgb(0x55, 0x5D, 0x4E),
            PIANO_WHITE_ROOT_TXT: Color32::from_rgb(0x24, 0x32, 0x1E),
            PIANO_WHITE_OUT_SCALE_TXT: Color32::from_rgb(0x82, 0x8A, 0x7E),
            PIANO_WHITE_OUT_SCALE_BG: Color32::from_rgb(0x56, 0x5E, 0x52),
            PIANO_BLACK_IN_SCALE_BG: Color32::from_rgb(0x28, 0x34, 0x24),
            PIANO_BLACK_ROOT_BG: Color32::from_rgb(0x42, 0x56, 0x38),
            PIANO_BLACK_IN_SCALE_TXT: Color32::from_rgb(0xC2, 0xD2, 0xB4),
            PIANO_BLACK_OUT_SCALE_TXT: Color32::from_rgb(0x4E, 0x54, 0x4A),
        }
    }
}
