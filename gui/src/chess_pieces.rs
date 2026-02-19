#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Piece {
    Empty,
    WhitePawn,
    WhiteBerolinaPawn,
    WhiteRook,
    WhiteKnight,
    WhiteBishop,
    WhiteQueen,
    WhiteKing,
    WhiteGrasshopper,
    BlackPawn,
    BlackBerolinaPawn,
    BlackRook,
    BlackKnight,
    BlackBishop,
    BlackQueen,
    BlackKing,
    BlackGrasshopper,
}

pub mod square {
    use crate::{chess_pieces::Piece, game::Player};

    pub const PAWN: u8 = 1;
    pub const BEROLINA_PAWN: u8 = 2;
    pub const ROOK: u8 = 3;
    pub const KNIGHT: u8 = 4;
    pub const BISHOP: u8 = 5;
    pub const QUEEN: u8 = 6;
    pub const KING: u8 = 7;
    pub const GRASSHOPPER: u8 = 8;
    const PIECE_MASK: u8 = 31;
    const OWNER: u8 = 32;
    const OCCUPIED: u8 = 64;
    const OUTSIDE: u8 = 128;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SquareContents {
        /*
        Bits:    | 0 | 1 | 2 | 3 | 4 |   5   |    6     |    7     |
        Meaning: |     piece         | owner | occupied | outside  |

        If piece == EMPTY then ignore owner.

        owner=0 is white is Player1
        owner=1 is black is Player2
        */
        pub state: u8,
    }

    impl SquareContents {
        pub fn outside() -> Self {
            Self { state: OUTSIDE }
        }

        pub fn is_outside(self) -> bool {
            self.state & OUTSIDE != 0
        }

        pub fn empty() -> Self {
            Self { state: 0 }
        }

        pub fn is_empty(self) -> bool {
            debug_assert!(!self.is_outside());
            self.state & OCCUPIED == 0
        }

        pub fn white_pawn() -> Self {
            Self {
                state: PAWN | OCCUPIED,
            }
        }
        pub fn white_berolina_pawn() -> Self {
            Self {
                state: BEROLINA_PAWN | OCCUPIED,
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
        pub fn white_grasshopper() -> Self {
            Self {
                state: GRASSHOPPER | OCCUPIED,
            }
        }

        pub fn black_pawn() -> Self {
            Self {
                state: PAWN | OCCUPIED | OWNER,
            }
        }
        pub fn black_berolina_pawn() -> Self {
            Self {
                state: BEROLINA_PAWN | OCCUPIED | OWNER,
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
        pub fn black_grasshopper() -> Self {
            Self {
                state: GRASSHOPPER | OCCUPIED | OWNER,
            }
        }

        pub fn from_piece_raw(turn: Player, piece_raw: u8) -> Self {
            debug_assert!(
                [PAWN, ROOK, KNIGHT, BISHOP, QUEEN, KING, GRASSHOPPER].contains(&piece_raw)
            );
            let mut state = piece_raw | OCCUPIED;
            if turn == Player::Second {
                state |= OWNER;
            }
            Self { state }
        }

        pub fn owner(self) -> Option<Player> {
            if self.state & OCCUPIED == 0 {
                None
            } else {
                #[allow(clippy::collapsible_else_if)]
                if self.state & OWNER == 0 {
                    Some(Player::First)
                } else {
                    Some(Player::Second)
                }
            }
        }

        pub fn piece_raw(self) -> u8 {
            self.state & PIECE_MASK
        }

        pub fn piece(self) -> Piece {
            use crate::game::Player;
            if self.state & OCCUPIED == 0 {
                Piece::Empty
            } else {
                let piece = self.state & PIECE_MASK;
                let owner = if self.state & OWNER == 0 {
                    Player::First
                } else {
                    Player::Second
                };
                match (piece, owner) {
                    (PAWN, Player::First) => Piece::WhitePawn,
                    (BEROLINA_PAWN, Player::First) => Piece::WhiteBerolinaPawn,
                    (ROOK, Player::First) => Piece::WhiteRook,
                    (KNIGHT, Player::First) => Piece::WhiteKnight,
                    (BISHOP, Player::First) => Piece::WhiteBishop,
                    (QUEEN, Player::First) => Piece::WhiteQueen,
                    (KING, Player::First) => Piece::WhiteKing,
                    (GRASSHOPPER, Player::First) => Piece::WhiteGrasshopper,

                    (PAWN, Player::Second) => Piece::BlackPawn,
                    (BEROLINA_PAWN, Player::Second) => Piece::BlackBerolinaPawn,
                    (ROOK, Player::Second) => Piece::BlackRook,
                    (KNIGHT, Player::Second) => Piece::BlackKnight,
                    (BISHOP, Player::Second) => Piece::BlackBishop,
                    (QUEEN, Player::Second) => Piece::BlackQueen,
                    (KING, Player::Second) => Piece::BlackKing,
                    (GRASSHOPPER, Player::Second) => Piece::BlackGrasshopper,

                    _ => {
                        panic!()
                    }
                }
            }
        }
    }
}
