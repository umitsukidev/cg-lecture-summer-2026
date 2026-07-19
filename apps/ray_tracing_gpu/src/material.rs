use nannou::glam::Vec3;

#[derive(Clone, Copy)]
pub enum Material {
    Diffuse { reflection: Vec3 },
    Specular { reflection: Vec3 },
    Emissive { emission: Vec3 },
    Refractive { reflection: Vec3, ior: f32 },
}

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

    pub fn to_gpu(self) -> GpuMaterial {
        match self {
            Material::Diffuse { reflection } => GpuMaterial {
                material_type: 0,
                pad0: 0,
                pad1: 0,
                pad2: 0,
                color: reflection.into(),
                pad3: 0.0,
            },
            Material::Specular { reflection } => GpuMaterial {
                material_type: 1,
                pad0: 0,
                pad1: 0,
                pad2: 0,
                color: reflection.into(),
                pad3: 0.0,
            },
            Material::Emissive { emission } => GpuMaterial {
                material_type: 2,
                pad0: 0,
                pad1: 0,
                pad2: 0,
                color: emission.into(),
                pad3: 0.0,
            },
            Material::Refractive { reflection, ior } => GpuMaterial {
                material_type: 3,
                pad0: 0,
                pad1: 0,
                pad2: 0,
                color: reflection.into(),
                pad3: ior,
            },
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct GpuMaterial {
    pub material_type: u32,
    pub pad0: u32,
    pub pad1: u32,
    pub pad2: u32,
    pub color: [f32; 3],
    pub pad3: f32,
}
