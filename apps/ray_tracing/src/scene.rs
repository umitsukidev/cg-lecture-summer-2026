use crate::{camera::Camera, material::Material, sphere::Sphere};
use nannou::prelude::*;

pub fn create_scene() -> (Camera, Material, Vec<Sphere<'static>>) {
    let camera = Camera::new(pt3(0.0, -10.0, 2.0), pt3(0.0, 0.0, 2.0), 55.0);
    let environment = Material::emissive(vec3(0.6, 0.7, 0.8));

    static WHITE: Material = Material::Diffuse {
        reflection: vec3(0.6, 0.6, 0.6),
    };
    static RED: Material = Material::Diffuse {
        reflection: vec3(0.8, 0.2, 0.2),
    };
    static GREEN: Material = Material::Diffuse {
        reflection: vec3(0.2, 0.8, 0.2),
    };
    static MIRROR: Material = Material::Specular {
        reflection: vec3(0.9, 0.6, 0.1),
    };
    static LIGHT: Material = Material::Emissive {
        emission: vec3(10.0, 10.0, 10.0),
    };

    static GLASS: Material = Material::Refractive {
        reflection: vec3(1.0, 0.64, 0.83),
        ior: 1.7,
    };

    let spheres = vec![
        Sphere {
            position: pt3(-2.0, -1.5, 0.0),
            radius: 2.0,
            material: &WHITE,
        }, // ball left
        Sphere {
            position: pt3(2.0, 1.5, 1.0),
            radius: 2.0,
            material: &MIRROR,
        }, // ball right
        Sphere {
            position: pt3(1.0, -3.0, -0.5),
            radius: 1.0,
            material: &GLASS,
        }, // ball center
        Sphere {
            position: pt3(0.0, -2.0, 10.0),
            radius: 3.0,
            material: &LIGHT,
        }, // light
        Sphere {
            position: pt3(105.0, 0.0, 0.0),
            radius: 100.0,
            material: &GREEN,
        }, // wall left
        Sphere {
            position: pt3(-105.0, 0.0, 0.0),
            radius: 100.0,
            material: &RED,
        }, // wall right
        Sphere {
            position: pt3(0.0, 0.0, -102.0),
            radius: 100.0,
            material: &WHITE,
        }, // floor
        Sphere {
            position: pt3(0.0, 110.0, 0.0),
            radius: 100.0,
            material: &WHITE,
        }, // wall back
    ];

    (camera, environment, spheres)
}
