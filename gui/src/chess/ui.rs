use super::game::BoardState;

pub struct State {
    board: BoardState,
}

impl State {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Self {
        Self {
            board: BoardState::initial_state_standard_chess(),
        }
    }
}

impl eframe::App for State {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Poo");
        });
    }
}
