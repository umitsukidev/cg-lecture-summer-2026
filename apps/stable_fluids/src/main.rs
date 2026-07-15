mod nannou_utils;
mod solver;
mod ui;

use crate::{
    nannou_utils::Point2Ext,
    solver::{Solver, X_N, Y_N},
    ui::{display_fps, display_vector},
};
use bevy::{
    camera::{Camera2d, RenderTarget},
    prelude::{
        AlignItems, BackgroundColor, Button, Camera, ClearColorConfig, JustifyContent, Node, Text,
        TextColor, TextFont, UiTargetCamera, Val,
    },
    window::WindowRef,
};
use nannou::{
    image::{Rgba, RgbaImage},
    prelude::*,
};
use rayon::prelude::*;

struct Model {
    _window: Entity,
    texture: Handle<Image>,
    image_buffer: RgbaImage,
    is_simulation_running: bool,
    show_display_grids: bool,
    show_display_velocity: bool,
    prev_mouse_pos: Option<Point2>,
    solver: Solver,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let window = app
        .new_window()
        .size(X_N as u32 * 10, Y_N as u32 * 10)
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

    app.command_scope(|mut commands| {
        let camera_entity = commands
            .spawn((
                Camera2d,
                Camera {
                    clear_color: ClearColorConfig::None,
                    order: 10,
                    ..default()
                },
                RenderTarget::Window(WindowRef::Entity(window)),
            ))
            .id();
        commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                UiTargetCamera(camera_entity),
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(150.0),
                            height: Val::Px(50.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(bevy::prelude::Color::srgb(0.2, 0.6, 1.0)),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("UI Test"),
                            TextFont {
                                font_size: bevy::text::FontSize::Px(20.0),
                                ..default()
                            },
                            TextColor(bevy::prelude::Color::WHITE),
                        ));
                    });
            });
    });
    Model {
        _window: window,
        texture,
        image_buffer,
        is_simulation_running: true,
        show_display_grids: false,
        show_display_velocity: true,
        prev_mouse_pos: None,
        solver,
    }
}

fn update(app: &App, model: &mut Model) {
    let window_rect = app.window_rect();
    let width = model.image_buffer.width();
    let _height = model.image_buffer.height();
    let mouse_pressed = app.mouse_buttons().pressed(MouseButton::Left);
    let mouse_pos = app.mouse();

    if model.is_simulation_running {
        model.solver.update_solver(
            mouse_pressed,
            mouse_pos.to_screen_coords(window_rect),
            model
                .prev_mouse_pos
                .map(|pos| pos.to_screen_coords(window_rect)),
        );
    }

    model
        .image_buffer
        .as_flat_samples_mut()
        .samples
        .par_chunks_mut(4)
        .enumerate()
        .for_each(|(index, chunk)| {
            let _x = (index as u32) % width;
            let _y = (index as u32) / width;

            let pixel = Rgba::from([0, 0, 0, 0]);

            chunk[0] = pixel[0];
            chunk[1] = pixel[1];
            chunk[2] = pixel[2];
            chunk[3] = pixel[3];
        });

    let pixels = model.image_buffer.as_raw().clone();
    app.modify_image(&model.texture, move |image| {
        image.data = Some(pixels);
    });

    model.prev_mouse_pos = if mouse_pressed { Some(mouse_pos) } else { None };
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();

    draw.background().color(BLACK);

    let solver = &model.solver;

    let window_rect = app.window_rect();

    draw.rect().wh(window_rect.wh()).texture(&model.texture);

    if model.show_display_velocity {
        display_vector(
            &draw,
            window_rect,
            solver.u[solver.velocity_index.0].view(),
            solver.v[solver.velocity_index.0].view(),
        );
    }
    display_fps(&draw, app, window_rect);
}
