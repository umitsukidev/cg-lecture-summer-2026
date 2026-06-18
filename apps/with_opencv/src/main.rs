mod opencv_utils;

use nannou::prelude::*;
use opencv::{core, objdetect, prelude::*, videoio};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use opencv_utils::MatExt;

struct ProcessedData {
    faces: Vec<core::Rect>,
    avg_flow: Vec2,
}

struct Model {
    texture: wgpu::Texture,
    image_receiver: Receiver<nannou::image::RgbaImage>,
    data_receiver: Receiver<ProcessedData>,
    faces: Vec<core::Rect>,
    avg_flow: Vec2,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let (cam_tx, cam_rx) = std::sync::mpsc::sync_channel::<core::Mat>(1);
    let (img_tx, img_rx) = channel::<nannou::image::RgbaImage>();
    let (data_tx, data_rx) = channel::<ProcessedData>();

    thread::spawn(move || {
        let mut cam = videoio::VideoCapture::new(0, videoio::CAP_AVFOUNDATION)
            .expect("Unable to open camera with AVFoundation");

        let opened =
            videoio::VideoCapture::is_opened(&cam).expect("Failed to check if camera is opened");
        if !opened {
            panic!("Camera is not opened!");
        }

        loop {
            let mut raw_frame = Mat::default();
            if let Ok(true) = cam.read(&mut raw_frame) {
                if raw_frame.size().map(|s| s.width > 0).unwrap_or(false) {
                    let mut frame = opencv::core::Mat::default();
                    if core::flip(&raw_frame, &mut frame, 1).is_ok() {
                        let _ = cam_tx.try_send(frame.clone());

                        if let Ok(rgba_image) = frame.to_rgba_image() {
                            if img_tx.send(rgba_image).is_err() {
                                break;
                            }
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let face_cascade_path =
        manifest_dir.join("assets/haarcascades/haarcascade_frontalface_default.xml");
    let face_cascade_path_str = face_cascade_path.to_str().unwrap().to_string();

    thread::spawn(move || {
        let mut face_detector = objdetect::CascadeClassifier::new(&face_cascade_path_str)
            .expect("Failed to load face cascade");
        let mut faces = core::Vector::<core::Rect>::new();
        let mut prev_gray: Option<core::Mat> = None;
        let process_interval = Duration::from_millis(100);

        loop {
            let start_time = Instant::now();

            let raw_frame = match cam_rx.recv() {
                Ok(f) => f,
                Err(_) => break,
            };

            let mut latest_frame = raw_frame;
            while let Ok(f) = cam_rx.try_recv() {
                latest_frame = f;
            }

            let mut gray = core::Mat::default();
            if opencv::imgproc::cvt_color(
                &latest_frame,
                &mut gray,
                opencv::imgproc::COLOR_BGR2GRAY,
                0,
                core::AlgorithmHint::ALGO_HINT_DEFAULT,
            )
            .is_ok()
            {
                face_detector
                    .detect_multi_scale(
                        &gray,
                        &mut faces,
                        1.1,
                        3,
                        0,
                        core::Size::new(30, 30),
                        core::Size::new(0, 0),
                    )
                    .expect("Failed to detect faces");

                let mut avg_flow = Vec2::ZERO;
                if let Some(ref prev_gray) = prev_gray {
                    let mut flow = core::Mat::default();
                    let flow_res = opencv::video::calc_optical_flow_farneback(
                        prev_gray, &gray, &mut flow, 0.5, 3, 15, 3, 5, 1.2, 0,
                    );

                    if flow_res.is_ok() {
                        if let Ok(mean) = core::mean(&flow, &core::no_array()) {
                            let dx = mean[0] as f32;
                            let dy = -mean[1] as f32;
                            avg_flow = vec2(dx, dy);
                        }
                    }
                }

                prev_gray = Some(gray.clone());

                let data = ProcessedData {
                    faces: faces.to_vec(),
                    avg_flow,
                };

                if data_tx.send(data).is_err() {
                    break;
                }
            }

            let elapsed = start_time.elapsed();
            if elapsed < process_interval {
                thread::sleep(process_interval - elapsed);
            }
        }
    });

    let first_image = img_rx.recv().expect("Failed to receive initial frame");
    let width = first_image.width();
    let height = first_image.height();

    let _window = app
        .new_window()
        .size(width, height)
        .view(view)
        .build()
        .unwrap();

    let dynamic_image = nannou::image::DynamicImage::ImageRgba8(first_image);
    let texture = wgpu::Texture::from_image(app, &dynamic_image);

    Model {
        texture,
        image_receiver: img_rx,
        data_receiver: data_rx,
        faces: Vec::new(),
        avg_flow: Vec2::ZERO,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let mut latest_image = None;
    while let Ok(img) = model.image_receiver.try_recv() {
        latest_image = Some(img);
    }

    if let Some(img) = latest_image {
        let window = app.main_window();
        let device = window.device();
        let queue = window.queue();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Texture Upload"),
        });

        model
            .texture
            .upload_data(device, &mut encoder, img.as_raw());
        queue.submit(Some(encoder.finish()));
    }

    let mut latest_data = None;
    while let Ok(data) = model.data_receiver.try_recv() {
        latest_data = Some(data);
    }

    if let Some(data) = latest_data {
        model.faces = data.faces;
        model.avg_flow = data.avg_flow;
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.texture(&model.texture);

    let win_rect = app.window_rect();
    let win_width = win_rect.w();
    let win_height = win_rect.h();

    for face in model.faces.iter() {
        let w = face.width as f32;
        let h = face.height as f32;
        let x = (face.x as f32 + w / 2.0) - (win_width / 2.0);
        let y = (win_height / 2.0) - (face.y as f32 + h / 2.0);

        draw.rect()
            .x_y(x, y)
            .w_h(w, h)
            .no_fill()
            .stroke_weight(4.0)
            .stroke_color(STEELBLUE);
    }

    draw.line()
        .start(pt2(0.0, 0.0))
        .end(model.avg_flow * 100.0)
        .color(STEELBLUE)
        .stroke_weight(4.0);

    draw.to_frame(app, &frame).unwrap();
}
