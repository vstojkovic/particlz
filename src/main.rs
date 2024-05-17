use bevy::app::{App, Startup};
use bevy::asset::AssetServer;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Res, ResMut, Resource};
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
use bevy::DefaultPlugins;
use board::{Board, BoardResource};
use strum_macros::EnumIter;
use tile::{Tile, TileAssets, TileKind};

mod board;
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
}

fn main() {
    let mut board = Board::new(2, 5);
    board.set_tile(0, 0, Tile::new(TileKind::Platform, Tint::White));
    board.set_tile(0, 1, Tile::new(TileKind::Platform, Tint::Green));
    board.set_tile(0, 2, Tile::new(TileKind::Platform, Tint::Yellow));
    board.set_tile(0, 3, Tile::new(TileKind::Platform, Tint::Red));
    board.set_tile(1, 4, Tile::new(TileKind::Collector, Tint::White));
    board.set_tile(1, 3, Tile::new(TileKind::Collector, Tint::Green));
    board.set_tile(1, 2, Tile::new(TileKind::Collector, Tint::Yellow));
    board.set_tile(1, 1, Tile::new(TileKind::Collector, Tint::Red));

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
        }
    }
}
