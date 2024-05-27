use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, Query, Resource};
use bevy::hierarchy::BuildChildren;
use bevy::math::Vec2;
use bevy::prelude::SpatialBundle;
use bevy::transform::components::Transform;

use crate::model::{Board, Piece};

use super::border::{spawn_horz_border, spawn_vert_border};
use super::focus::spawn_focus;
use super::manipulator::spawn_manipulator;
use super::particle::spawn_particle;
use super::tile::spawn_tile;
use super::{Assets, BoardCoords};

#[derive(Resource)]
pub struct BoardResource {
    pub board: Board,
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
                    self.tiles.push(
                        self.board
                            .get_tile(row, col)
                            .map(|tile| spawn_tile(parent, tile, (row, col).into(), &assets.tiles)),
                    );
                }
            }

            for row in 0..=self.board.rows {
                for col in 0..self.board.cols {
                    self.horz_borders
                        .push(self.board.get_horz_border(row, col).map(|border| {
                            spawn_horz_border(parent, border, (row, col).into(), &assets.borders)
                        }));
                }
            }

            for row in 0..self.board.rows {
                for col in 0..=self.board.cols {
                    self.vert_borders
                        .push(self.board.get_vert_border(row, col).map(|border| {
                            spawn_vert_border(parent, border, (row, col).into(), &assets.borders)
                        }));
                }
            }

            for row in 0..self.board.rows {
                for col in 0..self.board.cols {
                    self.pieces
                        .push(self.board.get_piece(row, col).map(|piece| match piece {
                            Piece::Particle(particle) => spawn_particle(
                                parent,
                                particle,
                                (row, col).into(),
                                &assets.particles,
                            ),
                            Piece::Manipulator(manipulator) => spawn_manipulator(
                                parent,
                                manipulator,
                                (row, col).into(),
                                &assets.manipulators,
                            ),
                        }));
                }
            }

            spawn_focus(parent, &assets.focus);
        });
    }

    pub fn coords_at_pos(
        &self,
        pos: Vec2,
        q_xform: &Query<&Transform>,
    ) -> Option<(BoardCoords, Vec2)> {
        let xform = q_xform.get(self.parent).unwrap();
        let origin = xform.translation.truncate();
        let pos = pos - origin;
        let coords = BoardCoords::from_xy(pos)?;
        if coords.row < self.board.rows && coords.col < self.board.cols {
            let center = coords.to_xy();
            Some((coords, pos - center))
        } else {
            None
        }
    }

    pub fn get_piece(&self, coords: BoardCoords) -> Option<Entity> {
        self.pieces[coords.row * self.board.cols + coords.col].clone()
    }

    pub fn move_piece(
        &mut self,
        from_coords: BoardCoords,
        to_coords: BoardCoords,
        q_anchor: &mut Query<(&mut BoardCoords, &mut Transform)>,
    ) {
        let from_idx = from_coords.row * self.board.cols + from_coords.col;

        let Some(anchor) = self.pieces[from_idx].take() else {
            return;
        };
        let to_idx = to_coords.row * self.board.cols + to_coords.col;
        self.pieces[to_idx] = Some(anchor);

        let piece = self.board.take_piece(from_coords.row, from_coords.col);
        self.board.set_piece(to_coords.row, to_coords.col, piece);

        let (mut anchor_coords, mut anchor_xform) = q_anchor.get_mut(anchor).unwrap();
        *anchor_coords = to_coords;
        anchor_xform.translation = to_coords.to_xy().extend(anchor_xform.translation.z);
    }
}
