use nannou::prelude::*;
mod ball;
use ball::Ball;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    _window: window::Id,
    mouse_left_pressed: bool,
    balls: Vec<Ball>,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .view(view)
        .key_pressed(key_pressed)
        .mouse_pressed(mouse_pressed)
        .mouse_released(mouse_released)
        .build()
        .unwrap();
    let balls = (-10..10)
        .map(|i| Ball::new(pt2(i as f32 * 30.0, 0.0)))
        .collect();
    Model {
        _window,
        mouse_left_pressed: false,
        balls,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    if model.mouse_left_pressed {
        for ball in &mut model.balls {
            let mouse = app.mouse.position();
            let velocity = ball.position - mouse;
            ball.velocity += velocity.normalize() * 0.1;
        }
    }
    for ball in &mut model.balls {
        ball.update();
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    for ball in &model.balls {
        draw.ellipse()
            .xy(ball.position)
            .radius(ball.radius)
            .color(BLACK);
    }
    draw.background().color(WHITE);
    draw.to_frame(app, &frame).unwrap();
}

fn key_pressed(_app: &App, _model: &mut Model, _key: Key) {}

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
