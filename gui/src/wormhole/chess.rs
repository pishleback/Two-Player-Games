use crate::chess_pieces::square::SquareContents;

pub struct BoardContent {
    pieces: [SquareContents; 144],
}

impl BoardContent {
    pub fn starting_position() -> Self {
        let mut pieces = [SquareContents::empty(); 144];

        // Top
        pieces[1] = SquareContents::white_pawn();
        pieces[9] = SquareContents::white_pawn();
        pieces[17] = SquareContents::white_pawn();
        pieces[25] = SquareContents::white_pawn();
        pieces[33] = SquareContents::white_pawn();
        pieces[41] = SquareContents::white_pawn();
        pieces[49] = SquareContents::white_pawn();
        pieces[57] = SquareContents::white_pawn();
        pieces[0] = SquareContents::white_rook();
        pieces[8] = SquareContents::white_knight();
        pieces[16] = SquareContents::white_bishop();
        pieces[24] = SquareContents::white_king();
        pieces[32] = SquareContents::white_queen();
        pieces[40] = SquareContents::white_bishop();
        pieces[48] = SquareContents::white_knight();
        pieces[56] = SquareContents::white_rook();

        pieces[6] = SquareContents::black_pawn();
        pieces[14] = SquareContents::black_pawn();
        pieces[22] = SquareContents::black_pawn();
        pieces[30] = SquareContents::black_pawn();
        pieces[38] = SquareContents::black_pawn();
        pieces[46] = SquareContents::black_pawn();
        pieces[54] = SquareContents::black_pawn();
        pieces[62] = SquareContents::black_pawn();
        pieces[7] = SquareContents::black_rook();
        pieces[15] = SquareContents::black_knight();
        pieces[23] = SquareContents::black_bishop();
        pieces[31] = SquareContents::black_king();
        pieces[39] = SquareContents::black_queen();
        pieces[47] = SquareContents::black_bishop();
        pieces[55] = SquareContents::black_knight();
        pieces[63] = SquareContents::black_rook();

        // Bottom
        pieces[70] = SquareContents::white_pawn();
        pieces[78] = SquareContents::white_pawn();
        pieces[86] = SquareContents::white_pawn();
        pieces[94] = SquareContents::white_pawn();
        pieces[102] = SquareContents::white_pawn();
        pieces[110] = SquareContents::white_pawn();
        pieces[118] = SquareContents::white_pawn();
        pieces[126] = SquareContents::white_pawn();
        pieces[71] = SquareContents::white_rook();
        pieces[79] = SquareContents::white_knight();
        pieces[87] = SquareContents::white_bishop();
        pieces[95] = SquareContents::white_king();
        pieces[103] = SquareContents::white_queen();
        pieces[111] = SquareContents::white_bishop();
        pieces[119] = SquareContents::white_knight();
        pieces[127] = SquareContents::white_rook();

        pieces[65] = SquareContents::black_pawn();
        pieces[73] = SquareContents::black_pawn();
        pieces[81] = SquareContents::black_pawn();
        pieces[89] = SquareContents::black_pawn();
        pieces[97] = SquareContents::black_pawn();
        pieces[105] = SquareContents::black_pawn();
        pieces[113] = SquareContents::black_pawn();
        pieces[121] = SquareContents::black_pawn();
        pieces[64] = SquareContents::black_rook();
        pieces[72] = SquareContents::black_knight();
        pieces[80] = SquareContents::black_bishop();
        pieces[88] = SquareContents::black_king();
        pieces[96] = SquareContents::black_queen();
        pieces[104] = SquareContents::black_bishop();
        pieces[112] = SquareContents::black_knight();
        pieces[120] = SquareContents::black_rook();

        Self { pieces }
    }

    pub fn map<T>(&self, f: impl Fn(&SquareContents) -> T) -> [T; 144] {
        std::array::from_fn(|i| f(&self.pieces[i]))
    }
}
