//! Engine-specific game data and logic

use bevy::asset::AssetServer;
use bevy::ecs::component::Component;
use bevy::ecs::system::Resource;
use bevy::math::Vec2;
use enumset::EnumSetType;
use strum_macros::EnumIter;

pub mod board;
pub mod border;
pub mod focus;
pub mod manipulator;
pub mod particle;
pub mod tile;

use self::border::BorderAssets;
use self::focus::FocusAssets;
use self::manipulator::ManipulatorAssets;
use self::particle::ParticleAssets;
use self::tile::TileAssets;

const TILE_WIDTH: f32 = 45.0;
const TILE_HEIGHT: f32 = 45.0;

#[derive(Debug, Hash, EnumIter, EnumSetType)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct BoardCoords {
    pub row: usize,
    pub col: usize,
}

#[derive(Resource)]
pub struct Assets {
    tiles: TileAssets,
    borders: BorderAssets,
    particles: ParticleAssets,
    manipulators: ManipulatorAssets,
    focus: FocusAssets,
}

impl BoardCoords {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    fn from_xy(pos: Vec2) -> Option<Self> {
        let pos = pos + Vec2::new(TILE_WIDTH, -TILE_HEIGHT) / 2.0;
        if pos.x < 0.0 || pos.y > 0.0 {
            return None;
        }
        let row = (-pos.y / TILE_HEIGHT).trunc() as usize;
        let col = (pos.x / TILE_WIDTH).trunc() as usize;
        Some(Self { row, col })
    }

    fn to_xy(&self) -> Vec2 {
        Vec2 {
            x: (self.col as f32) * TILE_WIDTH,
            y: -(self.row as f32) * TILE_HEIGHT,
        }
    }
}

impl From<(usize, usize)> for BoardCoords {
    fn from(value: (usize, usize)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl Assets {
    pub fn load(server: &AssetServer) -> Self {
        Self {
            tiles: TileAssets::load(server),
            borders: BorderAssets::load(server),
            particles: ParticleAssets::load(server),
            manipulators: ManipulatorAssets::load(server),
            focus: FocusAssets::load(server),
        }
    }
}