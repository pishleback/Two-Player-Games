use eframe::wgpu;
use egui::Color32;
use glam::Quat;
use wgpu_widgets::{
    texture_to_egui,
    widget::{VisiblePart, WgpuEguiRenderPipeline},
};

use crate::wormhole::board;

pub struct Pipeline {
    cube_pipeline: super::board_render::Pipeline,
    egui_pipeline: texture_to_egui::RenderTexturePipeline,
}

impl Pipeline {
    pub fn new(wgpu_ctx: &egui_wgpu::RenderState, pixels_size: (u32, u32)) -> Self {
        let cube_pipeline = super::board_render::Pipeline::new(
            &wgpu_ctx,
            pixels_size,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Depth32Float,
        );
        let egui_pipeline = texture_to_egui::RenderTexturePipeline::new(
            &wgpu_ctx,
            cube_pipeline.colour_texture_view(),
        );
        Self {
            cube_pipeline,
            egui_pipeline,
        }
    }

    pub fn set_selected(&mut self, pos: &board::Pos) {
        self.cube_pipeline.set_selected(pos);
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.cube_pipeline.set_rotation(rotation);
    }

    pub fn set_fill_colour(&mut self, fill_colour: Color32) {
        self.cube_pipeline.set_fill_colour(fill_colour);
    }
}

impl WgpuEguiRenderPipeline for Pipeline {
    fn prepare(&self, device: &wgpu::Device, queue: &wgpu::Queue, visible_part: &VisiblePart) {
        self.cube_pipeline.prepare(device, queue);
        self.egui_pipeline.prepare(device, queue, visible_part);
    }

    fn paint(&self, wgpu_render_pass: &mut wgpu::RenderPass<'_>) {
        self.cube_pipeline.render();
        self.egui_pipeline.paint(wgpu_render_pass);
    }
}
