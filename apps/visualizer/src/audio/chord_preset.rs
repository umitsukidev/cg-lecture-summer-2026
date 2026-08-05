#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordPreset {
    CMajor9,
    Dm9,
    Em7,
    FMaj9,
    G9sus4,
    AMinor9,
    AbMaj9,
    BbMaj9,
}

impl ChordPreset {
    pub fn frequencies(&self) -> [f32; 5] {
        match self {
            ChordPreset::CMajor9 => [261.63, 329.63, 392.00, 493.88, 587.33], // C4, E4, G4, B4, D5 [I]
            ChordPreset::Dm9     => [293.66, 349.23, 440.00, 523.25, 659.25], // D4, F4, A4, C5, E5 [ii]
            ChordPreset::Em7     => [329.63, 392.00, 493.88, 587.33, 739.99], // E4, G4, B4, D5, F#5 [iii]
            ChordPreset::FMaj9   => [174.61, 220.00, 261.63, 329.63, 392.00], // F3, A3, C4, E4, G4 [IV]
            ChordPreset::G9sus4  => [196.00, 261.63, 293.66, 349.23, 440.00], // G3, C4, D4, F4, A4 [V]
            ChordPreset::AMinor9 => [220.00, 261.63, 329.63, 392.00, 493.88], // A3, C4, E4, G4, B4 [vi]
            ChordPreset::AbMaj9  => [207.65, 261.63, 311.13, 392.00, 466.16], // Ab3, C4, Eb4, G4, Bb4 [bVI]
            ChordPreset::BbMaj9  => [233.08, 293.66, 349.23, 440.00, 523.25], // Bb3, D4, F4, A4, C5 [bVII]
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ChordPreset::CMajor9 => "C Major9 (Tonic)",
            ChordPreset::Dm9 => "D Minor9 (Subdominant)",
            ChordPreset::Em7 => "E Minor7 (Mediant)",
            ChordPreset::FMaj9 => "F Major9 (Subdominant)",
            ChordPreset::G9sus4 => "G9sus4 (Dominant)",
            ChordPreset::AMinor9 => "A Minor9 (Submediant)",
            ChordPreset::AbMaj9 => "Ab Major9 (Cinematic bVI)",
            ChordPreset::BbMaj9 => "Bb Major9 (Breezy bVII)",
        }
    }

    pub fn note_names(&self) -> [&'static str; 5] {
        match self {
            ChordPreset::CMajor9 => ["C4 (Root)", "E4 (3rd)", "G4 (5th)", "B4 (7th)", "D5 (9th)"],
            ChordPreset::Dm9 => ["D4 (Root)", "F4 (m3rd)", "A4 (5th)", "C5 (7th)", "E5 (9th)"],
            ChordPreset::Em7 => ["E4 (Root)", "G4 (m3rd)", "B4 (5th)", "D5 (7th)", "F#5 (9th)"],
            ChordPreset::FMaj9 => ["F3 (Root)", "A3 (3rd)", "C4 (5th)", "E4 (7th)", "G4 (9th)"],
            ChordPreset::G9sus4 => ["G3 (Root)", "C4 (sus4)", "D4 (5th)", "F4 (m7th)", "A4 (9th)"],
            ChordPreset::AMinor9 => ["A3 (Root)", "C4 (m3rd)", "E4 (5th)", "G4 (7th)", "B4 (9th)"],
            ChordPreset::AbMaj9 => ["Ab3 (Root)", "C4 (3rd)", "Eb4 (5th)", "G4 (7th)", "Bb4 (9th)"],
            ChordPreset::BbMaj9 => ["Bb3 (Root)", "D4 (3rd)", "F4 (5th)", "A4 (7th)", "C5 (9th)"],
        }
    }
}
