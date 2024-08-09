use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, EntityCommands, Query, Resource};
use bevy::hierarchy::{BuildChildren, DespawnRecursiveExt};
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::transform::components::Transform;

use crate::model::{
    Board, BoardCoords, Direction, GridMap, GridSet, LevelCampaign, LevelMetadata, LevelProgress,
    Piece, Tile, TileKind,
};

use super::border::{spawn_horz_border, spawn_vert_border};
use super::focus::spawn_focus;
use super::manipulator::spawn_manipulator;
use super::particle::spawn_particle;
use super::tile::spawn_tile;
use super::{BoardCoordsHolder, EngineCoords, GameAssets, Mutable, TILE_HEIGHT, TILE_WIDTH};

#[derive(Resource)]
pub struct Level {
    pub metadata: LevelMetadata,
    pub present: Board,
    pub future: Board,
    pub past: Vec<Board>,
    pub parent: Option<Entity>,
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

#[derive(Resource, Deref)]
pub struct Campaign(pub LevelCampaign);

impl Level {
    pub fn new(board: Board, metadata: LevelMetadata) -> Self {
        let present = board;
        let future = present.clone();
        let tiles = GridMap::like(&present.tiles);
        let horz_borders = GridMap::like(&present.horz_borders);
        let vert_borders = GridMap::like(&present.vert_borders);
        let pieces = GridMap::like(&present.pieces);
        let progress = LevelProgress::new(&present);
        Self {
            metadata,
            present,
            future,
            past: vec![],
            parent: None,
            tiles,
            horz_borders,
            vert_borders,
            pieces,
            progress,
        }
    }

    pub fn spawn(&mut self, play_area_size: Vec2, commands: &mut Commands, assets: &GameAssets) {
        if self.parent.is_some() {
            self.despawn(commands);
        }

        let mut parent = spawn_board(&self.present, play_area_size, commands, &|_| ());
        self.parent = Some(parent.id());
        parent.with_children(|parent| {
            self.tiles.clear();
            for (coords, tile) in self.present.tiles.iter() {
                self.tiles.set(
                    coords,
                    spawn_tile(parent, tile, coords, &assets.tiles, &|_| ()),
                );
            }

            self.horz_borders.clear();
            for (coords, border) in self.present.horz_borders.iter() {
                self.horz_borders.set(
                    coords,
                    spawn_horz_border(parent, border, coords, &assets.borders, &|_| ()),
                );
            }

            self.vert_borders.clear();
            for (coords, border) in self.present.vert_borders.iter() {
                self.vert_borders.set(
                    coords,
                    spawn_vert_border(parent, border, coords, &assets.borders, &|_| ()),
                );
            }

            self.pieces.clear();
            for (coords, piece) in self.present.pieces.iter() {
                let entity = match piece {
                    Piece::Particle(particle) => {
                        spawn_particle(parent, particle, coords, &assets.particles, &|_| ())
                    }
                    Piece::Manipulator(manipulator) => spawn_manipulator(
                        parent,
                        manipulator,
                        coords,
                        &self.present,
                        &assets,
                        &|_| (),
                    ),
                };
                self.pieces.set(coords, entity);
            }

            spawn_focus(parent, &assets.focus);
        });
    }

    pub fn despawn(&mut self, commands: &mut Commands) {
        commands
            .entity(self.parent.take().unwrap())
            .despawn_recursive();
    }

    pub fn coords_at_pos(
        &self,
        pos: Vec2,
        q_xform: &Query<&Transform>,
    ) -> Option<(BoardCoords, Vec2)> {
        let xform = q_xform.get(self.parent.unwrap()).unwrap();
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

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    pub fn undo(&mut self) {
        if let Some(board) = self.past.pop() {
            self.present.copy_state_from(&board);
            self.future.copy_state_from(&self.present);
            self.progress = LevelProgress::new(&self.present);
        }
    }

    pub fn reset(&mut self) {
        self.past.truncate(1);
        self.undo();
    }

    pub fn prepare_move(&mut self, move_set: &GridSet, direction: Direction) {
        self.past.push(self.present.clone());
        self.future.move_pieces(&move_set, direction);
        self.future.retarget_beams();
    }

    pub fn move_piece(&mut self, from_coords: BoardCoords, to_coords: BoardCoords) {
        let entity = self.pieces.take(from_coords).unwrap();
        self.pieces.set(to_coords, entity);
        if let Some(Piece::Particle(_)) = self.present.pieces.get(to_coords) {
            if let Some(Tile {
                kind: TileKind::Collector,
                ..
            }) = self.present.tiles.get(to_coords)
            {
                self.progress.particle_collected();
            }
        }
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

pub fn spawn_board<'c>(
    board: &Board,
    parent_area_size: Vec2,
    commands: &'c mut Commands,
    mutator: &impl Fn(&mut EntityCommands),
) -> EntityCommands<'c> {
    let board_size = Vec2::new(
        board.dims.cols as f32 * TILE_WIDTH,
        board.dims.rows as f32 * TILE_HEIGHT,
    )
    .abs();
    let mut board_origin = (parent_area_size - board_size) / 2.0;
    board_origin.y = -board_origin.y;

    commands
        .spawn(BoardBundle {
            spatial: SpatialBundle {
                transform: Transform {
                    translation: board_origin.extend(0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        })
        .mutate(mutator)
}

pub fn update_piece_coords(
    level: Res<Level>,
    mut q_coords: Query<&mut BoardCoordsHolder>,
    mut q_xform: Query<&mut Transform>,
    q_children: Query<&Children>,
) {
    for (coords, &anchor) in level.pieces.iter() {
        let mut holder = q_coords.get_mut(anchor).unwrap();
        if holder.0 == coords {
            continue;
        }
        holder.0 = coords;

        let mut xform = q_xform.get_mut(anchor).unwrap();
        xform.translation = coords.to_xy().extend(xform.translation.z);

        for child in q_children.iter_descendants(anchor) {
            if let Ok(mut holder) = q_coords.get_mut(child) {
                holder.0 = coords;
            }
        }
    }
}
