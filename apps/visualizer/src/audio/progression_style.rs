use super::ChordPreset;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressionStyle {
    CityPop,          // FMaj9 -> G9sus4 -> Em7 -> AMinor9 -> CMajor9
    NeoSoulJazz,      // Dm9 -> G9sus4 -> CMajor9 -> AMinor9
    CinematicModal,   // FMaj9 -> AbMaj9 -> BbMaj9 -> CMajor9
    GenerativeRandom, // Dynamic Functional Harmony Walk
}

impl ProgressionStyle {
    pub fn name(&self) -> &'static str {
        match self {
            ProgressionStyle::CityPop => "Pop / City Pop (王道・王道進行)",
            ProgressionStyle::NeoSoulJazz => "Jazz / Neo-Soul (2-5-1進行)",
            ProgressionStyle::CinematicModal => "Cinematic Modal (借用和音)",
            ProgressionStyle::GenerativeRandom => "Generative Harmonic Walk (音楽理論ランダム)",
        }
    }

    pub fn next_chord(&self, current: ChordPreset, rng_val: f32) -> ChordPreset {
        match self {
            ProgressionStyle::CityPop => match current {
                ChordPreset::FMaj9 => if rng_val < 0.6 { ChordPreset::G9sus4 } else { ChordPreset::Em7 },
                ChordPreset::G9sus4 => if rng_val < 0.7 { ChordPreset::Em7 } else { ChordPreset::CMajor9 },
                ChordPreset::Em7 => if rng_val < 0.8 { ChordPreset::AMinor9 } else { ChordPreset::Dm9 },
                ChordPreset::AMinor9 => if rng_val < 0.6 { ChordPreset::Dm9 } else { ChordPreset::FMaj9 },
                ChordPreset::Dm9 => if rng_val < 0.7 { ChordPreset::G9sus4 } else { ChordPreset::FMaj9 },
                _ => ChordPreset::FMaj9,
            },
            ProgressionStyle::NeoSoulJazz => match current {
                ChordPreset::Dm9 => ChordPreset::G9sus4,
                ChordPreset::G9sus4 => if rng_val < 0.7 { ChordPreset::CMajor9 } else { ChordPreset::Em7 },
                ChordPreset::CMajor9 => if rng_val < 0.6 { ChordPreset::AMinor9 } else { ChordPreset::FMaj9 },
                ChordPreset::AMinor9 => if rng_val < 0.7 { ChordPreset::Dm9 } else { ChordPreset::FMaj9 },
                _ => ChordPreset::Dm9,
            },
            ProgressionStyle::CinematicModal => match current {
                ChordPreset::FMaj9 => if rng_val < 0.6 { ChordPreset::AbMaj9 } else { ChordPreset::BbMaj9 },
                ChordPreset::AbMaj9 => ChordPreset::BbMaj9,
                ChordPreset::BbMaj9 => if rng_val < 0.7 { ChordPreset::CMajor9 } else { ChordPreset::FMaj9 },
                ChordPreset::CMajor9 => if rng_val < 0.6 { ChordPreset::AMinor9 } else { ChordPreset::FMaj9 },
                ChordPreset::AMinor9 => ChordPreset::FMaj9,
                _ => ChordPreset::FMaj9,
            },
            ProgressionStyle::GenerativeRandom => match current {
                ChordPreset::CMajor9 | ChordPreset::AMinor9 | ChordPreset::Em7 => {
                    if rng_val < 0.35 {
                        ChordPreset::FMaj9
                    } else if rng_val < 0.65 {
                        ChordPreset::Dm9
                    } else if rng_val < 0.85 {
                        ChordPreset::AbMaj9
                    } else {
                        ChordPreset::G9sus4
                    }
                }
                ChordPreset::FMaj9 | ChordPreset::Dm9 | ChordPreset::AbMaj9 | ChordPreset::BbMaj9 => {
                    if rng_val < 0.45 {
                        ChordPreset::G9sus4
                    } else if rng_val < 0.75 {
                        ChordPreset::CMajor9
                    } else {
                        ChordPreset::AMinor9
                    }
                }
                ChordPreset::G9sus4 => {
                    if rng_val < 0.6 {
                        ChordPreset::CMajor9
                    } else if rng_val < 0.85 {
                        ChordPreset::AMinor9
                    } else {
                        ChordPreset::Em7
                    }
                }
            },
        }
    }
}
