//! Settings persistence.
//!
//! Settings are saved to a small config file whenever the user changes one,
//! and re-applied on startup. There is no autosave/debounce — each setter
//! simply writes the file synchronously.

use std::fs;
use std::path::PathBuf;

use crate::mapping::LayoutPreset;
use crate::scale::{RootNote, ScaleType};
use crate::ui::AppTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    pub theme: AppTheme,
    pub root: RootNote,
    pub scale: ScaleType,
    pub layout_preset: LayoutPreset,
    pub base_octave: i8,
    pub mapping_padding: i8,
    pub velocity: u8,
    pub show_unmapped_note_names: bool,
    pub onboarding_completed: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: AppTheme::default(),
            root: RootNote::C,
            scale: ScaleType::NaturalMinor,
            layout_preset: LayoutPreset::LinearSplit,
            base_octave: 4,
            mapping_padding: 0,
            velocity: 100,
            show_unmapped_note_names: true,
            onboarding_completed: false,
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    let mut path = PathBuf::from(home);
    path.push("Library/Application Support/Qwerkeys");
    fs::create_dir_all(&path).ok()?;
    path.push("settings.conf");
    Some(path)
}

/// Load persisted settings, falling back to defaults for anything
/// missing or invalid.
pub fn load() -> Settings {
    let Some(path) = settings_path() else {
        return Settings::default();
    };
    let Ok(text) = fs::read_to_string(path) else {
        return Settings::default();
    };
    parse(&text)
}

fn parse(text: &str) -> Settings {
    let mut s = Settings::default();

    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "theme" => {
                if let Ok(idx) = value.parse::<usize>()
                    && let Some(theme) = AppTheme::ALL.iter().copied().find(|t| *t as usize == idx)
                {
                    s.theme = theme;
                }
            }
            "root" => {
                if let Ok(idx) = value.parse::<usize>()
                    && let Some(root) = RootNote::ALL.iter().copied().find(|r| *r as usize == idx)
                {
                    s.root = root;
                }
            }
            "scale" => {
                if let Ok(idx) = value.parse::<usize>()
                    && let Some(scale) = ScaleType::ALL
                        .iter()
                        .copied()
                        .find(|sc| *sc as usize == idx)
                {
                    s.scale = scale;
                }
            }
            "layout" => {
                if let Ok(idx) = value.parse::<usize>()
                    && let Some(preset) = LayoutPreset::ALL
                        .iter()
                        .copied()
                        .find(|p| *p as usize == idx)
                {
                    s.layout_preset = preset;
                }
            }
            "base_octave" => {
                if let Ok(v) = value.parse::<i8>() {
                    s.base_octave = v.clamp(0, 7);
                }
            }
            "mapping_padding" => {
                if let Ok(v) = value.parse::<i8>() {
                    s.mapping_padding = v.clamp(-7, 7);
                }
            }
            "velocity" => {
                if let Ok(v) = value.parse::<u8>() {
                    s.velocity = v.clamp(10, 127);
                }
            }
            "show_unmapped" => s.show_unmapped_note_names = value == "1",
            "onboarding_completed" => s.onboarding_completed = value == "1",
            _ => {}
        }
    }

    s
}

/// Persist the given settings. Writes to a temp file first and renames it
/// into place so a crash mid-write can never corrupt the config.
#[cfg(not(test))]
pub fn save(s: &Settings) {
    let Some(path) = settings_path() else {
        return;
    };

    let body = format!(
        "theme={}\nroot={}\nscale={}\nlayout={}\nbase_octave={}\nmapping_padding={}\nvelocity={}\nshow_unmapped={}\nonboarding_completed={}\n",
        s.theme as usize,
        s.root as usize,
        s.scale as usize,
        s.layout_preset as usize,
        s.base_octave,
        s.mapping_padding,
        s.velocity,
        u8::from(s.show_unmapped_note_names),
        u8::from(s.onboarding_completed),
    );

    use std::io::Write;
    let tmp = path.with_extension("conf.tmp");
    if fs::File::create(&tmp)
        .and_then(|mut f| f.write_all(body.as_bytes()))
        .is_ok()
    {
        let _ = fs::rename(&tmp, &path);
    }
}

/// No-op during tests so test runs never touch the real config file.
#[cfg(test)]
pub fn save(_s: &Settings) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_roundtrip() {
        let s = Settings {
            theme: AppTheme::Sage,
            root: RootNote::D,
            scale: ScaleType::Blues,
            layout_preset: LayoutPreset::LinearContinuous,
            base_octave: 2,
            mapping_padding: -3,
            velocity: 64,
            show_unmapped_note_names: false,
            onboarding_completed: true,
        };

        let body = format!(
            "theme={}\nroot={}\nscale={}\nlayout={}\nbase_octave={}\nmapping_padding={}\nvelocity={}\nshow_unmapped={}\nonboarding_completed={}\n",
            s.theme as usize,
            s.root as usize,
            s.scale as usize,
            s.layout_preset as usize,
            s.base_octave,
            s.mapping_padding,
            s.velocity,
            u8::from(s.show_unmapped_note_names),
            u8::from(s.onboarding_completed),
        );

        let parsed = parse(&body);
        assert_eq!(parsed.theme, s.theme);
        assert_eq!(parsed.root, s.root);
        assert_eq!(parsed.scale, s.scale);
        assert_eq!(parsed.layout_preset, s.layout_preset);
        assert_eq!(parsed.base_octave, s.base_octave);
        assert_eq!(parsed.mapping_padding, s.mapping_padding);
        assert_eq!(parsed.velocity, s.velocity);
        assert_eq!(parsed.show_unmapped_note_names, s.show_unmapped_note_names);
        assert_eq!(parsed.onboarding_completed, s.onboarding_completed);
    }

    #[test]
    fn test_parse_garbage_falls_back_to_defaults() {
        let d = Settings::default();
        // Invalid enum indexes are ignored (defaults kept, velocity clamped).
        let parsed = parse("theme=999\nroot=abc\nscale=42\nlayout=7\nvelocity=255\n");
        assert_eq!(parsed.theme, d.theme);
        assert_eq!(parsed.root, d.root);
        assert_eq!(parsed.scale, d.scale);
        assert_eq!(parsed.layout_preset, d.layout_preset);
        assert_eq!(parsed.velocity, 127);
    }
}
