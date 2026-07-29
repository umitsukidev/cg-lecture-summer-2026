use crate::cmyk::Cmyk;
use nannou::prelude::*;

#[allow(dead_code)]
pub trait Point2Ext {
    fn to_screen_coords(self, window_rect: Rect) -> Point2;
    fn to_mathematical_coords(self, window_rect: Rect) -> Point2;
}

impl Point2Ext for Point2 {
    fn to_screen_coords(self, window_rect: Rect) -> Point2 {
        let x = self.x + window_rect.w() / 2.0;
        let y = window_rect.h() / 2.0 - self.y;
        pt2(x, y)
    }

    fn to_mathematical_coords(self, window_rect: Rect) -> Point2 {
        let x = self.x - window_rect.w() / 2.0;
        let y = window_rect.h() / 2.0 - self.y;
        pt2(x, y)
    }
}

#[allow(dead_code)]
pub trait ColorExt {
    fn cmyk(cyan: f32, magenta: f32, yellow: f32, black: f32) -> Color;
    fn cmykw(cyan: f32, magenta: f32, yellow: f32, black: f32, white: f32) -> Color;
    fn to_cmyk(self) -> Cmyk;
}

impl ColorExt for Color {
    fn cmyk(cyan: f32, magenta: f32, yellow: f32, black: f32) -> Color {
        let red = (1.0 - cyan) * (1.0 - black);
        let green = (1.0 - magenta) * (1.0 - black);
        let blue = (1.0 - yellow) * (1.0 - black);

        Color::srgb(red, green, blue)
    }

    fn cmykw(cyan: f32, magenta: f32, yellow: f32, black: f32, white: f32) -> Color {
        let [red, green, blue] = Color::cmyk(cyan, magenta, yellow, black)
            .to_srgba()
            .to_f32_array_no_alpha();
        let white = white.clamp(0.0, 1.0);
        let color_amount = cyan.clamp(0.0, 1.0)
            + magenta.clamp(0.0, 1.0)
            + yellow.clamp(0.0, 1.0)
            + black.clamp(0.0, 1.0);
        let total_amount = color_amount + white;
        let white_ratio = if total_amount > f32::EPSILON {
            white / total_amount
        } else {
            0.0
        };

        Color::srgb(
            red + (1.0 - red) * white_ratio,
            green + (1.0 - green) * white_ratio,
            blue + (1.0 - blue) * white_ratio,
        )
    }

    fn to_cmyk(self) -> Cmyk {
        let [red, green, blue] = self.to_srgba().to_f32_array_no_alpha();

        let max_rgb = [red, green, blue]
            .into_iter()
            .reduce(f32::max)
            .unwrap_or(0.0);

        let (cyan, magenta, yellow, black) = if max_rgb == 0.0 {
            (0.0, 0.0, 0.0, 1.0)
        } else {
            (
                (max_rgb - red) / max_rgb,
                (max_rgb - green) / max_rgb,
                (max_rgb - blue) / max_rgb,
                1.0 - max_rgb,
            )
        };

        Cmyk::new(cyan, magenta, yellow, black)
    }
}
