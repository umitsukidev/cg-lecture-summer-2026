mod camera;
mod hit;
mod material;
mod nannou_utils;
mod ray;
mod sphere;

use crate::camera::Camera;
use crate::material::{Material, MaterialType};
use crate::nannou_utils::Point3Ext;
use crate::sphere::Sphere;
use nannou::image::{Rgba, RgbaImage};
use nannou::prelude::*;
use rayon::prelude::*;

struct Model {
    texture: Handle<Image>,
    image_buffer: RgbaImage,
    camera: Camera,
    environment: Material,
    sphere: Sphere,
    colors: Colors,
}

#[allow(dead_code)]
struct Colors {
    white: Material,
    red: Material,
    green: Material,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let _window = app.new_window().size(512, 512).view(view).build();

    let width = app.window_rect().w() as u32;
    let height = app.window_rect().h() as u32;

    let image_buffer = RgbaImage::new(width, height);

    // #region create scene
    let camera = Camera::new(pt3(0.0, -10.0, 2.0), pt3(0.0, 0.0, 0.0), 55.0);
    let environment = Material::new(Some(vec3(0.6, 0.7, 0.8)), None, None);

    let white = Material::new(None, Some(vec3(0.6, 0.6, 0.2)), Some(MaterialType::DIFFUSE));
    let red = Material::new(None, Some(vec3(0.8, 0.2, 0.2)), Some(MaterialType::DIFFUSE));
    let green = Material::new(None, Some(vec3(0.2, 0.8, 0.2)), Some(MaterialType::DIFFUSE));
    // #endregion

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
        sphere: Sphere {
            position: pt3(0.0, 0.0, 0.0),
            radius: 2.0,
            material: red.clone(),
        },
        colors: Colors { white, red, green },
    }
}

fn update(app: &App, model: &mut Model) {
    let width = model.image_buffer.width();
    let _height = model.image_buffer.height();

    let camera = &model.camera;
    let sphere = &model.sphere;
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

            let pixel = render(window_rect, x, y, camera, sphere, environment);

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

fn render(
    window_rect: Rect,
    x: u32,
    y: u32,
    camera: &Camera,
    sphere: &Sphere,
    environment: &Material,
) -> Rgba<u8> {
    let view = camera.ray(window_rect, UVec2::new(x, y), 0.5, 0.5);
    if let Some(hit) = sphere.intersect(view, 0.0001, 10000.0) {
        hit.material.color().to_color()
    } else {
        environment.emission.unwrap().to_color()
    }
}
