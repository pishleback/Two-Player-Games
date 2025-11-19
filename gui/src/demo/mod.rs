use std::{rc::Rc, sync::Arc};

use crate::{
    demo::{cube::CubeRenderer, texture_to_egui::TextureRenderer},
    root::AppState,
};
use eframe::egui_wgpu::wgpu;

mod cube;
mod texture_to_egui;

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub struct State {
    rotation: glam::Quat,
}

impl State {
    pub fn new(ctx: &egui::Context, frame: &mut eframe::Frame) -> Self {
        let wgpu_ctx = frame.wgpu_render_state.as_ref().unwrap();
        Self {
            rotation: glam::Quat::IDENTITY,
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
                egui::ScrollArea::both()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        if ui.button("Back").clicked() {
                            return Some(
                                Box::new(crate::menu::State::default()) as Box<dyn AppState>
                            );
                        }

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.label("The triangle is being painted using ");
                            ui.hyperlink_to("WGPU", "https://wgpu.rs");
                            ui.label(" (Portable Rust graphics API awesomeness)");
                        });
                        ui.label(
                            "\
It's not a very impressive demo, but it shows you can embed 3D inside of egui.",
                        );

                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                            let (rect, response) = ui
                                .allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());

                            self.rotation =
                                (glam::Quat::from_rotation_y(-response.drag_motion().x * 0.01)
                                    * glam::Quat::from_rotation_x(
                                        -response.drag_motion().y * 0.01,
                                    )
                                    * self.rotation)
                                    .normalize();

                            ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                                rect,
                                CustomCallback {
                                    renderer: Arc::new(texture_to_egui::TextureRenderer::new(
                                        wgpu_ctx,
                                        (
                                            (ctx.pixels_per_point() * rect.width()) as u32,
                                            (ctx.pixels_per_point() * rect.height()) as u32,
                                        ),
                                        |render_pass, target_format| {
                                            let renderer = CubeRenderer::new(
                                                wgpu_ctx,
                                                DEPTH_FORMAT,
                                                target_format,
                                            );
                                            renderer.prepare(
                                                &wgpu_ctx.device,
                                                &wgpu_ctx.queue,
                                                self.rotation,
                                            );
                                            renderer.paint(render_pass);
                                        },
                                    )),
                                    rotation: self.rotation,
                                },
                            ));
                        });
                        ui.label("Drag to rotate!");

                        None
                    })
                    .inner
            })
            .inner
    }
}

struct CustomCallback {
    renderer: Arc<texture_to_egui::TextureRenderer>,
    rotation: glam::Quat,
}

impl egui_wgpu::CallbackTrait for CustomCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        self.renderer.prepare(device, queue);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        self.renderer.paint(render_pass);
    }
}

pub fn render_to_texture(
    ctx: &egui_wgpu::RenderState,
    texture_view: &wgpu::TextureView,
    render: impl FnOnce(&mut wgpu::RenderPass),
) {
    let size = texture_view.texture().size();
    let depth_texture_desc = wgpu::TextureDescriptor {
        label: Some("TextureDescriptor"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };
    let depth_texture = ctx.device.create_texture(&depth_texture_desc);
    let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let render_pass_desc = wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &texture_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.5,
                    g: 0.5,
                    b: 0.5,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &depth_texture_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    };

    {
        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
        render(&mut render_pass);
    }

    ctx.queue.submit(Some(encoder.finish()));
}
