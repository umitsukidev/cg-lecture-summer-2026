mod nannou_utils;
mod solver;
mod ui;

use crate::{
    solver::{Solver, X_N, Y_N},
    ui::{display_grids, display_gui, display_vector},
};
use nannou::{image::RgbaImage, prelude::*};
use rayon::prelude::*;
use std::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

pub struct Model {
    _window: Entity,
    texture: Handle<Image>,
    is_simulation_running: bool,
    show_display_grids: bool,
    show_display_velocity: bool,
    prev_mouse_pos: Option<Point2>,
    solver: Solver,
    displayed_fps: f32,
    last_fps_update: std::time::Instant,
    pixel_rx: Mutex<Receiver<Vec<u8>>>,
    pixel_tx: Sender<Vec<u8>>,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let window = app
        .new_window()
        .size(X_N as u32 * 10, Y_N as u32 * 10)
        .key_pressed(key_pressed)
        .view(view)
        .build();
    app.set_update_rate(60.0);

    let window_rect = app.window_rect();

    let width = window_rect.w() as u32;
    let height = window_rect.h() as u32;

    let image_buffer = RgbaImage::new(width, height);

    let dynamic_image = nannou::image::DynamicImage::ImageRgba8(image_buffer.clone());
    let image = Image::from_dynamic(
        dynamic_image,
        true,
        bevy_asset::RenderAssetUsages::default(),
    );
    let texture = app.asset_server().add(image);

    let solver = Solver::new(window_rect);

    let (pixel_tx, pixel_rx) = std::sync::mpsc::channel();

    Model {
        _window: window,
        texture,
        is_simulation_running: true,
        show_display_grids: false,
        show_display_velocity: false,
        prev_mouse_pos: None,
        solver,
        displayed_fps: 0.0,
        last_fps_update: std::time::Instant::now(),
        pixel_rx: Mutex::new(pixel_rx),
        pixel_tx,
    }
}

fn update(app: &App, model: &mut Model) {
    let window_rect = app.window_rect();
    let width = window_rect.w() as u32;
    let height = window_rect.h() as u32;
    let mouse_pressed =
        app.mouse_buttons().pressed(MouseButton::Left) && !app.egui().egui_wants_pointer_input();
    let mouse_pos = app.mouse();

    if model.is_simulation_running {
        model
            .solver
            .update_solver(mouse_pressed, mouse_pos, model.prev_mouse_pos);
    }

    let raw_pixels = model
        .pixel_rx
        .lock()
        .unwrap()
        .try_recv()
        .unwrap_or_else(|_| vec![0; (width * height * 4) as usize]);

    let mut image_buffer = RgbaImage::from_raw(width, height, raw_pixels).unwrap();

    image_buffer
        .as_flat_samples_mut()
        .samples
        .par_chunks_mut(4)
        .enumerate()
        .for_each(|(index, chunk)| {
            let x = index % width as usize;
            let y = index / width as usize;

            let pixel = model.solver.get_pixel(x, y);

            chunk[0] = pixel[0];
            chunk[1] = pixel[1];
            chunk[2] = pixel[2];
            chunk[3] = pixel[3];
        });

    let pixels = image_buffer.into_raw();
    let tx = model.pixel_tx.clone();
    app.modify_image(&model.texture, move |image| {
        if let Some(old_pixels) = image.data.take() {
            let _ = tx.send(old_pixels);
        }
        image.data = Some(pixels);
    });

    model.prev_mouse_pos = if mouse_pressed { Some(mouse_pos) } else { None };

    if model.last_fps_update.elapsed() >= std::time::Duration::from_millis(500) {
        model.displayed_fps = app.fps() as f32;
        model.last_fps_update = std::time::Instant::now();
    }

    display_gui(app, model);
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();

    draw.background().color(BLACK);

    let solver = &model.solver;

    let window_rect = app.window_rect();

    draw.rect().wh(window_rect.wh()).texture(&model.texture);

    if model.show_display_grids {
        display_grids(&draw, window_rect);
    }

    if model.show_display_velocity {
        display_vector(
            &draw,
            window_rect,
            solver.u[solver.velocity_index.0].view(),
            solver.v[solver.velocity_index.0].view(),
        );
    }
}

fn key_pressed(app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::Escape => app.quit(),
        KeyCode::KeyR => model.solver.reset(),
        _ => {}
    }
}
