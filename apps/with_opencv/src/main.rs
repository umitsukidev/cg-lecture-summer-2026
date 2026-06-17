mod opencv_utils;

use nannou::prelude::*;
use opencv::{prelude::*, videoio};

use opencv_utils::MatExt;

struct Model {
    cam: videoio::VideoCapture,
    texture: wgpu::Texture,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    // カメラの初期化（デフォルトカメラ）
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_AVFOUNDATION)
        .expect("Unable to open camera with AVFoundation");

    // カメラが正しく開けたか確認
    let opened =
        videoio::VideoCapture::is_opened(&cam).expect("Failed to check if camera is opened");
    if !opened {
        panic!("Camera is not opened!");
    }

    // 最初のフレームを読み込んで初期の画像サイズを取得
    let mut frame = opencv::core::Mat::default();
    cam.read(&mut frame).expect("Failed to read initial frame");

    let size = frame.size().expect("Failed to get frame size");
    let width = size.width as u32;
    let height = size.height as u32;

    // Nannouのウィンドウサイズをカメラ画像に合わせる
    let _window = app
        .new_window()
        .size(width, height)
        .view(view)
        .build()
        .unwrap();

    // 初期テクスチャの生成
    let rgba_image = frame
        .to_rgba_image()
        .expect("Failed to convert initial frame to RgbaImage");
    let dynamic_image = nannou::image::DynamicImage::ImageRgba8(rgba_image);
    let texture = wgpu::Texture::from_image(app, &dynamic_image);

    Model { cam, texture }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let mut frame = opencv::core::Mat::default();
    // カメラからフレームを読み込み、正常に取得できたらテクスチャを更新
    if let Ok(true) = model.cam.read(&mut frame) {
        if frame.size().map(|s| s.width > 0).unwrap_or(false) {
            if let Ok(rgba_image) = frame.to_rgba_image() {
                let window = app.main_window();
                let device = window.device();
                let queue = window.queue();

                // テクスチャのアップロード用コマンドエンコーダを作成
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Texture Upload"),
                });

                // 新しいフレームデータをGPU上のテクスチャにアップロード
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
    // テクスチャをウィンドウいっぱいに描画
    draw.texture(&model.texture);
    draw.to_frame(app, &frame).unwrap();
}
