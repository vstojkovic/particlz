use enum_map::{Enum, EnumMap};
use enumset::{enum_set, EnumSet};
use strum_macros::{EnumIter, FromRepr};

use super::{BoardCoords, Direction, Tint};

#[derive(Debug, Clone)]
pub struct Tile {
    pub kind: TileKind,
    pub tint: Tint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enum, EnumIter, FromRepr)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enum, EnumIter, FromRepr)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeamTargetKind {
    Piece,
    Border,
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

    pub(super) fn set_target(&mut self, direction: Direction, target: BeamTarget) {
        self.targets[direction] = Some(target);
    }

    pub fn iter_targets(&self) -> impl Iterator<Item = BeamTarget> + '_ {
        self.emitters
            .directions()
            .iter()
            .map(|direction| self.targets[direction].unwrap())
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
