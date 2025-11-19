use std::num::NonZeroU64;

use eframe::{
    egui_wgpu::{
        self,
        wgpu::{self, util::DeviceExt as _},
    },
    wgpu::TextureFormat,
};
use glam::{Mat4, Quat, Vec3};

use crate::root::AppState;

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

pub struct State {
    rotation: Quat,
}

impl State {
    pub fn new(frame: &eframe::Frame) -> Option<Self> {
        let ctx = frame.wgpu_render_state.as_ref()?;

        let x = CubeRenderer::new(ctx, ctx.target_format);

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our `Custom3D` struct, we insert it into the
        // `paint_callback_resources` type map, which is stored alongside the render pass.
        ctx.renderer.write().callback_resources.insert(x);

        Some(Self {
            rotation: Quat::IDENTITY,
        })
    }
}

impl AppState for State {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) -> Option<Box<dyn AppState>> {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink(false)
                .show(ui, |ui| {
                     if ui.button("Back").clicked() {
                            return Some(Box::new(crate::menu::State::default()) as Box<dyn AppState>);
                        }

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("The triangle is being painted using ");
                        ui.hyperlink_to("WGPU", "https://wgpu.rs");
                        ui.label(" (Portable Rust graphics API awesomeness)");
                    });
                    ui.label("It's not a very impressive demo, but it shows you can embed 3D inside of egui.");

                    egui::Frame::canvas(ui.style()).show(ui, |ui| {
                        self.custom_painting(ui);
                    });
                    ui.label("Drag to rotate!");

                    None
                }).inner
        }).inner
    }
}



pub struct CubeRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    uniform_buffer: wgpu::Buffer,
}

impl CubeRenderer {
    pub fn new(ctx: &egui_wgpu::RenderState, target_format: TextureFormat) -> Self {
        let device = &ctx.device;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("custom3d"),
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
            label: Some("Triangle Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = indices.len() as u32;

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("custom3d"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(64),
                },
                count: None,
            }],
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
                // targets: &[Some(ctx.target_format.into())],
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("custom3d"),
            contents: bytemuck::cast_slice(&[0.0_f32; 16]), // 16 bytes aligned!
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("custom3d"),
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

    pub fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, rotation: Quat) {
        let projection = glam::Mat4::perspective_lh(std::f32::consts::FRAC_PI_2, 1.0, 0.1, 10.0);
        let view = Mat4::look_to_lh(
            Vec3::from_array([0.0, 0.0, -4.0]),
            Vec3::from_array([0.0, 0.0, 1.0]),
            Vec3::from_array([0.0, 1.0, 0.0]),
        );
        let model = Mat4::from_quat(rotation);

        let mat = (projection * view * model).to_cols_array();

        // Update our uniform buffer with the angle from the UI
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&mat));
    }

    pub fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        // Draw our triangle!
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        //  render_pass.draw(0..8, 0..1);
    }
}
