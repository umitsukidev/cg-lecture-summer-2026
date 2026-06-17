mod opencv_utils;

use nannou::prelude::*;
use opencv::{core, objdetect, prelude::*, videoio};

use opencv_utils::MatExt;

struct Model {
    cam: videoio::VideoCapture,
    texture: wgpu::Texture,
    face_detector: objdetect::CascadeClassifier,
    faces: core::Vector<core::Rect>,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_AVFOUNDATION)
        .expect("Unable to open camera with AVFoundation");

    let opened =
        videoio::VideoCapture::is_opened(&cam).expect("Failed to check if camera is opened");
    if !opened {
        panic!("Camera is not opened!");
    }

    let mut frame = Mat::default();
    cam.read(&mut frame).expect("Failed to read initial frame");
    let mut flipped_frame = Mat::default();
    core::flip(&frame, &mut flipped_frame, 1).expect("Failed to flip frame");
    let frame = flipped_frame;

    let face_detector = {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let face_cascade_path =
            manifest_dir.join("assets/haarcascades/haarcascade_frontalface_default.xml");
        objdetect::CascadeClassifier::new(face_cascade_path.to_str().unwrap())
            .expect("Failed to load face cascade")
    };

    let faces = core::Vector::<core::Rect>::new();

    let size = frame.size().expect("Failed to get frame size");
    let width = size.width as u32;
    let height = size.height as u32;

    let _window = app
        .new_window()
        .size(width, height)
        .view(view)
        .build()
        .unwrap();

    let rgba_image = frame
        .to_rgba_image()
        .expect("Failed to convert initial frame to RgbaImage");
    let dynamic_image = nannou::image::DynamicImage::ImageRgba8(rgba_image);
    let texture = wgpu::Texture::from_image(app, &dynamic_image);

    Model {
        cam,
        texture,
        face_detector,
        faces,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let mut raw_frame = opencv::core::Mat::default();
    if let Ok(true) = model.cam.read(&mut raw_frame) {
        if raw_frame.size().map(|s| s.width > 0).unwrap_or(false) {
            let mut frame = opencv::core::Mat::default();
            core::flip(&raw_frame, &mut frame, 1).expect("Failed to flip frame");

            if let Ok(rgba_image) = frame.to_rgba_image() {
                let window = app.main_window();
                let device = window.device();
                let queue = window.queue();

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Texture Upload"),
                });

                let mut gray = core::Mat::default();
                opencv::imgproc::cvt_color(
                    &frame,
                    &mut gray,
                    opencv::imgproc::COLOR_BGR2GRAY,
                    0,
                    core::AlgorithmHint::ALGO_HINT_DEFAULT,
                )
                .expect("Failed to convert to grayscale");

                model
                    .face_detector
                    .detect_multi_scale(
                        &gray,
                        &mut model.faces,
                        1.1,
                        3,
                        0,
                        core::Size::new(30, 30),
                        core::Size::new(0, 0),
                    )
                    .expect("Failed to detect faces");

                model
                    .texture
                    .upload_data(device, &mut encoder, rgba_image.as_raw());
                queue.submit(Some(encoder.finish()));
            }
        }
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

    draw.to_frame(app, &frame).unwrap();
}
