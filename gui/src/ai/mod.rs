use crate::game::{Game, GameLogic};

#[cfg(not(target_arch = "wasm32"))]
pub mod alphabeta;
pub mod random;

pub trait Ai<G: GameLogic>{
    fn new() -> Self;
    fn set_game(&mut self, game: Game<G>);
    fn think(&mut self, max_time: chrono::TimeDelta);
    fn best_move(&self) -> Option<G::Move>;
}
