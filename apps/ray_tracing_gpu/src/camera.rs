use nannou::prelude::*;

#[derive(Clone)]
pub struct Camera {
    pub position: Point3,
    pub flen: f32,
    pub camera_to_world: Mat4,
}

impl Camera {
    pub fn new(position: Point3, target: Vec3, f: f32) -> Self {
        let flen = f / 35.0;

        let z = (target - position).normalize();
        let x = z.cross(vec3(0.0, 0.0, 1.0)).normalize();
        let y = x.cross(z);
        let camera_to_world = mat4(x.extend(0.0), y.extend(0.0), z.extend(0.0), Vec4::ZERO);

        Self {
            position,
            flen,
            camera_to_world,
        }
    }

    pub fn to_uniform(&self) -> CameraUniform {
        CameraUniform {
            position: self.position.into(),
            flen: self.flen,
            camera_to_world: self.camera_to_world.to_cols_array_2d(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct CameraUniform {
    pub position: [f32; 3],
    pub flen: f32,
    pub camera_to_world: [[f32; 4]; 4],
}
