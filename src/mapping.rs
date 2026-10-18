use crate::scale::{format_midi_note, RootNote, ScaleType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutPreset {
    #[default]
    LinearSplit,
    OctaveStacked,
    IsomorphicFourths,
    ChordBassMelody,
    LinearContinuous,
}

impl LayoutPreset {
    pub const ALL: [LayoutPreset; 5] = [
        LayoutPreset::LinearSplit,
        LayoutPreset::OctaveStacked,
        LayoutPreset::IsomorphicFourths,
        LayoutPreset::ChordBassMelody,
        LayoutPreset::LinearContinuous,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            LayoutPreset::LinearSplit => "Linear Split (20+20)",
            LayoutPreset::OctaveStacked => "Octave Stacked (Vertical)",
            LayoutPreset::IsomorphicFourths => "Isomorphic Fourths (Guitar)",
            LayoutPreset::ChordBassMelody => "Chords Left + Lead Right",
            LayoutPreset::LinearContinuous => "Linear Continuous (40 Keys)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandSide {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct KeyMappingDef {
    pub label: &'static str,
    pub usage: u32,
    pub mac_keycode: i64,
    pub hand: HandSide,
    #[allow(dead_code)]
    pub row: usize,
    #[allow(dead_code)]
    pub col: usize,
    pub degree_index: usize, // Bottom-to-top ascending scale index
}

pub static KEY_DEFINITIONS: &[KeyMappingDef] = &[
    // ------------------------------------------------------------------------
    // LEFT HAND (20 keys: 5 keys x 4 rows)
    // ------------------------------------------------------------------------
    // Row 3: Bottom Row (Lowest pitch: Degrees 0..4)
    KeyMappingDef { label: "Z", usage: 0x1D, mac_keycode: 6, hand: HandSide::Left, row: 3, col: 0, degree_index: 0 },
    KeyMappingDef { label: "X", usage: 0x1B, mac_keycode: 7, hand: HandSide::Left, row: 3, col: 1, degree_index: 1 },
    KeyMappingDef { label: "C", usage: 0x06, mac_keycode: 8, hand: HandSide::Left, row: 3, col: 2, degree_index: 2 },
    KeyMappingDef { label: "V", usage: 0x19, mac_keycode: 9, hand: HandSide::Left, row: 3, col: 3, degree_index: 3 },
    KeyMappingDef { label: "B", usage: 0x05, mac_keycode: 11, hand: HandSide::Left, row: 3, col: 4, degree_index: 4 },

    // Row 2: Home Row (Degrees 5..9)
    KeyMappingDef { label: "A", usage: 0x04, mac_keycode: 0, hand: HandSide::Left, row: 2, col: 0, degree_index: 5 },
    KeyMappingDef { label: "S", usage: 0x16, mac_keycode: 1, hand: HandSide::Left, row: 2, col: 1, degree_index: 6 },
    KeyMappingDef { label: "D", usage: 0x07, mac_keycode: 2, hand: HandSide::Left, row: 2, col: 2, degree_index: 7 },
    KeyMappingDef { label: "F", usage: 0x09, mac_keycode: 3, hand: HandSide::Left, row: 2, col: 3, degree_index: 8 },
    KeyMappingDef { label: "G", usage: 0x0A, mac_keycode: 5, hand: HandSide::Left, row: 2, col: 4, degree_index: 9 },

    // Row 1: Top Row (Degrees 10..14)
    KeyMappingDef { label: "Q", usage: 0x14, mac_keycode: 12, hand: HandSide::Left, row: 1, col: 0, degree_index: 10 },
    KeyMappingDef { label: "W", usage: 0x1A, mac_keycode: 13, hand: HandSide::Left, row: 1, col: 1, degree_index: 11 },
    KeyMappingDef { label: "E", usage: 0x08, mac_keycode: 14, hand: HandSide::Left, row: 1, col: 2, degree_index: 12 },
    KeyMappingDef { label: "R", usage: 0x15, mac_keycode: 15, hand: HandSide::Left, row: 1, col: 3, degree_index: 13 },
    KeyMappingDef { label: "T", usage: 0x17, mac_keycode: 17, hand: HandSide::Left, row: 1, col: 4, degree_index: 14 },

    // Row 0: Number Row (Degrees 15..19)
    KeyMappingDef { label: "1", usage: 0x1E, mac_keycode: 18, hand: HandSide::Left, row: 0, col: 0, degree_index: 15 },
    KeyMappingDef { label: "2", usage: 0x1F, mac_keycode: 19, hand: HandSide::Left, row: 0, col: 1, degree_index: 16 },
    KeyMappingDef { label: "3", usage: 0x20, mac_keycode: 20, hand: HandSide::Left, row: 0, col: 2, degree_index: 17 },
    KeyMappingDef { label: "4", usage: 0x21, mac_keycode: 21, hand: HandSide::Left, row: 0, col: 3, degree_index: 18 },
    KeyMappingDef { label: "5", usage: 0x22, mac_keycode: 23, hand: HandSide::Left, row: 0, col: 4, degree_index: 19 },

    // ------------------------------------------------------------------------
    // RIGHT HAND (20 keys: 5 keys x 4 rows)
    // ------------------------------------------------------------------------
    // Row 3: Bottom Row (Degrees 0..4)
    KeyMappingDef { label: "N", usage: 0x11, mac_keycode: 45, hand: HandSide::Right, row: 3, col: 0, degree_index: 0 },
    KeyMappingDef { label: "M", usage: 0x10, mac_keycode: 46, hand: HandSide::Right, row: 3, col: 1, degree_index: 1 },
    KeyMappingDef { label: ",", usage: 0x36, mac_keycode: 43, hand: HandSide::Right, row: 3, col: 2, degree_index: 2 },
    KeyMappingDef { label: ".", usage: 0x37, mac_keycode: 47, hand: HandSide::Right, row: 3, col: 3, degree_index: 3 },
    KeyMappingDef { label: "/", usage: 0x38, mac_keycode: 44, hand: HandSide::Right, row: 3, col: 4, degree_index: 4 },

    // Row 2: Home Row (Degrees 5..9)
    KeyMappingDef { label: "H", usage: 0x0B, mac_keycode: 4, hand: HandSide::Right, row: 2, col: 0, degree_index: 5 },
    KeyMappingDef { label: "J", usage: 0x0D, mac_keycode: 38, hand: HandSide::Right, row: 2, col: 1, degree_index: 6 },
    KeyMappingDef { label: "K", usage: 0x0E, mac_keycode: 40, hand: HandSide::Right, row: 2, col: 2, degree_index: 7 },
    KeyMappingDef { label: "L", usage: 0x0F, mac_keycode: 37, hand: HandSide::Right, row: 2, col: 3, degree_index: 8 },
    KeyMappingDef { label: ";", usage: 0x33, mac_keycode: 41, hand: HandSide::Right, row: 2, col: 4, degree_index: 9 },

    // Row 1: Top Row (Degrees 10..14)
    KeyMappingDef { label: "Y", usage: 0x1C, mac_keycode: 16, hand: HandSide::Right, row: 1, col: 0, degree_index: 10 },
    KeyMappingDef { label: "U", usage: 0x18, mac_keycode: 32, hand: HandSide::Right, row: 1, col: 1, degree_index: 11 },
    KeyMappingDef { label: "I", usage: 0x0C, mac_keycode: 34, hand: HandSide::Right, row: 1, col: 2, degree_index: 12 },
    KeyMappingDef { label: "O", usage: 0x12, mac_keycode: 31, hand: HandSide::Right, row: 1, col: 3, degree_index: 13 },
    KeyMappingDef { label: "P", usage: 0x13, mac_keycode: 35, hand: HandSide::Right, row: 1, col: 4, degree_index: 14 },

    // Row 0: Number Row (Degrees 15..19)
    KeyMappingDef { label: "6", usage: 0x23, mac_keycode: 22, hand: HandSide::Right, row: 0, col: 0, degree_index: 15 },
    KeyMappingDef { label: "7", usage: 0x24, mac_keycode: 26, hand: HandSide::Right, row: 0, col: 1, degree_index: 16 },
    KeyMappingDef { label: "8", usage: 0x25, mac_keycode: 28, hand: HandSide::Right, row: 0, col: 2, degree_index: 17 },
    KeyMappingDef { label: "9", usage: 0x26, mac_keycode: 25, hand: HandSide::Right, row: 0, col: 3, degree_index: 18 },
    KeyMappingDef { label: "0", usage: 0x27, mac_keycode: 29, hand: HandSide::Right, row: 0, col: 4, degree_index: 19 },
];

pub fn find_key_by_usage(usage: u32) -> Option<&'static KeyMappingDef> {
    KEY_DEFINITIONS.iter().find(|k| k.usage == usage)
}

#[allow(dead_code)]
pub fn get_key(hand: HandSide, row: usize, col: usize) -> Option<&'static KeyMappingDef> {
    KEY_DEFINITIONS.iter().find(|k| k.hand == hand && k.row == row && k.col == col)
}

pub fn find_key_by_label(label: &str) -> Option<&'static KeyMappingDef> {
    KEY_DEFINITIONS.iter().find(|k| k.label == label)
}

pub fn hid_usage_to_mac_keycode(usage: u32) -> Option<i64> {
    match usage {
        0x2C => Some(49), // Space
        0x2D => Some(27), // -
        0x2E => Some(24), // =
        0x2F => Some(33), // [
        0x30 => Some(30), // ]
        0x31 => Some(42), // \
        0x34 => Some(39), // '
        0x35 => Some(50), // `
        _ => find_key_by_usage(usage).map(|k| k.mac_keycode),
    }
}

#[allow(dead_code)]
pub fn calculate_key_note(
    key: &KeyMappingDef,
    root: RootNote,
    scale: ScaleType,
    left_base_octave: i8,
    right_base_octave: i8,
) -> u8 {
    calculate_key_notes(
        key,
        root,
        scale,
        left_base_octave,
        right_base_octave,
        LayoutPreset::LinearSplit,
    )[0]
}

pub fn calculate_key_notes(
    key: &KeyMappingDef,
    root: RootNote,
    scale: ScaleType,
    left_base_octave: i8,
    right_base_octave: i8,
    preset: LayoutPreset,
) -> Vec<u8> {
    match preset {
        LayoutPreset::LinearSplit => {
            let base_octave = match key.hand {
                HandSide::Left => left_base_octave,
                HandSide::Right => right_base_octave,
            };
            vec![scale.get_midi_note(root, base_octave, key.degree_index)]
        }
        LayoutPreset::OctaveStacked => {
            let global_col = if key.hand == HandSide::Left {
                key.col
            } else {
                key.col + 5
            };
            let row_oct_offset = (3 - key.row as i8).max(0);
            let oct = left_base_octave + row_oct_offset;
            vec![scale.get_midi_note(root, oct, global_col)]
        }
        LayoutPreset::IsomorphicFourths => {
            let row_deg = (3 - key.row) * 3;
            let deg = row_deg + key.col;
            let base_octave = match key.hand {
                HandSide::Left => left_base_octave,
                HandSide::Right => right_base_octave,
            };
            vec![scale.get_midi_note(root, base_octave, deg)]
        }
        LayoutPreset::ChordBassMelody => match key.hand {
            HandSide::Left => match key.row {
                3 => {
                    let note = scale.get_midi_note(root, left_base_octave, key.col);
                    vec![note]
                }
                2 => {
                    let base_deg = key.col;
                    let r = scale.get_midi_note(root, left_base_octave + 1, base_deg);
                    let t = scale.get_midi_note(root, left_base_octave + 1, base_deg + 2);
                    let f = scale.get_midi_note(root, left_base_octave + 1, base_deg + 4);
                    vec![r, t, f]
                }
                1 => {
                    let base_deg = key.col + 5;
                    let r = scale.get_midi_note(root, left_base_octave + 1, base_deg);
                    let t = scale.get_midi_note(root, left_base_octave + 1, base_deg + 2);
                    let f = scale.get_midi_note(root, left_base_octave + 1, base_deg + 4);
                    vec![r, t, f]
                }
                _ => {
                    let base_deg = key.col;
                    let r = scale.get_midi_note(root, left_base_octave + 1, base_deg);
                    let t = scale.get_midi_note(root, left_base_octave + 1, base_deg + 2);
                    let f = scale.get_midi_note(root, left_base_octave + 1, base_deg + 4);
                    let s = scale.get_midi_note(root, left_base_octave + 1, base_deg + 6);
                    vec![r, t, f, s]
                }
            },
            HandSide::Right => {
                let deg = (3 - key.row) * 5 + key.col;
                vec![scale.get_midi_note(root, right_base_octave, deg)]
            }
        },
        LayoutPreset::LinearContinuous => {
            let global_col = if key.hand == HandSide::Left {
                key.col
            } else {
                key.col + 5
            };
            let deg = (3 - key.row) * 10 + global_col;
            vec![scale.get_midi_note(root, left_base_octave, deg)]
        }
    }
}

pub fn get_key_display_info(
    key: &KeyMappingDef,
    root: RootNote,
    scale: ScaleType,
    left_base_octave: i8,
    right_base_octave: i8,
    preset: LayoutPreset,
) -> (String, String, bool) {
    let notes = calculate_key_notes(key, root, scale, left_base_octave, right_base_octave, preset);
    let primary_note = notes.first().copied().unwrap_or(60);
    let is_root = scale.is_root(root, primary_note);

    match preset {
        LayoutPreset::LinearSplit => {
            let note_str = format_midi_note(primary_note);
            let degree = scale.degree_label(key.degree_index).to_string();
            (note_str, degree, is_root)
        }
        LayoutPreset::OctaveStacked => {
            let note_str = format_midi_note(primary_note);
            let global_col = if key.hand == HandSide::Left {
                key.col
            } else {
                key.col + 5
            };
            let degree = scale.degree_label(global_col).to_string();
            (note_str, degree, is_root)
        }
        LayoutPreset::IsomorphicFourths => {
            let note_str = format_midi_note(primary_note);
            let row_deg = (3 - key.row) * 3;
            let deg = row_deg + key.col;
            let degree = scale.degree_label(deg).to_string();
            (note_str, degree, is_root)
        }
        LayoutPreset::ChordBassMelody => {
            if key.hand == HandSide::Left {
                match key.row {
                    3 => {
                        let note_str = format_midi_note(primary_note);
                        let degree = format!("Bass {}", scale.degree_label(key.col));
                        (note_str, degree, is_root)
                    }
                    2 => {
                        let romans = ["I", "ii", "iii", "IV", "V"];
                        let roman = romans.get(key.col).copied().unwrap_or("Triad");
                        let note_str = format!("{roman} (3)");
                        let degree = format_midi_note(primary_note);
                        (note_str, degree, key.col == 0)
                    }
                    1 => {
                        let romans = ["vi", "vii°", "I+", "ii+", "iii+"];
                        let roman = romans.get(key.col).copied().unwrap_or("Triad");
                        let note_str = format!("{roman} (3)");
                        let degree = format_midi_note(primary_note);
                        (note_str, degree, false)
                    }
                    _ => {
                        let romans = ["I7", "ii7", "iii7", "IV7", "V7"];
                        let roman = romans.get(key.col).copied().unwrap_or("7th");
                        let note_str = format!("{roman} (4)");
                        let degree = format_midi_note(primary_note);
                        (note_str, degree, key.col == 0)
                    }
                }
            } else {
                let note_str = format_midi_note(primary_note);
                let deg = (3 - key.row) * 5 + key.col;
                let degree = scale.degree_label(deg).to_string();
                (note_str, degree, is_root)
            }
        }
        LayoutPreset::LinearContinuous => {
            let note_str = format_midi_note(primary_note);
            let global_col = if key.hand == HandSide::Left {
                key.col
            } else {
                key.col + 5
            };
            let deg = (3 - key.row) * 10 + global_col;
            let degree = scale.degree_label(deg).to_string();
            (note_str, degree, is_root)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_hand_split_counts() {
        let left_keys: Vec<_> = KEY_DEFINITIONS.iter().filter(|k| k.hand == HandSide::Left).collect();
        let right_keys: Vec<_> = KEY_DEFINITIONS.iter().filter(|k| k.hand == HandSide::Right).collect();

        assert_eq!(left_keys.len(), 20);
        assert_eq!(right_keys.len(), 20);
        assert_eq!(KEY_DEFINITIONS.len(), 40);
    }

    #[test]
    fn test_ascending_degrees() {
        // Left hand lowest: Z = degree 0
        let z = find_key_by_label("Z").unwrap();
        assert_eq!(z.degree_index, 0);

        // Left hand highest: 5 = degree 19
        let key_5 = find_key_by_label("5").unwrap();
        assert_eq!(key_5.degree_index, 19);

        // Right hand lowest: N = degree 0
        let n = find_key_by_label("N").unwrap();
        assert_eq!(n.degree_index, 0);

        // Right hand highest: 0 = degree 19
        let key_0 = find_key_by_label("0").unwrap();
        assert_eq!(key_0.degree_index, 19);
    }

    #[test]
    fn test_preset_octave_stacked_vertical_alignment() {
        let z = find_key_by_label("Z").unwrap(); // row 3, col 0
        let a = find_key_by_label("A").unwrap(); // row 2, col 0
        let q = find_key_by_label("Q").unwrap(); // row 1, col 0
        let key_1 = find_key_by_label("1").unwrap(); // row 0, col 0

        let root = RootNote::C;
        let scale = ScaleType::Major;
        let left_oct = 2;
        let right_oct = 4;
        let preset = LayoutPreset::OctaveStacked;

        let note_z = calculate_key_notes(z, root, scale, left_oct, right_oct, preset)[0];
        let note_a = calculate_key_notes(a, root, scale, left_oct, right_oct, preset)[0];
        let note_q = calculate_key_notes(q, root, scale, left_oct, right_oct, preset)[0];
        let note_1 = calculate_key_notes(key_1, root, scale, left_oct, right_oct, preset)[0];

        // Each row should be exactly 12 semitones (1 octave) higher
        assert_eq!(note_a, note_z + 12);
        assert_eq!(note_q, note_a + 12);
        assert_eq!(note_1, note_q + 12);
    }

    #[test]
    fn test_preset_chord_bass_melody() {
        let root = RootNote::C;
        let scale = ScaleType::Major;
        let left_oct = 2;
        let right_oct = 4;
        let preset = LayoutPreset::ChordBassMelody;

        // A key is Row 2, Col 0 -> C Major Triad (C3, E3, G3)
        let a = find_key_by_label("A").unwrap();
        let chord = calculate_key_notes(a, root, scale, left_oct, right_oct, preset);
        assert_eq!(chord.len(), 3);
        assert_eq!(chord[0], 48); // C3
        assert_eq!(chord[1], 52); // E3
        assert_eq!(chord[2], 55); // G3

        // 1 key is Row 0, Col 0 -> C Major 7th (C3, E3, G3, B3)
        let key_1 = find_key_by_label("1").unwrap();
        let chord_7th = calculate_key_notes(key_1, root, scale, left_oct, right_oct, preset);
        assert_eq!(chord_7th.len(), 4);
        assert_eq!(chord_7th[0], 48); // C3
        assert_eq!(chord_7th[1], 52); // E3
        assert_eq!(chord_7th[2], 55); // G3
        assert_eq!(chord_7th[3], 59); // B3
    }
}
