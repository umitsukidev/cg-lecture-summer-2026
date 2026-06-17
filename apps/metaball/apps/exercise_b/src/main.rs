use nannou::image::{Rgba, RgbaImage};
use nannou::prelude::*;

struct Model {
    texture: wgpu::Texture,
    image_buffer: RgbaImage,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let _window = app.new_window().size(512, 512).view(view).build().unwrap();

    let width = 512;
    let height = 512;
    let image_buffer = RgbaImage::new(width, height);

    let dynamic_image = nannou::image::DynamicImage::ImageRgba8(image_buffer.clone());
    let texture = wgpu::Texture::from_image(app, &dynamic_image);

    Model {
        texture,
        image_buffer,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let width = model.image_buffer.width();
    let height = model.image_buffer.height();

    for x in 0..width {
        for y in 0..height {
            let pixel = pt2(x as f32, y as f32);
            let distance = (pixel - pt2(width as f32, height as f32) / 2.0).length();
            let value = if distance < 100.0 {
                (1.0 - distance / 100.0).powf(2.0) * 255.0
            } else {
                0.0
            };
            let value_u8 = value as u8;
            model
                .image_buffer
                .put_pixel(x, y, Rgba([value_u8, value_u8, value_u8, 255]));
        }
    }

    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Texture Upload"),
    });
    model
        .texture
        .upload_data(device, &mut encoder, model.image_buffer.as_raw());
    queue.submit(Some(encoder.finish()));
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.texture(&model.texture);
    draw.to_frame(app, &frame).unwrap();
}
