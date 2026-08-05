use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

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
            rotation_progress: 0.0,
            current_hold_duration: 5.0,
            current_transition_duration: 1.0,
        }
    }
}

pub struct AudioSynth {
    pub shared_state: Arc<Mutex<SharedAudioState>>,
}

impl AudioSynth {
    pub fn new() -> Self {
        let shared_state = Arc::new(Mutex::new(SharedAudioState::default()));
        let state_clone = shared_state.clone();

        std::thread::spawn(move || {
            if let Ok(_stream) = Self::setup_cpal_stream(state_clone) {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                }
            }
        });

        Self { shared_state }
    }

    fn setup_cpal_stream(state: Arc<Mutex<SharedAudioState>>) -> Result<cpal::Stream, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "No default audio output device found".to_string())?;

        let config = device
            .default_output_config()
            .map_err(|e| e.to_string())?;
        let sample_rate = config.sample_rate().0 as f32;
        let channels = config.channels() as usize;

        let mut phases = [0.0f32; 5];
        let mut start_freqs = ChordPreset::CMajor9.frequencies();
        let mut target_freqs = start_freqs;
        let mut current_freqs = start_freqs;
        let mut is_in_transition = false;
        let mut phase_elapsed = 0.0f32;
        let mut hold_duration = 5.0f32;
        let mut transition_duration = 1.0f32;
        let mut rng_seed = 987654321u32;

        let err_fn = |err| eprintln!("an error occurred on audio stream: {}", err);

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device
                .build_output_stream(
                    &config.into(),
                    move |data: &mut [f32], _| {
                        Self::write_audio_data(
                            data,
                            channels,
                            sample_rate,
                            &state,
                            &mut phases,
                            &mut start_freqs,
                            &mut target_freqs,
                            &mut current_freqs,
                            &mut is_in_transition,
                            &mut phase_elapsed,
                            &mut hold_duration,
                            &mut transition_duration,
                            &mut rng_seed,
                        );
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| e.to_string())?,
            _ => return Err("Unsupported audio sample format".to_string()),
        };

        stream.play().map_err(|e| e.to_string())?;
        Ok(stream)
    }

    #[allow(clippy::too_many_arguments)]
    fn write_audio_data(
        output: &mut [f32],
        channels: usize,
        sample_rate: f32,
        state: &Arc<Mutex<SharedAudioState>>,
        phases: &mut [f32; 5],
        start_freqs: &mut [f32; 5],
        target_freqs: &mut [f32; 5],
        current_freqs: &mut [f32; 5],
        is_in_transition: &mut bool,
        phase_elapsed: &mut f32,
        hold_duration: &mut f32,
        transition_duration: &mut f32,
        rng_seed: &mut u32,
    ) {
        let mut guard = match state.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        if !guard.enabled || guard.volume <= 0.001 {
            for sample in output.iter_mut() {
                *sample = 0.0;
            }
            return;
        }

        let num_frames = (output.len() / channels) as f32;
        let buffer_duration = num_frames / sample_rate;

        // Auto Chord Rotation with Generative Progression Graph (3~10s Hold, 4s/oct Transition)
        if guard.auto_rotate_chords {
            *phase_elapsed += buffer_duration;

            if *is_in_transition {
                // Transition Phase (Duration = Max Octave Shift * 4.0 sec/octave)
                let t_raw = (*phase_elapsed / *transition_duration).clamp(0.0, 1.0);
                // Differentiable Cosine Ease-In-Out: 0.5 * (1 - cos(PI * t))
                let t_eased = 0.5 * (1.0 - (t_raw * std::f32::consts::PI).cos());

                for i in 0..5 {
                    current_freqs[i] = start_freqs[i] + (target_freqs[i] - start_freqs[i]) * t_eased;
                }

                guard.rotation_progress = t_raw;
                guard.is_transitioning = true;
                guard.current_transition_duration = *transition_duration;

                if *phase_elapsed >= *transition_duration {
                    // Transition complete -> switch to Holding phase
                    *phase_elapsed = 0.0;
                    *is_in_transition = false;
                    *current_freqs = *target_freqs;

                    // Pick random hold duration between 3.0s and 10.0s
                    *rng_seed = rng_seed.wrapping_mul(1664525).wrapping_add(1013904223);
                    let norm_rand = (*rng_seed as f32) / (u32::MAX as f32);
                    *hold_duration = 3.0 + norm_rand * 7.0;
                }
            } else {
                // Holding Phase (Random 3.0s ~ 10.0s duration)
                *current_freqs = *target_freqs;
                guard.rotation_progress = (*phase_elapsed / *hold_duration).clamp(0.0, 1.0);
                guard.is_transitioning = false;

                if *phase_elapsed >= *hold_duration {
                    // Hold complete -> pick next chord based on music theory progression style
                    *phase_elapsed = 0.0;
                    *is_in_transition = true;
                    *start_freqs = *target_freqs;

                    let current_preset = guard.preset;
                    let style = guard.progression_style;

                    *rng_seed = rng_seed.wrapping_mul(1664525).wrapping_add(1013904223);
                    let norm_rand = (*rng_seed as f32) / (u32::MAX as f32);

                    let next_preset = style.next_chord(current_preset, norm_rand);
                    guard.preset = next_preset;
                    *target_freqs = next_preset.frequencies();

                    // Speed: 4.0 seconds per octave shift!
                    let octave_dist = (0..5)
                        .map(|i| (target_freqs[i] / start_freqs[i]).log2().abs())
                        .fold(0.0f32, f32::max);
                    *transition_duration = (octave_dist * 4.0).max(0.2);
                    guard.current_transition_duration = *transition_duration;
                }
            }

            guard.current_hold_duration = *hold_duration;
        } else {
            let manual_target = guard.preset.frequencies();
            for i in 0..5 {
                current_freqs[i] += (manual_target[i] - current_freqs[i]) * 0.005;
            }
            *phase_elapsed = 0.0;
            *is_in_transition = false;
            *start_freqs = *current_freqs;
            guard.rotation_progress = 0.0;
        }

        let master_vol = guard.volume;
        let vorticity = guard.vorticity.clamp(0.0, 5.0);
        let flow_angle = guard.flow_angle;

        // Smooth volume envelopes quickly to open on motion & silence when still
        let smoothing = 0.03;
        for i in 0..5 {
            let diff = guard.target_amps[i] - guard.current_amps[i];
            guard.current_amps[i] += diff * smoothing;
            if guard.current_amps[i] < 0.0001 {
                guard.current_amps[i] = 0.0;
            }
        }
        let current_amps = guard.current_amps;
        let vel_rates = guard.velocity_rates;
        let voice_positions = guard.voice_positions;

        for frame in output.chunks_mut(channels) {
            let mut sample_val = 0.0;

            for i in 0..5 {
                let amp = current_amps[i];
                if amp <= 0.0001 {
                    continue;
                }

                let vel_rate = vel_rates[i].clamp(0.0, 3.0);
                let base_freq = current_freqs[i];

                // 1. Directional pitch glide driven by velocity vector angle
                let dir_pitch_bend = (flow_angle + (i as f32) * 0.5).sin() * 0.03 * vel_rate;

                // 2. Swirl/Vorticity tremolo
                let swirl_lfo = (phases[i] * (2.0 + vorticity * 4.0)).sin() * (vorticity * 0.03).clamp(0.0, 0.15);

                let freq = base_freq * (1.0 + dir_pitch_bend + swirl_lfo);
                let phase_step = freq / sample_rate;

                phases[i] = (phases[i] + phase_step) % 1.0;
                let phase = phases[i];

                // 3. Motion-driven gentle pulsing multiplier
                let pulse_freq = 1.5 + vel_rate * 6.0;
                let motion_pulse = 0.88 + 0.12 * (phase * pulse_freq * std::f32::consts::TAU).sin();

                // 4. Position-Driven Timbre Variation (Screen X & Y)
                let (px, py) = voice_positions[i];
                let px = px.clamp(0.0, 1.0);
                let py = py.clamp(0.0, 1.0);

                // Fundamental pure sine wave
                let sine1 = (phase * std::f32::consts::TAU).sin();

                // Top (py -> 0): Sub-octave warmth & depth
                let sub_sine = ((phase * 0.5) * std::f32::consts::TAU).sin() * (0.35 * (1.0 - py));

                // Right (px -> 1): Silky high-octave sine shimmer
                let high_sine = ((phase * 2.0) % 1.0 * std::f32::consts::TAU).sin() * (0.15 * px);

                // Bottom (py -> 1): Warm 5th harmonic sine overtone
                let fifth_sine = ((phase * 1.5) % 1.0 * std::f32::consts::TAU).sin() * (0.12 * py);

                // Ultra-round, warm acoustic sine waveform shaping
                let raw_wave = sine1 * 0.75 + sub_sine + high_sine + fifth_sine;
                let wave = raw_wave * (1.0 - 0.08 * raw_wave * raw_wave);

                sample_val += wave * amp * motion_pulse * 0.22;
            }

            let sample_out = (sample_val * master_vol).tanh();
            for sample in frame.iter_mut() {
                *sample = sample_out;
            }
        }
    }

    pub fn update_metrics(&self, metrics: &crate::solver::FluidAudioMetrics) {
        if let Ok(mut state) = self.shared_state.lock() {
            state.avg_velocity = metrics.avg_velocity;
            state.max_velocity = metrics.max_velocity;
            state.vorticity = metrics.vorticity;
            state.flow_angle = metrics.flow_angle;

            let max_possible_mass = 50.0;
            let velocity_threshold = 0.012; // Deadzone threshold for fluid motion

            match state.mapping_mode {
                AudioMappingMode::Spatial => {
                    for i in 0..5 {
                        let vel = metrics.spatial_velocities[i];
                        state.velocity_rates[i] = vel;
                        state.voice_positions[i] = metrics.spatial_positions[i];

                        if vel > velocity_threshold && metrics.spatial_masses[i] > 1e-3 {
                            let norm_mass = (metrics.spatial_masses[i] / max_possible_mass).clamp(0.0, 1.0);
                            let motion_factor = ((vel - velocity_threshold) * 4.5).clamp(0.0, 1.0);
                            state.target_amps[i] = motion_factor * (0.4 + 0.6 * norm_mass.powf(0.5));
                        } else {
                            state.target_amps[i] = 0.0;
                        }
                    }
                }
                AudioMappingMode::ColorMass => {
                    for i in 0..5 {
                        let vel = metrics.color_velocities[i];
                        state.velocity_rates[i] = vel;
                        state.voice_positions[i] = metrics.color_positions[i];

                        if vel > velocity_threshold && metrics.color_masses[i] > 1e-3 {
                            let norm_mass = (metrics.color_masses[i] / (max_possible_mass * 0.3)).clamp(0.0, 1.0);
                            let motion_factor = ((vel - velocity_threshold) * 4.5).clamp(0.0, 1.0);
                            state.target_amps[i] = motion_factor * (0.4 + 0.6 * norm_mass.powf(0.5));
                        } else {
                            state.target_amps[i] = 0.0;
                        }
                    }
                }
            }
        }
    }
}
