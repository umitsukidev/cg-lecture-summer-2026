use super::{AudioMappingMode, ChordPreset, SharedAudioState};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

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
        let mut lpf_states = [0.0f32; 5];
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
                            &mut lpf_states,
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
        lpf_states: &mut [f32; 5],
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

        // More visible ink means a shorter chord hold. The area term keeps the
        // mapping faithful to binary coverage, while contrast distinguishes a
        // faint wash from a dark mark occupying the same area.
        let visual_activity = (guard.ink_area_ratio * 0.65 + guard.ink_contrast * 0.35)
            .clamp(0.0, 1.0);
        let target_hold_duration = 10.0 - visual_activity * 7.0;

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

                    *hold_duration = target_hold_duration;
                }
            } else {
                // Holding Phase (3.0s ~ 10.0s, controlled by the rendered ink)
                *current_freqs = *target_freqs;

                // Adapt continuously, but over about half a second so a single
                // simulation frame does not cause an audible timing jump.
                let duration_alpha = (buffer_duration / 0.5).clamp(0.0, 1.0);
                *hold_duration +=
                    (target_hold_duration - *hold_duration) * duration_alpha;
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

                // 4. Pure Round Timbre & Dynamic 2D Screen-Coordinate EQ Filtering
                let (px, py) = voice_positions[i];
                let px = px.clamp(0.0, 1.0);
                let py = py.clamp(0.0, 1.0);

                // Fundamental pure, ultra-round sine wave
                let sine1 = (phase * std::f32::consts::TAU).sin();

                // Y-Axis (Vertical Position Top -> Bottom): EQ Sub-Bass Boost vs Mid-Clarity Boost
                // Top (py -> 0): Deep sub-octave bass boost (0.30 * (1 - py))
                let sub_boost = ((phase * 0.5) * std::f32::consts::TAU).sin() * (0.30 * (1.0 - py));
                // Bottom (py -> 1): Gentle mid-range clarity boost (0.12 * py)
                let mid_boost = ((phase * 1.5) % 1.0 * std::f32::consts::TAU).sin() * (0.12 * py);

                let raw_wave = sine1 * 0.75 + sub_boost + mid_boost;
                let wave_round = raw_wave * (1.0 - 0.08 * raw_wave * raw_wave);

                // X-Axis (Horizontal Position Left -> Right): Dynamic EQ Low-Pass Filter Cutoff Sweep
                // Left (px -> 0): Dark, muffled LPF cutoff (~600Hz, alpha ~ 0.08)
                // Right (px -> 1): Open, bright, clear LPF cutoff (~12000Hz, alpha ~ 0.92)
                let lpf_alpha = 0.08 + (px.powf(1.5)) * 0.84;
                lpf_states[i] += lpf_alpha * (wave_round - lpf_states[i]);
                let wave = lpf_states[i];

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
            state.ink_area_ratio = metrics.ink_area_ratio;
            state.ink_contrast = metrics.ink_contrast;

            match state.mapping_mode {
                AudioMappingMode::Spatial => {
                    for i in 0..5 {
                        let vel = metrics.spatial_velocities[i];
                        let mass = metrics.spatial_masses[i];
                        state.velocity_rates[i] = vel;
                        state.voice_positions[i] = metrics.spatial_positions[i];

                        // Normalized Velocity Vector (0.0 ~ 1.0)
                        let norm_vel = (vel / 0.4).clamp(0.0, 1.0);
                        // Normalized Effective Ink Concentration (0.0 ~ 1.0)
                        let norm_mass = (mass / 20.0).clamp(0.0, 1.0);

                        // Dynamic Volume = Vector Velocity * Ink Concentration Mass
                        state.target_amps[i] = norm_vel * norm_mass;
                    }
                }
                AudioMappingMode::ColorMass => {
                    for i in 0..5 {
                        let vel = metrics.color_velocities[i];
                        let mass = metrics.color_masses[i];
                        state.velocity_rates[i] = vel;
                        state.voice_positions[i] = metrics.color_positions[i];

                        // Normalized Velocity Vector (0.0 ~ 1.0)
                        let norm_vel = (vel / 0.4).clamp(0.0, 1.0);
                        // Normalized Effective Color Concentration (0.0 ~ 1.0)
                        let norm_mass = (mass / 10.0).clamp(0.0, 1.0);

                        // Dynamic Volume = Vector Velocity * Color Concentration Mass
                        state.target_amps[i] = norm_vel * norm_mass;
                    }
                }
            }
        }
    }
}
