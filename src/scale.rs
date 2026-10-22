#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootNote {
    C = 0,
    CSharp = 1,
    D = 2,
    DSharp = 3,
    E = 4,
    F = 5,
    FSharp = 6,
    G = 7,
    GSharp = 8,
    A = 9,
    ASharp = 10,
    B = 11,
}

impl RootNote {
    pub const ALL: [RootNote; 12] = [
        RootNote::C,
        RootNote::CSharp,
        RootNote::D,
        RootNote::DSharp,
        RootNote::E,
        RootNote::F,
        RootNote::FSharp,
        RootNote::G,
        RootNote::GSharp,
        RootNote::A,
        RootNote::ASharp,
        RootNote::B,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            RootNote::C => "C",
            RootNote::CSharp => "C#",
            RootNote::D => "D",
            RootNote::DSharp => "D#",
            RootNote::E => "E",
            RootNote::F => "F",
            RootNote::FSharp => "F#",
            RootNote::G => "G",
            RootNote::GSharp => "G#",
            RootNote::A => "A",
            RootNote::ASharp => "A#",
            RootNote::B => "B",
        }
    }

    pub fn semitone(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleType {
    NaturalMinor,
    Major,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    HarmonicMinor,
    MelodicMinor,
    MinorPentatonic,
    MajorPentatonic,
    Blues,
    JapaneseHirajoshi,
    ArabicDoubleHarmonic,
}

impl ScaleType {
    pub const ALL: [ScaleType; 13] = [
        ScaleType::NaturalMinor,
        ScaleType::Major,
        ScaleType::Dorian,
        ScaleType::Phrygian,
        ScaleType::Lydian,
        ScaleType::Mixolydian,
        ScaleType::HarmonicMinor,
        ScaleType::MelodicMinor,
        ScaleType::MinorPentatonic,
        ScaleType::MajorPentatonic,
        ScaleType::Blues,
        ScaleType::JapaneseHirajoshi,
        ScaleType::ArabicDoubleHarmonic,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ScaleType::NaturalMinor => "Natural Minor",
            ScaleType::Major => "Major",
            ScaleType::Dorian => "Dorian",
            ScaleType::Phrygian => "Phrygian",
            ScaleType::Lydian => "Lydian",
            ScaleType::Mixolydian => "Mixolydian",
            ScaleType::HarmonicMinor => "Harmonic Minor",
            ScaleType::MelodicMinor => "Melodic Minor",
            ScaleType::MinorPentatonic => "Minor Pentatonic",
            ScaleType::MajorPentatonic => "Major Pentatonic",
            ScaleType::Blues => "Blues",
            ScaleType::JapaneseHirajoshi => "Japanese (Hirajoshi)",
            ScaleType::ArabicDoubleHarmonic => "Arabic (Double Harmonic)",
        }
    }

    pub fn intervals(&self) -> &'static [u8] {
        match self {
            ScaleType::NaturalMinor => &[0, 2, 3, 5, 7, 8, 10],
            ScaleType::Major => &[0, 2, 4, 5, 7, 9, 11],
            ScaleType::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            ScaleType::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
            ScaleType::Lydian => &[0, 2, 4, 6, 7, 9, 11],
            ScaleType::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            ScaleType::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
            ScaleType::MelodicMinor => &[0, 2, 3, 5, 7, 9, 11],
            ScaleType::MinorPentatonic => &[0, 3, 5, 7, 10],
            ScaleType::MajorPentatonic => &[0, 2, 4, 7, 9],
            ScaleType::Blues => &[0, 3, 5, 6, 7, 10],
            ScaleType::JapaneseHirajoshi => &[0, 2, 3, 7, 8],
            ScaleType::ArabicDoubleHarmonic => &[0, 1, 4, 5, 7, 8, 11],
        }
    }

    pub fn degree_label_signed(&self, degree: i32) -> &'static str {
        let labels: &'static [&'static str] = match self {
            ScaleType::NaturalMinor => &["Root", "2nd", "m3rd", "4th", "5th", "m6th", "m7th"],
            ScaleType::Major => &["Root", "2nd", "3rd", "4th", "5th", "6th", "7th"],
            ScaleType::Dorian => &["Root", "2nd", "m3rd", "4th", "5th", "6th", "m7th"],
            ScaleType::Phrygian => &["Root", "b2nd", "m3rd", "4th", "5th", "m6th", "m7th"],
            ScaleType::Lydian => &["Root", "2nd", "3rd", "#4th", "5th", "6th", "7th"],
            ScaleType::Mixolydian => &["Root", "2nd", "3rd", "4th", "5th", "6th", "m7th"],
            ScaleType::HarmonicMinor => &["Root", "2nd", "m3rd", "4th", "5th", "m6th", "7th"],
            ScaleType::MelodicMinor => &["Root", "2nd", "m3rd", "4th", "5th", "6th", "7th"],
            ScaleType::MinorPentatonic => &["Root", "m3rd", "4th", "5th", "m7th"],
            ScaleType::MajorPentatonic => &["Root", "2nd", "3rd", "5th", "6th"],
            ScaleType::Blues => &["Root", "m3rd", "4th", "b5th", "5th", "m7th"],
            ScaleType::JapaneseHirajoshi => &["Root", "2nd", "m3rd", "5th", "m6th"],
            ScaleType::ArabicDoubleHarmonic => {
                &["Root", "b2nd", "3rd", "4th", "5th", "b6th", "7th"]
            }
        };

        let step = degree.rem_euclid(labels.len() as i32) as usize;
        labels[step]
    }

    pub fn get_midi_note_signed(&self, root: RootNote, base_octave: i8, degree: i32) -> u8 {
        let intervals = self.intervals();
        let num_notes = intervals.len() as i32;

        let octave_shift = degree.div_euclid(num_notes) as i8;
        let scale_step = degree.rem_euclid(num_notes) as usize;
        let interval_semitones = intervals[scale_step];

        let total_octave = base_octave + octave_shift;
        let note =
            (total_octave + 1) as i16 * 12 + root.semitone() as i16 + interval_semitones as i16;
        note.clamp(0, 127) as u8
    }

    pub fn is_in_scale(&self, root: RootNote, note: u8) -> bool {
        let pitch_class = (note % 12) as i16;
        let root_class = root.semitone() as i16;
        let interval = (pitch_class - root_class).rem_euclid(12) as u8;
        self.intervals().contains(&interval)
    }

    pub fn is_root(&self, root: RootNote, note: u8) -> bool {
        (note % 12) == root.semitone()
    }
}

pub fn format_midi_note(note: u8) -> String {
    let names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (note as i8 / 12) - 1;
    let name = names[(note % 12) as usize];
    format!("{name}{octave}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_minor_c2() {
        let scale = ScaleType::NaturalMinor;
        let root = RootNote::C;
        // Base octave 2: C2 is MIDI 36
        assert_eq!(scale.get_midi_note_signed(root, 2, 0), 36); // C2 (Root)
        assert_eq!(format_midi_note(36), "C2");
        assert_eq!(scale.degree_label_signed(0), "Root");

        assert_eq!(scale.get_midi_note_signed(root, 2, 1), 38); // D2 (2nd)
        assert_eq!(scale.degree_label_signed(1), "2nd");

        assert_eq!(scale.get_midi_note_signed(root, 2, 2), 39); // D#2 / Eb2 (m3rd)
        assert_eq!(scale.degree_label_signed(2), "m3rd");

        // Degree 7 = octave wrap to C3 (48)
        assert_eq!(scale.get_midi_note_signed(root, 2, 7), 48);
        assert_eq!(format_midi_note(48), "C3");
        assert_eq!(scale.degree_label_signed(7), "Root");
    }

    #[test]
    fn test_minor_pentatonic() {
        let scale = ScaleType::MinorPentatonic;
        let root = RootNote::A;
        // A2 is semitone 9, base octave 2 -> (2+1)*12 + 9 = 45 (A2)
        assert_eq!(scale.get_midi_note_signed(root, 2, 0), 45); // A2
        assert_eq!(scale.degree_label_signed(0), "Root");

        // Degree 5 = 1 octave wrap -> A3 (57)
        assert_eq!(scale.get_midi_note_signed(root, 2, 5), 57); // A3
    }
}
