use crate::cmyk::Cmyk;
use derive_more::{AsMut, AsRef, Deref, DerefMut, From, Into};

#[derive(Debug, Clone, Copy, Default, PartialEq, Deref, DerefMut, From, Into, AsRef, AsMut)]
pub struct Cmykw(pub [f32; 5]);

impl Cmykw {
    pub fn new(cyan: f32, magenta: f32, yellow: f32, black: f32, white: f32) -> Self {
        Self([cyan, magenta, yellow, black, white])
    }

    pub fn from_cmyk(cmyk: Cmyk, white: f32) -> Self {
        Self::new(cmyk.c(), cmyk.m(), cmyk.y(), cmyk.k(), white)
    }

    pub fn cmyk(&self) -> Cmyk {
        Cmyk::new(self[0], self[1], self[2], self[3])
    }

    pub fn set_cmyk(&mut self, cmyk: Cmyk) {
        self[..4].copy_from_slice(&cmyk[..]);
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

    pub fn white(&self) -> f32 {
        self[4]
    }

    pub fn white_mut(&mut self) -> &mut f32 {
        &mut self[4]
    }

    pub fn to_optical_density(self) -> Self {
        Self(self.map(|coverage| {
            let coverage = coverage.clamp(0.0, 1.0 - f32::EPSILON);
            -(1.0 - coverage).ln()
        }))
    }
}

impl From<Cmyk> for Cmykw {
    fn from(cmyk: Cmyk) -> Self {
        Self::from_cmyk(cmyk, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Cmykw;
    use crate::cmyk::Cmyk;

    #[test]
    fn replacing_cmyk_preserves_white() {
        let mut color = Cmykw::new(0.1, 0.2, 0.3, 0.4, 0.5);
        color.set_cmyk(Cmyk::new(0.6, 0.7, 0.8, 0.9));

        assert_eq!(color, Cmykw::new(0.6, 0.7, 0.8, 0.9, 0.5));
    }

    #[test]
    fn optical_density_round_trips_to_coverage() {
        let color = Cmykw::new(0.8, 0.2, 0.5, 1.0, 0.4);
        let density = color.to_optical_density();
        let restored = density.map(|value| 1.0 - (-value).exp());

        for (actual, expected) in restored.into_iter().zip(color.iter().copied()) {
            assert!((actual - expected).abs() < 1e-6);
        }
    }
}
