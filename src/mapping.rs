pub fn usage_to_midi_base(usage: u32) -> Option<u8> {
    match usage {
        // White Keys
        0x04 => Some(60), // A -> C4
        0x16 => Some(62), // S -> D4
        0x07 => Some(64), // D -> E4
        0x09 => Some(65), // F -> F4
        0x0A => Some(67), // G -> G4
        0x0B => Some(69), // H -> A4
        0x0D => Some(71), // J -> B4
        0x0E => Some(72), // K -> C5
        0x0F => Some(74), // L -> D5
        0x33 => Some(76), // ; -> E5
        0x34 => Some(77), // ' -> F5

        // Black Keys
        0x1A => Some(61), // W -> C#4
        0x08 => Some(63), // E -> D#4
        0x17 => Some(66), // T -> F#4
        0x1C => Some(68), // Y -> G#4
        0x18 => Some(70), // U -> A#4
        0x12 => Some(73), // O -> C#5
        0x13 => Some(75), // P -> D#5

        _ => None,
    }
}

pub fn hid_usage_to_mac_keycode(usage: u32) -> Option<i64> {
    match usage {
        0x04 => Some(0),  // A
        0x16 => Some(1),  // S
        0x07 => Some(2),  // D
        0x09 => Some(3),  // F
        0x0B => Some(4),  // H
        0x0A => Some(5),  // G
        0x1D => Some(6),  // Z
        0x1B => Some(7),  // X
        0x06 => Some(8),  // C
        0x19 => Some(9),  // V
        0x05 => Some(11), // B
        0x14 => Some(12), // Q
        0x1A => Some(13), // W
        0x08 => Some(14), // E
        0x15 => Some(15), // R
        0x1C => Some(16), // Y
        0x17 => Some(17), // T
        0x1E => Some(18), // 1
        0x1F => Some(19), // 2
        0x20 => Some(20), // 3
        0x21 => Some(21), // 4
        0x23 => Some(22), // 6
        0x22 => Some(23), // 5
        0x2E => Some(24), // =
        0x26 => Some(25), // 9
        0x24 => Some(26), // 7
        0x2D => Some(27), // -
        0x25 => Some(28), // 8
        0x27 => Some(29), // 0
        0x30 => Some(30), // ]
        0x12 => Some(31), // O
        0x18 => Some(32), // U
        0x2F => Some(33), // [
        0x0C => Some(34), // I
        0x13 => Some(35), // P
        0x28 => Some(36), // Return
        0x0F => Some(37), // L
        0x0D => Some(38), // J
        0x34 => Some(39), // '
        0x0E => Some(40), // K
        0x33 => Some(41), // ;
        0x31 => Some(42), // \
        0x36 => Some(43), // ,
        0x38 => Some(44), // /
        0x11 => Some(45), // N
        0x10 => Some(46), // M
        0x37 => Some(47), // .
        0x2B => Some(48), // Tab
        0x2C => Some(49), // Space
        _ => None,
    }
}

pub fn format_note_name(note: u8) -> String {
    let names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (note as i8 / 12) - 1;
    let name = names[(note % 12) as usize];
    format!("{name}{octave} ({note})")
}
