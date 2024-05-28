//! Engine-agnostic game data and logic

use enumset::{EnumSet, EnumSetType};
use strum_macros::{EnumCount, EnumIter, FromRepr};

mod pbc1;

pub use pbc1::Pbc1DecodeError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, FromRepr)]
#[repr(u8)]
pub enum Tint {
    White,
    Green,
    Yellow,
    Red,
}

#[derive(Debug, Hash, EnumIter, EnumCount, EnumSetType)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

pub struct Board {
    pub rows: usize,
    pub cols: usize,
    pub tiles: Vec<Option<Tile>>,
    pub horz_borders: Vec<Option<Border>>,
    pub vert_borders: Vec<Option<Border>>,
    pub pieces: Vec<Option<Piece>>,
}

pub struct Tile {
    pub kind: TileKind,
    pub tint: Tint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, FromRepr)]
#[repr(u8)]
pub enum TileKind {
    Platform,
    Collector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum Border {
    Wall,
    Window,
}

pub enum Piece {
    Particle(Particle),
    Manipulator(Manipulator),
}

pub struct Particle {
    pub tint: Tint,
}

pub struct Manipulator {
    pub emitters: Emitters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, FromRepr)]
#[repr(u8)]
pub enum Emitters {
    Left,
    Up,
    Right,
    Down,
    LeftUp,
    LeftDown,
    RightUp,
    RightDown,
    LeftRight,
    UpDown,
}

impl Board {
    pub fn new(rows: usize, cols: usize) -> Self {
        let num_tiles = rows * cols;
        let mut tiles = Vec::with_capacity(num_tiles);
        tiles.resize_with(num_tiles, || None);

        let num_horz_borders = (rows + 1) * cols;
        let mut horz_borders = Vec::with_capacity(num_horz_borders);
        horz_borders.resize_with(num_horz_borders, || None);

        let num_vert_borders = rows * (cols + 1);
        let mut vert_borders = Vec::with_capacity(num_vert_borders);
        vert_borders.resize_with(num_vert_borders, || None);

        let num_pieces = num_tiles;
        let mut pieces = Vec::with_capacity(num_pieces);
        pieces.resize_with(num_pieces, || None);

        Self {
            rows,
            cols,
            tiles,
            horz_borders,
            vert_borders,
            pieces,
        }
    }

    pub fn from_pbc1(code: &str) -> Result<Self, Pbc1DecodeError> {
        pbc1::decode(code)
    }

    pub fn get_tile(&self, row: usize, col: usize) -> Option<&Tile> {
        self.tiles[row * self.cols + col].as_ref()
    }

    pub fn set_tile<T: Into<Option<Tile>>>(&mut self, row: usize, col: usize, tile: T) {
        self.tiles[row * self.cols + col] = tile.into();
    }

    pub fn get_horz_border(&self, row: usize, col: usize) -> Option<&Border> {
        self.horz_borders[row * self.cols + col].as_ref()
    }

    pub fn set_horz_border<B: Into<Option<Border>>>(&mut self, row: usize, col: usize, border: B) {
        self.horz_borders[row * self.cols + col] = border.into();
    }

    pub fn get_vert_border(&self, row: usize, col: usize) -> Option<&Border> {
        self.vert_borders[row * (self.cols + 1) + col].as_ref()
    }

    pub fn set_vert_border<B: Into<Option<Border>>>(&mut self, row: usize, col: usize, border: B) {
        self.vert_borders[row * (self.cols + 1) + col] = border.into();
    }

    pub fn get_piece(&self, row: usize, col: usize) -> Option<&Piece> {
        self.pieces[row * self.cols + col].as_ref()
    }

    pub fn set_piece<T: Into<Option<Piece>>>(&mut self, row: usize, col: usize, piece: T) {
        self.pieces[row * self.cols + col] = piece.into();
    }

    pub fn take_piece(&mut self, row: usize, col: usize) -> Option<Piece> {
        self.pieces[row * self.cols + col].take()
    }

    pub fn compute_allowed_moves(&self, row: usize, col: usize) -> EnumSet<Direction> {
        let mut moves = EnumSet::empty();

        if row > 0
            && self.get_horz_border(row, col).is_none()
            && self.get_piece(row - 1, col).is_none()
        {
            moves.insert(Direction::Up);
        }
        if col > 0
            && self.get_vert_border(row, col).is_none()
            && self.get_piece(row, col - 1).is_none()
        {
            moves.insert(Direction::Left);
        }
        if row < (self.rows - 1)
            && self.get_horz_border(row + 1, col).is_none()
            && self.get_piece(row + 1, col).is_none()
        {
            moves.insert(Direction::Down);
        }
        if col < (self.cols - 1)
            && self.get_vert_border(row, col + 1).is_none()
            && self.get_piece(row, col + 1).is_none()
        {
            moves.insert(Direction::Right);
        }

        moves
    }
}

impl Tile {
    pub fn new(kind: TileKind, tint: Tint) -> Self {
        Self { kind, tint }
    }
}

impl Particle {
    pub fn new(tint: Tint) -> Self {
        assert!(tint != Tint::White);
        Self { tint }
    }
}

impl Manipulator {
    pub fn new(emitters: Emitters) -> Self {
        Self { emitters }
    }
}

impl Into<Option<Piece>> for Particle {
    fn into(self) -> Option<Piece> {
        Some(Piece::Particle(self))
    }
}

impl Into<Option<Piece>> for Manipulator {
    fn into(self) -> Option<Piece> {
        Some(Piece::Manipulator(self))
    }
}
