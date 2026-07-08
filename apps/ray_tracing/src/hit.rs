use nannou::{geom::Point3, glam::Vec3};

use crate::material::Material;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Hit {
    pub distance: f32,
    pub position: Point3,
    pub normal: Vec3,
    pub material: Material,
}
