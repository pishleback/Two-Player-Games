use crate::{
    game::{GameLogic, Player},
    grid::GridGame,
};

pub struct StandardChessGame {}
impl StandardChessGame {
    pub fn new() -> Self {
        Self {}
    }
}

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
        state: u8,
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
use egui::{Color32, Painter};
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

    const fn add(self, other: Self) -> Self {
        Self {
            idx: self.idx + other.idx,
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum SquareSeenBy {
    None,
}

#[derive(Debug, Clone)]
pub struct BoardContent {
    /*
    A 12x10 grid. The outer squares are for edge-detection.
    The inner 8x8 grid is the standard chess board.
     */
    data: [SquareContents; 120],
}
impl BoardContent {
    fn set(&mut self, pos: Pos, content: SquareContents) {
        debug_assert_ne!(self.get(pos), SquareContents::outside());
        self.data[pos.idx] = content;
    }

    fn get(&self, pos: Pos) -> SquareContents {
        self.data[pos.idx]
    }

    fn compute_visibility(&self) -> [[SquareSeenBy; 120]; 120] {
        println!("{:?}", self.data);
        std::array::from_fn(|_| std::array::from_fn(|_| SquareSeenBy::None))
    }
}

#[derive(Debug, Clone)]
pub struct BoardState {
    board: BoardContent,
    move_num: usize,
    // If a pawn just double-moved, store the phantom capture square and the move on which the pawn moved.
    en_croissant_info: Option<(Pos, usize)>,
    // For each square, which squares is it seen by and in what way?
    visibility: [[SquareSeenBy; 120]; 120],
}

impl BoardState {
    pub fn initial_state_standard_chess() -> Self {
        let board = vec![
            vec!['#', '#', '#', '#', '#', '#', '#', '#', '#', '#'],
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
            vec!['#', '#', '#', '#', '#', '#', '#', '#', '#', '#'],
        ];
        debug_assert_eq!(board.len(), 12);
        for row in &board {
            debug_assert_eq!(row.len(), 10);
        }
        let board_content = BoardContent {
            data: std::array::from_fn(|i| {
                let r = i / 10;
                let c = i % 10;
                match board[r][c] {
                    ' ' => SquareContents::empty(),
                    '#' => SquareContents::outside(),
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
                }
            }),
        };
        Self {
            visibility: board_content.compute_visibility(),
            board: board_content,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Move {
    Null,
    Teleport {
        from: Pos,
        from_content: SquareContents,
        to: Pos,
        to_content: SquareContents,
        capture: bool,
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
}

impl StandardChessGame {
    fn generate_pseudomoves(&self, turn: Player, board: &BoardState) -> Vec<Move> {
        let mut moves = vec![];
        moves.push(Move::Null);

        let forward = DPos::from_grid(
            match turn {
                Player::First => -1,
                Player::Second => 1,
            },
            0,
        );
        const LEFT: DPos = DPos::from_grid(0, -1);
        const RIGHT: DPos = DPos::from_grid(0, 1);

        fn sliding_moves(
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
                            });
                            break;
                        }
                    }
                    None => {
                        moves.push(Move::Teleport {
                            from,
                            from_content,
                            to,
                            to_content,
                            capture: false,
                        });
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
                            // Pawn move 1 ahead
                            let one_step = from + forward;
                            let one_step_content = board.get(one_step);
                            if !one_step_content.is_outside() && one_step_content.is_empty() {
                                moves.push(Move::Teleport {
                                    from,
                                    from_content,
                                    to: one_step,
                                    to_content: one_step_content,
                                    capture: false,
                                });
                                // Pawn move 2 ahead
                                if !from_content.is_moved() {
                                    let two_step = one_step + forward;
                                    let two_step_content = board.get(two_step);
                                    if !two_step_content.is_outside() && two_step_content.is_empty()
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
                                    moves.push(Move::Teleport {
                                        from,
                                        from_content,
                                        to,
                                        to_content,
                                        capture: to_content.owner().is_some(),
                                    })
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
                                    moves.push(Move::Teleport {
                                        from,
                                        from_content,
                                        to,
                                        to_content,
                                        capture: to_content.owner().is_some(),
                                    })
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
                                sliding_moves(board, turn, &mut moves, from, from_content, dir);
                            }
                        }
                        square::BISHOP => {
                            for dir in [
                                DPos::from_grid(1, 1),
                                DPos::from_grid(-1, 1),
                                DPos::from_grid(1, -1),
                                DPos::from_grid(-1, -1),
                            ] {
                                sliding_moves(board, turn, &mut moves, from, from_content, dir);
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
                                sliding_moves(board, turn, &mut moves, from, from_content, dir);
                            }
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
            }
        }
        moves
    }
}

impl GameLogic for StandardChessGame {
    type State = BoardState;
    type Move = Move;
    type Score = i64;

    fn initial_state(&self) -> Self::State {
        BoardState::initial_state_standard_chess()
    }

    fn generate_moves(&self, turn: Player, board: &Self::State) -> Vec<Self::Move> {
        self.generate_pseudomoves(turn, board)
    }

    fn make_move(&self, board: &mut Self::State, mv: &Self::Move) {
        match mv {
            Move::Teleport {
                from,
                from_content,
                to,
                to_content,
                capture,
            } => {
                debug_assert_eq!(*capture, to_content.owner().is_some());
            }
            _ => {}
        }

        match mv {
            Move::Null => {}
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
        }

        match mv {
            Move::PawnDoublePush {
                from,
                from_content,
                croissant,
                prev_en_croissant_info,
                to,
                to_content,
            } => board.en_croissant_info = Some((*croissant, board.move_num)),
            _ => {}
        }

        board.move_num += 1;
    }

    fn unmake_move(&self, board: &mut Self::State, mv: &Self::Move) {
        debug_assert!(board.move_num > 0);
        board.move_num -= 1;

        match mv {
            Move::Null => {}
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
        }

        match mv {
            Move::PawnDoublePush {
                from,
                from_content,
                croissant,
                prev_en_croissant_info,
                to,
                to_content,
            } => board.en_croissant_info = *prev_en_croissant_info,
            _ => {}
        }
    }

    fn score(&self, board: &Self::State) -> Self::Score {
        0
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

    fn update_move_selection(
        &self,
        turn: Player,
        board: &Self::State,
        action: super::MoveSelectionAction,
        move_selection_state: &mut Self::MoveSelectionState,
    ) -> Option<Self::Move> {
        let moves = self.generate_moves(turn, board);
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
                        Move::Null => {}
                        Move::Teleport { from, to, .. }
                        | Move::PawnDoublePush { from, to, .. }
                        | Move::PawnEnCroissantCapture { from, to, .. } => {
                            if from == piece_pos && to == pos {
                                return Some(mv);
                            }
                        }
                    }
                }
                let pos_content = board.get(pos);
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

        let draw_move = |from: Pos, to: Pos, capture: bool| {
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

        let moves = self.generate_moves(turn, board);
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
                        Move::Null => {}
                        Move::Teleport {
                            from,
                            from_content,
                            to,
                            to_content,
                            capture,
                        } => {
                            if from == selected_pos {
                                draw_move(from, to, capture);
                            }
                        }
                        Move::PawnDoublePush {
                            from,
                            from_content,
                            croissant,
                            prev_en_croissant_info,
                            to,
                            to_content,
                        } => {
                            if from == selected_pos {
                                draw_move(from, to, false);
                            }
                        }
                        Move::PawnEnCroissantCapture {
                            from,
                            from_content,
                            to,
                            to_content,
                            capture,
                            capture_content,
                        } => {
                            if from == selected_pos {
                                draw_move(from, to, true);
                            }
                        }
                    }
                }
            }
        }
    }
}
