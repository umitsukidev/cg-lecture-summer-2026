use crate::{
    Model,
    audio::{AudioMappingMode, ChordPreset, ProgressionStyle},
    nannou_utils::{ColorExt, Point2Ext},
    solver::{H, X_N, Y_N},
};
use nannou::prelude::{
    egui::{
        FontTweak, Shadow, Visuals,
        epaint::text::{Tag, VariationCoords},
        style::Interaction,
    },
    *,
};
use ndarray::ArrayView2;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

pub fn update_vector_mesh(
    vector_mesh: &mut [geom::Tri<(Point3, Color)>],
    u: ArrayView2<f32>,
    v: ArrayView2<f32>,
    window_rect: Rect,
) {
    let width = window_rect.w();
    let height = window_rect.h();
    let l = width * H * 0.5;

    vector_mesh
        .par_iter_mut()
        .enumerate()
        .for_each(|(idx, mesh_tri)| {
            let i = idx / Y_N;
            let j = idx % Y_N;

            let val_u = u[[i, j]];
            let val_v = v[[i, j]];

            let from = pt2(
                (i as f32 + 0.5) * width / X_N as f32,
                (j as f32 + 0.5) * height / Y_N as f32,
            );
            let vel = vec2(val_u, val_v) * l * 10.0;
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

                    let p1 = (from_math - normal).extend(0.0);
                    let p2 = (from_math + normal).extend(0.0);
                    let p3 = to_math.extend(0.0);

                    let color = Color::srgb_u8(255, 200, 0);

                    *mesh_tri = geom::Tri([(p1, color), (p2, color), (p3, color)]);
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
            *mesh_tri = zero_tri;
        });
}

pub fn display_vector(draw: &Draw, vector_mesh: &Mutex<Vec<geom::Tri<(Point3, Color)>>>) {
    let mut mesh_guard = vector_mesh.lock().unwrap();
    let mut tris = mesh_guard
        .drain(..)
        .filter(|tri| tri.0[0].0 != tri.0[1].0)
        .peekable();

    if tris.peek().is_some() {
        draw.mesh().tris_colored(tris);
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

    egui.set_global_style(egui::Style {
        visuals: Visuals {
            window_shadow: Shadow::NONE,
            ..Default::default()
        },
        interaction: Interaction {
            selectable_labels: false,
            ..Default::default()
        },
        ..Default::default()
    });

    egui::Area::new(egui::Id::new("fps_area"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 10.0))
        .show(&egui, |ui| {
            egui::Frame::window(ui.style()).show(ui, |ui| {
                ui.set_width(70.0);
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("fps: {:.0}", fps)).monospace());
                });
            });
        });

    egui::Window::new("設定 / 和音シミュレーター")
        .resizable(true)
        .vscroll(true)
        .default_width(320.0)
        .default_height(720.0)
        .show(&egui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
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

                ui.label("Press F3 to hide GUI");

                ui.separator();

                // ----------------------------------------------------
                // 1. Audio Synth & Chord Controls (Collapsible)
                // ----------------------------------------------------
                egui::CollapsingHeader::new(egui::RichText::new("🎵 和音・サウンド設定").heading())
                    .default_open(true)
                    .show(ui, |ui| {
                        if let Ok(mut audio_state) = model.audio.shared_state.lock() {
                            ui.checkbox(&mut audio_state.enabled, "サウンド有効化");

                            let vol_label = ui.label("マスター音量");
                            ui.add(
                                egui::Slider::new(&mut audio_state.volume, 0.0..=1.0)
                                    .step_by(0.01)
                                    .fixed_decimals(2),
                            )
                            .labelled_by(vol_label.id);

                            ui.horizontal(|ui| {
                                ui.label("手動コード:");
                                let current_preset = audio_state.preset;
                                egui::ComboBox::from_id_salt("chord_preset_combo")
                                    .selected_text(current_preset.name())
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut audio_state.preset, ChordPreset::CMajor9, ChordPreset::CMajor9.name());
                                        ui.selectable_value(&mut audio_state.preset, ChordPreset::Dm9, ChordPreset::Dm9.name());
                                        ui.selectable_value(&mut audio_state.preset, ChordPreset::Em7, ChordPreset::Em7.name());
                                        ui.selectable_value(&mut audio_state.preset, ChordPreset::FMaj9, ChordPreset::FMaj9.name());
                                        ui.selectable_value(&mut audio_state.preset, ChordPreset::G9sus4, ChordPreset::G9sus4.name());
                                        ui.selectable_value(&mut audio_state.preset, ChordPreset::AMinor9, ChordPreset::AMinor9.name());
                                        ui.selectable_value(&mut audio_state.preset, ChordPreset::AbMaj9, ChordPreset::AbMaj9.name());
                                        ui.selectable_value(&mut audio_state.preset, ChordPreset::BbMaj9, ChordPreset::BbMaj9.name());
                                    });
                            });

                            ui.checkbox(&mut audio_state.auto_rotate_chords, "自動コード進行（ローテーション）");
                            if audio_state.auto_rotate_chords {
                                ui.horizontal(|ui| {
                                    ui.label("進行スタイル:");
                                    let current_style = audio_state.progression_style;
                                    egui::ComboBox::from_id_salt("progression_style_combo")
                                        .selected_text(current_style.name())
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut audio_state.progression_style, ProgressionStyle::GenerativeRandom, ProgressionStyle::GenerativeRandom.name());
                                            ui.selectable_value(&mut audio_state.progression_style, ProgressionStyle::CityPop, ProgressionStyle::CityPop.name());
                                            ui.selectable_value(&mut audio_state.progression_style, ProgressionStyle::NeoSoulJazz, ProgressionStyle::NeoSoulJazz.name());
                                            ui.selectable_value(&mut audio_state.progression_style, ProgressionStyle::CinematicModal, ProgressionStyle::CinematicModal.name());
                                        });
                                });

                                ui.horizontal(|ui| {
                                    if audio_state.is_transitioning {
                                        ui.label(format!("状態: 遷移中 ({:.1}s)", audio_state.current_transition_duration));
                                        ui.add(egui::ProgressBar::new(audio_state.rotation_progress).text("4s/oct Morph"));
                                    } else {
                                        let total_hold = audio_state.current_hold_duration;
                                        let remaining = (total_hold * (1.0 - audio_state.rotation_progress)).max(0.0);
                                        ui.label(format!("状態: 保持中 (残り {:.1}s / {:.1}s)", remaining, total_hold));
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                ui.label("音響マッピング:");
                                let current_mode = audio_state.mapping_mode;
                                egui::ComboBox::from_id_salt("audio_mode_combo")
                                    .selected_text(current_mode.name())
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut audio_state.mapping_mode, AudioMappingMode::Spatial, AudioMappingMode::Spatial.name());
                                        ui.selectable_value(&mut audio_state.mapping_mode, AudioMappingMode::ColorMass, AudioMappingMode::ColorMass.name());
                                    });
                            });

                            ui.add_space(4.0);
                            ui.label("リアルタイム和音メーター:");
                            let labels = audio_state.mapping_mode.voice_labels();
                            let notes = audio_state.preset.note_names();

                            for i in 0..5 {
                                let amp = audio_state.current_amps[i];
                                ui.horizontal(|ui| {
                                    ui.style_mut().spacing.item_spacing.x = 4.0;
                                    ui.label(format!("{}:", labels[i]));
                                    ui.add(egui::ProgressBar::new(amp).text(format!("{:.0}% ({})", amp * 100.0, notes[i])));
                                });
                            }
                        }
                    });

                ui.separator();

                // ----------------------------------------------------
                // 2. Color Selection & Swatch Area (MOVED ABOVE Fluid Simulation)
                // ----------------------------------------------------
                ui.heading("🎨 インクの色選択 & スウォッチ");

                let switch_fill = ui.visuals().widgets.inactive.weak_bg_fill;
                let switch_stroke = ui.visuals().widgets.noninteractive.bg_stroke;
                let switch_corner_radius = ui.visuals().widgets.inactive.corner_radius;

                egui::Frame::new()
                    .fill(switch_fill)
                    .stroke(switch_stroke)
                    .corner_radius(switch_corner_radius)
                    .inner_margin(egui::Margin::same(1))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.x = 1.0;
                        ui.horizontal(|ui| {
                            if color_mode_button(ui, "CMYK+W", model.set_color_by_cmykw).clicked() {
                                model.set_color_by_cmykw = true;
                            }
                            if color_mode_button(ui, "RGB", !model.set_color_by_cmykw).clicked() {
                                model.set_color_by_cmykw = false;
                            }
                        });
                    });

                ui.add_space(4.0);

                // Two columns: Left = Color Sliders, Right = Swatch Palette
                let mut selected_swatch = None;
                ui.columns(2, |columns| {
                    // Left Column: Sliders & Color Picker
                    columns[0].vertical(|ui| {
                        if !model.set_color_by_cmykw {
                            let ink_color = model.solver.ink_color;
                            let cmyk = ink_color.cmyk();
                            let mut ink_color = Color::cmyk(cmyk.c(), cmyk.m(), cmyk.y(), cmyk.k())
                                .to_srgba()
                                .to_u8_array_no_alpha();

                            ui.color_edit_button_srgb(&mut ink_color);

                            let red_label = ui.label("Red");
                            ui.add(egui::Slider::new(&mut ink_color[0], 0..=255)).labelled_by(red_label.id);
                            let green_label = ui.label("Green");
                            ui.add(egui::Slider::new(&mut ink_color[1], 0..=255)).labelled_by(green_label.id);
                            let blue_label = ui.label("Blue");
                            ui.add(egui::Slider::new(&mut ink_color[2], 0..=255)).labelled_by(blue_label.id);

                            model.solver.ink_color.set_cmyk(
                                Color::srgb_u8(ink_color[0], ink_color[1], ink_color[2]).to_cmyk(),
                            );
                            *model.solver.ink_color.white_mut() = 0.0;
                        } else {
                            let ink_color = model.solver.ink_color;
                            let cmyk = ink_color.cmyk();
                            let mut color_picker_value =
                                Color::cmyk(cmyk.c(), cmyk.m(), cmyk.y(), cmyk.k())
                                    .to_srgba()
                                    .to_u8_array_no_alpha();

                            ui.horizontal(|ui| {
                                if ui.color_edit_button_srgb(&mut color_picker_value).changed() {
                                    model.solver.ink_color.set_cmyk(
                                        Color::srgb_u8(
                                            color_picker_value[0],
                                            color_picker_value[1],
                                            color_picker_value[2],
                                        )
                                        .to_cmyk(),
                                    );
                                }

                                let ink_color = model.solver.ink_color;
                                let cmyk = ink_color.cmyk();
                                let preview =
                                    Color::cmykw(cmyk.c(), cmyk.m(), cmyk.y(), cmyk.k(), ink_color.white())
                                        .to_srgba()
                                        .to_u8_array_no_alpha();
                                let preview = egui::Color32::from_rgb(preview[0], preview[1], preview[2]);
                                egui::color_picker::show_color(ui, preview, ui.spacing().interact_size)
                                    .on_hover_text("CMYK+W 最終色");
                            });

                            let ink_color = &mut model.solver.ink_color;

                            let cyan_label = ui.label("Cyan");
                            ui.add(egui::Slider::new(ink_color.cyan_mut(), 0.0..=1.0)).labelled_by(cyan_label.id);
                            let magenta_label = ui.label("Magenta");
                            ui.add(egui::Slider::new(ink_color.magenta_mut(), 0.0..=1.0)).labelled_by(magenta_label.id);
                            let yellow_label = ui.label("Yellow");
                            ui.add(egui::Slider::new(ink_color.yellow_mut(), 0.0..=1.0)).labelled_by(yellow_label.id);
                            let black_label = ui.label("Black");
                            ui.add(egui::Slider::new(ink_color.black_mut(), 0.0..=1.0)).labelled_by(black_label.id);
                            let white_label = ui.label("White");
                            ui.add(egui::Slider::new(ink_color.white_mut(), 0.0..=1.0)).labelled_by(white_label.id);
                        }
                    });

                    // Right Column: Color Swatch Palette & Registration Button
                    columns[1].vertical(|ui| {
                        ui.label(egui::RichText::new("🎨 スウォッチ一覧").strong());
                        if ui.button("➕ 現在の色を登録").clicked() {
                            model.color_swatches.push(model.solver.ink_color);
                        }

                        ui.add_space(4.0);

                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                            for swatch in &model.color_swatches {
                                let cmyk = swatch.cmyk();
                                let rgb = Color::cmykw(cmyk.c(), cmyk.m(), cmyk.y(), cmyk.k(), swatch.white())
                                    .to_srgba()
                                    .to_u8_array_no_alpha();
                                let fill = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);

                                let btn = egui::Button::new("")
                                    .fill(fill)
                                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::DARK_GRAY))
                                    .min_size(egui::vec2(22.0, 22.0));

                                let tooltip = format!(
                                    "C:{:.0}% M:{:.0}% Y:{:.0}%\nK:{:.0}% W:{:.0}%",
                                    cmyk.c() * 100.0,
                                    cmyk.m() * 100.0,
                                    cmyk.y() * 100.0,
                                    cmyk.k() * 100.0,
                                    swatch.white() * 100.0
                                );

                                if ui.add(btn).on_hover_text(tooltip).clicked() {
                                    selected_swatch = Some(*swatch);
                                }
                            }
                        });
                    });
                });

                if let Some(swatch) = selected_swatch {
                    model.solver.ink_color = swatch;
                }

                ui.separator();

                // ----------------------------------------------------
                // 3. Fluid Simulation Controls
                // ----------------------------------------------------
                ui.heading("🌊 流体シミュレーション");
                ui.checkbox(&mut model.display_grids, "グリッドを表示");
                ui.checkbox(&mut model.display_velocity, "速度ベクトルを表示");

                let src_vel_amp_label = ui.label("インクの勢い");
                ui.add(
                    egui::Slider::new(&mut model.solver.src_vel_amp, 0.0..=1.0)
                        .step_by(0.01)
                        .smart_aim(false)
                        .fixed_decimals(2),
                )
                .labelled_by(src_vel_amp_label.id);

                let velocity_dissipation_label = ui.label("力の減衰率");
                ui.add(
                    egui::Slider::new(&mut model.solver.velocity_dissipation, 0.0..=1.0)
                        .step_by(0.01)
                        .smart_aim(false)
                        .fixed_decimals(2),
                )
                .labelled_by(velocity_dissipation_label.id);

                let src_ink_amp_label = ui.label("インク量");
                ui.add(
                    egui::Slider::new(&mut model.solver.src_ink_amp, 0.0..=1.0)
                        .step_by(0.01)
                        .smart_aim(false)
                        .fixed_decimals(2),
                )
                .labelled_by(src_ink_amp_label.id);

                let src_water_amp_label = ui.label("水量");
                ui.add(
                    egui::Slider::new(&mut model.solver.src_water_amp, 0.0..=1.0)
                        .step_by(0.01)
                        .smart_aim(false)
                        .fixed_decimals(2),
                )
                .labelled_by(src_water_amp_label.id);

                let src_rad_label = ui.label("インクの注入半径");
                ui.add(
                    egui::Slider::new(&mut model.solver.src_rad, 0.01..=20.0)
                        .step_by(0.01)
                        .smart_aim(false)
                        .fixed_decimals(2),
                )
                .labelled_by(src_rad_label.id);

                let max_gs_iterate_label = ui.label("圧力反復計算の最大回数");
                ui.add(
                    egui::Slider::new(&mut model.solver.max_pressure_iterations, 1..=2000)
                        .step_by(1.0)
                        .smart_aim(false),
                )
                .labelled_by(max_gs_iterate_label.id);

                ui.separator();

                ui.checkbox(&mut model.is_simulation_running, "シミュレーション");
                if ui.button("流体をリセット").clicked() {
                    model.solver.reset_simulation();
                }
                let reset_all_button = ui.button("全てリセット");
                egui::Popup::menu(&reset_all_button)
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                    .show(|ui| {
                        ui.label("全てリセットしますか？");
                        if ui.button("リセット").clicked() {
                            model.solver.reset_all();
                            ui.close();
                        }
                        if ui.button("キャンセル").clicked() {
                            ui.close();
                        }
                    });
            });
        });
}

fn color_mode_button(ui: &mut egui::Ui, text: &str, is_selected: bool) -> egui::Response {
    let visuals = ui.visuals();

    let bg_fill = if is_selected {
        visuals.widgets.active.bg_fill
    } else {
        egui::Color32::TRANSPARENT
    };

    let text_color = if is_selected {
        visuals.widgets.active.fg_stroke.color
    } else {
        visuals.widgets.inactive.fg_stroke.color
    };

    let corner_radius = visuals.widgets.inactive.corner_radius;

    egui::Frame::new()
        .fill(bg_fill)
        .corner_radius(corner_radius)
        .inner_margin(egui::Margin::symmetric(8, 2))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).color(text_color))
        })
        .response
}
