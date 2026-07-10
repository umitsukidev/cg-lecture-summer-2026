use crate::{hit::Hit, material::Material, ray::Ray};
use nannou::geom::Point3;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct Sphere<'a> {
    pub position: Point3,
    pub radius: f32,
    pub material: &'a Material,
}

#[allow(dead_code)]
impl<'a> Sphere<'a> {
    pub fn distance(&self, ray: Ray) -> f32 {
        let position = ray.origin - self.position;
        let b = ray.direction.dot(position);
        let c = position.dot(position) - self.radius * self.radius;
        if c < b * b {
            let mut t = b + (b * b - c).sqrt();
            if 0.0 < t {
                t = b - (b * b - c).sqrt();
            }
            if t < 0.0 {
                return -t;
            }
        }
        -1.0
    }

    pub fn intersect(&self, ray: Ray, t_min: f32, t_max: f32) -> Option<Hit<'a>> {
        let t = self.distance(ray);
        if t_min < t && t < t_max {
            let position = ray.origin + (ray.direction * t);
            Some(Hit {
                distance: t,
                position,
                normal: (position - self.position).normalize(),
                material: self.material,
            })
        } else {
            None
        }
    }
}
