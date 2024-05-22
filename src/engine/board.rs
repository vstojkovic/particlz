use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, Resource};
use bevy::hierarchy::BuildChildren;
use bevy::prelude::SpatialBundle;

use crate::model::{Board, Piece};

use super::border::{BorderBundle, Orientation};
use super::manipulator::ManipulatorBundle;
use super::particle::ParticleBundle;
use super::tile::TileBundle;
use super::Assets;

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
