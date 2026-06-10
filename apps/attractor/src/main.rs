mod ball;

use ball::Ball;
use nannou::prelude::*;
use rayon::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    _window: window::Id,
    mouse_left_pressed: bool,
    ball_lines: Vec<Vec<Ball>>,
}

const BALL_COUNT: isize = 1000;
const LINE_COUNT: isize = 500;
const TIME_SCALE: f32 = 0.5;

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .view(view)
        .mouse_pressed(mouse_pressed)
        .mouse_released(mouse_released)
        .build()
        .unwrap();
    let ball_lines = (-LINE_COUNT / 2..LINE_COUNT / 2)
        .map(|i| {
            (-BALL_COUNT / 2..BALL_COUNT / 2)
                .map(|j| Ball::new(pt2(j as f32, i as f32 * 5.0), 1.0))
                .collect()
        })
        .collect();
    Model {
        _window,
        mouse_left_pressed: false,
        ball_lines,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let mouse = app.mouse.position();
    let mouse_left_pressed = model.mouse_left_pressed;
    let dt = TIME_SCALE;

    model.ball_lines.par_iter_mut().for_each(|ball_line| {
        if mouse_left_pressed {
            let radius = 150.0;
            let strength = -3.0;
            let ramp = 1.0;
            for ball in ball_line.iter_mut() {
                let delta = mouse - ball.position;
                let d = delta.length();
                if d > 0.0 && d < radius {
                    let s = (d / radius).powf(1.0 / ramp);
                    let f = s * 9.0 * strength * (1.0 / (s + 1.0) + (s - 3.0) / 4.0) / d;
                    ball.velocity.y += delta.y * f * dt;
                }
            }
        }
        for ball in ball_line.iter_mut() {
            ball.update(dt);
        }
    });
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(WHITE);
    let total_lines = model.ball_lines.len();
    for (i, ball_line) in model.ball_lines.iter().enumerate() {
        let t = i as f32 / (total_lines - 1) as f32;
        let hue = 0.83 - t * 0.37;
        let color = hsv(hue, 1.0, 1.0);
        draw.polyline()
            .weight(1.0)
            .points(ball_line.iter().map(|b| b.position))
            .color(color);
    }
    draw.to_frame(app, &frame).unwrap();
}

fn mouse_pressed(_app: &App, model: &mut Model, button: MouseButton) {
    match button {
        MouseButton::Left => {
            model.mouse_left_pressed = true;
        }
        MouseButton::Middle => {}
        MouseButton::Right => {}
        _ => {}
    }
}

fn mouse_released(_app: &App, model: &mut Model, button: MouseButton) {
    match button {
        MouseButton::Left => {
            model.mouse_left_pressed = false;
        }
        MouseButton::Middle => {}
        MouseButton::Right => {}
        _ => {}
    }
}
