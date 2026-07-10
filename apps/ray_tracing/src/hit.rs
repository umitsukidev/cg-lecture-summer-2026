use crate::material::Material;
use nannou::{geom::Point3, glam::Vec3};

#[allow(dead_code)]
#[derive(Clone)]
pub struct Hit {
    pub distance: f32,
    pub position: Point3,
    pub normal: Vec3,
    pub material: Material,
}
