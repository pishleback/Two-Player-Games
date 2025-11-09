use crate::game::GameLogic;

struct ChessGame {}

mod square {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Square {
        Empty,
        WhitePawn,
        WhiteRook,
        WhiteKnight,
        WhiteBishop,
        WhiteQueen,
        WhiteKing,
        BlackPawn,
        BlackRook,
        BlackKnight,
        BlackBishop,
        BlackQueen,
        BlackKing,
    }

    const PAWN: u8 = 1;
    const ROOK: u8 = 2;
    const KNIGHT: u8 = 3;
    const BISHOP: u8 = 4;
    const QUEEN: u8 = 5;
    const KING: u8 = 6;
    const PIECE_MASK: u8 = 15;
    const MOVED: u8 = 16;
    const OWNER: u8 = 32;
    const OCCUPIED: u8 = 64;
    const BOUNDARY: u8 = 128;

    #[derive(Debug, Clone, Copy)]
    pub struct RawSquare {
        /*
        Bits:    | 0 | 1 | 2 | 3 |   4   |   5   |    6     |     7    |
        Meaning: |     piece     | moved | owner | occupied | boundary |

        If boundary is set, ignore all other bits. This is a square outside the board.
        If piece == EMPTY then ignore owner.
        moved=1 iff the piece has moved.

        owner=0 is white is Player1
        owner=1 is black is Player2
        */
        state: u8,
    }

    impl RawSquare {
        pub fn boundary() -> Self {
            Self { state: BOUNDARY }
        }

        pub fn empty() -> Self {
            Self { state: 0 }
        }

        pub fn white_pawn() -> Self {
            Self {
                state: PAWN | OCCUPIED,
            }
        }
        pub fn white_rook() -> Self {
            Self {
                state: ROOK | OCCUPIED,
            }
        }
        pub fn white_knight() -> Self {
            Self {
                state: KNIGHT | OCCUPIED,
            }
        }
        pub fn white_bishop() -> Self {
            Self {
                state: BISHOP | OCCUPIED,
            }
        }
        pub fn white_queen() -> Self {
            Self {
                state: QUEEN | OCCUPIED,
            }
        }
        pub fn white_king() -> Self {
            Self {
                state: KING | OCCUPIED,
            }
        }

        pub fn black_pawn() -> Self {
            Self {
                state: PAWN | OCCUPIED | OWNER,
            }
        }
        pub fn black_rook() -> Self {
            Self {
                state: ROOK | OCCUPIED | OWNER,
            }
        }
        pub fn black_knight() -> Self {
            Self {
                state: KNIGHT | OCCUPIED | OWNER,
            }
        }
        pub fn black_bishop() -> Self {
            Self {
                state: BISHOP | OCCUPIED | OWNER,
            }
        }
        pub fn black_queen() -> Self {
            Self {
                state: QUEEN | OCCUPIED | OWNER,
            }
        }
        pub fn black_king() -> Self {
            Self {
                state: KING | OCCUPIED | OWNER,
            }
        }

        pub fn to_enum(&self) -> Square {
            use crate::game::Player;
            if self.state & BOUNDARY != 0 {
                panic!()
            } else if self.state & OCCUPIED == 0 {
                Square::Empty
            } else {
                let piece = self.state & PIECE_MASK;
                let owner = if self.state & OWNER == 0 {
                    Player::First
                } else {
                    Player::Second
                };
                match (piece, owner) {
                    (PAWN, Player::First) => Square::WhitePawn,
                    (ROOK, Player::First) => Square::WhiteRook,
                    (KNIGHT, Player::First) => Square::WhiteKnight,
                    (BISHOP, Player::First) => Square::WhiteBishop,
                    (QUEEN, Player::First) => Square::WhiteQueen,
                    (KING, Player::First) => Square::WhiteKing,
                    (PAWN, Player::Second) => Square::BlackPawn,
                    (ROOK, Player::Second) => Square::BlackRook,
                    (KNIGHT, Player::Second) => Square::BlackKnight,
                    (BISHOP, Player::Second) => Square::BlackBishop,
                    (QUEEN, Player::Second) => Square::BlackQueen,
                    (KING, Player::Second) => Square::BlackKing,
                    _ => {
                        panic!()
                    }
                }
            }
        }
    }
}
use square::RawSquare;
pub use square::Square;

#[derive(Debug, Clone)]
pub struct BoardState {
    /*
    A 10x10 grid. The outer squares are for edge-detection.
    The inner 8x8 grid is the standard chess board.
     */
    data: [RawSquare; 100],
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
                    '#' => RawSquare::boundary(),
                    ' ' => RawSquare::empty(),
                    'p' => RawSquare::white_pawn(),
                    'r' => RawSquare::white_rook(),
                    'n' => RawSquare::white_knight(),
                    'b' => RawSquare::white_bishop(),
                    'q' => RawSquare::white_queen(),
                    'k' => RawSquare::white_king(),
                    'P' => RawSquare::black_pawn(),
                    'R' => RawSquare::black_rook(),
                    'N' => RawSquare::black_knight(),
                    'B' => RawSquare::black_bishop(),
                    'Q' => RawSquare::black_queen(),
                    'K' => RawSquare::black_king(),
                    _ => unreachable!(),
                }
            }),
        }
    }

    pub fn square(&self, row: usize, col: usize) -> Square {
        self.data[row + 1 + 10 * (col + 1)].to_enum()
    }
}

#[derive(Debug, Clone)]
enum Move {
    Null,
}

impl GameLogic for ChessGame {
    type State = BoardState;
    type Move = Move;
    type Score = i64;

    fn initial_state(&self) -> Self::State {
        BoardState::initial_state_standard_chess()
    }

    fn generate_moves(&self, state: Self::State) -> Vec<Self::Move> {
        todo!()
    }

    fn make_move(&self, state: &mut Self::State, mv: &Self::Move) {
        match mv {
            Move::Null => {}
        }
    }

    fn unmake_move(&self, state: &mut Self::State, mv: &Self::Move) {
        match mv {
            Move::Null => {}
        }
    }

    fn score(&self, state: &Self::State) -> Self::Score {
        0
    }
}
