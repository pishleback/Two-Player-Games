use eframe::{
    egui_wgpu::{
        self,
        wgpu::{self, util::DeviceExt as _},
    },
    wgpu::Extent3d,
};
use egui::Color32;
use glam::{Mat4, Quat, Vec3};
use std::num::NonZeroU64;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_mat: [[f32; 4]; 4],
    proj_mat: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x4, // color
    ];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    model_mat: [[f32; 4]; 4],
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        2 => Float32x4,
        3 => Float32x4,
        4 => Float32x4,
        5 => Float32x4,
    ];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct RenderCubePipeline {
    wgpu_ctx: egui_wgpu::RenderState,
    pixels_size: (u32, u32),
    fill_colour: egui::Color32,
    colour_texture: wgpu::Texture,
    depth_texture: wgpu::Texture,
    uniforms: Uniforms,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
}

impl RenderCubePipeline {
    pub fn new(
        wgpu_ctx: &egui_wgpu::RenderState,
        pixels_size: (u32, u32),
        colour_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let colour_texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: pixels_size.0,
                height: pixels_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: colour_format,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            label: None,
            view_formats: &[],
        };
        let colour_texture: wgpu::Texture = wgpu_ctx.device.create_texture(&colour_texture_desc);
        let colour_texture_view =
            colour_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture_desc = wgpu::TextureDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            size: Extent3d {
                width: pixels_size.0,
                height: pixels_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: depth_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let depth_texture = wgpu_ctx.device.create_texture(&depth_texture_desc);

        let device = &wgpu_ctx.device;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let vertices = [
            Vertex {
                position: [-1.0, -1.0, -1.0],
                color: [0.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0, -1.0],
                color: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-1.0, 1.0, -1.0],
                color: [0.0, 1.0, 0.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0, -1.0],
                color: [1.0, 1.0, 0.0, 1.0],
            },
            Vertex {
                position: [-1.0, -1.0, 1.0],
                color: [0.0, 0.0, 1.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0, 1.0],
                color: [1.0, 0.0, 1.0, 1.0],
            },
            Vertex {
                position: [-1.0, 1.0, 1.0],
                color: [0.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ];

        let indices: &[u16] = &[
            0, 1, 3, 3, 2, 0, // bottom
            4, 5, 7, 7, 6, 4, // top
            0, 4, 6, 6, 2, 0, // left
            1, 5, 7, 7, 3, 1, // right
            0, 1, 5, 5, 4, 0, // front
            2, 3, 7, 7, 6, 2, // back
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = indices.len() as u32;

        let mut instances = vec![];
        for x in [-1.0, 0.0, 1.0] {
            for y in [-1.0, 0.0, 1.0] {
                instances.push(Instance {
                    model_mat: glam::Mat4::from_translation(Vec3 { x: x, y: y, z: 0.0 })
                        .to_cols_array_2d(),
                });
            }
        }
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(128),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), Instance::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: colour_texture_view.texture().format(),
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let uniforms = Uniforms {
            view_mat: glam::Mat4::ZERO.to_cols_array_2d(),
            proj_mat: glam::Mat4::ZERO.to_cols_array_2d(),
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            contents: &bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            wgpu_ctx: wgpu_ctx.clone(),
            pixels_size,
            fill_colour: Color32::PURPLE,
            colour_texture,
            depth_texture,
            uniforms,
            pipeline,
            bind_group,
            vertex_buffer,
            index_buffer,
            num_indices,
            instances,
            instance_buffer,
            uniform_buffer,
        }
    }

    pub fn colour_texture_view(&self) -> wgpu::TextureView {
        self.colour_texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn depth_texture_view(&self) -> wgpu::TextureView {
        self.depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        let projection = glam::Mat4::perspective_lh(
            0.7,
            (std::cmp::max(self.pixels_size.0, 1) as f32)
                / (std::cmp::max(self.pixels_size.1, 1) as f32),
            8.0,
            12.0,
        );
        let view = Mat4::look_to_lh(
            Vec3::from_array([0.0, 0.0, -10.0]),
            Vec3::from_array([0.0, 0.0, 1.0]),
            Vec3::from_array([0.0, 1.0, 0.0]),
        );
        self.instances = vec![];
        for x in [-2.0, 0.0, 2.0] {
            for y in [-2.0, 0.0, 2.0] {
                self.instances.push(Instance {
                    model_mat: (glam::Mat4::from_translation(Vec3 { x: x, y: y, z: 0.0 })
                        * glam::Mat4::from_quat(rotation))
                    .to_cols_array_2d(),
                });
            }
        }
        self.instance_buffer =
            self.wgpu_ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(wgpu_widgets::wgpu_label!()),
                    contents: bytemuck::cast_slice(&self.instances),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        self.uniforms = Uniforms {
            view_mat: view.to_cols_array_2d(),
            proj_mat: projection.to_cols_array_2d(),
        };
    }

    pub fn set_fill_colour(&mut self, fill_colour: Color32) {
        self.fill_colour = fill_colour;
    }

    pub fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue) {
        // Update uniform buffer with the angle from the UI
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&self.uniforms));
    }

    pub fn render(&self) {
        let mut encoder = self
            .wgpu_ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(wgpu_widgets::wgpu_label!()),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.colour_texture_view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: (self.fill_colour.r() as f64) / 255.0,
                            g: (self.fill_colour.g() as f64) / 255.0,
                            b: (self.fill_colour.b() as f64) / 255.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Draw the cube
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);
            //  render_pass.draw(0..8, 0..1);
        }

        self.wgpu_ctx.queue.submit(Some(encoder.finish()));
    }
}
