use crate::{
    camera::Camera,
    material::{Material, MaterialType},
    sphere::Sphere,
};
use nannou::prelude::*;

pub fn create_scene() -> (Camera, Material, Vec<Sphere>) {
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
