//! Engine-agnostic game data and logic

use enumset::{enum_set, EnumSet, EnumSetType};
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BoardCoords {
    pub row: usize,
    pub col: usize,
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

#[derive(Debug, Clone, Copy)]
pub struct BeamTarget {
    pub kind: BeamTargetKind,
    pub coords: BoardCoords,
}

#[derive(Debug, Clone, Copy)]
pub enum BeamTargetKind {
    Piece,
    Border,
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

    pub fn get_tile(&self, coords: BoardCoords) -> Option<&Tile> {
        self.tiles[coords.row * self.cols + coords.col].as_ref()
    }

    pub fn set_tile<T: Into<Option<Tile>>>(&mut self, coords: BoardCoords, tile: T) {
        self.tiles[coords.row * self.cols + coords.col] = tile.into();
    }

    pub fn get_horz_border(&self, coords: BoardCoords) -> Option<&Border> {
        self.horz_borders[coords.row * self.cols + coords.col].as_ref()
    }

    pub fn set_horz_border<B: Into<Option<Border>>>(&mut self, coords: BoardCoords, border: B) {
        self.horz_borders[coords.row * self.cols + coords.col] = border.into();
    }

    pub fn get_vert_border(&self, coords: BoardCoords) -> Option<&Border> {
        self.vert_borders[coords.row * (self.cols + 1) + coords.col].as_ref()
    }

    pub fn set_vert_border<B: Into<Option<Border>>>(&mut self, coords: BoardCoords, border: B) {
        self.vert_borders[coords.row * (self.cols + 1) + coords.col] = border.into();
    }

    pub fn get_piece(&self, coords: BoardCoords) -> Option<&Piece> {
        self.pieces[coords.row * self.cols + coords.col].as_ref()
    }

    pub fn set_piece<T: Into<Option<Piece>>>(&mut self, coords: BoardCoords, piece: T) {
        self.pieces[coords.row * self.cols + coords.col] = piece.into();
    }

    pub fn take_piece(&mut self, coords: BoardCoords) -> Option<Piece> {
        self.pieces[coords.row * self.cols + coords.col].take()
    }

    pub fn compute_allowed_moves(&self, coords: BoardCoords) -> EnumSet<Direction> {
        let mut moves = EnumSet::empty();

        if coords.row > 0
            && self.get_horz_border(coords).is_none()
            && self.get_piece(coords.move_to(Direction::Up)).is_none()
        {
            moves.insert(Direction::Up);
        }
        if coords.col > 0
            && self.get_vert_border(coords).is_none()
            && self.get_piece(coords.move_to(Direction::Left)).is_none()
        {
            moves.insert(Direction::Left);
        }
        if coords.row < (self.rows - 1)
            && self
                .get_horz_border(coords.move_to(Direction::Down))
                .is_none()
            && self.get_piece(coords.move_to(Direction::Down)).is_none()
        {
            moves.insert(Direction::Down);
        }
        if coords.col < (self.cols - 1)
            && self
                .get_vert_border(coords.move_to(Direction::Right))
                .is_none()
            && self.get_piece(coords.move_to(Direction::Right)).is_none()
        {
            moves.insert(Direction::Right);
        }

        moves
    }

    pub fn prev_manipulator(&self, coords: Option<BoardCoords>) -> Option<BoardCoords> {
        // NOTE: An active board should never have 0 manipulators
        let mut coords = coords.unwrap_or_default();
        let mut remaining = self.rows * self.cols;
        while remaining > 0 {
            if coords.col > 0 {
                coords.col -= 1;
            } else {
                coords.col = self.cols - 1;
                if coords.row > 0 {
                    coords.row -= 1;
                } else {
                    coords.row = self.rows - 1;
                }
            }
            if let Some(Piece::Manipulator(_)) = self.get_piece(coords) {
                if !self.compute_allowed_moves(coords).is_empty() {
                    return Some(coords);
                }
            }
            remaining -= 1;
        }
        None
    }

    pub fn next_manipulator(&self, coords: Option<BoardCoords>) -> Option<BoardCoords> {
        // NOTE: An active board should never have 0 manipulators
        let max_row = self.rows - 1;
        let max_col = self.cols - 1;
        let mut coords = coords.unwrap_or_else(|| BoardCoords::new(max_row, max_col));
        let mut remaining = self.rows * self.cols;
        while remaining > 0 {
            if coords.col < max_col {
                coords.col += 1;
            } else {
                coords.col = 0;
                if coords.row < max_row {
                    coords.row += 1;
                } else {
                    coords.row = 0;
                }
            }
            if let Some(Piece::Manipulator(_)) = self.get_piece(coords) {
                if !self.compute_allowed_moves(coords).is_empty() {
                    return Some(coords);
                }
            }
            remaining -= 1;
        }
        None
    }

    pub fn find_beam_target(&self, coords: BoardCoords, direction: Direction) -> BeamTarget {
        let (row_delta, col_delta, mut border_coords, get_border): (
            isize,
            isize,
            BoardCoords,
            fn(&Self, BoardCoords) -> Option<&Border>,
        ) = match direction {
            Direction::Up => (-1, 0, coords, Self::get_horz_border),
            Direction::Left => (0, -1, coords, Self::get_vert_border),
            Direction::Down => (1, 0, coords.move_to(Direction::Down), Self::get_horz_border),
            Direction::Right => (
                0,
                1,
                coords.move_to(Direction::Right),
                Self::get_vert_border,
            ),
        };
        let mut piece_coords = coords;

        loop {
            if let Some(Border::Wall) = get_border(self, border_coords) {
                return BeamTarget::border(border_coords);
            }
            match border_coords.row.checked_add_signed(row_delta) {
                Some(row) if (row <= self.rows) => border_coords.row = row,
                _ => return BeamTarget::border(border_coords),
            }
            match border_coords.col.checked_add_signed(col_delta) {
                Some(col) if (col <= self.cols) => border_coords.col = col,
                _ => return BeamTarget::border(border_coords),
            }
            piece_coords.row = piece_coords.row.wrapping_add_signed(row_delta);
            piece_coords.col = piece_coords.col.wrapping_add_signed(col_delta);
            if self.get_piece(piece_coords).is_some() {
                return BeamTarget::piece(piece_coords);
            }
        }
    }
}

impl BoardCoords {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub fn move_to(self, direction: Direction) -> Self {
        match direction {
            Direction::Up => (self.row - 1, self.col),
            Direction::Left => (self.row, self.col - 1),
            Direction::Down => (self.row + 1, self.col),
            Direction::Right => (self.row, self.col + 1),
        }
        .into()
    }
}

impl From<(usize, usize)> for BoardCoords {
    fn from(value: (usize, usize)) -> Self {
        Self::new(value.0, value.1)
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

impl Emitters {
    pub fn directions(self) -> EnumSet<Direction> {
        match self {
            Self::Left => enum_set!(Direction::Left),
            Self::Up => enum_set!(Direction::Up),
            Self::Right => enum_set!(Direction::Right),
            Self::Down => enum_set!(Direction::Down),
            Self::LeftUp => enum_set!(Direction::Left | Direction::Up),
            Self::LeftDown => enum_set!(Direction::Left | Direction::Down),
            Self::RightUp => enum_set!(Direction::Right | Direction::Up),
            Self::RightDown => enum_set!(Direction::Right | Direction::Down),
            Self::LeftRight => enum_set!(Direction::Left | Direction::Right),
            Self::UpDown => enum_set!(Direction::Up | Direction::Down),
        }
    }
}

impl BeamTarget {
    pub fn border(coords: BoardCoords) -> Self {
        Self {
            kind: BeamTargetKind::Border,
            coords,
        }
    }

    pub fn piece(coords: BoardCoords) -> Self {
        Self {
            kind: BeamTargetKind::Piece,
            coords,
        }
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
