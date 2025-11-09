use super::chess::BoardState;
use egui::{Color32, Pos2, Rect, Stroke, TextureHandle, Vec2};
use std::collections::HashMap;

pub struct State {
    board: BoardState,
    pieces: HashMap<&'static str, TextureHandle>,
}

impl State {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Self {
        let ctx = &cc.egui_ctx;
        // helper to load embedded PNGs
        let load = |name: &'static str, bytes: &'static [u8]| -> TextureHandle {
            let img = image::load_from_memory(bytes).expect("embedded image failed to load");
            let size = [img.width() as _, img.height() as _];
            let rgba = img.to_rgba8();
            let pixels = rgba.into_flat_samples().samples;
            ctx.load_texture(
                name,
                egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
                egui::TextureOptions::LINEAR,
            )
        };

        let mut pieces = HashMap::new();
        pieces.insert(
            "white_pawn",
            load("white_pawn", include_bytes!("icons/white pawn.png")),
        );
        pieces.insert(
            "white_rook",
            load("white_rook", include_bytes!("icons/white rook.png")),
        );
        pieces.insert(
            "white_knight",
            load("white_knight", include_bytes!("icons/white knight.png")),
        );
        pieces.insert(
            "white_bishop",
            load("white_bishop", include_bytes!("icons/white bishop.png")),
        );
        pieces.insert(
            "white_queen",
            load("white_queen", include_bytes!("icons/white queen.png")),
        );
        pieces.insert(
            "white_king",
            load("white_king", include_bytes!("icons/white king.png")),
        );

        pieces.insert(
            "black_pawn",
            load("black_pawn", include_bytes!("icons/black pawn.png")),
        );
        pieces.insert(
            "black_rook",
            load("black_rook", include_bytes!("icons/black rook.png")),
        );
        pieces.insert(
            "black_knight",
            load("black_knight", include_bytes!("icons/black knight.png")),
        );
        pieces.insert(
            "black_bishop",
            load("black_bishop", include_bytes!("icons/black bishop.png")),
        );
        pieces.insert(
            "black_queen",
            load("black_queen", include_bytes!("icons/black queen.png")),
        );
        pieces.insert(
            "black_king",
            load("black_king", include_bytes!("icons/black king.png")),
        );

        Self {
            board: BoardState::initial_state_standard_chess(),
            pieces,
        }
    }
}

impl eframe::App for State {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Reserve the available space
            let avail = ui.available_rect_before_wrap();
            let avail_size = avail.size();

            // Compute size of one cell: must be square, so use the smaller dimension
            let cell_size = avail_size.x.min(avail_size.y) / 8.0;

            // Compute total board size and center it in the available rect
            let board_size = Vec2::new(cell_size * 8.0, cell_size * 8.0);
            let board_top_left = Pos2::new(
                avail.left() + (avail_size.x - board_size.x) / 2.0,
                avail.top() + (avail_size.y - board_size.y) / 2.0,
            );

            let painter = ui.painter();

            // Define the colours of the squares
            let light = Color32::from_rgb(240, 217, 181); // light square
            let dark = Color32::from_rgb(181, 136, 99); // dark square
            let border = Stroke::new(1.0, Color32::BLACK);

            // Draw the grid
            for row in 0..8 {
                for col in 0..8 {
                    let x = board_top_left.x + (col as f32) * cell_size;
                    let y = board_top_left.y + (row as f32) * cell_size;
                    let r = Rect::from_min_size(Pos2::new(x, y), Vec2::new(cell_size, cell_size));
                    let color = if (row + col) % 2 == 0 { light } else { dark };
                    painter.rect_filled(r, 0.0, color);
                    painter.rect_stroke(r, 0.0, border, egui::StrokeKind::Inside);
                }
            }

            // Draw the pieces
            let draw_piece =
                |name: &str, row: usize, col: usize, pieces: &HashMap<&str, TextureHandle>| {
                    if let Some(tex) = pieces.get(name) {
                        let x = board_top_left.x + (col as f32) * cell_size;
                        let y = board_top_left.y + (row as f32) * cell_size;
                        let r = Rect::from_min_size(Pos2::new(x, y), Vec2::splat(cell_size));
                        painter.image(
                            tex.id(),
                            r,
                            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                            Color32::WHITE, // no tint
                        );
                    }
                };
            for row in 0..8 {
                for col in 0..8 {
                    match self.board.square(row, col) {
                        super::chess::Square::Empty => {}
                        super::chess::Square::WhitePawn => {
                            draw_piece("white_pawn", row, col, &self.pieces);
                        }
                        super::chess::Square::WhiteRook => {
                            draw_piece("white_rook", row, col, &self.pieces);
                        }
                        super::chess::Square::WhiteKnight => {
                            draw_piece("white_knight", row, col, &self.pieces);
                        }
                        super::chess::Square::WhiteBishop => {
                            draw_piece("white_bishop", row, col, &self.pieces);
                        }
                        super::chess::Square::WhiteQueen => {
                            draw_piece("white_queen", row, col, &self.pieces);
                        }
                        super::chess::Square::WhiteKing => {
                            draw_piece("white_king", row, col, &self.pieces);
                        }
                        super::chess::Square::BlackPawn => {
                            draw_piece("black_pawn", row, col, &self.pieces);
                        }
                        super::chess::Square::BlackRook => {
                            draw_piece("black_rook", row, col, &self.pieces);
                        }
                        super::chess::Square::BlackKnight => {
                            draw_piece("black_knight", row, col, &self.pieces);
                        }
                        super::chess::Square::BlackBishop => {
                            draw_piece("black_bishop", row, col, &self.pieces);
                        }
                        super::chess::Square::BlackQueen => {
                            draw_piece("black_queen", row, col, &self.pieces);
                        }
                        super::chess::Square::BlackKing => {
                            draw_piece("black_king", row, col, &self.pieces);
                        }
                    }
                }
            }
        });
    }
}
