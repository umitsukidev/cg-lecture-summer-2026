use crate::cmykw::Cmykw;

#[derive(Debug, Clone, Copy, Default)]
pub struct InkCell {
    pub color_mass: Cmykw,
    pub ink_amount: f32,
    pub water_amount: f32,
}
