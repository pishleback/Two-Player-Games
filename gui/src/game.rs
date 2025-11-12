use std::{fmt::Debug, ops::Neg};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    First,
    Second,
}

impl Player {
    pub fn flip(self) -> Self {
        match self {
            Self::First => Self::Second,
            Self::Second => Self::First,
        }
    }
}

pub trait Score: Neg<Output = Self> + Ord {
    fn pos_inf() -> Self;
    fn neg_inf() -> Self;
}

// A 2 player turn-based game.
// Turn switches every move.
// First is winning if score is positive
// Second is winning if score is negative
pub trait GameLogic: Debug + Clone + 'static {
    type State: Debug + Clone + PartialEq + Eq + Send;
    type Move: Debug + Clone + Send + PartialEq + Eq;
    type Score: Debug + Clone + Send + Score;

    fn initial_state(&self) -> Self::State;

    fn turn(&self, state: &Self::State) -> Player;

    fn hash_state(&self, state: &Self::State) -> u64;

    // The game ends when `generate_moves` returns no moves.
    fn generate_moves(&self, state: &mut Self::State) -> Vec<Self::Move>;
    // A subset of self.generate_moves(..) with only very active moves
    fn generate_quiescence_moves(&self, state: &mut Self::State) -> Vec<Self::Move> {
        vec![]
    }
    fn score(&self, state: &mut Self::State) -> Self::Score;

    fn make_move(&self, state: &mut Self::State, mv: &Self::Move);
    fn unmake_move(&self, state: &mut Self::State, mv: &Self::Move);
}

#[derive(Debug, Clone)]
pub struct Game<G: GameLogic> {
    logic: G,
    state: G::State,
    turn: Player,
    move_history: Vec<G::Move>,
}

impl<G: GameLogic> Game<G> {
    pub fn new(logic: G) -> Self {
        let state = logic.initial_state();
        Self {
            logic,
            state,
            turn: Player::First,
            move_history: vec![],
        }
    }

    pub fn logic(&self) -> &G {
        &self.logic
    }

    pub fn state(&self) -> &G::State {
        &self.state
    }

    pub fn turn(&self) -> Player {
        self.turn
    }

    pub fn num_moves(&self) -> usize {
        self.move_history.len()
    }

    pub fn make_move(&mut self, mv: G::Move) {
        debug_assert!(self.logic.generate_moves(&mut self.state).contains(&mv));
        self.logic.make_move(&mut self.state, &mv);
        self.turn = self.turn.flip();
        self.move_history.push(mv);
    }

    pub fn can_undo_move(&self) -> bool {
        !self.move_history.is_empty()
    }

    pub fn undo_move(&mut self) {
        let mv = self.move_history.pop().unwrap();
        self.logic.unmake_move(&mut self.state, &mv);
        self.turn = self.turn.flip();
    }
}
