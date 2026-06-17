mod ball;

use nannou::prelude::*;

use crate::ball::Ball;

fn main() {
    nannou::app(model).update(update).run();
}

use std::cell::RefCell;

struct Model {
    window_id: window::Id,
    balls: Vec<ball::Ball>,
    renderer: RefCell<nannou::draw::Renderer>,
    texture: wgpu::Texture,
    _texture_view: wgpu::TextureView,
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .size(1024, 1024)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .msaa_samples(1)
        .build()
        .unwrap();

    let balls = vec![];

    let window = app.window(window_id).unwrap();
    let device = window.device();

    // Create the offscreen render target texture
    let texture = wgpu::TextureBuilder::new()
        .size([1024, 1024])
        .format(Frame::TEXTURE_FORMAT)
        .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
        .sample_count(1)
        .build(device);

    let texture_view = texture.view().build();

    // Create the custom renderer for drawing to the texture
    let renderer = RefCell::new(nannou::draw::RendererBuilder::new().build(
        device,
        [1024, 1024],
        window.scale_factor(),
        1,
        Frame::TEXTURE_FORMAT,
    ));

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Metaball"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/metaball.wgsl").into()),
    });

    let sampler = wgpu::SamplerBuilder::new().build(device);

    let bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
        .texture_from(wgpu::ShaderStages::FRAGMENT, &texture)
        .sampler(wgpu::ShaderStages::FRAGMENT, true)
        .build(device);

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Metaball pipeline descriptor"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &shader)
        .fragment_shader(&shader)
        .primitive_topology(wgpu::PrimitiveTopology::TriangleList)
        .vertex_entry_point("vs_main")
        .fragment_entry_point("fs_main")
        .color_format(Frame::TEXTURE_FORMAT)
        .build(device);

    let bind_group = wgpu::BindGroupBuilder::new()
        .texture_view(&texture_view)
        .sampler(&sampler)
        .build(device, &bind_group_layout);

    Model {
        window_id,
        balls,
        renderer,
        texture,
        _texture_view: texture_view,
        render_pipeline,
        bind_group,
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {
    let window = app.window(model.window_id).unwrap();
    let device = window.device();

    // 1. Draw original balls display logic using CPU draw to the offscreen texture
    let draw = app.draw();
    draw.background().color(WHITE);

    for ball in &model.balls {
        draw.ellipse()
            .xy(ball.position)
            .radius(ball.radius)
            .color(BLACK);
    }

    let mut encoder = frame.command_encoder();
    model
        .renderer
        .borrow_mut()
        .render_to_texture(device, &mut encoder, &draw, &model.texture);

    // 2. Begin the screen pass to render our full-screen triangle post-process shader
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Metaball Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: frame.texture_view(),
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                store: true,
            },
        })],
        depth_stencil_attachment: None,
    });

    render_pass.set_pipeline(&model.render_pipeline);
    render_pass.set_bind_group(0, &model.bind_group, &[]);
    render_pass.draw(0..3, 0..1);
}

fn mouse_pressed(app: &App, model: &mut Model, button: MouseButton) {
    match button {
        MouseButton::Left => model.balls.push(Ball {
            position: app.mouse.position(),
            radius: 50.0,
        }),
        _ => {}
    }
}
