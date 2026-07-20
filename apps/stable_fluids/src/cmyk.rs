use derive_more::{Deref, DerefMut};

#[derive(Debug, Clone, Copy, Default, Deref, DerefMut)]
pub struct Cmyk(pub [f32; 4]);

impl Cmyk {
    pub fn new(cyan: f32, magenta: f32, yellow: f32, black: f32) -> Self {
        Self([cyan, magenta, yellow, black])
    }

    pub fn cyan(&self) -> f32 {
        self[0]
    }

    pub fn magenta(&self) -> f32 {
        self[1]
    }

    pub fn yellow(&self) -> f32 {
        self[2]
    }

    pub fn black(&self) -> f32 {
        self[3]
    }

    pub fn c(&self) -> f32 {
        self.cyan()
    }

    pub fn m(&self) -> f32 {
        self.magenta()
    }

    pub fn y(&self) -> f32 {
        self.yellow()
    }

    pub fn k(&self) -> f32 {
        self.black()
    }
}
