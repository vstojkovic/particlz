use bevy::app::{App, Startup};
use bevy::asset::AssetServer;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Res, ResMut, Resource};
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
use bevy::DefaultPlugins;
use board::{Board, BoardResource};
use border::{Border, BorderAssets};
use manipulator::{Emitters, Manipulator, ManipulatorAssets};
use particle::{Particle, ParticleAssets};
use strum_macros::EnumIter;
use tile::{Tile, TileAssets, TileKind};

mod board;
mod border;
mod manipulator;
mod particle;
mod tile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum Tint {
    White,
    Green,
    Yellow,
    Red,
}

#[derive(Resource)]
pub struct Assets {
    tiles: TileAssets,
    borders: BorderAssets,
    particles: ParticleAssets,
    manipulators: ManipulatorAssets,
}

fn main() {
    let board = make_test_board();
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Particlz".into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(BoardResource::new(board))
        .add_systems(Startup, (load_assets, setup_board).chain())
        .run();
}

fn load_assets(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(Assets::load(&server));
}

fn setup_board(mut commands: Commands, mut board: ResMut<BoardResource>, assets: Res<Assets>) {
    commands.spawn(Camera2dBundle::default());
    board.spawn(&mut commands, &assets);
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

fn make_test_board() -> Board {
    let mut board = Board::new(5, 5);
    board.set_tile(0, 0, Tile::new(TileKind::Platform, Tint::White));
    board.set_tile(0, 1, Tile::new(TileKind::Platform, Tint::Green));
    board.set_tile(0, 2, Tile::new(TileKind::Platform, Tint::Yellow));
    board.set_tile(0, 3, Tile::new(TileKind::Platform, Tint::Red));
    for row in 1..=3 {
        for col in 0..=4 {
            board.set_tile(row, col, Tile::new(TileKind::Platform, Tint::White));
        }
    }
    board.set_tile(4, 4, Tile::new(TileKind::Collector, Tint::White));
    board.set_tile(4, 3, Tile::new(TileKind::Collector, Tint::Green));
    board.set_tile(4, 2, Tile::new(TileKind::Collector, Tint::Yellow));
    board.set_tile(4, 1, Tile::new(TileKind::Collector, Tint::Red));
    board.set_horz_border(0, 0, Border::Wall);
    board.set_horz_border(1, 0, Border::Wall);
    board.set_horz_border(4, 4, Border::Window);
    board.set_horz_border(5, 4, Border::Window);
    board.set_vert_border(0, 0, Border::Window);
    board.set_vert_border(4, 5, Border::Wall);
    board.set_piece(1, 1, Particle::new(Tint::Green));
    board.set_piece(1, 2, Particle::new(Tint::Yellow));
    board.set_piece(1, 3, Particle::new(Tint::Red));
    board.set_piece(2, 0, Manipulator::new(Emitters::Left));
    board.set_piece(2, 1, Manipulator::new(Emitters::Up));
    board.set_piece(2, 2, Manipulator::new(Emitters::Right));
    board.set_piece(2, 3, Manipulator::new(Emitters::Down));
    board.set_piece(2, 4, Manipulator::new(Emitters::LeftRight));
    board.set_piece(3, 0, Manipulator::new(Emitters::LeftUp));
    board.set_piece(3, 1, Manipulator::new(Emitters::LeftDown));
    board.set_piece(3, 2, Manipulator::new(Emitters::RightUp));
    board.set_piece(3, 3, Manipulator::new(Emitters::RightDown));
    board.set_piece(3, 4, Manipulator::new(Emitters::UpDown));
    board
}
