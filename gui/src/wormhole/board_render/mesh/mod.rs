use crate::wormhole::board_render::BoardParams;
use core::f32;
use eframe::egui_wgpu::{
    self,
    wgpu::{self, util::DeviceExt as _},
};
use glam::{Mat4, Vec3, Vec4};
use std::num::NonZeroU64;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub colour: [f32; 4],
    pub tex_uv: [f32; 2],
    pub tex_idx: f32,
    pub colour_to_tex: f32,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x4, // colour
        2 => Float32x2, // tex_uv
        3 => Float32, // tex_idx
        4 => Float32, // colour_to_tex
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    #[allow(unused)]
    pub fn empty() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
        }
    }

    pub fn union(
        mut self,
        Mesh {
            mut vertices,
            indices,
        }: Self,
    ) -> Self {
        let n = self.vertices.len() as u32;
        self.vertices.append(&mut vertices);
        for i in indices {
            self.indices.push(n + i);
        }
        self
    }

    pub fn transform(mut self, t: &glam::Mat4) -> Self {
        for vertex in self.vertices.iter_mut() {
            let p = vertex.position;
            let p = Vec4::new(p[0], p[1], p[2], 1.0);
            let p = t.mul_vec4(p);
            let p = [p.x, p.y, p.z];
            vertex.position = p;
        }
        self
    }
}

pub fn board_border(board_params: &BoardParams) -> Mesh {
    let radius = 0.1;
    let steps = 12;

    let mut vertices = vec![];
    for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        for i in 0..steps {
            let angle = f32::consts::TAU * (i as f32) / (steps as f32);
            let cos = angle.cos();
            let sin = angle.sin();
            vertices.push(Vertex {
                position: [
                    (0.5 * (board_params.side_length) + radius) * dx + radius * cos * dx,
                    (0.5 * (board_params.side_length) + radius) * dy + radius * cos * dy,
                    radius * sin,
                ],
                colour: [0.5, 0.5, 0.5, 1.0],
                tex_uv: Default::default(),
                tex_idx: Default::default(),
                colour_to_tex: 0.0,
            });
        }
    }
    let mut indices = vec![];
    for i in 0..4 {
        let j = (i + 1) % 4;
        for s in 0..steps {
            let t = (s + 1) % steps;
            let a = i * steps + s;
            let b = i * steps + t;
            let c = j * steps + s;
            let d = j * steps + t;
            indices.extend([a, b, c]);
            indices.extend([c, b, d]);
        }
    }

    let mesh = Mesh { vertices, indices };

    mesh.clone()
        .transform(&Mat4::from_translation(Vec3::new(
            0.0,
            0.0,
            board_params.face_offset,
        )))
        .union(mesh.transform(&Mat4::from_translation(Vec3::new(
            0.0,
            0.0,
            -board_params.face_offset,
        ))))
}

#[derive( Clone)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    uniform_buffer: wgpu::Buffer,
}

impl Pipeline {
    pub fn new(
        wgpu_ctx: &egui_wgpu::RenderState,
        board_params: &BoardParams,
        colour_texture_view: wgpu::TextureView,
        depth_texture_view: wgpu::TextureView,
        depth_peel_view: wgpu::TextureView,
        icons_texture_array: &wgpu::Texture,
        blend: Option<wgpu::BlendState>,
        uniforms: &super::Uniforms,
        meshes: Vec<Mesh>,
        include_border: bool,
    ) -> Self {
        let device = &wgpu_ctx.device;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let mut mesh = Mesh::empty();
        if include_border {
            mesh = mesh.union(board_border(board_params));
        }
        for m in meshes {
            mesh = mesh.union(m);
        }
        let Mesh { vertices, indices } = mesh;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = indices.len() as u32;

        let depth_peel_texture_sampler = wgpu_ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let icons_texture_view = icons_texture_array.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let icons_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(
                            std::mem::size_of::<super::Uniforms>() as _
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
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
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: colour_texture_view.texture().format(),
                    blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_texture_view.texture().format(),
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            contents: bytemuck::bytes_of(uniforms),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&depth_peel_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&depth_peel_texture_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&icons_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&icons_sampler),
                },
            ],
        });

        Self {
            pipeline,
            bind_group,
            vertex_buffer,
            index_buffer,
            num_indices,
            uniform_buffer,
        }
    }

    pub fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, uniforms: &super::Uniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(uniforms));
    }

    pub fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        if self.vertex_buffer.size() != 0 && self.index_buffer.size() != 0 {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
    }
}
