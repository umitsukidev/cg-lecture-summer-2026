use crate::nannou_utils::Point3Ext;
use nannou::{glam::Vec3, image::Rgba};

#[allow(dead_code)]
#[derive(Clone)]
pub enum Material {
    Diffuse { reflection: Vec3 },
    Specular { reflection: Vec3 },
    Refractive { reflection: Vec3, ior: f32 },
    Emissive { emission: Vec3 },
}

#[allow(dead_code)]
impl Material {
    pub fn diffuse(reflection: Vec3) -> Self {
        Material::Diffuse { reflection }
    }

    pub fn specular(reflection: Vec3) -> Self {
        Material::Specular { reflection }
    }

    pub fn emissive(emission: Vec3) -> Self {
        Material::Emissive { emission }
    }

    pub fn refractive(reflection: Vec3, ior: f32) -> Self {
        Material::Refractive { reflection, ior }
    }

    pub fn to_color(&self) -> Rgba<u8> {
        match *self {
            Material::Diffuse { reflection } => reflection.to_color(),
            Material::Specular { reflection } => reflection.to_color(),
            Material::Emissive { emission } => emission.to_color(),
            Material::Refractive { reflection, .. } => reflection.to_color(),
        }
    }
}
