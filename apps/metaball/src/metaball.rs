use nannou::{
    geom::Point2,
    glam::{Vec2, vec2},
    rand,
};

#[derive(Debug)]
pub struct Metaball {
    pub position: Point2,
    pub radius: f32,
    velocity: Vec2,
    field_size: Point2,
}

impl Metaball {
    pub fn new(position: Point2, radius: f32, field_size: Point2) -> Self {
        Metaball {
            position,
            radius,
            velocity: vec2(rand::random_range(-5.0, 5.0), rand::random_range(-5.0, 5.0)),
            field_size,
        }
    }

    pub fn update(&mut self) {
        self.bounce();
        self.position += self.velocity;
    }

    fn bounce(&mut self) {
        let w = self.field_size.x;
        let h = self.field_size.y;
        if self.position.x < -w / 2.0 + self.radius || self.position.x > w / 2.0 - self.radius {
            self.velocity.x *= -1.0;
        }
        if self.position.y < -h / 2.0 + self.radius || self.position.y > h / 2.0 - self.radius {
            self.velocity.y *= -1.0;
        }
    }
}
