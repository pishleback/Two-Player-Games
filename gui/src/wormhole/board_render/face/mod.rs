use eframe::egui_wgpu::{
    self,
    wgpu::{self, util::DeviceExt as _},
};
use std::num::NonZeroU64;

use crate::wormhole::board_render::BoardParams;

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
        uniforms: &super::Uniforms,
    ) -> Self {
        let device = &wgpu_ctx.device;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let side_offset = board_params.side_length * 0.5;
        let vertices = [
            Vertex {
                position: [-side_offset, -side_offset, -board_params.face_offset],
                color: [0.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [side_offset, -side_offset, -board_params.face_offset],
                color: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-side_offset, side_offset, -board_params.face_offset],
                color: [0.0, 1.0, 0.0, 1.0],
            },
            Vertex {
                position: [side_offset, side_offset, -board_params.face_offset],
                color: [1.0, 1.0, 0.0, 1.0],
            },
            Vertex {
                position: [-side_offset, -side_offset, board_params.face_offset],
                color: [0.0, 0.0, 1.0, 1.0],
            },
            Vertex {
                position: [side_offset, -side_offset, board_params.face_offset],
                color: [1.0, 0.0, 1.0, 1.0],
            },
            Vertex {
                position: [-side_offset, side_offset, board_params.face_offset],
                color: [0.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: [side_offset, side_offset, board_params.face_offset],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ];

        let indices: &[u16] = &[
            0, 1, 3, 3, 2, 0, // bottom
            4, 5, 7, 7, 6, 4, // top
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

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(std::mem::size_of::<super::Uniforms>() as _),
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
                buffers: &[Vertex::desc()],
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
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
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

    pub fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, uniforms : &super::Uniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(uniforms));
    }

    pub fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        // Draw the cube
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        //  render_pass.draw(0..8, 0..1);
    }
}
