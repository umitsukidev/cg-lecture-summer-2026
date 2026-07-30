use nannou::{image::RgbaImage, prelude::*};
use rayon::prelude::*;
use std::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

pub struct Model {
    _window: Entity,
    texture: Handle<Image>,
    pixel_rx: Mutex<Receiver<Vec<u8>>>,
    pixel_tx: Sender<Vec<u8>>,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let window = app
        .new_window()
        .primary()
        .size(1200, 900)
        .resizable(false)
        .view(view)
        .build();
    app.set_update_rate(60.0);

    let image_buffer = RgbaImage::new(1200, 900);

    let dynamic_image = nannou::image::DynamicImage::ImageRgba8(image_buffer.clone());
    let mut image = Image::from_dynamic(
        dynamic_image,
        true,
        bevy_asset::RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::linear();
    let texture = app.asset_server().add(image);

    let (pixel_tx, pixel_rx) = std::sync::mpsc::channel();

    Model {
        _window: window,
        texture,
        pixel_rx: Mutex::new(pixel_rx),
        pixel_tx,
    }
}

fn update(app: &App, model: &mut Model) {
    let window_rect = app.window_rect();

    let width = window_rect.w() as u32;
    let height = window_rect.h() as u32;

    let raw_pixels = {
        let mut pixels = model
            .pixel_rx
            .lock()
            .unwrap()
            .try_recv()
            .unwrap_or_else(|_| vec![0; (width * height * 4) as usize]);
        let expected_size = (width * height * 4) as usize;
        if pixels.len() != expected_size {
            pixels.resize(expected_size, 0);
        }
        pixels
    };

    let mut image_buffer = RgbaImage::from_raw(width, height, raw_pixels).unwrap();

    image_buffer
        .as_flat_samples_mut()
        .samples
        .par_chunks_mut(4)
        .enumerate()
        .for_each(|(_, chunk)| {
            // let x = index % width as usize;
            // let y = index / width as usize;

            let pixel = [0, 0, 0, 0];

            chunk[0] = pixel[0];
            chunk[1] = pixel[1];
            chunk[2] = pixel[2];
            chunk[3] = pixel[3];
        });

    let pixels = image_buffer.into_raw();
    let tx = model.pixel_tx.clone();
    app.modify_image(&model.texture, move |image| {
        if let Some(old_pixels) = image.data.take() {
            let _ = tx.send(old_pixels);
        }
        image.data = Some(pixels);
    });
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();

    let window_rect = app.window_rect();

    draw.rect().wh(window_rect.wh()).texture(&model.texture);
}
