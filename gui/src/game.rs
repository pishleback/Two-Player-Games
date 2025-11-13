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

pub trait Neutral {
    fn neutral() -> Self;
}

pub trait HeuristicScore:
    PartialEq + Eq + PartialOrd + Ord + std::ops::Neg<Output = Self> + Neutral
{
}

pub enum AbsScore<T> {
    SecondPlayerWin,
    Draw,
    FirstPlayerWin,
    Heuristic(T),
}

impl<T: Neg<Output = T>> AbsScore<T> {
    pub fn into_rel(self, player: Player) -> RelScore<T> {
        match (player, self) {
            (Player::First, AbsScore::SecondPlayerWin) => RelScore::Terminal(RelTerminal::Loose, 0),
            (Player::First, AbsScore::Draw) => RelScore::Terminal(RelTerminal::Draw, 0),
            (Player::First, AbsScore::FirstPlayerWin) => RelScore::Terminal(RelTerminal::Win, 0),
            (Player::First, AbsScore::Heuristic(score)) => RelScore::Heuristic(score),
            (Player::Second, AbsScore::SecondPlayerWin) => RelScore::Terminal(RelTerminal::Win, 0),
            (Player::Second, AbsScore::Draw) => RelScore::Terminal(RelTerminal::Draw, 0),
            (Player::Second, AbsScore::FirstPlayerWin) => RelScore::Terminal(RelTerminal::Loose, 0),
            (Player::Second, AbsScore::Heuristic(score)) => RelScore::Heuristic(-score),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelTerminal {
    Loose,
    Draw,
    Win,
}

#[derive(Debug, Clone)]
pub enum RelScore<T> {
    Heuristic(T),
    Terminal(RelTerminal, usize),
}

impl<T> RelScore<T> {
    pub fn inc_time(self) -> Self {
        match self {
            RelScore::Heuristic(score) => RelScore::Heuristic(score),
            RelScore::Terminal(terminal, time) => RelScore::Terminal(terminal, time + 1),
        }
    }
}

impl<T: Ord + Neutral> PartialEq for RelScore<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl<T: Ord + Neutral> Eq for RelScore<T> {}

impl<T: Ord + Neutral> PartialOrd for RelScore<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord + Neutral> Ord for RelScore<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (RelScore::Heuristic(left), RelScore::Heuristic(right)) => left.cmp(right),
            (
                RelScore::Terminal(RelTerminal::Loose, left_t),
                RelScore::Terminal(RelTerminal::Loose, right_t),
            ) => left_t.cmp(right_t),
            (
                RelScore::Terminal(RelTerminal::Win, left_t),
                RelScore::Terminal(RelTerminal::Win, right_t),
            ) => right_t.cmp(left_t),
            (
                RelScore::Terminal(RelTerminal::Draw, _),
                RelScore::Terminal(RelTerminal::Draw, _),
            ) => std::cmp::Ordering::Equal,
            (RelScore::Terminal(RelTerminal::Win, _), _) => std::cmp::Ordering::Greater,
            (RelScore::Terminal(RelTerminal::Loose, _), _) => std::cmp::Ordering::Less,
            (_, RelScore::Terminal(RelTerminal::Win, _)) => std::cmp::Ordering::Less,
            (_, RelScore::Terminal(RelTerminal::Loose, _)) => std::cmp::Ordering::Greater,
            (RelScore::Heuristic(value), RelScore::Terminal(RelTerminal::Draw, _)) => {
                value.cmp(&T::neutral())
            }
            (RelScore::Terminal(RelTerminal::Draw, _), RelScore::Heuristic(value)) => {
                T::neutral().cmp(value)
            }
        }
    }
}

impl<T: Neg<Output = T>> Neg for RelScore<T> {
    type Output = RelScore<T>;

    fn neg(self) -> Self::Output {
        match self {
            RelScore::Heuristic(score) => RelScore::Heuristic(-score),
            RelScore::Terminal(terminal, time) => RelScore::Terminal(
                match terminal {
                    RelTerminal::Loose => RelTerminal::Win,
                    RelTerminal::Draw => RelTerminal::Draw,
                    RelTerminal::Win => RelTerminal::Loose,
                },
                time,
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WithPosInf<T: Eq + Ord> {
    Finite(T),
    PosInf,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WithNegInf<T: Eq + Ord> {
    NegInf,
    Finite(T),
}

impl<T: Eq + Ord> WithPosInf<T> {
    pub fn unwrap_finite(self) -> T {
        match self {
            WithPosInf::Finite(value) => value,
            WithPosInf::PosInf => panic!(),
        }
    }
}

impl<T: Eq + Ord> WithNegInf<T> {
    pub fn unwrap_finite(self) -> T {
        match self {
            WithNegInf::Finite(value) => value,
            WithNegInf::NegInf => panic!(),
        }
    }
}

impl<T: Eq + Ord + Neg<Output = T>> Neg for WithPosInf<T> {
    type Output = WithNegInf<T>;

    fn neg(self) -> Self::Output {
        match self {
            WithPosInf::Finite(value) => WithNegInf::Finite(-value),
            WithPosInf::PosInf => WithNegInf::NegInf,
        }
    }
}

impl<T: Eq + Ord + Neg<Output = T>> Neg for WithNegInf<T> {
    type Output = WithPosInf<T>;

    fn neg(self) -> Self::Output {
        match self {
            WithNegInf::Finite(value) => WithPosInf::Finite(-value),
            WithNegInf::NegInf => WithPosInf::PosInf,
        }
    }
}

impl<T: Eq + Ord> PartialEq<WithPosInf<T>> for WithNegInf<T> {
    fn eq(&self, other: &WithPosInf<T>) -> bool {
        match (self, other) {
            (WithNegInf::Finite(left), WithPosInf::Finite(right)) => left == right,
            _ => false,
        }
    }
}

impl<T: Eq + Ord> PartialOrd<WithPosInf<T>> for WithNegInf<T> {
    fn partial_cmp(&self, other: &WithPosInf<T>) -> Option<std::cmp::Ordering> {
        Some(match (self, other) {
            (WithNegInf::Finite(left), WithPosInf::Finite(right)) => left.cmp(right),
            (_, WithPosInf::PosInf) | (WithNegInf::NegInf, _) => std::cmp::Ordering::Less,
        })
    }
}

impl<T: Eq + Ord> PartialEq<WithNegInf<T>> for WithPosInf<T> {
    fn eq(&self, other: &WithNegInf<T>) -> bool {
        match (self, other) {
            (WithPosInf::Finite(left), WithNegInf::Finite(right)) => left == right,
            _ => false,
        }
    }
}

impl<T: Eq + Ord> PartialOrd<WithNegInf<T>> for WithPosInf<T> {
    fn partial_cmp(&self, other: &WithNegInf<T>) -> Option<std::cmp::Ordering> {
        Some(match (self, other) {
            (WithPosInf::Finite(left), WithNegInf::Finite(right)) => left.cmp(right),
            (_, WithNegInf::NegInf) | (WithPosInf::PosInf, _) => std::cmp::Ordering::Greater,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Neutral for i32 {
        fn neutral() -> Self {
            0
        }
    }

    #[test]
    fn test() {
        assert!(WithNegInf::Finite(0) > WithNegInf::NegInf);
        assert!(WithNegInf::NegInf < WithNegInf::Finite(0));
        assert!(WithPosInf::Finite(0) < WithPosInf::PosInf);
        assert!(WithPosInf::PosInf > WithPosInf::Finite(0));
        assert!(WithNegInf::Finite(-1) < WithPosInf::Finite(1));
        assert!(WithPosInf::Finite(1) > WithNegInf::Finite(-1));
        assert!(WithNegInf::Finite(0) < WithPosInf::PosInf);
        assert!(WithNegInf::NegInf < WithPosInf::Finite(0));
        assert!(WithNegInf::<i32>::NegInf < WithPosInf::PosInf);
        assert_eq!(WithPosInf::<i32>::PosInf, -WithNegInf::NegInf);
        assert_eq!(-WithPosInf::<i32>::PosInf, WithNegInf::NegInf);
        assert_eq!(WithPosInf::Finite(-1), -WithNegInf::Finite(1));

        assert!(RelScore::Heuristic(-1) < RelScore::Terminal(RelTerminal::Draw, 0));
    }
}

// A 2 player turn-based game.
// Turn switches every move.
// First is winning if score is positive
// Second is winning if score is negative
pub trait GameLogic: Debug + Clone + 'static {
    type State: Debug + Clone + PartialEq + Eq + Send;
    type Move: Debug + Clone + Send + PartialEq + Eq;
    type HeuristicScore: Debug + Clone + Send + HeuristicScore;

    fn initial_state(&self) -> Self::State;

    fn turn(&self, state: &Self::State) -> Player;

    fn hash_state(&self, state: &Self::State) -> u64;

    // The game ends when `generate_moves` returns no moves.
    fn generate_moves(&self, state: &mut Self::State) -> Vec<Self::Move>;
    // A subset of self.generate_moves(..) with only very active moves
    #[allow(unused_variables)]
    fn generate_quiescence_moves(&self, state: &mut Self::State) -> Vec<Self::Move> {
        vec![]
    }
    fn score(&self, state: &mut Self::State) -> AbsScore<Self::HeuristicScore>;

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
