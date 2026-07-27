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
            src_ink_amp: 0.1,
            src_water_amp: 0.0,
            u: std::array::from_fn(|_| Array2::zeros((X_N, Y_N))),
            v: std::array::from_fn(|_| Array2::zeros((X_N, Y_N))),
            div: Array2::zeros((X_N, Y_N)),
            prs: Array2::zeros((X_N, Y_N)),
            ink: std::array::from_fn(|_| Array2::from_elem((X_N, Y_N), InkCell::default())),
            ink_color: Cmykw::new(0.8, 0.2, 0.2, 0.5, 0.0),
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

                    // 0.5を足してグリッドの中心に補正
                    let pct = 1.0
                        - pt2(i as f32 + 0.5, j as f32 + 0.5).distance(pt2(mx, my)) / self.src_rad;
                    let pct = f32::max(pct, 0.0);

                    let mut vel = mouse_vel * pct;

                    vel.x += *u_val;
                    vel.y += *v_val;

                    // 速さ制限
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
            let ink_density = self.ink_color.to_optical_density();
            let Some((x_start, x_end, y_start, y_end)) = Self::source_bounds(mx, my, self.src_rad)
            else {
                return;
            };

            let mut ink_inner = self.ink[0].slice_mut(s![x_start..x_end, y_start..y_end]);

            Zip::indexed(&mut ink_inner).for_each(|(i, j), ink_val| {
                let i = i + x_start;
                let j = j + y_start;

                // 0.5を足してグリッドの中心に補正
                let pct =
                    1.0 - pt2(i as f32 + 0.5, j as f32 + 0.5).distance(pt2(mx, my)) / self.src_rad;
                let ink_pct = f32::max(pct, 0.0) * self.src_ink_amp;
                let water_pct = f32::max(pct, 0.0) * self.src_water_amp;

                for (ink_channel, density) in ink_val.color_mass.iter_mut().zip(ink_density.iter())
                {
                    *ink_channel += ink_pct * density;
                }

                ink_val.ink_amount += ink_pct;
                ink_val.water_amount += water_pct;
            });
        }
    }

    fn projection_velocity(&mut self) {
        // ---------------------------
        // 1. 反復計算前の事前計算
        // ---------------------------

        // 壁を取り除く
        #[allow(clippy::reversed_empty_ranges)]
        let mut div_inner = self.div.slice_mut(s![1..-1, 1..-1]);

        let u = &self.u[0];
        let v = &self.v[0];

        Zip::indexed(&mut div_inner).par_for_each(|(i, j), div_val| {
            // 壁を取り除いたぶんのインデックスの修正
            let i = i + 1;
            let j = j + 1;

            let div_u =
                ((u[[i + 1, j]] - u[[i - 1, j]]) + (v[[i, j + 1]] - v[[i, j - 1]])) / (2.0 * H);

            *div_val = -(H.powi(2) / self.dt) * div_u;
        });

        // ---------------------------
        // 2. ガウス=ザイデル反復法の計算
        // ---------------------------
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

            // 収束判定
            if err < tolerance {
                break;
            }
        }

        // ---------------------------
        // 3. 圧力勾配を求めて速度を更新
        // ---------------------------
        #[allow(clippy::reversed_empty_ranges)]
        let mut u_inner = self.u[0].slice_mut(s![1..-1, 1..-1]);
        #[allow(clippy::reversed_empty_ranges)]
        let mut v_inner = self.v[0].slice_mut(s![1..-1, 1..-1]);

        let prs = &self.prs;

        Zip::indexed(&mut u_inner)
            .and(&mut v_inner)
            .par_for_each(|(i, j), u_val, v_val| {
                // 壁を取り除いたぶんのインデックスの調整
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

        // 壁を取り除く
        #[allow(clippy::reversed_empty_ranges)]
        let mut u_inner = u_curr.slice_mut(s![1..-1, 1..-1]);
        #[allow(clippy::reversed_empty_ranges)]
        let mut v_inner = v_curr.slice_mut(s![1..-1, 1..-1]);

        Zip::indexed(&mut u_inner)
            .and(&mut v_inner)
            .par_for_each(|(i, j), u_val, v_val| {
                // 壁を取り除いたぶんのインデックスの調整
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

                *u_val = vx;
                *v_val = vy;
            })
    }

    fn advection_ink(&mut self) {
        self.ink.swap(0, 1);

        let [ink_curr, ink_prev] = &mut self.ink;

        // 壁を取り除く
        #[allow(clippy::reversed_empty_ranges)]
        let mut ink_inner = ink_curr.slice_mut(s![1..-1, 1..-1]);

        Zip::indexed(&mut ink_inner).par_for_each(|(i, j), ink_val| {
            // 壁を取り除いたぶんのインデックスの調整
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
        })
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
            return Rgba([255, 255, 255, 255]);
        }

        let ink = self.ink[0][[x, y]];
        if ink.ink_amount <= 1e-6 {
            return Rgba([255, 255, 255, 255]);
        }

        let concentration = ink.ink_amount / (1.0 + ink.water_amount);

        let [c, m, y, k, w] = ink.color_mass.map(|mass| {
            let mixed_density = mass / ink.ink_amount;
            let effective_density = mixed_density * concentration;
            1.0 - (-effective_density).exp()
        });

        Rgba(Color::cmykw(c, m, y, k, w).to_srgba().to_u8_array())
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.window_rect);
    }
}
