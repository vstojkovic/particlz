//! Engine-agnostic game data and logic

use std::fmt::Debug;

use enum_map::Enum;
use enumset::EnumSetType;
use strum_macros::{EnumCount, EnumIter, FromRepr};

mod board;
mod element;
mod grid;
mod movement;
mod pbc1;
mod support;

pub use board::Board;
pub use element::{
    BeamTarget, BeamTargetKind, Border, Emitters, Manipulator, Particle, Piece, Tile, TileKind,
};
pub use grid::{GridMap, GridSet};

pub const MAX_BOARD_ROWS: usize = 15;
pub const MAX_BOARD_COLS: usize = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, FromRepr)]
#[repr(u8)]
pub enum Tint {
    White,
    Green,
    Yellow,
    Red,
}

#[derive(Debug, Hash, EnumIter, EnumCount, EnumSetType, Enum)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimensions {
    pub rows: usize,
    pub cols: usize,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct BoardCoords {
    pub row: usize,
    pub col: usize,
}

impl Direction {
    pub fn orientation(self) -> Orientation {
        match self {
            Self::Up | Self::Down => Orientation::Vertical,
            Self::Left | Self::Right => Orientation::Horizontal,
        }
    }
}

impl Orientation {
    pub fn flip(self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}

impl Dimensions {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self { rows, cols }
    }

    pub fn contains(&self, coords: BoardCoords) -> bool {
        (coords.row < self.rows) && (coords.col < self.cols)
    }

    pub fn iter(self) -> impl DoubleEndedIterator<Item = BoardCoords> {
        (0..(self.rows * self.cols)).map(move |idx| self.coords(idx))
    }

    fn coords(&self, idx: usize) -> BoardCoords {
        BoardCoords::new(idx / self.cols, idx % self.cols)
    }

    fn index(&self, coords: BoardCoords) -> usize {
        coords.row * self.cols + coords.col
    }
}

impl BoardCoords {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub fn to_border_coords(self, direction: Direction) -> Self {
        match direction {
            Direction::Up | Direction::Left => self,
            Direction::Down => (self.row + 1, self.col).into(),
            Direction::Right => (self.row, self.col + 1).into(),
        }
    }
}

impl Debug for BoardCoords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
}

impl From<(usize, usize)> for BoardCoords {
    fn from(value: (usize, usize)) -> Self {
        Self::new(value.0, value.1)
    }
}
