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
use ndarray::{ArrayView2, Zip};
use std::sync::Arc;

pub fn update_vector_mesh(
    vector_mesh: &mut ndarray::Array2<[geom::Tri<(Point3, Color)>; 2]>,
    u: ArrayView2<f32>,
    v: ArrayView2<f32>,
    window_rect: Rect,
) {
    let width = window_rect.w();
    let height = window_rect.h();
    let l = width * H * 0.5;

    Zip::indexed(vector_mesh)
        .and(&u)
        .and(&v)
        .par_for_each(|(i, j), mesh_pair, &val_u, &val_v| {
            let from = pt2(
                (i as f32 + 0.5) * width / X_N as f32,
                (j as f32 + 0.5) * height / Y_N as f32,
            );
            let vel = vec2(val_u, val_v) * l * 4.1;
            let to = from + vel;

            if vel.length() > 1e-5 {
                let from_math = from.to_mathematical_coords(window_rect);
                let to_math = to.to_mathematical_coords(window_rect);
                let d = to_math - from_math;
                let len = d.length();
                if len > 1e-5 {
                    let dir = d / len;
                    let thick = 1.0;
                    let normal = vec2(-dir.y, dir.x) * (thick * 0.5);

                    let p1 = (from_math + normal).extend(0.0);
                    let p2 = (from_math - normal).extend(0.0);
                    let p3 = (to_math + normal).extend(0.0);
                    let p4 = (to_math - normal).extend(0.0);

                    let color = Color::srgb_u8(255, 200, 0);

                    mesh_pair[0] = geom::Tri([(p1, color), (p2, color), (p3, color)]);
                    mesh_pair[1] = geom::Tri([(p2, color), (p4, color), (p3, color)]);
                    return;
                }
            }

            let zero_pt = pt3(0.0, 0.0, 0.0);
            let zero_color = Color::srgb_u8(0, 0, 0);
            let zero_tri = geom::Tri([
                (zero_pt, zero_color),
                (zero_pt, zero_color),
                (zero_pt, zero_color),
            ]);
            mesh_pair[0] = zero_tri.clone();
            mesh_pair[1] = zero_tri;
        });
}

pub fn display_vector(draw: &Draw, vector_mesh: &ndarray::Array2<[geom::Tri<(Point3, Color)>; 2]>) {
    let tris = vector_mesh
        .iter()
        .flat_map(|pair| pair.iter())
        .filter(|tri| tri.0[0].0 != tri.0[1].0)
        .cloned();

    draw.mesh().tris_colored(tris);
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
                ui.set_width(60.0);
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("fps: {:.0}", fps)).monospace());
                });
            });
        });

    egui::Window::new("設定")
        .resizable(false)
        .max_width(150.0)
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

            ui.separator();

            let src_vel_amp = ui.label("src_vel_amp");
            ui.add(
                egui::Slider::new(&mut model.solver.src_vel_amp, 0.0..=0.4)
                    .step_by(0.01)
                    .smart_aim(false)
                    .fixed_decimals(2),
            )
            .labelled_by(src_vel_amp.id);

            let src_ink_amp = ui.label("src_ink_amp");
            ui.add(
                egui::Slider::new(&mut model.solver.src_ink_amp, 0.0..=0.4)
                    .step_by(0.01)
                    .smart_aim(false)
                    .fixed_decimals(2),
            )
            .labelled_by(src_ink_amp.id);

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

            ui.separator();

            ui.checkbox(&mut model.is_simulation_running, "シミュレーション");
            if ui.button("リセット").clicked() {
                model.solver.reset();
            }
        });
}
