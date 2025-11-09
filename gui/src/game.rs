use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    First,
    Second,
}

impl Player {
    fn flip(&mut self) {
        match self {
            Self::First => *self = Self::Second,
            Self::Second => *self = Self::First,
        }
    }
}

// A 2 player turn-based game.
// Turn switches every move.
// First is winning if score is positive
// Second is winning if score is negative
pub trait GameLogic {
    type State: Debug + Clone;
    type Move: Debug;
    type Score: Debug + Ord;

    fn initial_state(&self) -> Self::State;

    // The game ends when `generate_moves` returns no moves.
    fn generate_moves(&self, state: Self::State) -> Vec<Self::Move>;
    fn score(&self, state: &Self::State) -> Self::Score;

    fn make_move(&self, state: &mut Self::State, mv: &Self::Move);
    fn unmake_move(&self, state: &mut Self::State, mv: &Self::Move);
}

pub struct Game<G: GameLogic> {
    logic: G,
    state: G::State,
    turn: Player,
    moves: Vec<G::Move>,
}

impl<G: GameLogic> Game<G> {
    pub fn new(logic: G) -> Self {
        let state = logic.initial_state();
        Self {
            logic,
            state,
            turn: Player::First,
            moves: vec![],
        }
    }

    pub fn logic(&self) -> &G {
        &self.logic
    }

    pub fn state(&self) -> &G::State {
        &self.state
    }

    pub fn turn(&self) -> &Player {
        &self.turn
    }

    pub fn num_moves(&self) -> usize {
        self.moves.len()
    }

    pub fn make_move(&mut self, mv: G::Move) {
        self.logic.make_move(&mut self.state, &mv);
        self.turn.flip();
        self.moves.push(mv);
    }

    pub fn can_undo_move(&self) -> bool {
        !self.moves.is_empty()
    }

    pub fn undo_move(&mut self) {
        let mv = self.moves.pop().unwrap();
        self.logic.unmake_move(&mut self.state, &mv);
        self.turn.flip();
    }
}
