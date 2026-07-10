mod camera;
mod material;
mod scene;
mod sphere;

use crate::{
    camera::Camera,
    material::{GpuMaterial, Material},
    scene::create_scene,
    sphere::{GpuSphere, Sphere},
};
use nannou::prelude::*;
use nannou::wgpu::util::DeviceExt;
use std::sync::Arc;

#[derive(Clone)]
struct Model {
    _window_id: Entity,
    // wgpu assets (wrapped in Arc for Clone derivation)
    config_buffer: Arc<wgpu::Buffer>,

    compute_pipeline: Arc<wgpu::ComputePipeline>,
    compute_bind_group: Arc<wgpu::BindGroup>,

    pipeline_layout: Arc<wgpu::PipelineLayout>,
    shader: Arc<wgpu::ShaderModule>,
    render_pipeline: Arc<std::sync::OnceLock<wgpu::RenderPipeline>>,
    render_bind_group: Arc<wgpu::BindGroup>,

    // Scene data
    _camera: Camera,
    _spheres: Vec<Sphere>,
    _environment: Material,
    frame_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
struct GpuConfig {
    environment: GpuMaterial,
    frame_count: u32,
    sphere_count: u32,
    pad0: u32,
    pad1: u32,
}

fn main() {
    nannou::app(model).update(update).render(render).run();
}

fn model(app: &App) -> Model {
    let window_id = app.new_window::<Model>().size(512, 512).build();

    let window = app.window(window_id);
    let device = window.device();

    let (camera, environment, spheres) = create_scene();

    // 1. Create Output Texture
    let texture = wgpu::TextureBuilder::new()
        .size([512, 512])
        .format(wgpu::TextureFormat::Rgba8Unorm)
        .usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING)
        .sample_count(1)
        .build(device);
    let texture_view = texture.view().build();

    // 2. Create Buffers
    let camera_uniform = camera.to_uniform();
    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Uniform Buffer"),
        contents: bytemuck::bytes_of(&camera_uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let gpu_spheres: Vec<GpuSphere> = spheres.iter().map(|s| s.to_gpu()).collect();
    let spheres_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Spheres Storage Buffer"),
        contents: bytemuck::cast_slice(&gpu_spheres),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    let config = GpuConfig {
        environment: environment.to_gpu(),
        frame_count: 0,
        sphere_count: spheres.len() as u32,
        pad0: 0,
        pad1: 0,
    };
    let config_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Config Uniform Buffer"),
        contents: bytemuck::bytes_of(&config),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let accum_size = 512 * 512 * std::mem::size_of::<[f32; 4]>();
    let accumulation_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Accumulation Storage Buffer"),
        size: accum_size as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Load Shader
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Ray Tracer Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/ray_tracer.wgsl").into()),
    });

    // 3. Compute Pipeline Setup
    let compute_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Bind Group Layout"),
            entries: &[
                // Camera
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Spheres
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Config
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Accumulation
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Output Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Compute Pipeline Layout"),
        bind_group_layouts: &[Some(&compute_bind_group_layout)],
        ..Default::default()
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Compute Bind Group"),
        layout: &compute_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: spheres_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: config_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: accumulation_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
        ],
    });

    // 4. Render Pipeline Setup
    let sampler = wgpu::SamplerBuilder::new().build(device);

    let render_bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
        .texture_from(wgpu::ShaderStages::FRAGMENT, &texture)
        .sampler(wgpu::ShaderStages::FRAGMENT, true)
        .build(device);

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[Some(&render_bind_group_layout)],
        ..Default::default()
    });

    let render_bind_group = wgpu::BindGroupBuilder::new()
        .texture_view(&texture_view)
        .sampler(&sampler)
        .build(device, &render_bind_group_layout);

    Model {
        _window_id: window_id,
        config_buffer: Arc::new(config_buffer),
        compute_pipeline: Arc::new(compute_pipeline),
        compute_bind_group: Arc::new(compute_bind_group),
        pipeline_layout: Arc::new(render_pipeline_layout),
        shader: Arc::new(shader),
        render_pipeline: Arc::new(std::sync::OnceLock::new()),
        render_bind_group: Arc::new(render_bind_group),
        _camera: camera,
        _spheres: spheres,
        _environment: environment,
        frame_count: 0,
    }
}

fn update(app: &App, model: &mut Model) {
    model.frame_count = app.elapsed_frames() as u32;

    let window = app.window(model._window_id);
    let queue = window.queue();
    let config = GpuConfig {
        environment: model._environment.to_gpu(),
        frame_count: model.frame_count,
        sphere_count: model._spheres.len() as u32,
        pad0: 0,
        pad1: 0,
    };
    queue.write_buffer(&model.config_buffer, 0, bytemuck::bytes_of(&config));
}

fn render(_app: &RenderApp, model: &Model, frame: Frame) {
    let device = frame.device();
    let mut encoder = frame.command_encoder();

    // 1. Dispatch Compute Shader
    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Ray Tracing Compute Pass"),
            ..Default::default()
        });
        compute_pass.set_pipeline(&model.compute_pipeline);
        compute_pass.set_bind_group(0, &*model.compute_bind_group, &[]);
        // Grid size: 512x512. Workgroup size: 16x16.
        compute_pass.dispatch_workgroups(32, 32, 1);
    }

    // 2. Render Fullscreen Quad to frame view
    let render_pipeline = model.render_pipeline.get_or_init(|| {
        wgpu::RenderPipelineBuilder::from_layout(&model.pipeline_layout, &model.shader)
            .fragment_shader(&model.shader)
            .primitive_topology(wgpu::PrimitiveTopology::TriangleList)
            .vertex_entry_point("vs_main")
            .fragment_entry_point("fs_main")
            .color_format(frame.texture_format())
            .build(device)
    });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Ray Tracing Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: frame.texture_view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        render_pass.set_pipeline(render_pipeline);
        render_pass.set_bind_group(0, &*model.render_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
