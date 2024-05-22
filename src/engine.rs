//! Engine-specific game data and logic

use bevy::asset::AssetServer;
use bevy::ecs::system::Resource;

pub mod board;
pub mod border;
pub mod manipulator;
pub mod particle;
pub mod tile;

use self::border::BorderAssets;
use self::manipulator::ManipulatorAssets;
use self::particle::ParticleAssets;
use self::tile::TileAssets;

const TILE_WIDTH: f32 = 45.0;
const TILE_HEIGHT: f32 = 45.0;

#[derive(Resource)]
pub struct Assets {
    tiles: TileAssets,
    borders: BorderAssets,
    particles: ParticleAssets,
    manipulators: ManipulatorAssets,
}

impl Assets {
    pub fn load(server: &AssetServer) -> Self {
        Self {
            tiles: TileAssets::load(server),
            borders: BorderAssets::load(server),
            particles: ParticleAssets::load(server),
            manipulators: ManipulatorAssets::load(server),
        }
    }
}
