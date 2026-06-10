use nannou::{
    geom::Point2,
    glam::{Vec2, vec2},
};

#[derive(Clone)]
pub struct Ball {
    pub position: Point2,
    pub velocity: Vec2,
    _radius: f32,
    air_resistance: f32,
}

impl Ball {
    pub fn new(position: Point2, radius: f32) -> Self {
        Self {
            position,
            velocity: vec2(0.0, 0.0),
            _radius: radius,
            air_resistance: 0.9,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.position += self.velocity * dt;
        self.velocity *= self.air_resistance.powf(dt);
    }
}
