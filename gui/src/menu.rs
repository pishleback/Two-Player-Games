use crate::root::AppState;
use std::sync::Arc;

#[derive(PartialEq)]
enum GameSelection {
    Chess,
    BerolinaChess,
    GrasshopperChess,
}

#[derive(PartialEq)]
enum AiSelection {
    AlphaBetaSingleThread,
    AlphaBetaMultiThread,
    Random,
    Null,
}

pub struct State {
    game_selection: GameSelection,
    ai_selection: AiSelection,
}

impl Default for State {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        let ai_selection = AiSelection::AlphaBetaSingleThread;
        #[cfg(not(target_arch = "wasm32"))]
        let ai_selection = AiSelection::AlphaBetaMultiThread;
        Self {
            game_selection: GameSelection::Chess,
            ai_selection,
        }
    }
}

impl AppState for State {
    fn update(
        &mut self,
        ctx: &egui::Context,
        gl: &Arc<eframe::egui_glow::glow::Context>,
        _frame: &mut eframe::Frame,
    ) -> Option<Box<dyn AppState>> {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        ui.heading("Which Game?");

                        ui.radio_value(&mut self.game_selection, GameSelection::Chess, "Chess");
                        ui.radio_value(
                            &mut self.game_selection,
                            GameSelection::BerolinaChess,
                            "Berolina Chess",
                        );
                        ui.radio_value(
                            &mut self.game_selection,
                            GameSelection::GrasshopperChess,
                            "Grasshopper Chess",
                        );

                        ui.separator();
                        ui.heading("Which AI?");

                        #[cfg(not(target_arch = "wasm32"))]
                        ui.radio_value(
                            &mut self.ai_selection,
                            AiSelection::AlphaBetaMultiThread,
                            "Alpha-Beta Multi-Threaded",
                        );
                        #[cfg(target_arch = "wasm32")]
                        ui.add_enabled(
                            false,
                            egui::RadioButton::new(
                                self.ai_selection == AiSelection::AlphaBetaMultiThread,
                                "Alpha-Beta Multi-Threaded",
                            ),
                        )
                        .on_disabled_hover_text(
                            "\
Alpha-Beta Multi-Threaded is not supported on WASM. Build and run natively to use this AI.",
                        );

                        ui.radio_value(
                            &mut self.ai_selection,
                            AiSelection::AlphaBetaSingleThread,
                            "Alpha-Beta Single Thread",
                        );
                        ui.radio_value(&mut self.ai_selection, AiSelection::Random, "Random Moves");
                        ui.radio_value(&mut self.ai_selection, AiSelection::Null, "None");

                        ui.separator();

                        if ui.button("Start").clicked() {
                            return match self.game_selection {
                                GameSelection::Chess
                                | GameSelection::BerolinaChess
                                | GameSelection::GrasshopperChess => {
                                    let game_logic = match self.game_selection {
                                        GameSelection::Chess => crate::grid::chess::Chess::Standard,
                                        GameSelection::BerolinaChess => {
                                            crate::grid::chess::Chess::Berolina
                                        }
                                        GameSelection::GrasshopperChess => {
                                            crate::grid::chess::Chess::Grasshopper
                                        }
                                    };
                                    match self.ai_selection {
                                        AiSelection::Null => {
                                            Some(Box::new(crate::grid::ui::State::<
                                                _,
                                                crate::ai::null::NullAi<_>,
                                            >::new(
                                                ctx, game_logic
                                            ))
                                                as Box<dyn AppState>)
                                        }
                                        AiSelection::Random => {
                                            Some(Box::new(crate::grid::ui::State::<
                                                _,
                                                crate::ai::random::Random<_>,
                                            >::new(
                                                ctx, game_logic
                                            ))
                                                as Box<dyn AppState>)
                                        }
                                        AiSelection::AlphaBetaMultiThread => {
                                            #[cfg(not(target_arch = "wasm32"))]
                                            {
                                                Some(Box::new(crate::grid::ui::State::<
                                                    _,
                                                    crate::ai::alphabeta::multithreaded::AlphaBeta<
                                                        _,
                                                    >,
                                                >::new(
                                                    ctx, game_logic
                                                ))
                                                    as Box<dyn AppState>)
                                            }
                                            #[cfg(target_arch = "wasm32")]
                                            unreachable!()
                                        }
                                        AiSelection::AlphaBetaSingleThread => {
                                            Some(Box::new(crate::grid::ui::State::<
                                                _,
                                                crate::ai::alphabeta::singlethreaded::AlphaBeta<_>,
                                            >::new(
                                                ctx, game_logic
                                            ))
                                                as Box<dyn AppState>)
                                        }
                                    }
                                }
                            };
                        }

                        ui.separator();

                        if ui.button("GPU Demo").clicked() {
                            return Some(
                                Box::new(crate::demo::Custom3d::new(ctx, gl)) as Box<dyn AppState>
                            );
                        }

                        None
                    })
                    .inner
            })
            .inner
    }
}
