use crate::wormhole::board_render::BoardParams;
use eframe::egui_wgpu::{
    self,
    wgpu::{self, util::DeviceExt as _},
};
use std::num::NonZeroU64;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![
        0 => Float32x3, // position
    ];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[derive(Clone)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    uniform_buffer: wgpu::Buffer,
}

fn compute_tube_model(board_params: &BoardParams) -> (Vec<Vertex>, Vec<u32>) {
    let scale = board_params.side_length / 8.0;

    let num_ribs = 36usize;
    let num_rings = 24;

    debug_assert!(num_ribs >= 2);

    let pi = std::f32::consts::PI;
    let tau = std::f32::consts::TAU;
    let sqrt5 = (5.0 as f32).sqrt();

    let mut vertices: Vec<Vertex> = vec![];
    let mut indices = vec![];

    let mut first = true;
    // `r` is the distance is squares from the central z-axis
    // `z` is the height
    for (r, z) in [(sqrt5 + 1.0, 1.0), (sqrt5, 1.0)]
        .into_iter()
        .chain((0..num_rings).map(|i| {
            let angle = pi * ((i + 1) as f32) / ((num_rings + 1) as f32);
            (sqrt5 - angle.sin() * board_params.hole_offset, angle.cos())
        }))
        .chain([(sqrt5, -1.0), (sqrt5 + 1.0, -1.0)].into_iter())
    {
        for i in 0..num_ribs {
            // Compute the vertex
            let angle = tau * (i as f32) / (num_ribs as f32);
            let x = r * angle.cos();
            let y = r * angle.sin();
            vertices.push(Vertex {
                position: [
                    scale * (x as f32),
                    scale * (y as f32),
                    board_params.face_offset * (z as f32),
                ],
            });
        }
        if !first {
            // Fill in a square using the 2 triangles (a, b, c) and (b, c, d)
            for i in 0..num_ribs {
                let j = (i + 1) % num_ribs;
                // adjacent vertivies i and j of the ring we just made
                let a = (vertices.len() - num_ribs + i) as _;
                let b = (vertices.len() - num_ribs + j) as _;
                // adjacent verticies i and j of the previous ring
                let c = (vertices.len() - 2 * num_ribs + i) as _;
                let d = (vertices.len() - 2 * num_ribs + j) as _;
                indices.extend_from_slice(&[a, b, c, b, c, d]);
            }
        }
        first = false;
    }
    debug_assert!(vertices.len() < u32::MAX as usize);
    (vertices, indices)
}

impl Pipeline {
    pub fn new(
        wgpu_ctx: &egui_wgpu::RenderState,
        board_params: &BoardParams,
        colour_texture_view: wgpu::TextureView,
        depth_texture_view: wgpu::TextureView,
        depth_peel_view: wgpu::TextureView,
        blend: Option<wgpu::BlendState>,
        uniforms: &super::Uniforms,
    ) -> Self {
        let device = &wgpu_ctx.device;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let (vertices, indices) = compute_tube_model(board_params);

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
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}
