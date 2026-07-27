use derive_more::{AsMut, AsRef, Deref, DerefMut, From, Into};

#[derive(Debug, Clone, Copy, Default, PartialEq, Deref, DerefMut, From, Into, AsRef, AsMut)]
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

    pub fn cyan_mut(&mut self) -> &mut f32 {
        &mut self[0]
    }

    pub fn magenta_mut(&mut self) -> &mut f32 {
        &mut self[1]
    }

    pub fn yellow_mut(&mut self) -> &mut f32 {
        &mut self[2]
    }

    pub fn black_mut(&mut self) -> &mut f32 {
        &mut self[3]
    }

    pub fn to_optical_density(self) -> Self {
        Self(self.map(|coverage| {
            let coverage = coverage.clamp(0.0, 1.0 - f32::EPSILON);
            -(1.0 - coverage).ln()
        }))
    }
}
