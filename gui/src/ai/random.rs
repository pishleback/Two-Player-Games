use std::cell::RefCell;

use crate::{
    ai::Ai,
    game::{Game, GameLogic},
};

/// Tiny WASM-friendly pseudo-random number generator
#[derive(Debug)]
struct SimpleRng(u32);

impl SimpleRng {
    fn new(seed: u32) -> Self {
        Self(seed)
    }

    fn next_u32(&mut self) -> u32 {
        // Linear congruential generator
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        self.0
    }

    fn next_usize(&mut self, max: usize) -> usize {
        (self.next_u32() as usize) % max
    }
}

/// Random AI using the tiny RNG
#[derive(Debug)]
pub struct Random<G: GameLogic> {
    game: Option<Game<G>>,
    rng: RefCell<SimpleRng>,
    best_move: Option<G::Move>,
}

impl<G: GameLogic> Default for Random<G> {
    fn default() -> Self {
        Self {
            game: None,
            rng: RefCell::new(SimpleRng::new(12345)), // fixed seed for determinism
            best_move: None,
        }
    }
}

fn random_move<G: GameLogic>(rng: &mut SimpleRng, game: &Game<G>) -> Option<G::Move> {
    let moves = game
        .logic()
        .generate_moves(game.turn(), &mut game.state().clone());
    if moves.is_empty() {
        None
    } else {
        Some(moves[rng.next_usize(moves.len())].clone())
    }
}

impl<G: GameLogic> Ai<G> for Random<G> {
    fn set_game(&mut self, game: Game<G>) {
        self.best_move = random_move(&mut self.rng.borrow_mut(), &game);
        self.game = Some(game);
    }

    fn think(&mut self, _max_time: chrono::Duration) {
        // No thinking needed, random AI is instant
    }

    fn best_move(&self) -> Option<G::Move> {
        self.best_move.clone()
    }
}
