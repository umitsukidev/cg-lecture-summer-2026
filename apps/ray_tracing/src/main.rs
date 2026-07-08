mod camera;
mod hit;
mod material;
mod nannou_utils;
mod ray;
mod sphere;

use crate::camera::Camera;
use crate::hit::Hit;
use crate::material::{Material, MaterialType};
use crate::nannou_utils::Point3Ext;
use crate::ray::Ray;
use crate::sphere::Sphere;
use nannou::{
    image::{Rgba, RgbaImage},
    prelude::*,
};
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

fn render(
    window_rect: Rect,
    x: u32,
    y: u32,
    camera: &Camera,
    spheres: &Vec<Sphere>,
    environment: &Material,
) -> Rgba<u8> {
    let view = camera.ray(window_rect, UVec2::new(x, y), random_f32(), random_f32());
    if let Some(hit) = find_nearest_intersection(spheres, &view, 0.001, f32::MAX) {
        // hit.material.to_color()
        hit.normal.to_color()
    } else {
        environment.emission.unwrap().to_color()
    }
}

// fn create_scene() -> (Camera, Material, Sphere) {
//     let camera = Camera::new(pt3(0.0, -20.0, 2.0), pt3(0.0, 0.0, 0.0), 55.0);
//     let environment = Material::new(Some(vec3(0.6, 0.7, 0.8)), None, None);
//
//     let white = Material::new(None, Some(vec3(0.6, 0.6, 0.2)), Some(MaterialType::DIFFUSE));
//     let red = Material::new(None, Some(vec3(0.8, 0.2, 0.2)), Some(MaterialType::DIFFUSE));
//     let green = Material::new(None, Some(vec3(0.2, 0.8, 0.2)), Some(MaterialType::DIFFUSE));
//
//     let sphere = Sphere {
//         position: pt3(0.0, 10.0, 0.0),
//         radius: 2.0,
//         material: red.clone(),
//     };
//
//     (camera, environment, sphere)
// }

fn create_scene() -> (Camera, Material, Vec<Sphere>) {
    let camera = Camera::new(pt3(0.0, -10.0, 2.0), pt3(0.0, 0.0, 2.0), 55.0);
    let environment = Material::new(Some(vec3(0.6, 0.7, 0.8)), None, None);

    let white = Material::new(None, Some(vec3(0.6, 0.6, 0.2)), Some(MaterialType::DIFFUSE));
    let red = Material::new(None, Some(vec3(0.8, 0.2, 0.2)), Some(MaterialType::DIFFUSE));
    let green = Material::new(None, Some(vec3(0.2, 0.8, 0.2)), Some(MaterialType::DIFFUSE));
    let mirror = Material::new(
        None,
        Some(vec3(0.9, 0.6, 0.1)),
        Some(MaterialType::SPECULAR),
    );
    let light = Material::new(Some(vec3(10.0, 10.0, 10.0)), None, None);

    let spheres = vec![
        Sphere {
            position: vec3(-2.0, -1.5, 0.0),
            radius: 2.0,
            material: white,
        }, // ball left
        Sphere {
            position: vec3(2.0, 1.5, 1.0),
            radius: 2.0,
            material: mirror,
        }, // ball right
        Sphere {
            position: vec3(0.0, -2.0, 10.0),
            radius: 3.0,
            material: light,
        }, // light
        Sphere {
            position: vec3(105.0, 0.0, 0.0),
            radius: 100.0,
            material: green,
        }, // wall left
        Sphere {
            position: vec3(-105.0, 0.0, 0.0),
            radius: 100.0,
            material: red,
        }, // wall right
        Sphere {
            position: vec3(0.0, 0.0, -102.0),
            radius: 100.0,
            material: white,
        }, // floor
        Sphere {
            position: vec3(0.0, 110.0, 0.0),
            radius: 100.0,
            material: white,
        }, // wall back
    ];

    (camera, environment, spheres)
}

fn find_nearest_intersection(
    spheres: &Vec<Sphere>,
    ray: &Ray,
    t_min: f32,
    t_max: f32,
) -> Option<Hit> {
    let mut hit = spheres
        .par_iter()
        .filter_map(|sphere| sphere.intersect(ray, t_min, t_max))
        .min_by(|a, b| a.distance.total_cmp(&b.distance));

    if let Some(hit) = &mut hit {
        if ray.direction.dot(hit.normal) > 0.0 {
            hit.normal *= -1.0;
        }
    }
    hit
}
