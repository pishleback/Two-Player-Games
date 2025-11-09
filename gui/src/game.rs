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

    // The game ends when `generate_moves` returns no moves.
    fn generate_moves(&self, state: Self::State) -> Vec<Self::Move>;
    fn score(&self, state: &Self::State) -> Self::Score;

    fn make_move(&self, state: &mut Self::State, mv: &Self::Move);
    fn unmake_move(&self, state: &mut Self::State, mv: &Self::Move);
}
