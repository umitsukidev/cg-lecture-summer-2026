#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioMappingMode {
    Spatial,
    ColorMass,
}

impl AudioMappingMode {
    pub fn name(&self) -> &'static str {
        match self {
            AudioMappingMode::Spatial => "Spatial Quadrants",
            AudioMappingMode::ColorMass => "CMYKW Color Mass",
        }
    }

    pub fn voice_labels(&self) -> [&'static str; 5] {
        match self {
            AudioMappingMode::Spatial => [
                "Top-Left Zone",
                "Top-Right Zone",
                "Center Zone",
                "Bottom-Left Zone",
                "Bottom-Right Zone",
            ],
            AudioMappingMode::ColorMass => ["Cyan", "Magenta", "Yellow", "Black", "White"],
        }
    }
}
