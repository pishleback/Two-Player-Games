use std::i64;

use crate::{
    game::{GameLogic, Player, Score},
    grid::GridGame,
};

#[derive(Default, Debug, Clone)]
pub struct StandardChessGame {}

mod square {
    use crate::{game::Player, grid::Piece};

    pub const PAWN: u8 = 1;
    pub const ROOK: u8 = 2;
    pub const KNIGHT: u8 = 3;
    pub const BISHOP: u8 = 4;
    pub const QUEEN: u8 = 5;
    pub const KING: u8 = 6;
    const PIECE_MASK: u8 = 15;
    const MOVED: u8 = 16;
    const OWNER: u8 = 32;
    const OCCUPIED: u8 = 64;
    const OUTSIDE: u8 = 128;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SquareContents {
        /*
        Bits:    | 0 | 1 | 2 | 3 |   4   |   5   |    6     |    7     |
        Meaning: |     piece     | moved | owner | occupied | outside  |

        If piece == EMPTY then ignore owner.
        moved=1 iff the piece has moved.

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

        pub fn moved(self) -> Self {
            Self {
                state: self.state | MOVED,
            }
        }

        pub fn is_moved(self) -> bool {
            debug_assert!(!self.is_outside());
            self.state & MOVED != 0
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
                    (ROOK, Player::First) => Piece::WhiteRook,
                    (KNIGHT, Player::First) => Piece::WhiteKnight,
                    (BISHOP, Player::First) => Piece::WhiteBishop,
                    (QUEEN, Player::First) => Piece::WhiteQueen,
                    (KING, Player::First) => Piece::WhiteKing,
                    (PAWN, Player::Second) => Piece::BlackPawn,
                    (ROOK, Player::Second) => Piece::BlackRook,
                    (KNIGHT, Player::Second) => Piece::BlackKnight,
                    (BISHOP, Player::Second) => Piece::BlackBishop,
                    (QUEEN, Player::Second) => Piece::BlackQueen,
                    (KING, Player::Second) => Piece::BlackKing,
                    _ => {
                        panic!()
                    }
                }
            }
        }
    }
}
use egui::{Color32, Painter, Rect, Stroke};
use square::SquareContents;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    idx: usize,
}

impl Pos {
    pub fn from_grid(row: usize, col: usize) -> Self {
        debug_assert!(row < 8);
        debug_assert!(col < 8);
        Self {
            idx: 10 * (row + 2) + col + 1,
        }
    }

    pub fn to_grid(self) -> Option<(usize, usize)> {
        let mut r = self.idx / 10;
        let mut c = self.idx % 10;
        if r <= 1 || c == 0 {
            None
        } else {
            r -= 2;
            c -= 1;
            if r < 8 && c < 8 { Some((r, c)) } else { None }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DPos {
    idx: isize,
}

impl DPos {
    const fn from_grid(row: isize, col: isize) -> Self {
        Self {
            idx: 10 * row + col,
        }
    }
}

impl std::ops::Add<DPos> for Pos {
    type Output = Pos;

    fn add(self, other: DPos) -> Self::Output {
        let idx = (self.idx as isize) + other.idx;
        debug_assert!(idx >= 0);
        Pos { idx: idx as usize }
    }
}

impl std::ops::Sub<DPos> for Pos {
    type Output = Pos;

    fn sub(self, other: DPos) -> Self::Output {
        let idx = (self.idx as isize) - other.idx;
        debug_assert!(idx >= 0);
        Pos { idx: idx as usize }
    }
}

impl std::ops::Add<DPos> for DPos {
    type Output = DPos;

    fn add(self, other: DPos) -> Self::Output {
        DPos {
            idx: self.idx + other.idx,
        }
    }
}

impl std::ops::Sub<DPos> for DPos {
    type Output = DPos;

    fn sub(self, other: DPos) -> Self::Output {
        DPos {
            idx: self.idx - other.idx,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BoardContent {
    /*
    A 12x10 grid. The outer squares are for edge-detection.
    The inner 8x8 grid is the standard chess board.
     */
    pieces: [SquareContents; 120],
    hash_bits: u64,
}
impl BoardContent {
    fn new() -> Self {
        Self {
            pieces: std::array::from_fn(|idx| {
                if (Pos { idx }).to_grid().is_none() {
                    SquareContents::outside()
                } else {
                    SquareContents::empty()
                }
            }),
            hash_bits: 0,
        }
    }

    #[cfg(debug_assertions)]
    fn validate_hash_bits(&self) {
        let mut expected_bits = 0u64;
        for row in 0..8usize {
            for col in 0..8usize {
                expected_bits ^= (self.get(Pos::from_grid(row, col)).state as u64)
                    .rotate_left(19 * ((8 * row + col) as u32));
            }
        }
        assert_eq!(expected_bits, self.hash_bits);
    }

    fn set(&mut self, pos: Pos, content: SquareContents) {
        #[cfg(debug_assertions)]
        self.validate_hash_bits();
        debug_assert_ne!(self.get(pos), SquareContents::outside());
        debug_assert!(!content.is_outside());
        let (row, col) = pos.to_grid().unwrap();
        self.hash_bits ^= ((self.get(pos).state ^ content.state) as u64)
            .rotate_left(19 * ((8 * row + col) as u32));
        self.pieces[pos.idx] = content;

        #[cfg(debug_assertions)]
        self.validate_hash_bits();
    }

    fn get(&self, pos: Pos) -> SquareContents {
        self.pieces[pos.idx]
    }

    fn hash_bits(&self) -> u64 {
        self.hash_bits
    }
}

/// Fast bijective 64 -> 64 using a 3-round Feistel network on 32-bit halves.
///
/// - `hash64(x)` maps u64 -> u64 bijectively.
/// - `unhash64(y)` is the inverse (recovers original).
///
/// This is simple, fast, and invertible. If you need *stronger* mixing,
/// increase rounds (e.g. 5 or 7) or make the round function stronger.
fn hash64(v: u64) -> u64 {
    #[inline]
    fn round_f(x: u32, k: u32) -> u32 {
        // cheap non-linear mixing: multiply (odd constant), rotate, xor
        // keep operations on 32-bit halves for speed and locality.
        let mut v = x.wrapping_mul(k);
        v = v.rotate_left(13);
        v ^ (x >> 5)
    }

    #[inline]
    fn split64(x: u64) -> (u32, u32) {
        ((x >> 32) as u32, x as u32)
    }

    #[inline]
    fn join64(hi: u32, lo: u32) -> u64 {
        ((hi as u64) << 32) | (lo as u64)
    }

    // Random odd constants
    const K0: u32 = 0x9E3779B1u32;
    const K1: u32 = 0xC2B2AE35u32;
    const K2: u32 = 0x165667B1u32;

    let (mut l, mut r) = split64(v);

    // 3-round Feistel (L, R swapped each round as usual)
    // Round 0
    let t = round_f(r, K0);
    l ^= t;
    // Round 1
    let t = round_f(l, K1);
    r ^= t;
    // Round 2
    let t = round_f(r, K2);
    l ^= t;

    // after odd number of rounds, swap halves to follow Feistel convention
    join64(r, l)
}

#[derive(Debug, Clone)]
pub struct BoardState {
    board: BoardContent,
    white_king: Pos,
    black_king: Pos,
    move_num: usize,
    // If a pawn just double-moved, store the phantom capture square and the move on which the pawn moved.
    en_croissant_info: Option<(Pos, usize)>,
}

impl PartialEq for BoardState {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board
            && self.move_num % 2 == other.move_num % 2
            && self.en_croissant_info.map(|(pos, _)| pos)
                == other.en_croissant_info.map(|(pos, _)| pos)
    }
}

impl Eq for BoardState {}

impl BoardState {
    #[cfg(debug_assertions)]
    fn validate(&self) {
        let white_king = self.board.get(self.white_king);
        assert!(!white_king.is_outside());
        assert!(!white_king.is_empty());
        assert!(white_king.piece_raw() == square::KING);
        assert_eq!(white_king.owner(), Some(Player::First));

        let black_king = self.board.get(self.black_king);
        assert!(!black_king.is_outside());
        assert!(!black_king.is_empty());
        assert!(black_king.piece_raw() == square::KING);
        assert_eq!(black_king.owner(), Some(Player::Second));
    }

    pub fn initial_state_standard_chess() -> Self {
        // let board = vec![
        //     vec!['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'],
        //     vec!['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
        //     vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        //     vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        //     vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        //     vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        //     vec!['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
        //     vec!['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r'],
        // ];
        let board = vec![
            vec!['R', ' ', ' ', ' ', 'K', ' ', ' ', 'R'],
            vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            vec!['r', ' ', ' ', ' ', 'k', ' ', ' ', 'r'],
        ];
        // let board = vec![
        //     vec!['R', ' ', ' ', ' ', 'K', ' ', ' ', 'R'],
        //     vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        //     vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        //     vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        //     vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        //     vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        //     vec![' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        //     vec![' ', ' ', ' ', ' ', 'k', ' ', ' ', ' '],
        // ];
        debug_assert_eq!(board.len(), 8);
        for row in &board {
            debug_assert_eq!(row.len(), 8);
        }
        let mut board_content = BoardContent::new();
        let mut white_king = None;
        let mut black_king = None;
        #[allow(clippy::needless_range_loop)]
        for row in 0..8 {
            for col in 0..8 {
                let pos = Pos::from_grid(row, col);
                board_content.set(
                    pos,
                    match board[row][col] {
                        ' ' => SquareContents::empty(),
                        'p' => SquareContents::white_pawn(),
                        'r' => SquareContents::white_rook(),
                        'n' => SquareContents::white_knight(),
                        'b' => SquareContents::white_bishop(),
                        'q' => SquareContents::white_queen(),
                        'k' => SquareContents::white_king(),
                        'P' => SquareContents::black_pawn(),
                        'R' => SquareContents::black_rook(),
                        'N' => SquareContents::black_knight(),
                        'B' => SquareContents::black_bishop(),
                        'Q' => SquareContents::black_queen(),
                        'K' => SquareContents::black_king(),
                        _ => unreachable!(),
                    },
                );
                match board[row][col] {
                    'k' => {
                        if white_king.is_none() {
                            white_king = Some(pos);
                        } else {
                            panic!()
                        }
                    }
                    'K' => {
                        if black_king.is_none() {
                            black_king = Some(pos);
                        } else {
                            panic!()
                        }
                    }
                    _ => {}
                }
            }
        }

        Self {
            board: board_content,
            white_king: white_king.unwrap(),
            black_king: black_king.unwrap(),
            move_num: 0,
            en_croissant_info: None,
        }
    }

    fn set(&mut self, pos: Pos, content: SquareContents) {
        self.board.set(pos, content);
    }

    fn get(&self, pos: Pos) -> SquareContents {
        self.board.get(pos)
    }

    fn hash_bits(&self) -> u64 {
        let hash_bits = if let Some((pos, _)) = self.en_croissant_info {
            self.board.hash_bits().wrapping_add(pos.idx as u64)
        } else {
            self.board.hash_bits()
        };
        hash_bits + (self.move_num as u64) % 2
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Move {
    Teleport {
        from: Pos,
        from_content: SquareContents,
        to: Pos,
        to_content: SquareContents,
        capture: bool,
        king_move: bool,
    },
    PawnDoublePush {
        from: Pos,
        from_content: SquareContents,
        croissant: Pos,
        prev_en_croissant_info: Option<(Pos, usize)>,
        to: Pos,
        to_content: SquareContents,
    },
    PawnEnCroissantCapture {
        from: Pos,
        from_content: SquareContents,
        to: Pos,
        to_content: SquareContents,
        capture: Pos,
        capture_content: SquareContents,
    },
    Castle {
        king_from: Pos,
        king_from_content: SquareContents,
        king_to: Pos,
        king_to_content: SquareContents,
        rook_from: Pos,
        rook_from_content: SquareContents,
        rook_to: Pos,
        rook_to_content: SquareContents,
    },
}

impl StandardChessGame {
    fn is_check(&self, player: Player, board: &BoardState) -> bool {
        let king_pos = match player {
            Player::First => board.white_king,
            Player::Second => board.black_king,
        };
        !self.attackers(player, board, king_pos).is_empty()
    }

    // A list of pieces on the other team which are attacking pos
    fn attackers_naive(&self, turn: Player, board: &BoardState, pos: Pos) -> Vec<Pos> {
        let mut attackers = vec![];
        for mv in self.pseudolegal_moves::<false>(turn.flip(), board) {
            match mv {
                Move::Teleport { from, to, .. } => {
                    if to == pos {
                        attackers.push(from);
                    }
                }
                Move::PawnDoublePush { .. } => {}
                Move::PawnEnCroissantCapture { .. } => {}
                Move::Castle { .. } => {}
            }
        }
        attackers
    }

    fn attackers(&self, turn: Player, board: &BoardState, pos: Pos) -> Vec<Pos> {
        let mut attackers = vec![];

        let forward = DPos::from_grid(
            match turn {
                Player::First => -1,
                Player::Second => 1,
            },
            0,
        );
        const LEFT: DPos = DPos::from_grid(0, -1);
        const RIGHT: DPos = DPos::from_grid(0, 1);

        // check for knights
        for other_pos in [
            pos + DPos::from_grid(1, 2),
            pos + DPos::from_grid(-1, 2),
            pos + DPos::from_grid(-2, 1),
            pos + DPos::from_grid(-2, -1),
            pos + DPos::from_grid(-1, -2),
            pos + DPos::from_grid(1, -2),
            pos + DPos::from_grid(2, -1),
            pos + DPos::from_grid(2, 1),
        ] {
            let other_pos_content = board.get(other_pos);
            if !other_pos_content.is_outside()
                && let Some(other_owner) = other_pos_content.owner()
                && other_owner == turn.flip()
                && other_pos_content.piece_raw() == square::KNIGHT
            {
                attackers.push(other_pos);
            }
        }

        // check for kings
        for other_pos in [
            pos + DPos::from_grid(0, 1),
            pos + DPos::from_grid(-1, 1),
            pos + DPos::from_grid(-1, 0),
            pos + DPos::from_grid(-1, -1),
            pos + DPos::from_grid(0, -1),
            pos + DPos::from_grid(1, -1),
            pos + DPos::from_grid(1, 0),
            pos + DPos::from_grid(1, 1),
        ] {
            let other_pos_content = board.get(other_pos);
            if !other_pos_content.is_outside()
                && let Some(other_owner) = other_pos_content.owner()
                && other_owner == turn.flip()
                && other_pos_content.piece_raw() == square::KING
            {
                attackers.push(other_pos);
            }
        }

        // check for pawns
        for other_pos in [pos + LEFT + forward, pos + RIGHT + forward] {
            let other_pos_content = board.get(other_pos);
            if !other_pos_content.is_outside()
                && let Some(other_owner) = other_pos_content.owner()
                && other_owner == turn.flip()
                && other_pos_content.piece_raw() == square::PAWN
            {
                attackers.push(other_pos);
            }
        }

        // check for sliding attacks
        for (dir, possible_enemies) in [
            (DPos::from_grid(0, 1), [square::QUEEN, square::ROOK]),
            (DPos::from_grid(-1, 1), [square::QUEEN, square::BISHOP]),
            (DPos::from_grid(-1, 0), [square::QUEEN, square::ROOK]),
            (DPos::from_grid(-1, -1), [square::QUEEN, square::BISHOP]),
            (DPos::from_grid(0, -1), [square::QUEEN, square::ROOK]),
            (DPos::from_grid(1, -1), [square::QUEEN, square::BISHOP]),
            (DPos::from_grid(1, 0), [square::QUEEN, square::ROOK]),
            (DPos::from_grid(1, 1), [square::QUEEN, square::BISHOP]),
        ] {
            let mut other_pos = pos;
            loop {
                other_pos = other_pos + dir;
                let other_pos_content = board.get(other_pos);
                if other_pos_content.is_outside() {
                    break;
                }
                if let Some(other_owner) = other_pos_content.owner() {
                    if other_owner == turn {
                        break;
                    } else {
                        if other_pos_content.piece_raw() == possible_enemies[0]
                            || other_pos_content.piece_raw() == possible_enemies[1]
                        {
                            attackers.push(other_pos);
                        }
                        break;
                    }
                }
            }
        }

        #[cfg(debug_assertions)]
        {
            let attackers_debug = self.attackers_naive(turn, board, pos);
            if attackers.len() != attackers_debug.len() {
                println!("{:?} {:?}", turn, pos.to_grid());
                println!("{:?} {:?}", attackers.len(), attackers_debug.len());
                println!("attackers_debug");
                for attack_pos in attackers_debug {
                    let attack_pos_content = board.get(attack_pos);
                    println!(
                        "{:?} {:?} {:?}",
                        pos.to_grid(),
                        attack_pos.to_grid(),
                        attack_pos_content.piece()
                    );
                }
                println!("attackers");
                for attack_pos in attackers {
                    let attack_pos_content = board.get(attack_pos);
                    println!(
                        "{:?} {:?} {:?}",
                        pos.to_grid(),
                        attack_pos.to_grid(),
                        attack_pos_content.piece()
                    );
                }
                panic!();
            }
        }

        attackers
    }

    fn pseudolegal_moves<const CAPTURES_ONLY: bool>(
        &self,
        turn: Player,
        board: &BoardState,
    ) -> Vec<Move> {
        #[cfg(debug_assertions)]
        board.validate();

        let mut moves = vec![];
        let forward = DPos::from_grid(
            match turn {
                Player::First => -1,
                Player::Second => 1,
            },
            0,
        );
        const LEFT: DPos = DPos::from_grid(0, -1);
        const RIGHT: DPos = DPos::from_grid(0, 1);

        fn sliding_moves<const CAPTURES_ONLY: bool>(
            board: &BoardState,
            turn: Player,
            moves: &mut Vec<Move>,
            from: Pos,
            from_content: SquareContents,
            dir: DPos,
        ) {
            let mut to = from + dir;
            loop {
                let to_content = board.get(to);
                if to_content.is_outside() {
                    break;
                }
                match to_content.owner() {
                    Some(owner) => {
                        if owner == turn {
                            break;
                        } else {
                            moves.push(Move::Teleport {
                                from,
                                from_content,
                                to,
                                to_content,
                                capture: true,
                                king_move: false,
                            });
                            break;
                        }
                    }
                    None => {
                        if !CAPTURES_ONLY {
                            moves.push(Move::Teleport {
                                from,
                                from_content,
                                to,
                                to_content,
                                capture: false,
                                king_move: false,
                            });
                        }
                        to = to + dir;
                    }
                }
            }
        }

        for row in 0..8 {
            for col in 0..8 {
                let from = Pos::from_grid(row, col);
                let from_content = board.get(from);
                if let Some(owner) = from_content.owner()
                    && owner == turn
                {
                    let piece_raw = from_content.piece_raw();
                    match piece_raw {
                        square::PAWN => {
                            let one_step = from + forward;
                            if !CAPTURES_ONLY {
                                // Pawn move 1 ahead
                                let one_step_content = board.get(one_step);
                                if !one_step_content.is_outside() && one_step_content.is_empty() {
                                    moves.push(Move::Teleport {
                                        from,
                                        from_content,
                                        to: one_step,
                                        to_content: one_step_content,
                                        capture: false,
                                        king_move: false,
                                    });

                                    // Pawn move 2 ahead
                                    if !from_content.is_moved() {
                                        let two_step = one_step + forward;
                                        let two_step_content = board.get(two_step);
                                        if !two_step_content.is_outside()
                                            && two_step_content.is_empty()
                                        {
                                            moves.push(Move::PawnDoublePush {
                                                from,
                                                from_content,
                                                croissant: one_step,
                                                prev_en_croissant_info: board.en_croissant_info,
                                                to: two_step,
                                                to_content: two_step_content,
                                            });
                                        }
                                    }
                                }
                            }
                            // Pawn captures left
                            {
                                let forward_left = one_step + LEFT;
                                let forward_left_content = board.get(forward_left);
                                if !forward_left_content.is_outside() {
                                    if forward_left_content.owner() == Some(turn.flip()) {
                                        moves.push(Move::Teleport {
                                            from,
                                            from_content,
                                            to: forward_left,
                                            to_content: forward_left_content,
                                            capture: true,
                                            king_move: false,
                                        });
                                    } else if let Some((en_croissant_pos, en_croissant_move_num)) =
                                        board.en_croissant_info
                                        && en_croissant_move_num + 1 == board.move_num
                                        && en_croissant_pos == forward_left
                                    {
                                        let capture = forward_left - forward;
                                        let capture_content = board.get(capture);
                                        moves.push(Move::PawnEnCroissantCapture {
                                            from,
                                            from_content,
                                            capture,
                                            capture_content,
                                            to: forward_left,
                                            to_content: forward_left_content,
                                        });
                                    }
                                }
                            }
                            // Pawn captures right
                            {
                                let forward_right = one_step + RIGHT;
                                let forward_right_content = board.get(forward_right);
                                if !forward_right_content.is_outside()
                                    && forward_right_content.owner() == Some(turn.flip())
                                {
                                    moves.push(Move::Teleport {
                                        from,
                                        from_content,
                                        to: forward_right,
                                        to_content: forward_right_content,
                                        capture: true,
                                        king_move: false,
                                    });
                                } else if let Some((en_croissant_pos, en_croissant_move_num)) =
                                    board.en_croissant_info
                                    && en_croissant_move_num + 1 == board.move_num
                                    && en_croissant_pos == forward_right
                                {
                                    let capture = forward_right - forward;
                                    let capture_content = board.get(capture);
                                    moves.push(Move::PawnEnCroissantCapture {
                                        from,
                                        from_content,
                                        capture,
                                        capture_content,
                                        to: forward_right,
                                        to_content: forward_right_content,
                                    });
                                }
                            }
                        }
                        square::KNIGHT => {
                            for to in [
                                from + DPos::from_grid(1, 2),
                                from + DPos::from_grid(-1, 2),
                                from + DPos::from_grid(-2, 1),
                                from + DPos::from_grid(-2, -1),
                                from + DPos::from_grid(-1, -2),
                                from + DPos::from_grid(1, -2),
                                from + DPos::from_grid(2, -1),
                                from + DPos::from_grid(2, 1),
                            ] {
                                let to_content = board.get(to);
                                if !to_content.is_outside() && to_content.owner() != Some(owner) {
                                    let capture = to_content.owner().is_some();
                                    if !CAPTURES_ONLY || capture {
                                        moves.push(Move::Teleport {
                                            from,
                                            from_content,
                                            to,
                                            to_content,
                                            capture,
                                            king_move: false,
                                        })
                                    }
                                }
                            }
                        }
                        square::KING => {
                            for to in [
                                from + DPos::from_grid(0, 1),
                                from + DPos::from_grid(-1, 1),
                                from + DPos::from_grid(-1, 0),
                                from + DPos::from_grid(-1, -1),
                                from + DPos::from_grid(0, -1),
                                from + DPos::from_grid(1, -1),
                                from + DPos::from_grid(1, 0),
                                from + DPos::from_grid(1, 1),
                            ] {
                                let to_content = board.get(to);
                                if !to_content.is_outside() && to_content.owner() != Some(owner) {
                                    let capture = to_content.owner().is_some();
                                    if !CAPTURES_ONLY || capture {
                                        moves.push(Move::Teleport {
                                            from,
                                            from_content,
                                            to,
                                            to_content,
                                            capture,
                                            king_move: true,
                                        })
                                    }
                                }
                            }
                        }
                        square::ROOK => {
                            for dir in [
                                DPos::from_grid(0, 1),
                                DPos::from_grid(-1, 0),
                                DPos::from_grid(0, -1),
                                DPos::from_grid(1, 0),
                            ] {
                                sliding_moves::<CAPTURES_ONLY>(
                                    board,
                                    turn,
                                    &mut moves,
                                    from,
                                    from_content,
                                    dir,
                                );
                            }
                        }
                        square::BISHOP => {
                            for dir in [
                                DPos::from_grid(1, 1),
                                DPos::from_grid(-1, 1),
                                DPos::from_grid(1, -1),
                                DPos::from_grid(-1, -1),
                            ] {
                                sliding_moves::<CAPTURES_ONLY>(
                                    board,
                                    turn,
                                    &mut moves,
                                    from,
                                    from_content,
                                    dir,
                                );
                            }
                        }
                        square::QUEEN => {
                            for dir in [
                                DPos::from_grid(0, 1),
                                DPos::from_grid(-1, 0),
                                DPos::from_grid(0, -1),
                                DPos::from_grid(1, 0),
                                DPos::from_grid(1, 1),
                                DPos::from_grid(-1, 1),
                                DPos::from_grid(1, -1),
                                DPos::from_grid(-1, -1),
                            ] {
                                sliding_moves::<CAPTURES_ONLY>(
                                    board,
                                    turn,
                                    &mut moves,
                                    from,
                                    from_content,
                                    dir,
                                );
                            }
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
            }
        }

        // Castling
        if !CAPTURES_ONLY {
            let castle_row = match turn {
                Player::First => 7,
                Player::Second => 0,
            };
            let king_from = Pos::from_grid(castle_row, 4);
            let king_from_content = board.get(king_from);
            if !king_from_content.is_empty() && !king_from_content.is_moved() {
                debug_assert_eq!(king_from_content.piece_raw(), square::KING);
                debug_assert_eq!(king_from_content.owner(), Some(turn));
                // Left rook
                {
                    let rook_from = Pos::from_grid(castle_row, 0);
                    let rook_from_content = board.get(rook_from);
                    if !rook_from_content.is_empty() && !rook_from_content.is_moved() {
                        debug_assert_eq!(rook_from_content.owner(), Some(turn));
                        debug_assert_eq!(rook_from_content.piece_raw(), square::ROOK);
                        let rook_mid = Pos::from_grid(castle_row, 1);
                        let rook_mid_content = board.get(rook_mid);
                        let king_to = Pos::from_grid(castle_row, 2);
                        let king_to_content = board.get(king_to);
                        let rook_to = Pos::from_grid(castle_row, 3);
                        let rook_to_content = board.get(rook_to);
                        if rook_mid_content.is_empty()
                            && king_to_content.is_empty()
                            && rook_to_content.is_empty()
                        {
                            moves.push(Move::Castle {
                                king_from,
                                king_from_content,
                                king_to,
                                king_to_content,
                                rook_from,
                                rook_from_content,
                                rook_to,
                                rook_to_content,
                            });
                        }
                    }
                }

                // Right rook
                {
                    let rook_from = Pos::from_grid(castle_row, 7);
                    let rook_from_content = board.get(rook_from);
                    if !rook_from_content.is_empty() && !rook_from_content.is_moved() {
                        debug_assert_eq!(rook_from_content.owner(), Some(turn));
                        debug_assert_eq!(rook_from_content.piece_raw(), square::ROOK);
                        let king_to = Pos::from_grid(castle_row, 6);
                        let king_to_content = board.get(king_to);
                        let rook_to = Pos::from_grid(castle_row, 5);
                        let rook_to_content = board.get(rook_to);
                        if king_to_content.is_empty() && rook_to_content.is_empty() {
                            moves.push(Move::Castle {
                                king_from,
                                king_from_content,
                                king_to,
                                king_to_content,
                                rook_from,
                                rook_from_content,
                                rook_to,
                                rook_to_content,
                            });
                        }
                    }
                }
            }
        }

        moves
    }

    fn legal_moves<const CAPTURES_ONLY: bool>(
        &self,
        turn: Player,
        board: &mut BoardState,
    ) -> Vec<Move> {
        let mut legal_moves = vec![];
        for mv in self.pseudolegal_moves::<CAPTURES_ONLY>(turn, board) {
            // TODO this is slow
            // Don't want to clone or modify the board here
            // But can use this for debug mode

            self.make_move(board, &mv);
            let mut is_legal = !self.is_check(turn, board);
            self.unmake_move(board, &mv);

            if let Move::Castle {
                king_from, rook_to, ..
            } = mv
            {
                // Can't castle through check
                if !self.attackers(turn, board, rook_to).is_empty() {
                    is_legal = false;
                }
                // Can't castle when in check
                if !self.attackers(turn, board, king_from).is_empty() {
                    is_legal = false;
                }
            }

            if is_legal {
                legal_moves.push(mv);
            }
        }
        legal_moves
    }
}

impl Score for i64 {
    fn pos_inf() -> Self {
        i64::MAX
    }

    fn neg_inf() -> Self {
        i64::MIN + 1
    }
}

impl GameLogic for StandardChessGame {
    type State = BoardState;
    type Move = Move;
    type Score = i64;

    fn turn(&self, state: &Self::State) -> Player {
        if state.move_num % 2 == 0 {
            Player::First
        } else {
            Player::Second
        }
    }

    fn initial_state(&self) -> Self::State {
        BoardState::initial_state_standard_chess()
    }

    fn hash_state(&self, board: &Self::State) -> u64 {
        hash64(board.hash_bits())
    }

    fn generate_moves(&self, board: &mut Self::State) -> Vec<Self::Move> {
        self.legal_moves::<false>(self.turn(board), board)
    }

    fn generate_quiescence_moves(&self, board: &mut Self::State) -> Vec<Self::Move> {
        self.legal_moves::<true>(self.turn(board), board)
    }

    fn make_move(&self, board: &mut Self::State, mv: &Self::Move) {
        #[cfg(debug_assertions)]
        board.validate();

        if let Move::Teleport {
            from_content,
            to,
            to_content,
            capture,
            king_move,
            ..
        } = mv
        {
            debug_assert_eq!(*capture, to_content.owner().is_some());
            if *king_move {
                match from_content.owner() {
                    Some(Player::First) => board.white_king = *to,
                    Some(Player::Second) => board.black_king = *to,
                    None => unreachable!(),
                }
            }
        }

        match mv {
            Move::Teleport {
                from,
                from_content,
                to,
                to_content,
                ..
            }
            | Move::PawnDoublePush {
                from,
                from_content,
                to,
                to_content,
                ..
            } => {
                debug_assert_ne!(from, to);
                debug_assert!(!from_content.is_outside());
                debug_assert!(!to_content.is_outside());
                debug_assert_ne!(from_content.owner(), to_content.owner());
                board.set(*from, SquareContents::empty());
                board.set(*to, from_content.moved());
            }
            Move::PawnEnCroissantCapture {
                from,
                from_content,
                capture,
                capture_content,
                to,
                to_content,
            } => {
                debug_assert!(!from_content.is_outside());
                debug_assert!(from_content.owner().is_some());
                debug_assert!(!to_content.is_outside());
                debug_assert!(to_content.owner().is_none());
                debug_assert!(!capture_content.is_outside());
                debug_assert!(capture_content.owner().is_some());
                debug_assert_ne!(from_content.owner(), capture_content.owner());
                board.set(*from, SquareContents::empty());
                board.set(*to, from_content.moved());
                board.set(*capture, SquareContents::empty());
            }
            Move::Castle {
                king_from,
                king_from_content,
                king_to,
                king_to_content,
                rook_from,
                rook_from_content,
                rook_to,
                rook_to_content,
            } => {
                debug_assert!(!king_from_content.is_outside());
                debug_assert!(!king_to_content.is_outside());
                debug_assert!(!rook_from_content.is_outside());
                debug_assert!(!rook_to_content.is_outside());
                debug_assert!(!king_from_content.is_empty());
                debug_assert!(king_to_content.is_empty());
                debug_assert!(!rook_from_content.is_empty());
                debug_assert!(rook_to_content.is_empty());
                debug_assert_eq!(king_from_content.piece_raw(), square::KING);
                debug_assert_eq!(rook_from_content.piece_raw(), square::ROOK);
                debug_assert_ne!(king_from, king_to);
                debug_assert_ne!(king_from, rook_from);
                debug_assert_ne!(king_from, rook_to);
                debug_assert_ne!(king_to, rook_from);
                debug_assert_ne!(king_to, rook_to);
                debug_assert_ne!(rook_from, rook_to);
                board.set(*king_from, SquareContents::empty());
                board.set(*king_to, king_from_content.moved());
                board.set(*rook_from, SquareContents::empty());
                board.set(*rook_to, rook_from_content.moved());
                match king_from_content.owner() {
                    Some(Player::First) => board.white_king = *king_to,
                    Some(Player::Second) => board.black_king = *king_to,
                    None => unreachable!(),
                }
            }
        }

        if let Move::PawnDoublePush { croissant, .. } = mv {
            board.en_croissant_info = Some((*croissant, board.move_num))
        }

        board.move_num += 1;

        #[cfg(debug_assertions)]
        board.validate();
    }

    fn unmake_move(&self, board: &mut Self::State, mv: &Self::Move) {
        #[cfg(debug_assertions)]
        board.validate();
        debug_assert!(board.move_num > 0);
        board.move_num -= 1;

        if let Move::Teleport {
            from,
            from_content,
            king_move,
            ..
        } = mv
            && *king_move
        {
            match from_content.owner() {
                Some(Player::First) => board.white_king = *from,
                Some(Player::Second) => board.black_king = *from,
                None => unreachable!(),
            }
        }

        match mv {
            Move::Teleport {
                from,
                from_content,
                to,
                to_content,
                ..
            }
            | Move::PawnDoublePush {
                from,
                from_content,
                to,
                to_content,
                ..
            } => {
                debug_assert_ne!(from, to);
                board.set(*from, *from_content);
                board.set(*to, *to_content);
            }
            Move::PawnEnCroissantCapture {
                from,
                from_content,
                to,
                to_content,
                capture,
                capture_content,
            } => {
                board.set(*from, *from_content);
                board.set(*capture, *capture_content);
                board.set(*to, *to_content);
            }
            Move::Castle {
                king_from,
                king_from_content,
                king_to,
                king_to_content,
                rook_from,
                rook_from_content,
                rook_to,
                rook_to_content,
            } => {
                debug_assert!(!king_from_content.is_outside());
                debug_assert!(!king_to_content.is_outside());
                debug_assert!(!rook_from_content.is_outside());
                debug_assert!(!rook_to_content.is_outside());
                debug_assert!(!king_from_content.is_empty());
                debug_assert!(king_to_content.is_empty());
                debug_assert!(!rook_from_content.is_empty());
                debug_assert!(rook_to_content.is_empty());
                debug_assert_eq!(king_from_content.piece_raw(), square::KING);
                debug_assert_eq!(rook_from_content.piece_raw(), square::ROOK);
                board.set(*king_to, *king_to_content);
                board.set(*king_from, *king_from_content);
                board.set(*rook_to, *rook_to_content);
                board.set(*rook_from, *rook_from_content);
                match king_from_content.owner() {
                    Some(Player::First) => board.white_king = *king_from,
                    Some(Player::Second) => board.black_king = *king_from,
                    None => unreachable!(),
                }
            }
        }

        if let Move::PawnDoublePush {
            prev_en_croissant_info,
            ..
        } = mv
        {
            board.en_croissant_info = *prev_en_croissant_info
        }

        #[cfg(debug_assertions)]
        board.validate();
    }

    fn score(&self, board: &mut Self::State) -> Self::Score {
        let turn = self.turn(board);
        let legal_moves = self.legal_moves::<false>(turn, board);
        if legal_moves.is_empty() {
            if self.is_check(turn, board) {
                match turn {
                    Player::First => Self::Score::neg_inf(),
                    Player::Second => Self::Score::pos_inf(),
                }
            } else {
                0
            }
        } else {
            let mut total: Self::Score = 0;

            total += self.pseudolegal_moves::<false>(Player::First, board).len() as Self::Score;
            total -= self.pseudolegal_moves::<false>(Player::Second, board).len() as Self::Score;

            for row in 0..8 {
                for col in 0..8 {
                    let pos = Pos::from_grid(row, col);
                    let content = board.get(pos);
                    debug_assert!(!content.is_outside());
                    if !content.is_empty() {
                        let piece = content.piece_raw();
                        let score = match piece {
                            square::PAWN => 100,
                            square::ROOK => 500,
                            square::KNIGHT => 300,
                            square::BISHOP => 300,
                            square::QUEEN => 900,
                            square::KING => 10000,
                            _ => unreachable!(),
                        };
                        match content.owner() {
                            Some(Player::First) => {
                                total += score;
                            }
                            Some(Player::Second) => {
                                total -= score;
                            }
                            None => unreachable!(),
                        }
                    }
                }
            }
            total
        }
    }
}

#[derive(Debug, Clone)]
pub enum MoveSelectionState {
    Initial,
    PieceSelected { row: usize, col: usize },
}

impl GridGame for StandardChessGame {
    const ROWS: usize = 8;
    const COLS: usize = 8;

    fn piece(&self, state: &Self::State, row: usize, col: usize) -> super::Piece {
        state.get(Pos::from_grid(row, col)).piece()
    }

    type MoveSelectionState = MoveSelectionState;

    fn initial_move_selection(&self) -> Self::MoveSelectionState {
        MoveSelectionState::Initial
    }

    fn show_move(
        &self,
        turn: Player,
        state: &Self::State,
        mv: Self::Move,
        cell_size: f32,
        cell_to_rect: impl Fn(usize, usize) -> egui::Rect,
        painter: &Painter,
    ) {
        let pos_to_rect =
            |pos: Pos| -> Option<Rect> { pos.to_grid().map(|(row, col)| cell_to_rect(row, col)) };

        let highlight = |pos: Pos, color: Color32| {
            if let Some(rect) = pos_to_rect(pos) {
                let rect = rect.shrink(0.03 * cell_size).shrink(0.03 * cell_size);
                painter.rect_filled(rect, 0.2 * cell_size, color);
            }
        };

        let show_arrow = |from: Pos, to: Pos| {
            highlight(from, Color32::ORANGE.gamma_multiply(0.5));
            highlight(to, Color32::ORANGE.gamma_multiply(0.5));
            painter.arrow(
                pos_to_rect(from).unwrap().center(),
                pos_to_rect(to).unwrap().center() - pos_to_rect(from).unwrap().center(),
                Stroke::new(0.05 * cell_size, Color32::ORANGE),
            );
        };

        match mv {
            Move::Teleport { from, to, .. }
            | Move::PawnDoublePush { from, to, .. }
            | Move::PawnEnCroissantCapture { from, to, .. } => show_arrow(from, to),
            Move::Castle {
                king_from, king_to, ..
            } => show_arrow(king_from, king_to),
        }
    }

    fn update_move_selection(
        &self,
        turn: Player,
        board: &Self::State,
        action: super::MoveSelectionAction,
        move_selection_state: &mut Self::MoveSelectionState,
    ) -> Option<Self::Move> {
        let moves = self.legal_moves::<false>(turn, &mut board.clone());
        match (action, move_selection_state.clone()) {
            (super::MoveSelectionAction::ClickSquare { row, col }, MoveSelectionState::Initial) => {
                let pos = Pos::from_grid(row, col);
                let pos_content = board.get(pos);
                if pos_content.owner() == Some(turn) {
                    *move_selection_state = MoveSelectionState::PieceSelected { row, col };
                } else {
                    *move_selection_state = MoveSelectionState::Initial;
                }
                None
            }
            (
                super::MoveSelectionAction::ClickSquare { row, col },
                MoveSelectionState::PieceSelected {
                    row: piece_row,
                    col: piece_col,
                },
            ) => {
                let pos = Pos::from_grid(row, col);
                let pos_content = board.get(pos);
                let piece_pos = Pos::from_grid(piece_row, piece_col);
                for mv in moves {
                    match mv {
                        Move::Teleport { from, to, .. }
                        | Move::PawnDoublePush { from, to, .. }
                        | Move::PawnEnCroissantCapture { from, to, .. } => {
                            if from == piece_pos && to == pos {
                                return Some(mv);
                            }
                        }
                        Move::Castle {
                            king_from,
                            king_from_content,
                            king_to,
                            king_to_content,
                            rook_from,
                            rook_from_content,
                            rook_to,
                            rook_to_content,
                        } => {
                            if king_from == piece_pos && king_to == pos {
                                return Some(mv);
                            }
                        }
                    }
                }
                if pos_content.owner() == Some(turn) {
                    *move_selection_state = MoveSelectionState::PieceSelected { row, col };
                } else {
                    *move_selection_state = MoveSelectionState::Initial;
                }
                None
            }
            _ => {
                *move_selection_state = MoveSelectionState::Initial;
                None
            }
        }
    }

    fn draw_move_selection(
        &self,
        turn: Player,
        board: &Self::State,
        move_selection_state: &Self::MoveSelectionState,
        cell_size: f32,
        cell_to_rect: impl Fn(usize, usize) -> egui::Rect,
        painter: &Painter,
    ) {
        let highlight = |pos: Pos, color: Color32| {
            if let Some((row, col)) = pos.to_grid() {
                let rect = cell_to_rect(row, col)
                    .shrink(0.03 * cell_size)
                    .shrink(0.03 * cell_size);
                painter.rect_filled(rect, 0.2 * cell_size, color);
            }
        };

        let draw_move = |to: Pos, capture: bool| {
            highlight(
                to,
                if capture {
                    Color32::RED.gamma_multiply(0.5)
                } else {
                    Color32::CYAN
                        .lerp_to_gamma(Color32::GREEN, 0.5)
                        .gamma_multiply(0.5)
                },
            );
        };

        let moves = self.legal_moves::<false>(turn, &mut board.clone());
        match move_selection_state {
            MoveSelectionState::Initial => {}
            MoveSelectionState::PieceSelected { row, col } => {
                let selected_pos = Pos::from_grid(*row, *col);
                highlight(
                    selected_pos,
                    Color32::CYAN
                        .lerp_to_gamma(Color32::BLUE, 0.5)
                        .gamma_multiply(0.5),
                );
                for mv in moves {
                    match mv {
                        Move::Teleport {
                            from, to, capture, ..
                        } => {
                            if from == selected_pos {
                                draw_move(to, capture);
                            }
                        }
                        Move::PawnDoublePush { from, to, .. } => {
                            if from == selected_pos {
                                draw_move(to, false);
                            }
                        }
                        Move::PawnEnCroissantCapture { from, to, .. } => {
                            if from == selected_pos {
                                draw_move(to, true);
                            }
                        }
                        Move::Castle {
                            king_from, king_to, ..
                        } => {
                            if king_from == selected_pos {
                                draw_move(king_to, false);
                            }
                        }
                    }
                }
            }
        }
    }
}
