use nannou::{geom::Point2, prelude::*};

#[allow(dead_code)]
pub trait Point2Ext {
    fn translate_to_screen_coords(&self, window_rect: Rect) -> Point2;
    fn translate_to_mathematical_coords(&self, window_rect: Rect) -> Point2;
}

impl Point2Ext for Point2 {
    fn translate_to_screen_coords(&self, window_rect: Rect) -> Point2 {
        let x = self.x + window_rect.w() / 2.0;
        let y = window_rect.h() / 2.0 - self.y;
        *self + vec2(x, y)
    }

    fn translate_to_mathematical_coords(&self, window_rect: Rect) -> Point2 {
        let x = self.x - window_rect.w() / 2.0;
        let y = window_rect.h() / 2.0 - self.y;
        *self + vec2(x, y)
    }
}
