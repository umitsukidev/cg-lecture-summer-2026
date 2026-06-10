use nannou::{
    geom::Point2,
    glam::{Vec2, vec2},
};

#[derive(Clone)]
pub struct Ball {
    pub position: Point2,
    pub velocity: Vec2,
    pub radius: f32,
    air_resistance: f32,
}

impl Ball {
    pub fn new(position: Point2) -> Self {
        Self {
            position,
            velocity: vec2(0.0, 0.0),
            radius: 10.0,
            air_resistance: 0.95,
        }
    }

    pub fn update(&mut self) {
        self.position += self.velocity;
        self.velocity *= self.air_resistance;
    }
}
