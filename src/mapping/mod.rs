//! Keyboard-to-MIDI mapping: the physical key table (see [`keys`]) plus the
//! layout presets that turn keys into MIDI notes (including the Ableton
//! piano-style preset).

mod keys;

pub use keys::{
    HandSide, KEY_DEFINITIONS, KeyMappingDef, find_key_by_label, find_key_by_usage,
    hid_usage_to_mac_keycode,
};

use std::collections::HashMap;

use crate::scale::{RootNote, ScaleMap, chromatic_degree_label, format_midi_note};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutPreset {
    #[default]
    LinearSplit,
    OctaveStacked,
    LinearContinuous,
    Custom,
    /// Keep `Ableton` last in the declaration order: presets are persisted by
    /// their index, so appending preserves saved settings files.
    Ableton,
}

impl LayoutPreset {
    pub const ALL: [LayoutPreset; 5] = [
        LayoutPreset::LinearSplit,
        LayoutPreset::OctaveStacked,
        LayoutPreset::LinearContinuous,
        LayoutPreset::Custom,
        LayoutPreset::Ableton,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            LayoutPreset::LinearSplit => "Linear Split",
            LayoutPreset::OctaveStacked => "Octave Stacked",
            LayoutPreset::LinearContinuous => "Linear Continuous",
            LayoutPreset::Custom => "Custom",
            LayoutPreset::Ableton => "Ableton",
        }
    }

    pub fn is_custom(&self) -> bool {
        *self == LayoutPreset::Custom
    }
}

/// Usage-code → MIDI-note table for [`LayoutPreset::Custom`].
pub type CustomLayoutMap = HashMap<u32, u8>;

/// The Ableton Live "Computer MIDI Keyboard" piano layout.
///
/// Home row plays the white keys, the row above plays the black keys, and the
/// piano continues into the next octave from `K` onward. Offsets are chromatic
/// semitones relative to the selected root (so `A` is always the root), and
/// keys Live doesn't map are silent.
///
/// A=C · W=C# · S=D · E=D# · D=E · F=F · T=F# · G=G · Y=G# · H=A · U=A# ·
/// J=B · K=C(+1 oct) · O=C#(+1 oct) · L=D(+1 oct) · P=D#(+1 oct) · ;=E(+1 oct)
pub static ABLETON_OFFSETS: &[(&str, i32)] = &[
    // Center row: the white keys of the base octave
    ("A", 0),
    ("S", 2),
    ("D", 4),
    ("F", 5),
    ("G", 7),
    ("H", 9),
    ("J", 11),
    // Upper row: the black keys of the base octave
    ("W", 1),
    ("E", 3),
    ("T", 6),
    ("Y", 8),
    ("U", 10),
    // From K the piano continues one octave up
    ("K", 12),
    ("O", 13),
    ("L", 14),
    ("P", 15),
    (";", 16),
];

pub fn ableton_offset(label: &str) -> Option<i32> {
    ABLETON_OFFSETS
        .iter()
        .find(|(l, _)| *l == label)
        .map(|&(_, offset)| offset)
}

/// A snapshot of everything needed to turn a keypress into a note.
#[derive(Clone)]
pub struct MappingConfig {
    pub root: RootNote,
    pub scale: ScaleMap,
    pub left_oct: i8,
    pub right_oct: i8,
    pub preset: LayoutPreset,
    pub padding: i8,
    pub custom: CustomLayoutMap,
}

impl MappingConfig {
    pub fn from_state(state: &crate::state::AppState) -> Self {
        use std::sync::atomic::Ordering;
        Self {
            root: state.get_root(),
            scale: state.get_scale_map(),
            left_oct: state.left_base_octave.load(Ordering::SeqCst),
            right_oct: state.right_base_octave.load(Ordering::SeqCst),
            preset: state.get_layout_preset(),
            padding: state.mapping_padding.load(Ordering::SeqCst),
            custom: state.get_custom_layout(),
        }
    }
}

/// Map a keypress to its MIDI note(s) under the given config. For the
/// Custom layout an unassigned key maps to nothing (silent).
pub fn calculate_key_notes(key: &KeyMappingDef, cfg: &MappingConfig) -> Vec<u8> {
    let MappingConfig {
        root,
        scale,
        left_oct: left_base_octave,
        right_oct: right_base_octave,
        preset,
        padding,
        custom,
    } = cfg;
    match preset {
        LayoutPreset::Custom => custom.get(&key.usage).copied().into_iter().collect(),
        LayoutPreset::LinearSplit => {
            let base_octave = match key.hand {
                HandSide::Left => *left_base_octave,
                HandSide::Right => *right_base_octave,
            };
            let deg = key.degree_index as i32 - *padding as i32;
            vec![scale.midi_note(*root, base_octave, deg)]
        }
        LayoutPreset::OctaveStacked => {
            let global_col = match key.hand {
                HandSide::Left => key.col,
                HandSide::Right => match key.row {
                    3 => key.col + 5,
                    _ => key.col + 6,
                },
            };
            let deg = global_col as i32 - *padding as i32;
            let row_oct_offset = (3 - key.row as i8).max(0);
            let oct = left_base_octave + row_oct_offset;
            vec![scale.midi_note(*root, oct, deg)]
        }
        LayoutPreset::Ableton => {
            // The Ableton layout is a fixed piano: chromatic semitone steps
            // from the selected root, one octave per 12, on the left octave.
            let Some(offset) = ableton_offset(key.label) else {
                return vec![]; // Keys Live doesn't map stay silent.
            };
            let oct = *left_base_octave + (offset / 12) as i8;
            let note =
                (oct as i16 + 1) * 12 + root.semitone() as i16 + (offset.rem_euclid(12)) as i16;
            vec![note.clamp(0, 127) as u8]
        }
        LayoutPreset::LinearContinuous => {
            let row_base = match key.row {
                3 => 0,
                2 => 9,
                1 => 19,
                _ => 30,
            };
            let global_col = match key.hand {
                HandSide::Left => key.col,
                HandSide::Right => match key.row {
                    3 => key.col + 5,
                    _ => key.col + 6,
                },
            };
            let deg = row_base + global_col as i32 - *padding as i32;
            vec![scale.midi_note(*root, *left_base_octave, deg)]
        }
    }
}

pub fn get_key_display_info(key: &KeyMappingDef, cfg: &MappingConfig) -> (String, String, bool) {
    let notes = calculate_key_notes(key, cfg);
    let primary_note = notes.first().copied().unwrap_or(60);
    // In the custom layout there is no scale, so no root highlighting.
    let is_root = !cfg.preset.is_custom()
        && !cfg.scale.chromatic
        && cfg.scale.is_root(cfg.root, primary_note);

    if cfg.preset.is_custom() {
        let note_str = cfg
            .custom
            .get(&key.usage)
            .map(|&n| format_midi_note(n))
            .unwrap_or_default();
        // Unassigned keys show nothing on the cap.
        return (note_str, String::new(), is_root);
    }

    match cfg.preset {
        LayoutPreset::LinearSplit => {
            let note_str = format_midi_note(primary_note);
            let deg = key.degree_index as i32 - cfg.padding as i32;
            let degree = cfg.scale.degree_label(deg).to_string();
            (note_str, degree, is_root)
        }
        LayoutPreset::OctaveStacked => {
            let note_str = format_midi_note(primary_note);
            let global_col = match key.hand {
                HandSide::Left => key.col,
                HandSide::Right => match key.row {
                    3 => key.col + 5,
                    _ => key.col + 6,
                },
            };
            let deg = global_col as i32 - cfg.padding as i32;
            let degree = cfg.scale.degree_label(deg).to_string();
            (note_str, degree, is_root)
        }
        LayoutPreset::LinearContinuous => {
            let note_str = format_midi_note(primary_note);
            let row_base = match key.row {
                3 => 0,
                2 => 9,
                1 => 19,
                _ => 30,
            };
            let global_col = match key.hand {
                HandSide::Left => key.col,
                HandSide::Right => match key.row {
                    3 => key.col + 5,
                    _ => key.col + 6,
                },
            };
            let deg = row_base + global_col as i32 - cfg.padding as i32;
            let degree = cfg.scale.degree_label(deg).to_string();
            (note_str, degree, is_root)
        }
        LayoutPreset::Ableton => {
            let Some(offset) = ableton_offset(key.label) else {
                // Unmapped keys are silent — show nothing on the cap.
                return (String::new(), String::new(), false);
            };
            let note_str = format_midi_note(primary_note);
            let degree = chromatic_degree_label(offset).to_string();
            (note_str, degree, is_root)
        }
        LayoutPreset::Custom => {
            // Handled by the early return above.
            unreachable!("custom layout display is handled before this match")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::{ScaleMap, ScaleType};

    #[test]
    fn test_two_hand_split_counts() {
        let left_keys: Vec<_> = KEY_DEFINITIONS
            .iter()
            .filter(|k| k.hand == HandSide::Left)
            .collect();
        let right_keys: Vec<_> = KEY_DEFINITIONS
            .iter()
            .filter(|k| k.hand == HandSide::Right)
            .collect();

        assert_eq!(left_keys.len(), 23);
        assert_eq!(right_keys.len(), 19);
        assert_eq!(KEY_DEFINITIONS.len(), 42);
    }

    #[test]
    fn test_ascending_degrees() {
        // Left hand lowest: Z = degree 0
        let z = find_key_by_label("Z").unwrap();
        assert_eq!(z.degree_index, 0);

        // Left hand highest: 6 = degree 22
        let key_6 = find_key_by_label("6").unwrap();
        assert_eq!(key_6.degree_index, 22);

        // Right hand lowest: N = degree 0
        let n = find_key_by_label("N").unwrap();
        assert_eq!(n.degree_index, 0);

        // Right hand highest: = = degree 18
        let key_eq = find_key_by_label("=").unwrap();
        assert_eq!(key_eq.degree_index, 18);
    }

    #[test]
    fn test_right_hand_keys_config() {
        // Removed keys should not exist in KEY_DEFINITIONS
        assert!(find_key_by_label("/").is_none());
        assert!(find_key_by_label("'").is_none());
        assert!(find_key_by_label("]").is_none());

        // Kept keys
        let open_bracket = find_key_by_label("[").unwrap();
        assert_eq!(open_bracket.usage, 0x2F);
        assert_eq!(open_bracket.mac_keycode, 33);
        assert_eq!(open_bracket.row, 1);
        assert_eq!(open_bracket.col, 4);
        assert_eq!(open_bracket.degree_index, 12);

        let minus = find_key_by_label("-").unwrap();
        assert_eq!(minus.usage, 0x2D);
        assert_eq!(minus.mac_keycode, 27);
        assert_eq!(minus.row, 0);
        assert_eq!(minus.col, 4);
        assert_eq!(minus.degree_index, 17);

        let equals = find_key_by_label("=").unwrap();
        assert_eq!(equals.usage, 0x2E);
        assert_eq!(equals.mac_keycode, 24);
        assert_eq!(equals.row, 0);
        assert_eq!(equals.col, 5);
        assert_eq!(equals.degree_index, 18);
    }

    #[test]
    fn test_preset_linear_continuous_strictly_ascending() {
        // Verify across rows that each subsequent key is higher pitch
        let z = find_key_by_label("Z").unwrap();
        let dot = find_key_by_label(".").unwrap();
        let a = find_key_by_label("A").unwrap();
        let semi = find_key_by_label(";").unwrap();
        let q = find_key_by_label("Q").unwrap();
        let open_bracket = find_key_by_label("[").unwrap();
        let key_1 = find_key_by_label("1").unwrap();
        let equals = find_key_by_label("=").unwrap();

        let cfg = map_cfg(
            RootNote::C,
            ScaleType::Major,
            2,
            4,
            LayoutPreset::LinearContinuous,
            0,
        );
        let n_z = calculate_key_notes(z, &cfg)[0];
        let n_dot = calculate_key_notes(dot, &cfg)[0];
        let n_a = calculate_key_notes(a, &cfg)[0];
        let n_semi = calculate_key_notes(semi, &cfg)[0];
        let n_q = calculate_key_notes(q, &cfg)[0];
        let n_open_b = calculate_key_notes(open_bracket, &cfg)[0];
        let n_1 = calculate_key_notes(key_1, &cfg)[0];
        let n_equals = calculate_key_notes(equals, &cfg)[0];

        assert!(n_z < n_dot);
        assert!(n_dot < n_a);
        assert!(n_a < n_semi);
        assert!(n_semi < n_q);
        assert!(n_q < n_open_b);
        assert!(n_open_b < n_1);
        assert!(n_1 < n_equals);
    }

    #[test]
    fn test_preset_octave_stacked_vertical_alignment() {
        let z = find_key_by_label("Z").unwrap(); // row 3, col 0
        let a = find_key_by_label("A").unwrap(); // row 2, col 0
        let q = find_key_by_label("Q").unwrap(); // row 1, col 0
        let key_1 = find_key_by_label("1").unwrap(); // row 0, col 0

        let cfg = map_cfg(
            RootNote::C,
            ScaleType::Major,
            2,
            4,
            LayoutPreset::OctaveStacked,
            0,
        );
        let note_z = calculate_key_notes(z, &cfg)[0];
        let note_a = calculate_key_notes(a, &cfg)[0];
        let note_q = calculate_key_notes(q, &cfg)[0];
        let note_1 = calculate_key_notes(key_1, &cfg)[0];

        // Each row should be exactly 12 semitones (1 octave) higher
        assert_eq!(note_a, note_z + 12);
        assert_eq!(note_q, note_a + 12);
        assert_eq!(note_1, note_q + 12);
    }

    #[test]
    fn test_mapping_padding_shifts_degrees() {
        let z = find_key_by_label("Z").unwrap();
        // With padding 0, Z (degree 0) is C3 (MIDI 48)
        let note_pad_0 = calculate_key_notes(
            z,
            &map_cfg(
                RootNote::C,
                ScaleType::Major,
                3,
                4,
                LayoutPreset::LinearSplit,
                0,
            ),
        )[0];
        assert_eq!(note_pad_0, 48);

        // With padding 1, Z plays degree -1 -> B2 (MIDI 47)
        let note_pad_1 = calculate_key_notes(
            z,
            &map_cfg(
                RootNote::C,
                ScaleType::Major,
                3,
                4,
                LayoutPreset::LinearSplit,
                1,
            ),
        )[0];
        assert_eq!(note_pad_1, 47);

        // With padding -1, Z plays degree 1 -> D3 (MIDI 50)
        let note_pad_neg1 = calculate_key_notes(
            z,
            &map_cfg(
                RootNote::C,
                ScaleType::Major,
                3,
                4,
                LayoutPreset::LinearSplit,
                -1,
            ),
        )[0];
        assert_eq!(note_pad_neg1, 50);
    }

    #[test]
    fn test_custom_layout_direct_assignment() {
        let cfg = MappingConfig {
            root: RootNote::C,
            scale: ScaleMap::from(ScaleType::Major),
            left_oct: 4,
            right_oct: 4,
            preset: LayoutPreset::Custom,
            padding: 0,
            custom: CustomLayoutMap::from([(0x04, 60), (0x05, 61)]),
        };

        // Assigned keys map straight to their note.
        let a_key = find_key_by_usage(0x04).unwrap();
        assert_eq!(calculate_key_notes(a_key, &cfg), vec![60]);

        // Unassigned keys are silent.
        let other = find_key_by_usage(0x09).unwrap();
        assert!(calculate_key_notes(other, &cfg).is_empty());
    }

    #[test]
    fn test_ableton_layout_piano_mapping() {
        let cfg = map_cfg(
            RootNote::C,
            ScaleType::Major,
            2,
            4,
            LayoutPreset::Ableton,
            0,
        );

        // Home row = white keys: A plays the root, S D F G H J continue up.
        let a = find_key_by_label("A").unwrap();
        let s = find_key_by_label("S").unwrap();
        let d = find_key_by_label("D").unwrap();
        let j = find_key_by_label("J").unwrap();
        assert_eq!(calculate_key_notes(a, &cfg), vec![36]); // C2 (Root)
        assert_eq!(calculate_key_notes(s, &cfg), vec![38]); // D2
        assert_eq!(calculate_key_notes(d, &cfg), vec![40]); // E2
        assert_eq!(calculate_key_notes(j, &cfg), vec![47]); // B2

        // Top row = black keys: W is one semitone above A.
        let w = find_key_by_label("W").unwrap();
        let e = find_key_by_label("E").unwrap();
        assert_eq!(calculate_key_notes(w, &cfg), vec![37]); // C#2
        assert_eq!(calculate_key_notes(e, &cfg), vec![39]); // D#2

        // From K the piano continues one octave up; ; reaches E3.
        let k = find_key_by_label("K").unwrap();
        let semi = find_key_by_label(";").unwrap();
        assert_eq!(calculate_key_notes(k, &cfg), vec![48]); // C3
        assert_eq!(calculate_key_notes(semi, &cfg), vec![52]); // E3
    }

    #[test]
    fn test_ableton_layout_unmapped_keys_silent() {
        let cfg = map_cfg(
            RootNote::C,
            ScaleType::Major,
            2,
            4,
            LayoutPreset::Ableton,
            0,
        );
        // Keys Live doesn't use stay silent, like the custom layout.
        for label in ["Z", "1", ",", "."] {
            let key = find_key_by_label(label).unwrap();
            assert!(
                calculate_key_notes(key, &cfg).is_empty(),
                "{label} should be silent"
            );
        }
    }

    #[test]
    fn test_ableton_layout_transposes_with_root_and_octave() {
        // Root D on octave 4: A plays D4, W plays D#4.
        let cfg = map_cfg(
            RootNote::D,
            ScaleType::Major,
            4,
            4,
            LayoutPreset::Ableton,
            0,
        );
        let a = find_key_by_label("A").unwrap();
        let w = find_key_by_label("W").unwrap();
        assert_eq!(calculate_key_notes(a, &cfg), vec![62]); // D4
        assert_eq!(calculate_key_notes(w, &cfg), vec![63]); // D#4
    }

    /// Test helper: build a `MappingConfig` with no custom bindings.
    fn map_cfg(
        root: RootNote,
        scale: ScaleType,
        left_oct: i8,
        right_oct: i8,
        preset: LayoutPreset,
        padding: i8,
    ) -> MappingConfig {
        MappingConfig {
            root,
            scale: ScaleMap::from(scale),
            left_oct,
            right_oct,
            preset,
            padding,
            custom: CustomLayoutMap::new(),
        }
    }
}
