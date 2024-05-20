use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, Resource};
use bevy::hierarchy::BuildChildren;
use bevy::prelude::SpatialBundle;

use crate::border::{Border, BorderBundle, Orientation};
use crate::manipulator::{Manipulator, ManipulatorBundle};
use crate::particle::{Particle, ParticleBundle};
use crate::tile::{Tile, TileBundle};
use crate::Assets;

pub struct Board {
    rows: usize,
    cols: usize,
    tiles: Vec<Option<Tile>>,
    horz_borders: Vec<Option<Border>>,
    vert_borders: Vec<Option<Border>>,
    pieces: Vec<Option<Piece>>,
}

pub enum Piece {
    Particle(Particle),
    Manipulator(Manipulator),
}

#[derive(Resource)]
pub struct BoardResource {
    board: Board,
    parent: Entity,
    tiles: Vec<Option<Entity>>,
    horz_borders: Vec<Option<Entity>>,
    vert_borders: Vec<Option<Entity>>,
    pieces: Vec<Option<Entity>>,
}

#[derive(Bundle, Default)]
struct BoardBundle {
    spatial: SpatialBundle,
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
}

impl BoardResource {
    pub fn new(board: Board) -> Self {
        let tiles = Vec::with_capacity(board.tiles.len());
        let horz_borders = Vec::with_capacity(board.horz_borders.len());
        let vert_borders = Vec::with_capacity(board.vert_borders.len());
        let pieces = Vec::with_capacity(board.pieces.len());
        Self {
            board,
            parent: Entity::PLACEHOLDER,
            tiles,
            horz_borders,
            vert_borders,
            pieces,
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

            for row in 0..=self.board.rows {
                for col in 0..self.board.cols {
                    self.horz_borders
                        .push(self.board.get_horz_border(row, col).map(|border| {
                            parent
                                .spawn(BorderBundle::new(
                                    border,
                                    row,
                                    col,
                                    Orientation::Horizontal,
                                    &assets.borders,
                                ))
                                .id()
                        }));
                }
            }

            for row in 0..self.board.rows {
                for col in 0..=self.board.cols {
                    self.vert_borders
                        .push(self.board.get_vert_border(row, col).map(|border| {
                            parent
                                .spawn(BorderBundle::new(
                                    border,
                                    row,
                                    col,
                                    Orientation::Vertical,
                                    &assets.borders,
                                ))
                                .id()
                        }));
                }
            }

            for row in 0..self.board.rows {
                for col in 0..self.board.cols {
                    self.pieces
                        .push(self.board.get_piece(row, col).map(|piece| {
                            match piece {
                                Piece::Particle(particle) => parent.spawn(ParticleBundle::new(
                                    particle,
                                    row,
                                    col,
                                    &assets.particles,
                                )),
                                Piece::Manipulator(manipulator) => {
                                    parent.spawn(ManipulatorBundle::new(
                                        manipulator,
                                        row,
                                        col,
                                        &assets.manipulators,
                                    ))
                                }
                            }
                            .id()
                        }));
                }
            }
        });
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
