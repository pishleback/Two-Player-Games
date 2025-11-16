use crate::game::{Game, GameLogic};

pub mod alphabeta;
pub mod null;
pub mod random;

pub trait Ai<G: GameLogic> {
    fn new() -> Self;
    fn set_game(&mut self, game: Game<G>);
    fn think(&mut self, max_time: chrono::TimeDelta);
    fn best_moves(&self) -> Vec<(String, G::Move)>;
}
