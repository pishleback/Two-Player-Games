use crate::{
    root::AppState,
    wormhole::{
        board::{self, Pos},
        board_render::{BoardParams, create_icons_texture_array},
        board_to_egui,
        chess::BoardContent,
    },
};
use eframe::wgpu;
use futures::FutureExt;
use std::{pin::Pin, task::Context};
use wgpu_widgets::widget::WgpuWidget;

pub struct State {
    icons_texture_array: wgpu::Texture,
    board_widget: WgpuWidget<board_to_egui::Pipeline>,
    rotation: glam::Quat,
    board_content: BoardContent,
    pending_square_click: Option<Pin<Box<dyn Future<Output = Option<Pos>>>>>,
    selected_pos: u8,
    radius: u64,
}

impl State {
    pub fn new(ctx: &egui::Context, frame: &mut eframe::Frame) -> Self {
        Self {
            icons_texture_array: create_icons_texture_array(
                frame.wgpu_render_state.as_ref().unwrap(),
            ),
            board_widget: WgpuWidget::new(ctx),
            rotation: glam::Quat::IDENTITY,
            board_content: BoardContent::starting_position(),
            pending_square_click: None,
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

                        ui.label("Wormhole Board");

                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                            let x = ui.available_width();
                            let y = ui.available_height();
                            let (rect, response) = ui.allocate_exact_size(
                                egui::Vec2 { x, y },
                                egui::Sense::click_and_drag(),
                            );

                            self.rotation =
                                (glam::Quat::from_rotation_y(response.drag_motion().x * 0.01)
                                    * glam::Quat::from_rotation_x(response.drag_motion().y * 0.01)
                                    * self.rotation)
                                    .normalize();

                            self.board_widget.set_rect(rect);
                            if self.board_widget.changed() {
                                let mut pixels_size = self.board_widget.pixels_size();
                                pixels_size.0 = pixels_size.0.max(1);
                                pixels_size.1 = pixels_size.1.max(1);
                                self.board_widget.set_pipeline(board_to_egui::Pipeline::new(
                                    wgpu_ctx,
                                    pixels_size,
                                    &self.board_content.map(|x| *x),
                                    &BoardParams {
                                        side_length: 11.0,
                                        face_offset: 1.4,
                                        hole_offset: self.radius as f32 / 1000.0,
                                    },
                                    &self.icons_texture_array,
                                ));
                            }

                            if let Some(mut pipeline) = self.board_widget.pipeline() {
                                pipeline.set_rotation(self.rotation);
                                pipeline.set_fill_colour(ui.visuals().extreme_bg_color);
                                pipeline.set_selected(&board::Pos::new(self.selected_pos));
                                if response.clicked() {
                                    if let Some(pos) = response.interact_pointer_pos() {
                                        let pos_frac = (
                                            (pos.x - rect.min.x) / (rect.max.x - rect.min.x),
                                            (pos.y - rect.min.y) / (rect.max.y - rect.min.y),
                                        );
                                        self.pending_square_click = Some(Box::pin(
                                            pipeline.clone().pixels_to_square(pos_frac),
                                        ));
                                    }
                                }
                            }

                            if let Some(pending_square_click) = self.pending_square_click.as_mut() {
                                let waker = futures::task::noop_waker();
                                let mut async_ctx = Context::from_waker(&waker);
                                match pending_square_click.poll_unpin(&mut async_ctx) {
                                    std::task::Poll::Ready(square_click) => {
                                        if let Some(pos) = square_click {
                                            self.selected_pos = pos.u8_idx();
                                        }
                                        self.pending_square_click = None;
                                    }
                                    std::task::Poll::Pending => {}
                                }
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
