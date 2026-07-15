use crate::{
    nannou_utils::Point2Ext,
    solver::{H, X_N, Y_N},
};
use nannou::prelude::*;
use ndarray::ArrayView2;

pub fn display_vector(draw: &Draw, window_rect: Rect, u: ArrayView2<f32>, v: ArrayView2<f32>) {
    let width = window_rect.w();
    let height = window_rect.h();

    let l = width * H * 0.5;
    for i in 0..X_N {
        for j in 0..Y_N {
            let from = pt2(
                (i as f32 + 0.5) * width / X_N as f32,
                (j as f32 + 0.5) * height / Y_N as f32,
            );
            let v = vec2(u[[i, j]], v[[i, j]]) * l * 4.1;
            let to = from + v;

            if v.length() > 0.0 {
                draw.line()
                    .start(from.to_mathematical_coords(window_rect))
                    .end(to.to_mathematical_coords(window_rect))
                    .color(Color::srgb_u8(255, 200, 0));
            }
        }
    }
}

pub fn display_grids(draw: &Draw, window_rect: Rect) {
    for i in 1..X_N {
        let px = i as f32 / X_N as f32 * window_rect.w();
        draw.line()
            .start(pt2(px, 0.0).to_mathematical_coords(window_rect))
            .end(pt2(px, window_rect.h()).to_mathematical_coords(window_rect))
            .color(Color::srgb_u8(255, 255, 255));
    }

    for j in 1..Y_N {
        let py = j as f32 / Y_N as f32 * window_rect.h();
        draw.line()
            .start(pt2(0.0, py).to_mathematical_coords(window_rect))
            .end(pt2(window_rect.w(), py).to_mathematical_coords(window_rect))
            .color(Color::srgb_u8(255, 200, 0));
    }
}

pub fn display_fps(draw: &Draw, app: &App, window_rect: Rect) {
    let fps = app.fps();

    let text_wh = vec2(80.0, -20.0);
    draw.rect()
        .xy(window_rect.top_left() + text_wh / 2.0)
        .wh(text_wh)
        .color(BLACK);
    draw.text(&format!("fps: {:.0}", fps))
        .xy(window_rect.top_left() + text_wh / 2.0)
        .wh(text_wh)
        .center_justify()
        .align_text_middle_y()
        .color(Color::srgb_u8(255, 200, 0));
}
