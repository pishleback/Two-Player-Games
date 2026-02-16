use eframe::wgpu::{self, Extent3d};
use egui::Color32;
use glam::{Mat4, Quat, Vec3};

use crate::wormhole::board;

mod face;
mod tube;

pub struct BoardParams {
    side_length: f32,
    face_offset: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    mat: [[f32; 4]; 4],
    side_length: f32,
    face_offset: f32,
    _padding: [f32; 2], // To make as a multiple of 16 bytes
    colours: [[f32; 4]; 144],
}

pub struct Pipeline {
    wgpu_ctx: egui_wgpu::RenderState,
    pixels_size: (u32, u32),
    fill_colour: egui::Color32,
    uniforms: Uniforms,
    colour_texture: wgpu::Texture,
    depth_texture: wgpu::Texture,
    face_pipeline: face::Pipeline,
    tube_pipeline: tube::Pipeline,
}

impl Pipeline {
    pub fn new(
        wgpu_ctx: &egui_wgpu::RenderState,
        pixels_size: (u32, u32),
        colour_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let board_params = BoardParams {
            side_length: 11.0,
            face_offset: 1.4,
        };

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
        let colour_texture = wgpu_ctx.device.create_texture(&colour_texture_desc);

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

        let mut uniforms = Uniforms {
            mat: Default::default(),
            side_length: board_params.side_length,
            face_offset: board_params.face_offset,
            colours: [[0.0, 0.0, 0.0, 1.0]; 144],
            _padding: Default::default(),
        };

        let face_pipeline = face::Pipeline::new(
            wgpu_ctx,
            &board_params,
            colour_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            depth_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            &uniforms,
        );

        let tube_pipeline = tube::Pipeline::new(
            wgpu_ctx,
            &board_params,
            colour_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            depth_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            &uniforms,
        );

        Self {
            wgpu_ctx: wgpu_ctx.clone(),
            pixels_size,
            fill_colour: Color32::PURPLE,
            colour_texture,
            depth_texture,
            uniforms,
            face_pipeline,
            tube_pipeline,
        }
    }

    pub fn set_selected(&mut self, pos: &super::board::Pos) {
        self.uniforms.colours = std::array::from_fn(|n| {
            let n = n as u8;
            let p = board::Pos { n };

            // match p.get_type() {
            //     board::PosType::Top => [1.0, 0.0, 0.0, 1.0],
            //     board::PosType::Bottom => [0.0, 1.0, 0.0, 1.0],
            //     board::PosType::HoleTop => [1.0, 0.0, 1.0, 1.0],
            //     board::PosType::HoleBottom => [0.0, 1.0, 1.0, 1.0],
            //     board::PosType::HolePent => [1.0, 1.0, 0.0, 1.0],
            // }

            // // if i.is_multiple_of(2) {
            // //     [0.0, 0.0, 0.0, 1.0]
            // // } else {
            // //     [1.0, 1.0, 1.0, 1.0]
            // // }
            [1.0, 1.0, 1.0, 1.0]
        });

        // println!("{:?}", pos.orbit());

        self.uniforms.colours[pos.n as usize] = [0.0, 0.0, 0.0, 1.0];
        self.uniforms.colours[pos.flip_x().n as usize] = [1.0, 0.0, 0.0, 1.0];
        self.uniforms.colours[pos.flip_y().n as usize] = [0.0, 1.0, 0.0, 1.0];
        self.uniforms.colours[pos.flip_z().n as usize] = [0.0, 0.0, 1.0, 1.0];
        self.uniforms.colours[pos.flip_xy().n as usize] = [1.0, 1.0, 0.0, 1.0];
    }

    pub fn colour_texture_view(&self) -> wgpu::TextureView {
        self.colour_texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn depth_texture_view(&self) -> wgpu::TextureView {
        self.depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn set_fill_colour(&mut self, fill_colour: Color32) {
        self.fill_colour = fill_colour;
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        let projection = glam::Mat4::perspective_lh(
            0.7,
            (std::cmp::max(self.pixels_size.0, 1) as f32)
                / (std::cmp::max(self.pixels_size.1, 1) as f32),
            0.1,
            100.0,
        );
        let view = Mat4::look_to_lh(
            Vec3::from_array([0.0, 0.0, -20.0]),
            Vec3::from_array([0.0, 0.0, 1.0]),
            Vec3::from_array([0.0, 1.0, 0.0]),
        );
        let model = Mat4::from_quat(rotation);

        self.uniforms.mat = (projection * view * model).to_cols_array_2d();
    }

    pub fn prepare(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.face_pipeline.prepare(device, queue, &self.uniforms);
        self.tube_pipeline.prepare(device, queue, &self.uniforms);
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

            self.face_pipeline.paint(&mut render_pass);
            self.tube_pipeline.paint(&mut render_pass);
        }

        self.wgpu_ctx.queue.submit(Some(encoder.finish()));
    }
}
