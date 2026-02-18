use crate::{
    root::AppState,
    wormhole::{board, board_render::BoardParams, board_to_egui},
};
use egui::Color32;
use wgpu_widgets::widget::WgpuWidget;

pub struct State {
    rotation: glam::Quat,
    board_widget: WgpuWidget<board_to_egui::Pipeline>,
    selected_pos: u8,
    radius: u64,
}

impl State {
    pub fn new(ctx: &egui::Context, _frame: &mut eframe::Frame) -> Self {
        Self {
            rotation: glam::Quat::IDENTITY,
            board_widget: WgpuWidget::new(ctx),
            selected_pos: 0,
            radius: 1000,
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

                        ui.scope(|ui| {
                            ui.spacing_mut().slider_width *= 3.0;
                            ui.add(
                                egui::Slider::new(&mut self.selected_pos, 0u8..=(144 - 1))
                                    .text("Pos"),
                            );
                            let resp = ui.add(
                                egui::Slider::new(&mut self.radius, 0u64..=2000).text("Radius"),
                            );
                            if resp.changed() {
                                self.board_widget.set_changed();
                            }
                        });

                        ui.label("Wormhole woowoos");

                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                            let x = ui.available_width();
                            let y = ui.available_height();
                            let (rect, response) =
                                ui.allocate_exact_size(egui::Vec2 { x, y }, egui::Sense::drag());

                            self.rotation =
                                (glam::Quat::from_rotation_y(-response.drag_motion().x * 0.01)
                                    * glam::Quat::from_rotation_x(
                                        -response.drag_motion().y * 0.01,
                                    )
                                    * self.rotation)
                                    .normalize();

                            self.board_widget.set_rect(rect);
                            if self.board_widget.changed() {
                                let pixels_size = self.board_widget.pixels_size();
                                self.board_widget.set_pipeline(board_to_egui::Pipeline::new(
                                    wgpu_ctx,
                                    pixels_size,
                                    &BoardParams {
                                        side_length: 11.0,
                                        face_offset: 1.4,
                                        hole_offset: self.radius as f32 / 1000.0,
                                    },
                                ));
                            }

                            if let Some(mut pipeline) = self.board_widget.pipeline() {
                                pipeline.set_rotation(self.rotation);
                                pipeline.set_fill_colour(ui.visuals().code_bg_color);
                                pipeline.set_selected(&board::Pos::new(self.selected_pos));
                            }

                            self.board_widget.add(ui);
                        });
                        ui.label("Drag to rotate!");

                        None
                    })
                    .inner
            })
            .inner
    }
}
