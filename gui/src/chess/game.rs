use crate::game::GameLogic;

struct ChessGame {}

mod square {
    const EMPTY: u8 = 0;
    const PAWN: u8 = 1;
    const ROOK: u8 = 2;
    const KNIGHT: u8 = 3;
    const BISHOP: u8 = 4;
    const QUEEN: u8 = 5;
    const KING: u8 = 6;
    const PIECE_MASK: u8 = 63;
    const OWNER: u8 = 64;
    const BOUNDARY: u8 = 128;

    #[derive(Debug, Clone, Copy)]
    pub struct SquareState {
        /*
        Bits:    | 0 | 1 | 2 | 3 | 4 | 5 |   6   |     7    |
        Meaning: |        piece          | owner | boundary |

        If boundary is set, ignore all other bits. This is a square outside the board.
        If piece == EMPTY then ignore owner

        owner=0 is white is Player1
        owner=1 is black is Player2
        */
        state: u8,
    }

    impl SquareState {
        pub fn boundary() -> Self {
            Self { state: BOUNDARY }
        }

        pub fn empty() -> Self {
            Self { state: EMPTY }
        }

        pub fn white_pawn() -> Self {
            Self { state: PAWN }
        }
        pub fn white_rook() -> Self {
            Self { state: ROOK }
        }
        pub fn white_knight() -> Self {
            Self { state: KNIGHT }
        }
        pub fn white_bishop() -> Self {
            Self { state: BISHOP }
        }
        pub fn white_queen() -> Self {
            Self { state: QUEEN }
        }
        pub fn white_king() -> Self {
            Self { state: KING }
        }

        pub fn black_pawn() -> Self {
            Self {
                state: PAWN | OWNER,
            }
        }
        pub fn black_rook() -> Self {
            Self {
                state: ROOK | OWNER,
            }
        }
        pub fn black_knight() -> Self {
            Self {
                state: KNIGHT | OWNER,
            }
        }
        pub fn black_bishop() -> Self {
            Self {
                state: BISHOP | OWNER,
            }
        }
        pub fn black_queen() -> Self {
            Self {
                state: QUEEN | OWNER,
            }
        }
        pub fn black_king() -> Self {
            Self {
                state: KING | OWNER,
            }
        }
    }
}
use square::SquareState;

#[derive(Debug, Clone)]
pub struct BoardState {
    /*
    A 10x10 grid. The outer squares are for edge-detection.
    The inner 8x8 grid is the standard chess board.
     */
    data: [SquareState; 100],
}

impl BoardState {
    pub fn initial_state_standard_chess() -> Self {
        let board = vec![
            vec!['#', '#', '#', '#', '#', '#', '#', '#', '#', '#'],
            vec!['#', 'R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R', '#'],
            vec!['#', 'P', 'P', 'P', 'P', 'P', 'P', 'P', 'P', '#'],
            vec!['#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '#'],
            vec!['#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '#'],
            vec!['#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '#'],
            vec!['#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '#'],
            vec!['#', 'p', 'p', 'p', 'p', 'p', 'p', 'p', 'p', '#'],
            vec!['#', 'r', 'n', 'b', 'q', 'k', 'b', 'n', 'r', '#'],
            vec!['#', '#', '#', '#', '#', '#', '#', '#', '#', '#'],
        ];
        debug_assert_eq!(board.len(), 10);
        for row in &board {
            debug_assert_eq!(row.len(), 10);
        }
        Self {
            data: std::array::from_fn(|i| {
                let r = i % 10;
                let c = i / 10;
                match board[r][c] {
                    '#' => SquareState::boundary(),
                    ' ' => SquareState::empty(),
                    'p' => SquareState::white_pawn(),
                    'r' => SquareState::white_rook(),
                    'n' => SquareState::white_knight(),
                    'b' => SquareState::white_bishop(),
                    'q' => SquareState::white_queen(),
                    'k' => SquareState::white_king(),
                    'P' => SquareState::black_pawn(),
                    'R' => SquareState::black_rook(),
                    'N' => SquareState::black_knight(),
                    'B' => SquareState::black_bishop(),
                    'Q' => SquareState::black_queen(),
                    'K' => SquareState::black_king(),
                    _ => unreachable!(),
                }
            }),
        }
    }
}

#[derive(Debug, Clone)]
enum Move {}

impl GameLogic for ChessGame {
    type State = BoardState;
    type Move = Move;
    type Score = i64;

    fn generate_moves(&self, state: Self::State) -> Vec<Self::Move> {
        todo!()
    }

    fn make_move(&self, state: &mut Self::State, mv: &Self::Move) {
        todo!()
    }

    fn unmake_move(&self, state: &mut Self::State, mv: &Self::Move) {
        todo!()
    }

    fn score(&self, state: &Self::State) -> Self::Score {
        todo!()
    }
}
