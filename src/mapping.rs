use crate::scale::{RootNote, ScaleType};

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
    // LEFT HAND (Bass & Chords - 22 keys)
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

    // Row 1: Top Row (Degrees 10..15)
    KeyMappingDef { label: "Q", usage: 0x14, mac_keycode: 12, hand: HandSide::Left, row: 1, col: 0, degree_index: 10 },
    KeyMappingDef { label: "W", usage: 0x1A, mac_keycode: 13, hand: HandSide::Left, row: 1, col: 1, degree_index: 11 },
    KeyMappingDef { label: "E", usage: 0x08, mac_keycode: 14, hand: HandSide::Left, row: 1, col: 2, degree_index: 12 },
    KeyMappingDef { label: "R", usage: 0x15, mac_keycode: 15, hand: HandSide::Left, row: 1, col: 3, degree_index: 13 },
    KeyMappingDef { label: "T", usage: 0x17, mac_keycode: 17, hand: HandSide::Left, row: 1, col: 4, degree_index: 14 },
    KeyMappingDef { label: "Y", usage: 0x1C, mac_keycode: 16, hand: HandSide::Left, row: 1, col: 5, degree_index: 15 },

    // Row 0: Number Row (Degrees 16..21)
    KeyMappingDef { label: "1", usage: 0x1E, mac_keycode: 18, hand: HandSide::Left, row: 0, col: 0, degree_index: 16 },
    KeyMappingDef { label: "2", usage: 0x1F, mac_keycode: 19, hand: HandSide::Left, row: 0, col: 1, degree_index: 17 },
    KeyMappingDef { label: "3", usage: 0x20, mac_keycode: 20, hand: HandSide::Left, row: 0, col: 2, degree_index: 18 },
    KeyMappingDef { label: "4", usage: 0x21, mac_keycode: 21, hand: HandSide::Left, row: 0, col: 3, degree_index: 19 },
    KeyMappingDef { label: "5", usage: 0x22, mac_keycode: 23, hand: HandSide::Left, row: 0, col: 4, degree_index: 20 },
    KeyMappingDef { label: "6", usage: 0x23, mac_keycode: 22, hand: HandSide::Left, row: 0, col: 5, degree_index: 21 },

    // ------------------------------------------------------------------------
    // RIGHT HAND (Melodies & Leads - 23 keys, including -, =, [, ], ')
    // ------------------------------------------------------------------------
    // Row 3: Bottom Row (Degrees 0..4)
    KeyMappingDef { label: "N", usage: 0x11, mac_keycode: 45, hand: HandSide::Right, row: 3, col: 0, degree_index: 0 },
    KeyMappingDef { label: "M", usage: 0x10, mac_keycode: 46, hand: HandSide::Right, row: 3, col: 1, degree_index: 1 },
    KeyMappingDef { label: ",", usage: 0x36, mac_keycode: 43, hand: HandSide::Right, row: 3, col: 2, degree_index: 2 },
    KeyMappingDef { label: ".", usage: 0x37, mac_keycode: 47, hand: HandSide::Right, row: 3, col: 3, degree_index: 3 },
    KeyMappingDef { label: "/", usage: 0x38, mac_keycode: 44, hand: HandSide::Right, row: 3, col: 4, degree_index: 4 },

    // Row 2: Home Row (Degrees 5..10)
    KeyMappingDef { label: "H", usage: 0x0B, mac_keycode: 4, hand: HandSide::Right, row: 2, col: 0, degree_index: 5 },
    KeyMappingDef { label: "J", usage: 0x0D, mac_keycode: 38, hand: HandSide::Right, row: 2, col: 1, degree_index: 6 },
    KeyMappingDef { label: "K", usage: 0x0E, mac_keycode: 40, hand: HandSide::Right, row: 2, col: 2, degree_index: 7 },
    KeyMappingDef { label: "L", usage: 0x0F, mac_keycode: 37, hand: HandSide::Right, row: 2, col: 3, degree_index: 8 },
    KeyMappingDef { label: ";", usage: 0x33, mac_keycode: 41, hand: HandSide::Right, row: 2, col: 4, degree_index: 9 },
    KeyMappingDef { label: "'", usage: 0x34, mac_keycode: 39, hand: HandSide::Right, row: 2, col: 5, degree_index: 10 },

    // Row 1: Top Row (Degrees 11..16)
    KeyMappingDef { label: "U", usage: 0x18, mac_keycode: 32, hand: HandSide::Right, row: 1, col: 0, degree_index: 11 },
    KeyMappingDef { label: "I", usage: 0x0C, mac_keycode: 34, hand: HandSide::Right, row: 1, col: 1, degree_index: 12 },
    KeyMappingDef { label: "O", usage: 0x12, mac_keycode: 31, hand: HandSide::Right, row: 1, col: 2, degree_index: 13 },
    KeyMappingDef { label: "P", usage: 0x13, mac_keycode: 35, hand: HandSide::Right, row: 1, col: 3, degree_index: 14 },
    KeyMappingDef { label: "[", usage: 0x2F, mac_keycode: 33, hand: HandSide::Right, row: 1, col: 4, degree_index: 15 },
    KeyMappingDef { label: "]", usage: 0x30, mac_keycode: 30, hand: HandSide::Right, row: 1, col: 5, degree_index: 16 },

    // Row 0: Number Row (Degrees 17..22)
    KeyMappingDef { label: "7", usage: 0x24, mac_keycode: 26, hand: HandSide::Right, row: 0, col: 0, degree_index: 17 },
    KeyMappingDef { label: "8", usage: 0x25, mac_keycode: 28, hand: HandSide::Right, row: 0, col: 1, degree_index: 18 },
    KeyMappingDef { label: "9", usage: 0x26, mac_keycode: 25, hand: HandSide::Right, row: 0, col: 2, degree_index: 19 },
    KeyMappingDef { label: "0", usage: 0x27, mac_keycode: 29, hand: HandSide::Right, row: 0, col: 3, degree_index: 20 },
    KeyMappingDef { label: "-", usage: 0x2D, mac_keycode: 27, hand: HandSide::Right, row: 0, col: 4, degree_index: 21 },
    KeyMappingDef { label: "=", usage: 0x2E, mac_keycode: 24, hand: HandSide::Right, row: 0, col: 5, degree_index: 22 },
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
    if usage == 0x2C {
        return Some(49); // Space
    }
    find_key_by_usage(usage).map(|k| k.mac_keycode)
}

pub fn calculate_key_note(
    key: &KeyMappingDef,
    root: RootNote,
    scale: ScaleType,
    left_base_octave: i8,
    right_base_octave: i8,
) -> u8 {
    let base_octave = match key.hand {
        HandSide::Left => left_base_octave,
        HandSide::Right => right_base_octave,
    };
    scale.get_midi_note(root, base_octave, key.degree_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_hand_split_counts() {
        let left_keys: Vec<_> = KEY_DEFINITIONS.iter().filter(|k| k.hand == HandSide::Left).collect();
        let right_keys: Vec<_> = KEY_DEFINITIONS.iter().filter(|k| k.hand == HandSide::Right).collect();

        assert_eq!(left_keys.len(), 22);
        assert_eq!(right_keys.len(), 23);
        assert_eq!(KEY_DEFINITIONS.len(), 45);
    }

    #[test]
    fn test_ascending_degrees() {
        // Left hand lowest: Z = degree 0
        let z = find_key_by_label("Z").unwrap();
        assert_eq!(z.degree_index, 0);

        // Left hand highest: 6 = degree 21
        let key_6 = find_key_by_label("6").unwrap();
        assert_eq!(key_6.degree_index, 21);

        // Right hand lowest: N = degree 0
        let n = find_key_by_label("N").unwrap();
        assert_eq!(n.degree_index, 0);

        // Right hand highest: = (equals) = degree 22
        let key_eq = find_key_by_label("=").unwrap();
        assert_eq!(key_eq.degree_index, 22);
    }
}
