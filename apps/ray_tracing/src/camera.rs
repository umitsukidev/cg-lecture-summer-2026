use nannou::prelude::*;

use crate::ray::Ray;

#[allow(dead_code)]
pub struct Camera {
    pub position: Point3,
    pub flen: f32,
    pub camera_to_world: Mat4,
}

#[allow(dead_code)]
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

    pub fn ray(&self, window_rect: Rect, a: UVec2, u1: f32, u2: f32) -> Ray {
        let dir = vec3(
            (a.x as f32 + u1) / (window_rect.w() / 2.0) - 1.0,
            -(a.y as f32 + u2) / (window_rect.h() / 2.0) + 1.0,
            self.flen,
        );
        Ray {
            origin: self.position,
            direction: self.camera_to_world.transform_vector3(dir).normalize(),
        }
    }
}
