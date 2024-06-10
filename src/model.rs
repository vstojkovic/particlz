//! Engine-agnostic game data and logic

use enum_map::{Enum, EnumMap};
use enumset::{enum_set, EnumSet, EnumSetType};
pub use grid::GridMap;
use strum::IntoEnumIterator;
use strum_macros::{EnumCount, EnumIter, FromRepr};

mod grid;
mod pbc1;

pub use pbc1::Pbc1DecodeError;

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

#[derive(Clone)]
pub struct Board {
    pub dims: Dimensions,
    pub tiles: GridMap<Tile>,
    pub horz_borders: GridMap<Border>,
    pub vert_borders: GridMap<Border>,
    pub pieces: GridMap<Piece>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimensions {
    pub rows: usize,
    pub cols: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BoardCoords {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Piece {
    Particle(Particle),
    Manipulator(Manipulator),
}

#[derive(Debug, Clone)]
pub struct Particle {
    pub tint: Tint,
}

#[derive(Debug, Clone)]
pub struct Manipulator {
    pub emitters: Emitters,
    targets: EnumMap<Direction, Option<BeamTarget>>,
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
        let dims = Dimensions::new(rows, cols);
        let tiles = GridMap::new(rows, cols);
        let horz_borders = GridMap::new(rows + 1, cols);
        let vert_borders = GridMap::new(rows, cols + 1);
        let pieces = GridMap::new(rows, cols);

        Self {
            dims,
            tiles,
            horz_borders,
            vert_borders,
            pieces,
        }
    }

    pub fn from_pbc1(code: &str) -> Result<Self, Pbc1DecodeError> {
        pbc1::decode(code)
    }

    pub fn copy_state_from(&mut self, other: &Self) {
        assert_eq!(self.dims.rows, other.dims.rows);
        assert_eq!(self.dims.cols, other.dims.cols);

        self.tiles.mirror(&other.tiles);
        self.horz_borders.mirror(&other.horz_borders);
        self.vert_borders.mirror(&other.vert_borders);
        self.pieces.mirror(&other.pieces);
    }

    pub fn neighbor(&self, coords: BoardCoords, direction: Direction) -> Option<BoardCoords> {
        match direction {
            Direction::Up => coords
                .row
                .checked_add_signed(-1)
                .map(|row| (row, coords.col).into()),
            Direction::Left => coords
                .col
                .checked_add_signed(-1)
                .map(|col| (coords.row, col).into()),
            Direction::Down => Some(coords.row + 1)
                .filter(|&row| row < self.dims.rows)
                .map(|row| (row, coords.col).into()),
            Direction::Right => Some(coords.col + 1)
                .filter(|&col| col < self.dims.cols)
                .map(|col| (coords.row, col).into()),
        }
    }

    pub fn borders(&self, orientation: Orientation) -> &GridMap<Border> {
        match orientation {
            Orientation::Horizontal => &self.horz_borders,
            Orientation::Vertical => &self.vert_borders,
        }
    }

    pub fn move_piece(&mut self, from_coords: BoardCoords, to_coords: BoardCoords) {
        let piece = self.pieces.take(from_coords);
        self.pieces.set(to_coords, piece);
    }

    pub fn retarget_beams(&mut self) {
        for coords in self.dims.iter() {
            let emitters = match self.pieces.get(coords) {
                Some(Piece::Manipulator(manipulator)) => manipulator.emitters,
                _ => continue,
            };
            for direction in emitters.directions() {
                let target = self.find_beam_target(coords, direction);
                let manipulator = self
                    .pieces
                    .get_mut(coords)
                    .unwrap()
                    .as_manipulator_mut()
                    .unwrap();
                manipulator.targets[direction] = Some(target);
            }
        }
    }

    pub fn compute_allowed_moves(&self, coords: BoardCoords) -> EnumSet<Direction> {
        let mut moves = EnumSet::empty();

        for direction in Direction::iter() {
            let Some(neighbor) = self.neighbor(coords, direction) else {
                continue;
            };
            let border_coords = coords.to_border_coords(direction);
            let border_orientation = direction.orientation().flip();
            if self.pieces.get(neighbor).is_none()
                && self
                    .borders(border_orientation)
                    .get(border_coords)
                    .is_none()
            {
                moves.insert(direction);
            }
        }

        moves
    }

    pub fn prev_manipulator(&self, coords: Option<BoardCoords>) -> Option<BoardCoords> {
        // NOTE: An active board should never have 0 manipulators
        let mut coords = coords.unwrap_or_default();
        let mut remaining = self.dims.rows * self.dims.cols;
        while remaining > 0 {
            if coords.col > 0 {
                coords.col -= 1;
            } else {
                coords.col = self.dims.cols - 1;
                if coords.row > 0 {
                    coords.row -= 1;
                } else {
                    coords.row = self.dims.rows - 1;
                }
            }
            if let Some(Piece::Manipulator(_)) = self.pieces.get(coords) {
                return Some(coords);
            }
            remaining -= 1;
        }
        None
    }

    pub fn next_manipulator(&self, coords: Option<BoardCoords>) -> Option<BoardCoords> {
        // NOTE: An active board should never have 0 manipulators
        let max_row = self.dims.rows - 1;
        let max_col = self.dims.cols - 1;
        let mut coords = coords.unwrap_or_else(|| BoardCoords::new(max_row, max_col));
        let mut remaining = self.dims.rows * self.dims.cols;
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
            if let Some(Piece::Manipulator(_)) = self.pieces.get(coords) {
                return Some(coords);
            }
            remaining -= 1;
        }
        None
    }

    pub fn find_beam_target(&self, coords: BoardCoords, direction: Direction) -> BeamTarget {
        let mut piece_coords = coords;
        let border_orientation = direction.orientation().flip();

        loop {
            let border_coords = piece_coords.to_border_coords(direction);
            if let Some(Border::Wall) = self.borders(border_orientation).get(border_coords) {
                return BeamTarget::border(border_coords);
            }
            piece_coords = match self.neighbor(piece_coords, direction) {
                Some(neighbor) => neighbor,
                None => return BeamTarget::border(border_coords),
            };
            if self.pieces.get(piece_coords).is_some() {
                return BeamTarget::piece(piece_coords);
            }
        }
    }
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

    pub fn iter(self) -> impl Iterator<Item = BoardCoords> {
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

impl Piece {
    pub fn as_manipulator(&self) -> Option<&Manipulator> {
        if let Self::Manipulator(manipulator) = self {
            Some(manipulator)
        } else {
            None
        }
    }

    pub fn as_manipulator_mut(&mut self) -> Option<&mut Manipulator> {
        if let Self::Manipulator(manipulator) = self {
            Some(manipulator)
        } else {
            None
        }
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
        Self {
            emitters,
            targets: EnumMap::default(),
        }
    }

    pub fn target(&self, direction: Direction) -> Option<BeamTarget> {
        self.targets[direction]
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
