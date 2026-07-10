use nannou::{image::Rgba, prelude::*};

#[allow(dead_code)]
pub trait Point3Ext {
    fn to_color(self) -> Rgba<u8>;
}

impl Point3Ext for Point3 {
    fn to_color(self) -> Rgba<u8> {
        Rgba([
            (255.0 * self.x) as u8,
            (255.0 * self.y) as u8,
            (255.0 * self.z) as u8,
            255,
        ])
    }
}
