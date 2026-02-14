use eframe::wgpu;
use egui::Color32;
use glam::Quat;
use wgpu_widgets::WgpuEguiRenderPipeline;

pub struct Pipeline {
    cube_pipeline: super::cube::RenderCubePipeline,
    egui_pipeline: super::texture_to_egui::RenderTexturePipeline,
}

impl Pipeline {
    pub fn new(
        wgpu_ctx: &egui_wgpu::RenderState,
        pixels_size: (u32, u32),
    ) -> Self {
        let texture_desc = wgpu::TextureDescriptor {
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
            label: None,
            view_formats: &[],
        };
        let texture: wgpu::Texture = wgpu_ctx.device.create_texture(&texture_desc);
        let texture_view: wgpu::TextureView = texture.create_view(&Default::default());
        Self {
            cube_pipeline: super::cube::RenderCubePipeline::new(
                &wgpu_ctx,
                pixels_size,
                texture_view.clone(),
                wgpu::TextureFormat::Depth32Float,
            ),
            egui_pipeline: super::texture_to_egui::RenderTexturePipeline::new(
                &wgpu_ctx,
                texture_view,
            ),
        }
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.cube_pipeline.set_rotation(rotation);
    }

pub fn set_fill_colour(&mut self, fill_colour: Color32) {
        self.cube_pipeline.set_fill_colour(fill_colour);
    }

}

impl WgpuEguiRenderPipeline for Pipeline {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        visible_part: &wgpu_widgets::VisiblePart,
    ) {
        self.cube_pipeline.prepare(device, queue);
        self.egui_pipeline.prepare(device, queue, visible_part);
    }

    fn paint(&self, wgpu_render_pass: &mut wgpu::RenderPass<'_>) {
        self.cube_pipeline.render();
        self.egui_pipeline.paint(wgpu_render_pass);
    }
}
