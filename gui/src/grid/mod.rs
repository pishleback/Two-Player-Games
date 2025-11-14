use egui::{Painter, Rect};

use crate::game::{GameLogic, Player};
use std::fmt::Debug;

pub mod chess;
pub mod ui;

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

pub enum MoveSelectionAction {
    Reset,
    ClickSquare { row: usize, col: usize },
}

pub trait GridGame: GameLogic {
    const ROWS: usize;
    const COLS: usize;

    fn piece(&self, state: &Self::State, row: usize, col: usize) -> Piece;

    type MoveSelectionState: Debug + Clone;

    fn initial_move_selection(&self) -> Self::MoveSelectionState;

    fn show_move(
        &self,
        turn: Player,
        state: &Self::State,
        mv: Self::Move,
        cell_size: f32,
        cell_to_rect: impl Fn(usize, usize) -> Rect,
        painter: &Painter,
    );

    fn update_move_selection(
        &self,
        turn: Player,
        state: &Self::State,
        action: MoveSelectionAction,
        move_selection_state: &mut Self::MoveSelectionState,
    ) -> Option<Self::Move>;

    fn draw_move_selection_on_grid(
        &self,
        turn: Player,
        state: &Self::State,
        move_selection_state: &Self::MoveSelectionState,
        cell_size: f32,
        cell_to_rect: impl Fn(usize, usize) -> Rect,
        painter: &Painter,
    );

    fn update_move_selection_ui(
        &self,
        turn: Player,
        state: &Self::State,
        move_selection_state: &Self::MoveSelectionState,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
    ) -> Option<Self::Move>;
}
