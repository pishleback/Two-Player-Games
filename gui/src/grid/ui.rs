use crate::{
    ai::Ai,
    game::Game,
    grid::{GridGame, Piece},
};
use egui::{Color32, Pos2, Rect, Stroke, TextureHandle, Vec2};
use std::collections::HashMap;

pub struct State<G: GridGame, A: Ai<G>> {
    game: Game<G>,
    ai: A,
    move_selection: G::MoveSelectionState,
    pieces: HashMap<Piece, TextureHandle>,
}

impl<G: GridGame, A: Ai<G>> State<G, A> {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>, game_logic: G) -> Self {
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
            Piece::WhitePawn,
            load("white_pawn", include_bytes!("icons/white pawn.png")),
        );
        pieces.insert(
            Piece::WhiteBerolinaPawn,
            load("white_berolina", include_bytes!("icons/white berolina.png")),
        );
        pieces.insert(
            Piece::WhiteRook,
            load("white_rook", include_bytes!("icons/white rook.png")),
        );
        pieces.insert(
            Piece::WhiteKnight,
            load("white_knight", include_bytes!("icons/white knight.png")),
        );
        pieces.insert(
            Piece::WhiteBishop,
            load("white_bishop", include_bytes!("icons/white bishop.png")),
        );
        pieces.insert(
            Piece::WhiteQueen,
            load("white_queen", include_bytes!("icons/white queen.png")),
        );
        pieces.insert(
            Piece::WhiteKing,
            load("white_king", include_bytes!("icons/white king.png")),
        );
        pieces.insert(
            Piece::WhiteGrasshopper,
            load(
                "white_grasshopper",
                include_bytes!("icons/white grasshopper.png"),
            ),
        );

        pieces.insert(
            Piece::BlackPawn,
            load("black_pawn", include_bytes!("icons/black pawn.png")),
        );
        pieces.insert(
            Piece::BlackBerolinaPawn,
            load("black_berolina", include_bytes!("icons/black berolina.png")),
        );
        pieces.insert(
            Piece::BlackRook,
            load("black_rook", include_bytes!("icons/black rook.png")),
        );
        pieces.insert(
            Piece::BlackKnight,
            load("black_knight", include_bytes!("icons/black knight.png")),
        );
        pieces.insert(
            Piece::BlackBishop,
            load("black_bishop", include_bytes!("icons/black bishop.png")),
        );
        pieces.insert(
            Piece::BlackQueen,
            load("black_queen", include_bytes!("icons/black queen.png")),
        );
        pieces.insert(
            Piece::BlackKing,
            load("black_king", include_bytes!("icons/black king.png")),
        );
        pieces.insert(
            Piece::BlackGrasshopper,
            load(
                "black_grasshopper",
                include_bytes!("icons/black grasshopper.png"),
            ),
        );

        let game = Game::new(game_logic.clone());
        let mut ai = A::new();
        ai.set_game(game.clone());
        Self {
            move_selection: game_logic.initial_move_selection(),
            ai,
            game,
            pieces,
        }
    }

    fn make_move(&mut self, mv: G::Move) {
        self.game.make_move(mv);
        self.move_selection = self.game.logic().initial_move_selection();
        self.ai.set_game(self.game.clone());
    }

    fn undo_move(&mut self) {
        self.game.undo_move();
        self.move_selection = self.game.logic().initial_move_selection();
        self.ai.set_game(self.game.clone());
    }
}

impl<G: GridGame, A: Ai<G>> eframe::App for State<G, A> {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.ai.think(chrono::TimeDelta::milliseconds(16));

        if let Some(mv) = self.game.logic().update_move_selection_ui(
            self.game.turn(),
            self.game.state(),
            &self.move_selection,
            ctx,
            frame,
        ) {
            self.make_move(mv);
        }

        let best_moves = self.ai.best_moves();
        let mut show_best_moves = vec![false; best_moves.len()];

        egui::SidePanel::left("left panel").show(ctx, |ui| {
            ui.heading("Game");
            ui.label(format!("Move {}", self.game.num_moves() + 1));

            match self.game.logic().score(&mut self.game.state().clone()) {
                crate::game::AbsScore::SecondPlayerWin => {
                    ui.label("Black Wins");
                }
                crate::game::AbsScore::Draw => {
                    ui.label("Draw");
                }
                crate::game::AbsScore::FirstPlayerWin => {
                    ui.label("White Wins");
                }
                crate::game::AbsScore::Heuristic(_) => match self.game.turn() {
                    crate::game::Player::First => {
                        ui.label("White's Turn");
                    }
                    crate::game::Player::Second => {
                        ui.label("Black's Turn");
                    }
                },
            }

            if self.game.can_undo_move() && ui.button("Undo").clicked() {
                self.undo_move();
            }

            ui.separator();
            ui.heading("Ai");
            for (idx, (label, best_move)) in best_moves.iter().enumerate() {
                let button = ui.button(label);
                if button.hovered() {
                    show_best_moves[idx] = true;
                }
                if button.clicked() {
                    self.make_move(best_move.clone());
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Reserve the available space
            let avail = ui.available_rect_before_wrap();
            let avail_size = avail.size();

            // Compute size of one cell: must be square, so use the smaller dimension
            let cell_size = (avail_size.x / (G::COLS as f32)).min(avail_size.y / (G::ROWS as f32));

            // Compute total board size and center it in the available rect
            let board_size = Vec2::new(cell_size * (G::COLS as f32), cell_size * (G::ROWS as f32));
            let board_top_left = Pos2::new(
                avail.left() + (avail_size.x - board_size.x) / 2.0,
                avail.top() + (avail_size.y - board_size.y) / 2.0,
            );

            let cell_to_rect = |row, col| {
                let x = board_top_left.x + (col as f32) * cell_size;
                let y = board_top_left.y + (row as f32) * cell_size;

                Rect::from_min_size(Pos2::new(x, y), Vec2::new(cell_size, cell_size))
            };

            let painter = ui.painter();

            // Define the colours of the squares
            let light = Color32::from_rgb(240, 217, 181); // light square
            let dark = Color32::from_rgb(181, 136, 99); // dark square
            let border = Stroke::new(1.0, Color32::BLACK);

            // Draw the grid
            for row in 0..G::ROWS {
                for col in 0..G::COLS {
                    let rect = cell_to_rect(row, col);
                    let color = if (row + col) % 2 == 0 { light } else { dark };
                    painter.rect_filled(rect, 0.0, color);
                    painter.rect_stroke(rect, 0.0, border, egui::StrokeKind::Inside);
                }
            }

            // Draw the pieces
            let draw_piece = |row: usize, col: usize, piece: Piece| {
                if let Some(tex) = self.pieces.get(&piece) {
                    let rect = cell_to_rect(row, col);
                    painter.image(
                        tex.id(),
                        rect,
                        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                        Color32::WHITE, // no tint
                    );
                } else if piece != Piece::Empty {
                    panic!("No icon for piece {:?}", piece);
                }
            };
            for row in 0..G::ROWS {
                for col in 0..G::COLS {
                    draw_piece(
                        row,
                        col,
                        self.game.logic().piece(self.game.state(), row, col),
                    );
                }
            }

            // Draw the move selection state
            self.game.logic().draw_move_selection_on_grid(
                self.game.turn(),
                self.game.state(),
                &self.move_selection,
                cell_size,
                cell_to_rect,
                painter,
            );

            // Handle clicks
            if ui.input(|i| {
                i.pointer.primary_pressed()
                    && if let Some(pos) = i.pointer.latest_pos() {
                        ui.max_rect().contains(pos)
                    } else {
                        false
                    }
            }) && !ui.ctx().wants_pointer_input()
            {
                let mut clicked = None;
                for row in 0..G::ROWS {
                    for col in 0..G::COLS {
                        let rect = cell_to_rect(row, col);
                        let pointer = ctx.input(|i| i.pointer.interact_pos());
                        if let Some(pos) = pointer
                            && ui.input(|i| i.pointer.primary_pressed())
                            && rect.contains(pos)
                        {
                            clicked = Some((row, col));
                        }
                    }
                }
                if let Some(mv) = if let Some((row, col)) = clicked {
                    self.game.logic().update_move_selection(
                        self.game.turn(),
                        self.game.state(),
                        super::MoveSelectionAction::ClickSquare { row, col },
                        &mut self.move_selection,
                    )
                } else {
                    self.game.logic().update_move_selection(
                        self.game.turn(),
                        self.game.state(),
                        super::MoveSelectionAction::Reset,
                        &mut self.move_selection,
                    )
                } {
                    self.make_move(mv);
                }
            }

            // Show best move
            for (idx, (_label, best_move)) in best_moves.iter().enumerate() {
                if show_best_moves[idx] {
                    self.game.logic().show_move(
                        self.game.turn(),
                        self.game.state(),
                        best_move.clone(),
                        cell_size,
                        cell_to_rect,
                        painter,
                    );
                }
            }
        });

        ctx.request_repaint();
    }
}
