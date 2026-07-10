use crate::{
    camera::Camera,
    hit::Hit,
    material::{Material, MaterialType},
    nannou_utils::Point3Ext,
    ray::Ray,
    sphere::Sphere,
};
use nannou::{image::Rgba, prelude::*};
use rayon::prelude::*;

pub fn find_nearest_intersection(
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

pub fn render(
    window_rect: Rect,
    x: u32,
    y: u32,
    camera: &Camera,
    spheres: &Vec<Sphere>,
    environment: &Material,
    count: &u64,
    pixel: &mut Vec3,
) -> Rgba<u8> {
    let view = camera.ray(window_rect, UVec2::new(x, y), random_f32(), random_f32());
    *pixel += trace(environment, spheres, view, 0);
    let final_color = *pixel / (*count as f32 + 1.0);
    final_color.to_color()
}

// pub fn render(
//     window_rect: Rect,
//     x: u32,
//     y: u32,
//     camera: &Camera,
//     spheres: &Vec<Sphere>,
//     environment: &Material,
// ) -> Rgba<u8> {
//     let view = camera.ray(window_rect, UVec2::new(x, y), random_f32(), random_f32());
//     if let Some(hit) = find_nearest_intersection(spheres, &view, 0.001, f32::MAX) {
//         // hit.material.to_color()
//         hit.normal.to_color()
//     } else {
//         environment.emission.unwrap().to_color()
//     }
// }

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

pub fn trace(environment: &Material, spheres: &Vec<Sphere>, ray: Ray, depth: u32) -> Vec3 {
    // if 10 < depth {
    //     return vec3(0.0, 0.0, 0.0);
    // }

    let mut ray = ray;

    let hit = find_nearest_intersection(spheres, &ray, 0.001, f32::MAX);

    let mut result = vec3(0.0, 0.0, 0.0);

    if let Some(hit) = hit {
        if let Some(emission) = hit.material.emission {
            result += emission;
        }

        if let Some(reflection) = hit.material.reflection {
            let (t, b) = tangentspace_basis(&hit.normal);

            match hit.material.material_type {
                MaterialType::DIFFUSE => {
                    let dir = sample_hemisphere_cosine(random_f32(), random_f32());
                    ray.origin = hit.position;
                    ray.direction = dir.x * t + dir.y * b + dir.z * hit.normal;
                }
                MaterialType::SPECULAR => {
                    let direction = ray.direction;
                    let normal = hit.normal;
                    ray.origin = hit.position;
                    ray.direction = direction - 2.0 * (direction.dot(normal)) * normal;
                }
            }
            result += trace(environment, spheres, ray, depth + 1) * reflection;
        }
    } else {
        return environment.emission.unwrap_or(vec3(0.0, 0.0, 0.0));
    }
    result
}

pub fn tangentspace_basis(n: &Vec3) -> (Vec3, Vec3) {
    let sg = if n.z < 0.0 { -1.0 } else { 1.0 };
    let a_factor = -1.0 / (sg + n.z);
    let b_factor = n.x * n.y * a_factor;
    let t = vec3(1.0 + sg * n.x * n.x * a_factor, sg * b_factor, -sg * n.x);
    let b = vec3(b_factor, sg + n.y * n.y * a_factor, -n.y);
    (t, b)
}

pub fn sample_hemisphere_cosine(u1: f32, u2: f32) -> Vec3 {
    let r = u1.sqrt();
    let theta = u2 * 2.0 * PI;
    vec3(r * theta.cos(), r * theta.sin(), (1.0 - u1).sqrt())
}
