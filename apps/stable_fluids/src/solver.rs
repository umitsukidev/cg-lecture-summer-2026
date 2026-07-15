use nannou::prelude::*;

pub const X_N: usize = 120;
pub const Y_N: usize = 90;
pub const H: f32 = 1.0 / (if X_N > Y_N { X_N } else { Y_N }) as f32;

#[derive(Debug, Clone)]
pub struct Solver {
    window_rect: Rect,
    dt: f32,
    max_gs_iterate: u32,
    src_rad: u32,
    src_vel_amp: f32,
    pub u: [[[f32; X_N]; Y_N]; 2],
    pub v: [[[f32; X_N]; Y_N]; 2],
    div: [[f32; X_N]; Y_N],
    prs: [[f32; X_N]; Y_N],
    pub ink: [[[f32; X_N]; Y_N]; 2],
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
            src_rad: 4,
            src_vel_amp: 0.1,
            u: [[[0.0; X_N]; Y_N]; 2],
            v: [[[0.0; X_N]; Y_N]; 2],
            div: [[0.0; X_N]; Y_N],
            prs: [[0.0; X_N]; Y_N],
            ink: [[[0.0; X_N]; Y_N]; 2],
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

        for i in 1..X_N - 1 {
            for j in 1..Y_N - 1 {
                let pct = 1.0
                    - (pt2(i as f32, j as f32) - pt2(mx as f32, my as f32)).length()
                        / self.src_rad as f32;
                let pct = 0.0.max(pct);

                let mut vel = mouse_vel * pct;

                vel.x += self.u[self.velocity_index.0][j][i];
                vel.y += self.v[self.velocity_index.0][j][i];

                // 速さ制限
                let vel = vel.clamp_length_max(5.0);

                self.u[self.velocity_index.0][j][i] = vel.x;
                self.v[self.velocity_index.0][j][i] = vel.y;
            }
        }
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
}
