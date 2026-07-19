mod metaball;

use nannou::image::{Rgba, RgbaImage};
use nannou::{prelude::*, rand};
use rayon::prelude::*;

use crate::metaball::Metaball;

struct Model {
    texture: Handle<Image>,
    metaballs: Vec<Metaball>,
    window: Entity,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let width = 1024;
    let height = 1024;
    let window = app
        .new_window()
        .size(width, height)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .build();

    let field_size = pt2(width as f32, height as f32);

    let metaballs = vec![
        Metaball::new(pt2(-50.0, 0.0), 100.0, field_size),
        Metaball::new(pt2(50.0, 0.0), 100.0, field_size),
    ];

    let image_buffer = RgbaImage::new(width, height);
    let dynamic_image = nannou::image::DynamicImage::ImageRgba8(image_buffer);
    let image = Image::from_dynamic(
        dynamic_image,
        true,
        bevy_asset::RenderAssetUsages::default(),
    );
    let texture = app.asset_server().add(image);

    Model {
        texture,
        metaballs,
        window,
    }
}

fn update(app: &App, model: &mut Model) {
    let w_h = app.window(model.window).size_pixels();
    let (width, height) = (w_h.x, w_h.y);

    let mut image_buffer = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 255]));

    image_buffer
        .as_flat_samples_mut()
        .samples
        .par_chunks_mut(4)
        .enumerate()
        .for_each(|(index, chunk)| {
            let x = (index as u32) % width;
            let y = (index as u32) / width;

            let px = x as f32 - width as f32 / 2.0;
            let py = height as f32 / 2.0 - y as f32;
            let pixel = pt2(px, py);

            let mut sum = 0.0;
            for metaball in &model.metaballs {
                sum += 50.0 * metaball.radius / (pixel - metaball.position).length();
            }

            let value = quantize_n_levels(sum as u8, 4);

            chunk[0] = value;
            chunk[1] = value;
            chunk[2] = value;
            chunk[3] = 255;
        });

    for metaball in &mut model.metaballs {
        metaball.update();
    }

    let dynamic_image = nannou::image::DynamicImage::ImageRgba8(image_buffer);
    let image = Image::from_dynamic(
        dynamic_image,
        true,
        bevy_asset::RenderAssetUsages::default(),
    );
    app.modify_image(&model.texture, move |img| {
        *img = image;
    });
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(BLACK);
    draw.rect().w_h(1024.0, 1024.0).texture(&model.texture);

    // for metaball in model.metaballs.iter() {
    //     draw.ellipse()
    //         .xy(metaball.position)
    //         .radius(metaball.radius)
    //         .color(BLACK);
    // }
}

fn mouse_pressed(app: &App, model: &mut Model, button: MouseButton) {
    let w_h = app.window(model.window).size_pixels();
    let (width, height) = (w_h.x, w_h.y);
    if button == MouseButton::Left {
        let field_size = pt2(width as f32, height as f32);
        model.metaballs.push(Metaball::new(
            app.mouse(),
            rand::random_range(10.0, 50.0),
            field_size,
        ));
    }
}

fn quantize_n_levels(value: u8, n: u8) -> u8 {
    if n < 2 {
        return 0;
    }
    if n == 255 {
        return value;
    }
    let step = 255.0 / (n - 1) as f32;
    let index = (value as f32 / step).round() as u8;
    (index as f32 * step).round() as u8
}
