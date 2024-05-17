use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, Resource};
use bevy::hierarchy::BuildChildren;
use bevy::prelude::SpatialBundle;

use crate::tile::{Tile, TileBundle};
use crate::Assets;

pub struct Board {
    rows: usize,
    cols: usize,
    tiles: Vec<Option<Tile>>,
}

#[derive(Resource)]
pub struct BoardResource {
    board: Board,
    parent: Entity,
    tiles: Vec<Option<Entity>>,
}

#[derive(Bundle, Default)]
struct BoardBundle {
    spatial: SpatialBundle,
}

impl Board {
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut tiles = Vec::with_capacity(rows * cols);
        tiles.resize_with(rows * cols, || None);
        Self { rows, cols, tiles }
    }

    pub fn get_tile(&self, row: usize, col: usize) -> Option<&Tile> {
        self.tiles[row * self.cols + col].as_ref()
    }

    pub fn set_tile<T: Into<Option<Tile>>>(&mut self, row: usize, col: usize, tile: T) {
        self.tiles[row * self.cols + col] = tile.into();
    }
}

impl BoardResource {
    pub fn new(board: Board) -> Self {
        let tiles = Vec::with_capacity(board.rows * board.cols);
        Self {
            board,
            parent: Entity::PLACEHOLDER,
            tiles,
        }
    }

    pub fn spawn(&mut self, commands: &mut Commands, assets: &Assets) {
        let mut parent = commands.spawn(BoardBundle::default());
        self.parent = parent.id();
        parent.with_children(|parent| {
            for row in 0..self.board.rows {
                for col in 0..self.board.cols {
                    self.tiles.push(self.board.get_tile(row, col).map(|tile| {
                        parent
                            .spawn(TileBundle::new(tile, row, col, &assets.tiles))
                            .id()
                    }));
                }
            }
        });
    }
}
