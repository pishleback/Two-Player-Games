use crate::wormhole::{board, board_render::BoardParams};
use eframe::wgpu::{self};
use egui::Color32;
use glam::Quat;
use wgpu_widgets::{
    texture_to_egui,
    widget::{VisiblePart, WgpuEguiRenderPipeline},
};

// Draw the board using depth-peeling for order-independent transparency
pub struct Pipeline {
    wgpu_ctx: egui_wgpu::RenderState,
    cube_pipeline_1: super::board_render::Pipeline,
    cube_pipeline_2: super::board_render::Pipeline,
    cube_pipeline_3: super::board_render::Pipeline,
    cube_pipeline_4: super::board_render::Pipeline,
    fill_colour: Color32,
    blit_texture_view: wgpu::TextureView,
    blit_pipeline_1: wgpu_widgets::blit::Pipeline,
    blit_pipeline_2: wgpu_widgets::blit::Pipeline,
    blit_pipeline_3: wgpu_widgets::blit::Pipeline,
    blit_pipeline_4: wgpu_widgets::blit::Pipeline,
    egui_pipeline: texture_to_egui::RenderTexturePipeline,
}

impl Pipeline {
    pub fn new(
        wgpu_ctx: &egui_wgpu::RenderState,
        pixels_size: (u32, u32),
        board_params: &BoardParams,
        icons_texture_array: &wgpu::Texture,
    ) -> Self {
        let cube_pipeline_1 = super::board_render::Pipeline::new(
            &wgpu_ctx,
            pixels_size,
            Color32::TRANSPARENT,
            &board_params,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Depth32Float,
            None,
            icons_texture_array,
            Some(wgpu::BlendState::REPLACE),
        );

        let cube_pipeline_2 = super::board_render::Pipeline::new(
            &wgpu_ctx,
            pixels_size,
            Color32::TRANSPARENT,
            &board_params,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Depth32Float,
            Some(cube_pipeline_1.depth_texture_view()),
            icons_texture_array,
            Some(wgpu::BlendState::REPLACE),
        );

        let cube_pipeline_3 = super::board_render::Pipeline::new(
            &wgpu_ctx,
            pixels_size,
            Color32::TRANSPARENT,
            &board_params,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Depth32Float,
            Some(cube_pipeline_2.depth_texture_view()),
            icons_texture_array,
            Some(wgpu::BlendState::REPLACE),
        );

        let cube_pipeline_4 = super::board_render::Pipeline::new(
            &wgpu_ctx,
            pixels_size,
            Color32::TRANSPARENT,
            &board_params,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Depth32Float,
            Some(cube_pipeline_3.depth_texture_view()),
            icons_texture_array,
            Some(wgpu::BlendState::REPLACE),
        );

        let blit_texture = wgpu_ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            size: wgpu::Extent3d {
                width: pixels_size.0,
                height: pixels_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,

            view_formats: &[],
        });
        let blit_texture_view = blit_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let blit_pipeline_1 = wgpu_widgets::blit::Pipeline::new(
            wgpu_ctx,
            pixels_size,
            cube_pipeline_4.colour_texture_view(),
            blit_texture_view.clone(),
        );

        let blit_pipeline_2 = wgpu_widgets::blit::Pipeline::new(
            wgpu_ctx,
            pixels_size,
            cube_pipeline_3.colour_texture_view(),
            blit_texture_view.clone(),
        );

        let blit_pipeline_3 = wgpu_widgets::blit::Pipeline::new(
            wgpu_ctx,
            pixels_size,
            cube_pipeline_2.colour_texture_view(),
            blit_texture_view.clone(),
        );

        let blit_pipeline_4 = wgpu_widgets::blit::Pipeline::new(
            wgpu_ctx,
            pixels_size,
            cube_pipeline_1.colour_texture_view(),
            blit_texture_view.clone(),
        );

        let egui_pipeline =
            texture_to_egui::RenderTexturePipeline::new(&wgpu_ctx, blit_texture_view.clone());

        Self {
            wgpu_ctx: wgpu_ctx.clone(),
            cube_pipeline_1,
            cube_pipeline_2,
            cube_pipeline_3,
            cube_pipeline_4,
            fill_colour: Color32::GREEN,
            blit_texture_view,
            blit_pipeline_1,
            blit_pipeline_2,
            blit_pipeline_3,
            blit_pipeline_4,
            egui_pipeline,
        }
    }

    pub fn set_selected(&mut self, pos: &board::Pos) {
        self.cube_pipeline_1.set_selected(pos);
        self.cube_pipeline_2.set_selected(pos);
        self.cube_pipeline_3.set_selected(pos);
        self.cube_pipeline_4.set_selected(pos);
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.cube_pipeline_1.set_rotation(rotation);
        self.cube_pipeline_2.set_rotation(rotation);
        self.cube_pipeline_3.set_rotation(rotation);
        self.cube_pipeline_4.set_rotation(rotation);
    }

    pub fn set_fill_colour(&mut self, fill_colour: Color32) {
        self.fill_colour = fill_colour;
    }
}

impl WgpuEguiRenderPipeline for Pipeline {
    fn prepare(&self, device: &wgpu::Device, queue: &wgpu::Queue, visible_part: &VisiblePart) {
        self.cube_pipeline_1.prepare(device, queue);
        self.cube_pipeline_2.prepare(device, queue);
        self.cube_pipeline_3.prepare(device, queue);
        self.cube_pipeline_4.prepare(device, queue);
        self.blit_pipeline_1.prepare(device, queue);
        self.blit_pipeline_2.prepare(device, queue);
        self.egui_pipeline.prepare(device, queue, visible_part);
    }

    fn paint(&self, wgpu_render_pass: &mut wgpu::RenderPass<'_>) {
        self.cube_pipeline_1.render();
        self.cube_pipeline_2.render();
        self.cube_pipeline_3.render();
        self.cube_pipeline_4.render();

        let mut encoder =
            self.wgpu_ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some(wgpu_widgets::wgpu_label!()),
                });
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.blit_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: (self.fill_colour.r() as f64) / 255.0,
                        g: (self.fill_colour.g() as f64) / 255.0,
                        b: (self.fill_colour.b() as f64) / 255.0,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        self.wgpu_ctx.queue.submit(Some(encoder.finish()));

        self.blit_pipeline_1.render();
        self.blit_pipeline_2.render();
        self.blit_pipeline_3.render();
        self.blit_pipeline_4.render();

        self.egui_pipeline.paint(wgpu_render_pass);
    }
}
