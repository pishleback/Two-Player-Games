#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    First,
    Second,
}

// A 2 player turn-based game.
// Turn switches every move.
// First is winning if score is positive
// Second is winning if score is negative
pub trait GameLogic {
    type State: Clone;
    type Move;
    type Score: Ord;

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
}

impl<G: GameLogic> Game<G> {
    pub fn new(logic: G) -> Self {
        let state = logic.initial_state();
        Self {
            logic,
            state,
            turn: Player::First,
        }
    }
}
