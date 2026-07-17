use nannou::prelude::*;
use ndarray::{Array2, Zip, s};

pub const X_N: usize = 120;
pub const Y_N: usize = 90;
pub const H: f32 = 1.0 / (if X_N > Y_N { X_N } else { Y_N }) as f32;

#[derive(Debug, Clone)]
pub struct Solver {
    window_rect: Rect,
    dt: f32,
    pub max_gs_iterate: u32,
    pub src_rad: f32,
    pub src_vel_amp: f32,
    pub u: [Array2<f32>; 2],
    pub v: [Array2<f32>; 2],
    pub div: Array2<f32>,
    pub prs: Array2<f32>,
    pub ink: [Array2<f32>; 2],
    /// (current, prev)
    pub velocity_index: (usize, usize),
    /// (current, prev)
    pub ink_index: (usize, usize),
}

impl Solver {
    pub fn new(window_rect: Rect) -> Self {
        Self {
            window_rect,
            dt: 1.0 / 60.0,
            max_gs_iterate: 50,
            src_rad: 4.0,
            src_vel_amp: 0.1,
            u: std::array::from_fn(|_| Array2::zeros((X_N, Y_N))),
            v: std::array::from_fn(|_| Array2::zeros((X_N, Y_N))),
            div: Array2::zeros((X_N, Y_N)),
            prs: Array2::zeros((X_N, Y_N)),
            ink: std::array::from_fn(|_| Array2::zeros((X_N, Y_N))),
            velocity_index: (0, 1),
            ink_index: (0, 1),
        }
    }

    pub fn update_solver(
        &mut self,
        mouse_pressed: bool,
        mouse_pos: Point2,
        prev_mouse_pos: Option<Point2>,
    ) {
        self.add_source_velocity(mouse_pressed, mouse_pos, prev_mouse_pos);
        // self.add_source_ink();
        // self.projection_velocity();
        // self.advection_velocity();
        // self.advection_ink();
    }

    fn add_source_velocity(
        &mut self,
        mouse_pressed: bool,
        mouse_pos: Point2,
        prev_mouse_pos: Option<Point2>,
    ) {
        let width = self.window_rect.w();
        let height = self.window_rect.h();
        if !mouse_pressed {
            return;
        }
        let mut mouse_vel = if let Some(prev_mouse_pos) = prev_mouse_pos {
            mouse_pos - prev_mouse_pos
        } else {
            vec2(0.0, 0.0)
        };

        mouse_vel *= self.src_vel_amp;

        let mx = mouse_pos.x * X_N as f32 / width;
        let my = mouse_pos.y * Y_N as f32 / height;

        // 壁を取り除く
        let mut u_inner = self.u[self.velocity_index.0].slice_mut(s![1..-1, 1..-1]);
        let mut v_inner = self.v[self.velocity_index.0].slice_mut(s![1..-1, 1..-1]);

        Zip::indexed(&mut u_inner)
            .and(&mut v_inner)
            .par_for_each(|(i, j), u_val, v_val| {
                // 壁を取り除いたぶんのインデックスの調整
                let i = i + 1;
                let j = j + 1;

                let pct = 1.0
                    - (pt2(i as f32, j as f32) - pt2(mx as f32, my as f32)).length()
                        / self.src_rad as f32;
                let pct = 0.0.max(pct);

                let mut vel = mouse_vel * pct;

                vel.x += *u_val;
                vel.y += *v_val;

                // 速さ制限
                let vel = vel.clamp_length_max(5.0);

                *u_val = vel.x;
                *v_val = vel.y;
            });
    }

    fn add_source_ink() {
        todo!()
    }

    fn projection_velocity() {
        todo!()
    }

    fn advection_velocity() {
        todo!()
    }

    fn advection_ink() {
        todo!()
    }

    pub fn reset(&mut self) {
        todo!()
    }
}
