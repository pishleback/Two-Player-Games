use crate::{root::AppState, wormhole::board_to_egui};
use wgpu_widgets::widget::WgpuWidget;

pub struct State {
    rotation: glam::Quat,
    cube_widget: WgpuWidget<board_to_egui::Pipeline>,
}

impl State {
    pub fn new(ctx: &egui::Context, _frame: &mut eframe::Frame) -> Self {
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

                            self.cube_widget.set_rect(rect);
                            if self.cube_widget.changed() {
                                let pixels_size = self.cube_widget.pixels_size();
                                self.cube_widget.set_pipeline(board_to_egui::Pipeline::new(
                                    wgpu_ctx,
                                    pixels_size,
                                ));
                            }

                            if let Some(mut pipeline) = self.cube_widget.pipeline() {
                                pipeline.set_rotation(self.rotation);
                                pipeline.set_fill_colour(ui.visuals().extreme_bg_color);
                            }

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
