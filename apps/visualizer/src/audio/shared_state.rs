use super::{AudioMappingMode, ChordPreset, ProgressionStyle};

pub struct SharedAudioState {
    pub enabled: bool,
    pub volume: f32,
    pub preset: ChordPreset,
    pub auto_rotate_chords: bool,
    pub progression_style: ProgressionStyle,
    pub is_transitioning: bool,
    pub mapping_mode: AudioMappingMode,
    pub target_amps: [f32; 5],
    pub current_amps: [f32; 5],
    pub velocity_rates: [f32; 5],
    pub voice_positions: [(f32, f32); 5],
    pub avg_velocity: f32,
    pub max_velocity: f32,
    pub vorticity: f32,
    pub flow_angle: f32,
    pub ink_area_ratio: f32,
    pub ink_contrast: f32,
    pub rotation_progress: f32,
    pub current_hold_duration: f32,
    pub current_transition_duration: f32,
}

impl Default for SharedAudioState {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 0.35,
            preset: ChordPreset::CMajor9,
            auto_rotate_chords: true,
            progression_style: ProgressionStyle::GenerativeRandom,
            is_transitioning: false,
            mapping_mode: AudioMappingMode::ColorMass,
            target_amps: [0.0; 5],
            current_amps: [0.0; 5],
            velocity_rates: [0.0; 5],
            voice_positions: [(0.5, 0.5); 5],
            avg_velocity: 0.0,
            max_velocity: 0.0,
            vorticity: 0.0,
            flow_angle: 0.0,
            ink_area_ratio: 0.0,
            ink_contrast: 0.0,
            rotation_progress: 0.0,
            current_hold_duration: 5.0,
            current_transition_duration: 1.0,
        }
    }
}
