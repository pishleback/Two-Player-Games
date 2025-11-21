use crate::{
    demo::{
        cube::{Cube, CubeRenderer},
        texture_to_egui::WgpuWidget,
    },
    root::AppState,
};
use eframe::egui_wgpu::wgpu;
use std::sync::Arc;

mod cube;
mod texture_to_egui;

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

                            ui.add(
                                WgpuWidget::new(
                                    ctx,
                                    frame,
                                    rect,
                                    response,
                                    Cube::new(self.rotation),
                                )
                                .unwrap(),
                            );
                        });
                        ui.label("Drag to rotate!");

                        None
                    })
                    .inner
            })
            .inner
    }
}
