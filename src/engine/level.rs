use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, Query, Resource};
use bevy::hierarchy::{BuildChildren, DespawnRecursiveExt};
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::transform::components::Transform;

use crate::model::{
    Board, BoardCoords, Direction, GridMap, GridSet, LevelProgress, Piece, Tile, TileKind,
};

use super::border::{spawn_horz_border, spawn_vert_border};
use super::focus::spawn_focus;
use super::manipulator::spawn_manipulator;
use super::particle::spawn_particle;
use super::tile::spawn_tile;
use super::{Assets, BoardCoordsHolder, EngineCoords};

#[derive(Resource)]
pub struct Level {
    pub present: Board,
    pub future: Board,
    pub parent: Entity,
    pub tiles: GridMap<Entity>,
    pub horz_borders: GridMap<Entity>,
    pub vert_borders: GridMap<Entity>,
    pub pieces: GridMap<Entity>,
    pub progress: LevelProgress,
}

#[derive(Bundle, Default)]
struct BoardBundle {
    spatial: SpatialBundle,
}

impl Level {
    pub fn new(board: Board) -> Self {
        let present = board;
        let future = present.clone();
        let tiles = GridMap::like(&present.tiles);
        let horz_borders = GridMap::like(&present.horz_borders);
        let vert_borders = GridMap::like(&present.vert_borders);
        let pieces = GridMap::like(&present.pieces);
        let progress = LevelProgress::new(&present);
        Self {
            present,
            future,
            parent: Entity::PLACEHOLDER,
            tiles,
            horz_borders,
            vert_borders,
            pieces,
            progress,
        }
    }

    pub fn spawn(&mut self, commands: &mut Commands, assets: &Assets) {
        let mut parent = commands.spawn(BoardBundle::default());
        self.parent = parent.id();
        parent.with_children(|parent| {
            for (coords, tile) in self.present.tiles.iter() {
                self.tiles
                    .set(coords, spawn_tile(parent, tile, coords, &assets.tiles));
            }

            for (coords, border) in self.present.horz_borders.iter() {
                self.horz_borders.set(
                    coords,
                    spawn_horz_border(parent, border, coords, &assets.borders),
                );
            }

            for (coords, border) in self.present.vert_borders.iter() {
                self.vert_borders.set(
                    coords,
                    spawn_vert_border(parent, border, coords, &assets.borders),
                );
            }

            for (coords, piece) in self.present.pieces.iter() {
                let entity = match piece {
                    Piece::Particle(particle) => {
                        spawn_particle(parent, particle, coords, &assets.particles)
                    }
                    Piece::Manipulator(manipulator) => spawn_manipulator(
                        parent,
                        manipulator,
                        coords,
                        &self.present,
                        &assets.manipulators,
                    ),
                };
                self.pieces.set(coords, entity);
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
        if self.present.dims.contains(coords) {
            let center = coords.to_xy();
            Some((coords, pos - center))
        } else {
            None
        }
    }

    pub fn update_present(&mut self) {
        self.present.copy_state_from(&self.future);
    }

    pub fn move_piece(
        &mut self,
        from_coords: BoardCoords,
        to_coords: BoardCoords,
        q_piece: &mut Query<(&mut BoardCoordsHolder, &mut Transform)>,
    ) {
        let entity = self.pieces.take(from_coords).unwrap();
        self.pieces.set(to_coords, entity);

        let (mut coords, mut xform) = q_piece.get_mut(entity).unwrap();
        coords.0 = to_coords;
        xform.translation = to_coords.to_xy().extend(xform.translation.z);
    }

    pub fn move_pieces(
        &mut self,
        pieces: &GridSet,
        direction: Direction,
        q_piece: &mut Query<(&mut BoardCoordsHolder, &mut Transform)>,
    ) {
        pieces.for_each(direction, |from_coords| {
            let to_coords = self.present.neighbor(from_coords, direction).unwrap();
            self.move_piece(from_coords, to_coords, q_piece);
            if let Some(Piece::Particle(_)) = self.present.pieces.get(to_coords) {
                if let Some(Tile {
                    kind: TileKind::Collector,
                    ..
                }) = self.present.tiles.get(to_coords)
                {
                    self.progress.particle_collected();
                }
            }
        });
    }

    pub fn remove_piece(&mut self, coords: BoardCoords, commands: &mut Commands) {
        let outcome = self
            .progress
            .piece_lost(self.present.pieces.get(coords).unwrap());
        self.present.remove_piece(coords);
        self.future.remove_piece(coords);
        let entity = self.pieces.take(coords).unwrap();
        commands.entity(entity).despawn_recursive();
        outcome
    }

    pub fn remove_pieces(&mut self, pieces: &GridSet, commands: &mut Commands) {
        for coords in pieces.iter() {
            self.remove_piece(coords, commands);
        }
    }
}
