use crate::{
    demo::{cube::RenderCubePipeline, texture_to_egui::RenderTexturePipeline},
    root::AppState,
};
use eframe::wgpu;
use wgpu_widgets::WgpuWidget;

pub struct State {
    rotation: glam::Quat,
    cube_widget: WgpuWidget<RenderTexturePipeline>,
}

impl State {
    pub fn new(ctx: &egui::Context, frame: &mut eframe::Frame) -> Self {
        Self {
            rotation: glam::Quat::IDENTITY,
            cube_widget: WgpuWidget::new(ctx),
        }
    }
}

impl AppState for State {
    fn update(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
    ) -> Option<Box<dyn AppState>> {
        let wgpu_ctx = frame.wgpu_render_state.as_ref().unwrap();

        egui::CentralPanel::default()
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        if ui.button("Back").clicked() {
                            return Some(
                                Box::new(crate::menu::State::default()) as Box<dyn AppState>
                            );
                        }

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.label("The cube is being painted using ");
                            ui.hyperlink_to("WGPU", "https://wgpu.rs");
                            ui.label(" (Portable Rust graphics API awesomeness)");
                        });
                        ui.label(
                            "\
The cube is being rendered to the UI via an intermediate texture.",
                        );

                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                            let (rect, response) = ui.allocate_exact_size(
                                egui::Vec2 { x: 500.0, y: 300.0 },
                                egui::Sense::drag(),
                            );

                            self.rotation =
                                (glam::Quat::from_rotation_y(-response.drag_motion().x * 0.01)
                                    * glam::Quat::from_rotation_x(
                                        -response.drag_motion().y * 0.01,
                                    )
                                    * self.rotation)
                                    .normalize();

                            self.cube_widget.set_rect(rect);
                            if self.cube_widget.changed() {
                                let pixels_size = self.cube_widget.pixels_size();
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
                                let texture: wgpu::Texture =
                                    wgpu_ctx.device.create_texture(&texture_desc);
                                let texture_view: wgpu::TextureView =
                                    texture.create_view(&Default::default());
                                self.cube_widget.set_pipeline(RenderTexturePipeline::new(
                                    ctx,
                                    wgpu_ctx,
                                    texture_view,
                                ));
                            }

                            self.cube_widget.pipeline().unwrap().render_to_texture(
                                ui.visuals().extreme_bg_color,
                                |wgpu_ctx, render_pass, size, color_format, depth_format| {
                                    let renderer = RenderCubePipeline::new(
                                        wgpu_ctx,
                                        size,
                                        color_format,
                                        depth_format,
                                    );
                                    renderer.prepare(
                                        &wgpu_ctx.device,
                                        &wgpu_ctx.queue,
                                        self.rotation,
                                    );
                                    renderer.paint(render_pass);
                                },
                            );

                            self.cube_widget.add(ui);
                        });
                        ui.label("Drag to rotate!");

                        None
                    })
                    .inner
            })
            .inner
    }
}
