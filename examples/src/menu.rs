use crate::root::AppState;

pub struct State {}

impl Default for State {
    fn default() -> Self {
        Self {}
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
                    .show(ui, |ui| {
                        ui.separator();
                        if frame.wgpu_render_state.is_some() {
                            if ui.button("Cube").clicked() {
                                return Some(Box::new(crate::examples::cube::app::State::new(
                                    ctx, frame,
                                ))
                                    as Box<dyn AppState>);
                            }
                        } else {
                            ui.add_enabled(false, egui::Button::new("GPU Demo"))
                                .on_disabled_hover_text("Requires wgpu.");
                        }
                        None
                    })
                    .inner
            })
            .inner
    }
}
