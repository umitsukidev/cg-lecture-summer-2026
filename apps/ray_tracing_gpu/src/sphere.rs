use crate::material::{GpuMaterial, Material};
use nannou::geom::Point3;

#[derive(Clone, Copy)]
pub struct Sphere {
    pub position: Point3,
    pub radius: f32,
    pub material: Material,
}

impl Sphere {
    pub fn to_gpu(self) -> GpuSphere {
        GpuSphere {
            position: self.position.into(),
            radius: self.radius,
            material: self.material.to_gpu(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct GpuSphere {
    pub position: [f32; 3],
    pub radius: f32,
    pub material: GpuMaterial,
}
