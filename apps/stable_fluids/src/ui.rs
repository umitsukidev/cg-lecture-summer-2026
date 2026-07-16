use crate::{
    Model,
    nannou_utils::Point2Ext,
    solver::{H, X_N, Y_N},
};
use nannou::prelude::{
    egui::{
        FontTweak,
        epaint::text::{Tag, VariationCoords},
    },
    *,
};
use ndarray::ArrayView2;
use std::sync::Arc;

pub fn display_vector(draw: &Draw, window_rect: Rect, u: ArrayView2<f32>, v: ArrayView2<f32>) {
    let width = window_rect.w();
    let height = window_rect.h();

    let l = width * H * 0.5;
    for i in 0..X_N {
        for j in 0..Y_N {
            let from = pt2(
                (i as f32 + 0.5) * width / X_N as f32,
                (j as f32 + 0.5) * height / Y_N as f32,
            );
            let v = vec2(u[[i, j]], v[[i, j]]) * l * 4.1;
            let to = from + v;

            if v.length() > 0.0 {
                draw.line()
                    .start(from.to_mathematical_coords(window_rect))
                    .end(to.to_mathematical_coords(window_rect))
                    .color(Color::srgb_u8(255, 200, 0));
            }
        }
    }
}

pub fn display_grids(draw: &Draw, window_rect: Rect) {
    for i in 1..X_N {
        let px = i as f32 / X_N as f32 * window_rect.w();
        draw.line()
            .start(pt2(px, 0.0).to_mathematical_coords(window_rect))
            .end(pt2(px, window_rect.h()).to_mathematical_coords(window_rect))
            .color(Color::srgba_u8(127, 127, 127, 127));
    }

    for j in 1..Y_N {
        let py = j as f32 / Y_N as f32 * window_rect.h();
        draw.line()
            .start(pt2(0.0, py).to_mathematical_coords(window_rect))
            .end(pt2(window_rect.w(), py).to_mathematical_coords(window_rect))
            .color(Color::srgba_u8(127, 127, 127, 127));
    }
}

pub fn display_gui(app: &App, model: &mut Model) {
    let egui = app.egui();
    let fps = model.displayed_fps;

    egui::Area::new(egui::Id::new("fps_area"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 10.0))
        .show(&egui, |ui| {
            ui.style_mut().interaction.selectable_labels = false;

            egui::Frame::window(ui.style()).show(ui, |ui| {
                ui.set_min_width(60.0);
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("fps: {:.0}", fps)).monospace());
                });
            });
        });

    egui::Window::new("設定")
        .resizable(false)
        .show(&egui, |ui| {
            let mut fonts = egui::FontDefinitions::default();

            fonts.font_data.insert(
                "NotoSansJP".to_owned(),
                Arc::new({
                    let axes_settings = [(Tag::new(b"wght"), 400.0)];
                    let coords = VariationCoords::new(axes_settings);

                    egui::FontData::from_static(include_bytes!(
                        "../assets/NotoSansJP-VariableFont_wght.ttf"
                    ))
                    .tweak(FontTweak {
                        coords,
                        ..Default::default()
                    })
                }),
            );

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "NotoSansJP".to_owned());

            ui.set_fonts(fonts);

            ui.style_mut().interaction.selectable_labels = false;

            ui.checkbox(&mut model.show_display_grids, "グリッドを表示");
            ui.checkbox(&mut model.show_display_velocity, "速度ベクトルを表示");

            let src_vel_amp = ui.label("src_vel_amp");
            ui.add(
                egui::Slider::new(&mut model.solver.src_vel_amp, 0.0..=0.4)
                    .step_by(0.01)
                    .smart_aim(false)
                    .fixed_decimals(2),
            )
            .labelled_by(src_vel_amp.id);

            let src_rad = ui.label("src_rad");
            ui.add(
                egui::Slider::new(&mut model.solver.src_rad, 0.01..=10.0)
                    .step_by(0.01)
                    .smart_aim(false)
                    .fixed_decimals(2),
            )
            .labelled_by(src_rad.id);

            let max_gs_iterate = ui.label("max_gs_iterate");
            ui.add(
                egui::Slider::new(&mut model.solver.max_gs_iterate, 1..=20000)
                    .step_by(1.0)
                    .smart_aim(false),
            )
            .labelled_by(max_gs_iterate.id);
        });
}
