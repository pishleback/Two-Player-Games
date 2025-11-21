use eframe::wgpu::{self, util::DeviceExt};
use egui::{Pos2, Rect};
use std::{
    num::NonZeroU64,
    sync::{Arc, Mutex},
};

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x2, // position
        1 => Float32x4, // color
        2 => Float32x2, // tex_coords
    ];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[derive(Debug)]
struct VisiblePart {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl VisiblePart {
    fn new(rect: Rect, viewport: Rect) -> Self {
        let intersection_rect = rect.intersect(viewport);
        fn frac(range: (f32, f32), value: f32) -> f32 {
            (value - range.0) / (range.1 - range.0)
        }
        Self {
            min_x: frac((rect.min.x, rect.max.x), intersection_rect.min.x),
            max_x: frac((rect.min.x, rect.max.x), intersection_rect.max.x),
            min_y: frac((rect.min.y, rect.max.y), intersection_rect.min.y),
            max_y: frac((rect.min.y, rect.max.y), intersection_rect.max.y),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Key {
    ppp: f32,
    texture_size: (u32, u32),
}

struct RenderTexturePipeline {
    ctx: egui::Context,
    wgpu_ctx: egui_wgpu::RenderState,
    key: Key,
    texture_view: wgpu::TextureView,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
}

impl RenderTexturePipeline {
    fn new_with_size(
        ctx: &egui::Context,
        wgpu_ctx: &egui_wgpu::RenderState,
        texture_size: (u32, u32),
    ) -> Self {
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: texture_size.0,
                height: texture_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            label: None,
            view_formats: &[],
        };
        let texture: wgpu::Texture = wgpu_ctx.device.create_texture(&texture_desc);
        let texture_view: wgpu::TextureView = texture.create_view(&Default::default());
        let texture_sampler = wgpu_ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let device = &wgpu_ctx.device;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("custom3d"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let vertices = [
            Vertex {
                position: [-1.0, -1.0],
                color: [0.0, 0.0, 0.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0],
                color: [1.0, 0.0, 0.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [-1.0, 1.0],
                color: [0.0, 1.0, 0.0, 1.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0],
                color: [1.0, 1.0, 0.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Triangle Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("custom3d"),
            contents: bytemuck::cast_slice(&[0.0_f32; 4]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("custom3d"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(16),
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("custom3d"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("custom3d"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu_ctx.target_format.into())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("custom3d"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            ctx: ctx.clone(),
            wgpu_ctx: wgpu_ctx.clone(),
            key: Key {
                ppp: ctx.pixels_per_point(),
                texture_size,
            },
            texture_view,
            pipeline,
            bind_group,
            vertex_buffer,
            uniform_buffer,
        }
    }

    fn set_rect(&mut self, rect: Rect) {
        let ppp = self.ctx.pixels_per_point();
        let texture_size = ((rect.width() * ppp) as u32, (rect.height() * ppp) as u32);
        let key = Key { ppp, texture_size };
        if self.key != key {
            *self = Self::new_with_size(&self.ctx, &self.wgpu_ctx, texture_size)
        }
    }

    fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, visible_part: &VisiblePart) {
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[
                visible_part.min_x,
                visible_part.min_y,
                visible_part.max_x,
                visible_part.max_y,
            ]),
        );
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..4, 0..1);
    }

    fn render_to_texture(
        &self,
        fill_colour: egui::Color32,
        render: impl FnOnce(
            &egui_wgpu::RenderState,
            &mut wgpu::RenderPass,
            (u32, u32),
            wgpu::TextureFormat,
            wgpu::TextureFormat,
        ),
    ) {
        let size = self.texture_view.texture().size();
        let depth_texture_desc = wgpu::TextureDescriptor {
            label: Some("TextureDescriptor"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let depth_texture = self.wgpu_ctx.device.create_texture(&depth_texture_desc);
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .wgpu_ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let render_pass_desc = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: (fill_colour.r() as f64) / 255.0,
                        g: (fill_colour.g() as f64) / 255.0,
                        b: (fill_colour.b() as f64) / 255.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        };

        {
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
            render(
                &self.wgpu_ctx,
                &mut render_pass,
                (size.width, size.height),
                wgpu::TextureFormat::Rgba8UnormSrgb,
                DEPTH_FORMAT,
            );
        }

        self.wgpu_ctx.queue.submit(Some(encoder.finish()));
    }
}

struct CustomCallback {
    visible_part: VisiblePart,
    pipeline: Arc<Mutex<RenderTexturePipeline>>,
}

impl egui_wgpu::CallbackTrait for CustomCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        _resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        self.pipeline
            .lock()
            .unwrap()
            .prepare(device, queue, &self.visible_part);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        _resources: &egui_wgpu::CallbackResources,
    ) {
        self.pipeline.lock().unwrap().paint(render_pass);
    }
}

pub struct RenderTextureWidget {
    ctx: egui::Context,
    rect: egui::Rect,
    pipeline: Arc<Mutex<RenderTexturePipeline>>,
}

impl RenderTextureWidget {
    pub fn new(ctx: &egui::Context, frame: &eframe::Frame) -> Self {
        let wgpu_ctx: &egui_wgpu::RenderState = frame.wgpu_render_state.as_ref().unwrap();
        Self {
            ctx: ctx.clone(),
            rect: Rect {
                min: Pos2 { x: 0.0, y: 0.0 },
                max: Pos2 { x: 0.0, y: 0.0 },
            },
            pipeline: Arc::new(Mutex::new(RenderTexturePipeline::new_with_size(
                ctx,
                wgpu_ctx,
                (1, 1),
            ))),
        }
    }

    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
        self.pipeline.lock().unwrap().set_rect(rect);
    }

    pub fn render_to_texture(
        &self,
        fill_colour: egui::Color32,
        render: impl FnOnce(
            &egui_wgpu::RenderState,
            &mut wgpu::RenderPass,
            (u32, u32),
            wgpu::TextureFormat,
            wgpu::TextureFormat,
        ),
    ) {
        self.pipeline
            .lock()
            .unwrap()
            .render_to_texture(fill_colour, render);
    }

    pub fn add(&self, ui: &egui::Ui) {
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            self.rect,
            CustomCallback {
                visible_part: VisiblePart::new(self.rect, self.ctx.viewport_rect()),
                pipeline: self.pipeline.clone(),
            },
        ));
    }
}
