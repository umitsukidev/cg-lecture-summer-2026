mod audio;
mod cmyk;
mod cmykw;
mod ink_cell;
mod nannou_utils;
mod solver;
mod ui;

use crate::{
    audio::AudioSynth,
    cmykw::Cmykw,
    solver::{Solver, X_N, Y_N},
    ui::{display_grids, display_gui, display_vector, update_vector_mesh},
};
use nannou::{image::RgbaImage, prelude::*};
use rayon::prelude::*;
use std::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};
use web_time::{Duration, Instant};

pub struct Model {
    _window: Entity,
    texture: Handle<Image>,
    is_simulation_running: bool,
    display_grids: bool,
    display_velocity: bool,
    show_gui: bool,
    set_color_by_cmykw: bool,
    pub color_swatches: Vec<Cmykw>,
    prev_mouse_pos: Option<Point2>,
    solver: Solver,
    audio: AudioSynth,
    displayed_fps: f32,
    last_fps_update: Instant,
    pixel_rx: Mutex<Receiver<Vec<u8>>>,
    pixel_tx: Sender<Vec<u8>>,
    vector_mesh: Mutex<Vec<geom::Tri<(Point3, Color)>>>,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let window = app
        .new_window()
        .primary()
        .size(1200, 900)
        .resizable(false)
        .key_pressed(key_pressed)
        .view(view)
        .build();
    app.set_update_rate(60.0);

    let window_rect = app.window_rect();
    let image_buffer = RgbaImage::new(X_N as u32, Y_N as u32);

    let dynamic_image = nannou::image::DynamicImage::ImageRgba8(image_buffer.clone());
    let mut image = Image::from_dynamic(
        dynamic_image,
        true,
        bevy_asset::RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::linear();
    let texture = app.asset_server().add(image);

    let solver = Solver::new(window_rect);
    let audio = AudioSynth::new();

    let (pixel_tx, pixel_rx) = std::sync::mpsc::channel();

    let zero_pt = pt3(0.0, 0.0, 0.0);
    let zero_color = Color::srgb_u8(0, 0, 0);
    let zero_tri = geom::Tri([
        (zero_pt, zero_color),
        (zero_pt, zero_color),
        (zero_pt, zero_color),
    ]);
    let mut initial_mesh = Vec::with_capacity(X_N * Y_N);
    initial_mesh.resize(X_N * Y_N, zero_tri);
    let vector_mesh = Mutex::new(initial_mesh);

    let default_swatches = vec![
        Cmykw::new(1.0, 0.0, 0.0, 0.0, 0.0), // Pure Cyan 100%
        Cmykw::new(0.0, 1.0, 0.0, 0.0, 0.0), // Pure Magenta 100%
        Cmykw::new(0.0, 0.0, 1.0, 0.0, 0.0), // Pure Yellow 100%
        Cmykw::new(0.0, 0.0, 0.0, 1.0, 0.0), // Pure Black 100%
        Cmykw::new(0.0, 0.0, 0.0, 0.0, 1.0), // Pure White 100%
        Cmykw::new(0.7, 0.0, 0.0, 0.0, 0.3), // Sky Blue (Cyan 70% + White 30%)
        Cmykw::new(0.0, 0.7, 0.0, 0.0, 0.3), // Rose Pink (Magenta 70% + White 30%)
        Cmykw::new(0.0, 0.0, 0.7, 0.0, 0.3), // Sun Gold (Yellow 70% + White 30%)
        Cmykw::new(0.4, 0.4, 0.0, 0.0, 0.2), // Lavender Violet (Cyan 40% + Magenta 40% + White 20%)
        Cmykw::new(0.5, 0.0, 0.3, 0.0, 0.2), // Turquoise Teal (Cyan 50% + Yellow 30% + White 20%)
        Cmykw::new(0.7, 0.5, 0.0, 0.3, 0.1), // Deep Navy (Cyan 70% + Magenta 50% + Black 30% + White 10%)
    ];

    Model {
        _window: window,
        texture,
        is_simulation_running: true,
        display_grids: true,
        display_velocity: false,
        show_gui: true,
        set_color_by_cmykw: true,
        color_swatches: default_swatches,
        prev_mouse_pos: None,
        solver,
        audio,
        displayed_fps: 0.0,
        last_fps_update: Instant::now(),
        pixel_rx: Mutex::new(pixel_rx),
        pixel_tx,
        vector_mesh,
    }
}

fn update(app: &App, model: &mut Model) {
    let window_rect = app.window_rect();
    let width = X_N as u32;
    let height = Y_N as u32;
    let mouse_pressed =
        app.mouse_buttons().pressed(MouseButton::Left) && !app.egui().egui_wants_pointer_input();
    let mouse_pos = app.mouse();

    if model.is_simulation_running {
        model
            .solver
            .update_solver(mouse_pressed, mouse_pos, model.prev_mouse_pos);

        // Analyze fluid state and update audio synth metrics
        let metrics = model.solver.analyze_fluid_state();
        model.audio.update_metrics(&metrics);

        let raw_pixels = {
            let mut pixels = model
                .pixel_rx
                .lock()
                .unwrap()
                .try_recv()
                .unwrap_or_else(|_| vec![0; (width * height * 4) as usize]);
            let expected_size = (width * height * 4) as usize;
            if pixels.len() != expected_size {
                pixels.resize(expected_size, 0);
            }
            pixels
        };

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
    }

    model.prev_mouse_pos = if mouse_pressed { Some(mouse_pos) } else { None };

    if model.last_fps_update.elapsed() >= Duration::from_millis(500) {
        model.displayed_fps = app.fps() as f32;
        model.last_fps_update = Instant::now();
    }

    if model.display_velocity {
        let mut mesh_guard = model.vector_mesh.lock().unwrap();
        if mesh_guard.is_empty() {
            let zero_pt = pt3(0.0, 0.0, 0.0);
            let zero_color = Color::srgb_u8(0, 0, 0);
            let zero_tri = geom::Tri([
                (zero_pt, zero_color),
                (zero_pt, zero_color),
                (zero_pt, zero_color),
            ]);
            mesh_guard.resize(X_N * Y_N, zero_tri);
        }
        let u = model.solver.u[0].view();
        let v = model.solver.v[0].view();
        update_vector_mesh(&mut mesh_guard, u, v, window_rect);
    }

    if model.show_gui {
        display_gui(app, model);
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();

    draw.background().color(WHITE);

    let window_rect = app.window_rect();

    if model.display_grids {
        display_grids(&draw, window_rect);
    }

    draw.rect().wh(window_rect.wh()).texture(&model.texture);

    if model.display_velocity {
        display_vector(&draw, &model.vector_mesh);
    }
}

fn key_pressed(app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::Escape =>
        {
            #[cfg(not(target_arch = "wasm32"))]
            app.quit()
        }
        KeyCode::KeyQ =>
        {
            #[cfg(not(target_arch = "wasm32"))]
            app.quit()
        }
        KeyCode::KeyR => model.solver.reset_simulation(),
        KeyCode::Space => model.is_simulation_running = !model.is_simulation_running,
        KeyCode::F3 => model.show_gui = !model.show_gui,
        _ => {}
    }
}
