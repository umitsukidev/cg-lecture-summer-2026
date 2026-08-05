use crate::{
    cmykw::Cmykw,
    ink_cell::InkCell,
    nannou_utils::{ColorExt, Point2Ext},
};
use nannou::{image::Rgba, prelude::*};
use ndarray::{Array2, Zip, s};

pub const X_N: usize = 320;
pub const Y_N: usize = 240;
pub const H: f32 = 1.0 / (if X_N > Y_N { X_N } else { Y_N }) as f32;

#[derive(Debug, Clone, Copy, Default)]
pub struct FluidAudioMetrics {
    /// Ink mass per color channel: [C, M, Y, K, W]
    pub color_masses: [f32; 5],
    /// Ink mass in 5 spatial zones: [Top-Left, Top-Right, Center, Bottom-Left, Bottom-Right]
    pub spatial_masses: [f32; 5],
    /// Velocity magnitude per color channel
    pub color_velocities: [f32; 5],
    /// Velocity magnitude in 5 spatial zones
    pub spatial_velocities: [f32; 5],
    /// Integrated momentum (Vector Velocity * Color Mass) per color channel
    pub color_momentums: [f32; 5],
    /// Integrated momentum (Vector Velocity * Ink Density Mass) in 5 spatial zones
    pub spatial_momentums: [f32; 5],
    /// Centroid position (X, Y) normalized in [0.0, 1.0] for color channels
    pub color_positions: [(f32, f32); 5],
    /// Centroid position (X, Y) normalized in [0.0, 1.0] for spatial zones
    pub spatial_positions: [(f32, f32); 5],
    /// Average velocity magnitude of the fluid field
    pub avg_velocity: f32,
    /// Peak velocity vector magnitude
    pub max_velocity: f32,
    /// Average vorticity (swirl intensity)
    pub vorticity: f32,
    /// Dominant flow angle (radians)
    pub flow_angle: f32,
    /// Total ink amount in the system
    pub total_ink: f32,
}

#[derive(Debug, Clone)]
pub struct Solver {
    window_rect: Rect,
    dt: f32,
    pub max_pressure_iterations: u32,
    pub src_rad: f32,
    pub src_vel_amp: f32,
    pub src_ink_amp: f32,
    pub src_water_amp: f32,
    /// [0]: current, [1]: prev
    pub u: [Array2<f32>; 2],
    /// [0]: current, [1]: prev
    pub v: [Array2<f32>; 2],
    pub div: Array2<f32>,
    pub prs: Array2<f32>,
    /// [0]: current, [1]: prev
    pub ink: [Array2<InkCell>; 2],
    pub ink_color: Cmykw,
    pub velocity_dissipation: f32,
    mouse_pressed: bool,
    mouse_pos: Option<Point2>,
    prev_mouse_pos: Option<Point2>,
}

impl Solver {
    pub fn new(window_rect: Rect) -> Self {
        Self {
            window_rect,
            dt: 1.0 / 60.0,
            max_pressure_iterations: 50,
            src_rad: 8.0,
            src_vel_amp: 0.1,
            src_ink_amp: 0.22,
            src_water_amp: 0.0,
            u: std::array::from_fn(|_| Array2::zeros((X_N, Y_N))),
            v: std::array::from_fn(|_| Array2::zeros((X_N, Y_N))),
            div: Array2::zeros((X_N, Y_N)),
            prs: Array2::zeros((X_N, Y_N)),
            ink: std::array::from_fn(|_| Array2::from_elem((X_N, Y_N), InkCell::default())),
            ink_color: Cmykw::new(0.69, 0.46, 0.0, 0.0, 0.0),
            velocity_dissipation: 0.1,
            mouse_pressed: false,
            mouse_pos: None,
            prev_mouse_pos: None,
        }
    }

    pub fn update_solver(
        &mut self,
        mouse_pressed: bool,
        mouse_pos: Point2,
        prev_mouse_pos: Option<Point2>,
    ) {
        self.mouse_pressed = mouse_pressed;
        self.mouse_pos = Some(mouse_pos.to_screen_coords(self.window_rect));
        self.prev_mouse_pos = prev_mouse_pos.map(|it| it.to_screen_coords(self.window_rect));

        self.add_source_velocity();
        self.add_source_ink();
        self.projection_velocity();
        self.advection_velocity();
        self.advection_ink();
    }

    fn source_bounds(mx: f32, my: f32, radius: f32) -> Option<(usize, usize, usize, usize)> {
        if mx + radius < 1.0
            || mx - radius > (X_N - 1) as f32
            || my + radius < 1.0
            || my - radius > (Y_N - 1) as f32
        {
            return None;
        }

        let x_start = (mx - radius).floor().clamp(1.0, (X_N - 2) as f32) as usize;
        let x_end = ((mx + radius).ceil() + 1.0).clamp(1.0, (X_N - 1) as f32) as usize;
        let y_start = (my - radius).floor().clamp(1.0, (Y_N - 2) as f32) as usize;
        let y_end = ((my + radius).ceil() + 1.0).clamp(1.0, (Y_N - 1) as f32) as usize;

        (x_start < x_end && y_start < y_end).then_some((x_start, x_end, y_start, y_end))
    }

    fn add_source_velocity(&mut self) {
        let width = self.window_rect.w();
        let height = self.window_rect.h();

        if !self.mouse_pressed {
            return;
        }

        if let Some(mouse_pos) = self.mouse_pos {
            let mut mouse_vel = if let Some(prev_mouse_pos) = self.prev_mouse_pos {
                mouse_pos - prev_mouse_pos
            } else {
                vec2(0.0, 0.0)
            };

            mouse_vel *= self.src_vel_amp;

            let mx = mouse_pos.x * X_N as f32 / width;
            let my = mouse_pos.y * Y_N as f32 / height;
            let Some((x_start, x_end, y_start, y_end)) = Self::source_bounds(mx, my, self.src_rad)
            else {
                return;
            };

            let mut u_inner = self.u[0].slice_mut(s![x_start..x_end, y_start..y_end]);
            let mut v_inner = self.v[0].slice_mut(s![x_start..x_end, y_start..y_end]);

            Zip::indexed(&mut u_inner)
                .and(&mut v_inner)
                .for_each(|(i, j), u_val, v_val| {
                    let i = i + x_start;
                    let j = j + y_start;

                    let pct = 1.0
                        - pt2(i as f32 + 0.5, j as f32 + 0.5).distance(pt2(mx, my)) / self.src_rad;
                    let pct = f32::max(pct, 0.0);

                    let mut vel = mouse_vel * pct;

                    vel.x += *u_val;
                    vel.y += *v_val;

                    let vel = vel.clamp_length_max(5.0);

                    *u_val = vel.x;
                    *v_val = vel.y;
                });
        }
    }

    fn add_source_ink(&mut self) {
        if !self.mouse_pressed {
            return;
        }

        if let Some(mouse_pos) = self.mouse_pos {
            let width = self.window_rect.w();
            let height = self.window_rect.h();

            let mx = mouse_pos.x * X_N as f32 / width;
            let my = mouse_pos.y * Y_N as f32 / height;
            let ink_color = self.ink_color;
            let Some((x_start, x_end, y_start, y_end)) = Self::source_bounds(mx, my, self.src_rad)
            else {
                return;
            };

            let mut ink_inner = self.ink[0].slice_mut(s![x_start..x_end, y_start..y_end]);

            Zip::indexed(&mut ink_inner).for_each(|(i, j), ink_val| {
                let i = i + x_start;
                let j = j + y_start;

                let pct =
                    1.0 - pt2(i as f32 + 0.5, j as f32 + 0.5).distance(pt2(mx, my)) / self.src_rad;
                let ink_pct = f32::max(pct, 0.0) * self.src_ink_amp;
                let water_pct = f32::max(pct, 0.0) * self.src_water_amp;

                for (color_mass, color_channel) in
                    ink_val.color_mass.iter_mut().zip(ink_color.iter())
                {
                    *color_mass += ink_pct * color_channel;
                }

                ink_val.ink_amount += ink_pct;
                ink_val.water_amount += water_pct;
            });
        }
    }

    fn projection_velocity(&mut self) {
        #[allow(clippy::reversed_empty_ranges)]
        let mut div_inner = self.div.slice_mut(s![1..-1, 1..-1]);

        let u = &self.u[0];
        let v = &self.v[0];

        Zip::indexed(&mut div_inner).par_for_each(|(i, j), div_val| {
            let i = i + 1;
            let j = j + 1;

            let div_u =
                ((u[[i + 1, j]] - u[[i - 1, j]]) + (v[[i, j + 1]] - v[[i, j - 1]])) / (2.0 * H);

            *div_val = -(H.powi(2) / self.dt) * div_u;
        });

        let tolerance = 0.001;
        for _ in 0..self.max_pressure_iterations {
            let div = &self.div;

            let prs = &mut self.prs;

            let mut err = 0.0;

            for i in 1..X_N - 1 {
                for j in 1..Y_N - 1 {
                    let prev_prs = prs[[i, j]];

                    prs[[i, j]] = (prs[[i + 1, j]]
                        + prs[[i - 1, j]]
                        + prs[[i, j + 1]]
                        + prs[[i, j - 1]]
                        + div[[i, j]])
                        / 4.0;

                    err = f32::max((prs[[i, j]] - prev_prs).abs(), err);
                }
            }

            self.enforce_wall_pressure();

            if err < tolerance {
                break;
            }
        }

        #[allow(clippy::reversed_empty_ranges)]
        let mut u_inner = self.u[0].slice_mut(s![1..-1, 1..-1]);
        #[allow(clippy::reversed_empty_ranges)]
        let mut v_inner = self.v[0].slice_mut(s![1..-1, 1..-1]);

        let prs = &self.prs;

        Zip::indexed(&mut u_inner)
            .and(&mut v_inner)
            .par_for_each(|(i, j), u_val, v_val| {
                let i = i + 1;
                let j = j + 1;

                let grad_prs_x = (prs[[i + 1, j]] - prs[[i - 1, j]]) / (H * 2.0);
                let grad_prs_y = (prs[[i, j + 1]] - prs[[i, j - 1]]) / (H * 2.0);

                *u_val += -self.dt * grad_prs_x;
                *v_val += -self.dt * grad_prs_y;
            });
    }

    fn advection_velocity(&mut self) {
        self.u.swap(0, 1);
        self.v.swap(0, 1);

        let [u_curr, u_prev] = &mut self.u;
        let [v_curr, v_prev] = &mut self.v;

        #[allow(clippy::reversed_empty_ranges)]
        let mut u_inner = u_curr.slice_mut(s![1..-1, 1..-1]);
        #[allow(clippy::reversed_empty_ranges)]
        let mut v_inner = v_curr.slice_mut(s![1..-1, 1..-1]);

        Zip::indexed(&mut u_inner)
            .and(&mut v_inner)
            .par_for_each(|(i, j), u_val, v_val| {
                let i = i + 1;
                let j = j + 1;

                let px = ((i as f32) * H - u_prev[[i, j]] * self.dt) / H;
                let py = ((j as f32) * H - v_prev[[i, j]] * self.dt) / H;

                let (i0, j0) = (
                    (px.floor()).clamp(1.0, X_N as f32 - 2.0) as usize,
                    (py.floor()).clamp(1.0, Y_N as f32 - 2.0) as usize,
                );
                let (i1, j1) = (i0 + 1, j0 + 1);

                let s = px - i0 as f32;
                let t = py - j0 as f32;

                let u = (
                    (u_prev[[i0, j0]], u_prev[[i0, j1]]),
                    (u_prev[[i1, j0]], u_prev[[i1, j1]]),
                );
                let vx = Self::bilinear(s, t, u);

                let v = (
                    (v_prev[[i0, j0]], v_prev[[i0, j1]]),
                    (v_prev[[i1, j0]], v_prev[[i1, j1]]),
                );
                let vy = Self::bilinear(s, t, v);

                let decay = (-self.velocity_dissipation * self.dt).exp();

                *u_val = vx * decay;
                *v_val = vy * decay;
            });
    }

    fn advection_ink(&mut self) {
        self.ink.swap(0, 1);

        let [ink_curr, ink_prev] = &mut self.ink;

        #[allow(clippy::reversed_empty_ranges)]
        let mut ink_inner = ink_curr.slice_mut(s![1..-1, 1..-1]);

        Zip::indexed(&mut ink_inner).par_for_each(|(i, j), ink_val| {
            let i = i + 1;
            let j = j + 1;

            let px = ((i as f32) * H - self.u[0][[i, j]] * self.dt) / H;
            let py = ((j as f32) * H - self.v[0][[i, j]] * self.dt) / H;

            let (i0, j0) = (
                (px.floor()).clamp(1.0, X_N as f32 - 2.0) as usize,
                (py.floor()).clamp(1.0, Y_N as f32 - 2.0) as usize,
            );
            let (i1, j1) = (i0 + 1, j0 + 1);

            let s = px - i0 as f32;
            let t = py - j0 as f32;

            for (channel, ink_curr) in ink_val.color_mass.iter_mut().enumerate() {
                let ink = (
                    (
                        ink_prev[[i0, j0]].color_mass[channel],
                        ink_prev[[i0, j1]].color_mass[channel],
                    ),
                    (
                        ink_prev[[i1, j0]].color_mass[channel],
                        ink_prev[[i1, j1]].color_mass[channel],
                    ),
                );

                let ink = Self::bilinear(s, t, ink);

                *ink_curr = ink;
            }

            ink_val.ink_amount = {
                let ink_amount = (
                    (ink_prev[[i0, j0]].ink_amount, ink_prev[[i0, j1]].ink_amount),
                    (ink_prev[[i1, j0]].ink_amount, ink_prev[[i1, j1]].ink_amount),
                );

                Self::bilinear(s, t, ink_amount)
            };

            ink_val.water_amount = {
                let water_amount = (
                    (
                        ink_prev[[i0, j0]].water_amount,
                        ink_prev[[i0, j1]].water_amount,
                    ),
                    (
                        ink_prev[[i1, j0]].water_amount,
                        ink_prev[[i1, j1]].water_amount,
                    ),
                );

                Self::bilinear(s, t, water_amount)
            };
        });
    }

    fn bilinear(x: f32, y: f32, ((v00, v01), (v10, v11)): ((f32, f32), (f32, f32))) -> f32 {
        let x = x.clamp(0.0, 1.0);
        let y = y.clamp(0.0, 1.0);

        let x_a = 1.0 - x;
        let y_a = 1.0 - y;

        v00 * x_a * y_a + v01 * x_a * y + v10 * x * y_a + v11 * x * y
    }

    fn enforce_wall_pressure(&mut self) {
        let prs = &mut self.prs;
        for n in 0..X_N {
            prs[[n, 0]] = prs[[n, 1]];
            prs[[n, Y_N - 1]] = prs[[n, Y_N - 2]];
        }
        for m in 0..Y_N {
            prs[[0, m]] = prs[[1, m]];
            prs[[X_N - 1, m]] = prs[[X_N - 2, m]];
        }
        prs[[0, 0]] = (prs[[1, 0]] + prs[[0, 1]]) / 2.0;
        prs[[0, Y_N - 1]] = (prs[[1, Y_N - 1]] + prs[[0, Y_N - 2]]) / 2.0;
        prs[[X_N - 1, 0]] = (prs[[X_N - 2, 0]] + prs[[X_N - 1, 1]]) / 2.0;
        prs[[X_N - 1, Y_N - 1]] = (prs[[X_N - 2, Y_N - 1]] + prs[[X_N - 1, Y_N - 2]]) / 2.0;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Rgba<u8> {
        if x == 0 || y == 0 || x >= X_N - 1 || y >= Y_N - 1 {
            return Rgba([255, 255, 255, 0]);
        }

        let ink = self.ink[0][[x, y]];
        if ink.ink_amount <= 1e-6 {
            return Rgba([255, 255, 255, 0]);
        }

        let [c, m, y, k, w] = ink.color_mass.map(|mass| mass / ink.ink_amount);
        let [red, green, blue] = Color::cmykw(c, m, y, k, w)
            .to_srgba()
            .to_f32_array_no_alpha();

        let effective_amount = ink.ink_amount / (1.0 + ink.water_amount);
        let opacity = 1.0 - (-effective_amount * 0.5).exp();

        Rgba(
            Color::srgba(red, green, blue, opacity)
                .to_srgba()
                .to_u8_array(),
        )
    }

    pub fn analyze_fluid_state(&self) -> FluidAudioMetrics {
        let mut metrics = FluidAudioMetrics::default();
        let half_x = X_N / 2;
        let half_y = Y_N / 2;

        let ink_grid = &self.ink[0];
        let u_grid = &self.u[0];
        let v_grid = &self.v[0];

        let mut total_vel_sq = 0.0;
        let mut total_vorticity = 0.0;
        let mut sum_u = 0.0f32;
        let mut sum_v = 0.0f32;
        let sample_step = 2;

        for x in (1..X_N - 1).step_by(sample_step) {
            for y in (1..Y_N - 1).step_by(sample_step) {
                let cell = &ink_grid[[x, y]];
                let amt = cell.ink_amount;
                metrics.total_ink += amt;

                let u_val = u_grid[[x, y]];
                let v_val = v_grid[[x, y]];
                let vel_sq = u_val * u_val + v_val * v_val;
                let vel_mag = vel_sq.sqrt();
                total_vel_sq += vel_sq;
                metrics.max_velocity = f32::max(metrics.max_velocity, vel_mag);
                sum_u += u_val;
                sum_v += v_val;

                let norm_x = x as f32 / X_N as f32;
                let norm_y = y as f32 / Y_N as f32;

                // Vorticity approximation: |dv/dx - du/dy|
                let dv_dx = (v_grid[[x + 1, y]] - v_grid[[x - 1, y]]) / (2.0 * H);
                let du_dy = (u_grid[[x, y + 1]] - u_grid[[x, y - 1]]) / (2.0 * H);
                total_vorticity += (dv_dx - du_dy).abs();

                let dilution_factor = 1.0 + cell.water_amount;
                let effective_ink = amt / dilution_factor;

                for c in 0..5 {
                    let c_mass = cell.color_mass[c];
                    let c_conc = c_mass / dilution_factor; // True concentration taking water dilution into account!
                    let c_mom = c_conc * vel_mag;

                    metrics.color_masses[c] += c_conc;
                    metrics.color_momentums[c] += c_mom;
                    metrics.color_positions[c].0 += c_conc * norm_x;
                    metrics.color_positions[c].1 += c_conc * norm_y;
                }

                let zone_idx = if (x > X_N / 4 && x < X_N * 3 / 4) && (y > Y_N / 4 && y < Y_N * 3 / 4) {
                    2 // Center
                } else if x <= half_x && y <= half_y {
                    0 // Top-Left
                } else if x > half_x && y <= half_y {
                    1 // Top-Right
                } else if x <= half_x && y > half_y {
                    3 // Bottom-Left
                } else {
                    4 // Bottom-Right
                };

                let s_mom = effective_ink * vel_mag;
                metrics.spatial_masses[zone_idx] += effective_ink;
                metrics.spatial_momentums[zone_idx] += s_mom;
                metrics.spatial_positions[zone_idx].0 += effective_ink * norm_x;
                metrics.spatial_positions[zone_idx].1 += effective_ink * norm_y;
            }
        }

        let num_samples = ((X_N / sample_step) * (Y_N / sample_step)) as f32;
        metrics.avg_velocity = (total_vel_sq / num_samples).sqrt();
        metrics.vorticity = total_vorticity / num_samples;
        metrics.flow_angle = sum_v.atan2(sum_u);

        // Normalize color & spatial velocities and spatial centroid positions
        for i in 0..5 {
            if metrics.color_masses[i] > 1e-4 {
                metrics.color_velocities[i] = metrics.color_momentums[i] / metrics.color_masses[i];
                metrics.color_positions[i].0 /= metrics.color_masses[i];
                metrics.color_positions[i].1 /= metrics.color_masses[i];
            } else {
                metrics.color_positions[i] = (0.5, 0.5);
            }

            if metrics.spatial_masses[i] > 1e-4 {
                metrics.spatial_velocities[i] = metrics.spatial_momentums[i] / metrics.spatial_masses[i];
                metrics.spatial_positions[i].0 /= metrics.spatial_masses[i];
                metrics.spatial_positions[i].1 /= metrics.spatial_masses[i];
            } else {
                metrics.spatial_positions[i] = (0.5, 0.5);
            }
        }

        metrics
    }

    pub fn reset_simulation(&mut self) {
        for velocity in &mut self.u {
            velocity.fill(0.0);
        }
        for velocity in &mut self.v {
            velocity.fill(0.0);
        }

        self.div.fill(0.0);
        self.prs.fill(0.0);

        for ink in &mut self.ink {
            ink.fill(InkCell::default());
        }

        self.mouse_pressed = false;
        self.mouse_pos = None;
        self.prev_mouse_pos = None;
    }

    pub fn reset_all(&mut self) {
        *self = Self::new(self.window_rect);
    }
}
