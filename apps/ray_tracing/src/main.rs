mod camera;
mod hit;
mod material;
mod nannou_utils;
mod ray;
mod sphere;
mod utils;

use crate::{
    camera::Camera,
    material::Material,
    sphere::Sphere,
    utils::{create_scene, render},
};
use nannou::{image::RgbaImage, prelude::*};
use rayon::prelude::*;

struct Model {
    texture: Handle<Image>,
    image_buffer: RgbaImage,
    camera: Camera,
    environment: Material,
    spheres: Vec<Sphere>,
}

static RAY_COMPUTE_LIMIT: u64 = 1000000;

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let _window = app.new_window().size(512, 512).view(view).build();

    let width = app.window_rect().w() as u32;
    let height = app.window_rect().h() as u32;

    let image_buffer = RgbaImage::new(width, height);

    let (camera, environment, spheres) = create_scene();

    let dynamic_image = nannou::image::DynamicImage::ImageRgba8(image_buffer.clone());
    let image = Image::from_dynamic(
        dynamic_image,
        true,
        bevy_asset::RenderAssetUsages::default(),
    );
    let texture = app.asset_server().add(image);

    Model {
        texture,
        image_buffer,
        camera,
        environment,
        spheres,
    }
}

fn update(app: &App, model: &mut Model) {
    if app.elapsed_frames() > RAY_COMPUTE_LIMIT {
        return;
    }
    let width = model.image_buffer.width();
    let _height = model.image_buffer.height();

    let camera = &model.camera;
    let spheres = &model.spheres;
    let environment = &model.environment;
    let window_rect = app.window_rect();

    model
        .image_buffer
        .as_flat_samples_mut()
        .samples
        .par_chunks_mut(4)
        .enumerate()
        .for_each(|(index, chunk)| {
            let x = (index as u32) % width;
            let y = (index as u32) / width;

            let pixel = render(window_rect, x, y, camera, spheres, environment);

            chunk[0] = pixel[0];
            chunk[1] = pixel[1];
            chunk[2] = pixel[2];
            chunk[3] = pixel[3];
        });

    let pixels = model.image_buffer.as_raw().clone();
    app.modify_image(&model.texture, move |image| {
        image.data = Some(pixels);
    });
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.rect().w_h(512.0, 512.0).texture(&model.texture);
}
