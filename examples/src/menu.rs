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
                        if ui.button("Single Cube").clicked() {
                            return Some(Box::new(crate::examples::single_cube::app::State::new(
                                ctx, frame,
                            )) as Box<dyn AppState>);
                        }

                        if ui.button("Many Cube").clicked() {
                            return Some(Box::new(crate::examples::many_cube::app::State::new(
                                ctx, frame,
                            )) as Box<dyn AppState>);
                        }

                        None
                    })
                    .inner
            })
            .inner
    }
}
