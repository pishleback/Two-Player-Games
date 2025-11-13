use crate::{
    ai::Ai,
    game::{Game, GameLogic},
};
use std::marker::PhantomData;

/// Random AI using the tiny RNG
#[derive(Debug)]
pub struct NullAi<G: GameLogic> {
    _g: PhantomData<G>,
}

impl<G: GameLogic> Default for NullAi<G> {
    fn default() -> Self {
        Self {
            _g: Default::default(),
        }
    }
}

impl<G: GameLogic> Ai<G> for NullAi<G> {
    fn new() -> Self {
        Self::default()
    }

    fn set_game(&mut self, game: Game<G>) {}

    fn think(&mut self, _max_time: chrono::Duration) {}

    fn best_move(&self) -> Option<G::Move> {
        None
    }
}
