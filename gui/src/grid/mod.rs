use egui::{Painter, Rect};

use crate::game::{GameLogic, Player};
use std::fmt::Debug;

pub mod chess;
pub mod ui;

pub enum MoveSelectionAction {
    Reset,
    ClickSquare { row: usize, col: usize },
}

pub trait GridGame: GameLogic {
    const ROWS: usize;
    const COLS: usize;
    type Square;

    fn square(&self, state: &Self::State, row: usize, col: usize) -> Self::Square;
    fn square_to_icon(&self, square: &Self::Square) -> Option<&'static str>;

    type MoveSelectionState: Debug;

    fn initial_move_selection(&self) -> Self::MoveSelectionState;
    fn update_move_selection(
        &self,
        turn: &Player,
        action: MoveSelectionAction,
        move_selection_state: &mut Self::MoveSelectionState,
    ) -> Option<Self::Move>;
    fn draw_move_selection(
        &self,
        move_selection_state: &Self::MoveSelectionState,
        cell_size: f32,
        cell_to_rect: impl Fn(usize, usize) -> Rect,
        painter: &Painter,
    );
}
