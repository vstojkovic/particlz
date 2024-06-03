use bevy::app::{FixedPostUpdate, Plugin};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::{Event, EventReader};
use bevy::ecs::system::{Query, Res};
use bevy::hierarchy::{BuildChildren, ChildBuilder, HierarchyQueryExt, Parent};
use bevy::math::Vec2;
use bevy::prelude::SpatialBundle;
use bevy::sprite::{Anchor, Sprite, SpriteBundle};
use bevy::transform::components::Transform;

use crate::engine::{TILE_HEIGHT, TILE_WIDTH};
use crate::model::{BeamTarget, BeamTargetKind, Board, BoardCoords, Direction, Emitters};

use super::animation::AnimationBundle;
use super::board::BoardResource;
use super::border::{BORDER_OFFSET_X, BORDER_OFFSET_Y};
use super::BoardCoordsHolder;

pub struct BeamPlugin;
#[derive(Component, Debug)]
pub struct Beam {
    direction: Direction,
    target: BeamTarget,
}

#[derive(Bundle)]
pub struct BeamBundle {
    beam: Beam,
    sprite: SpriteBundle,
}

#[derive(Event)]
pub struct RetargetBeams;

impl BeamBundle {
    fn new(origin: BoardCoords, direction: Direction, target: BeamTarget) -> Self {
        let sprite_anchor = match direction {
            Direction::Up => Anchor::BottomCenter,
            Direction::Left => Anchor::CenterRight,
            Direction::Down => Anchor::TopCenter,
            Direction::Right => Anchor::CenterLeft,
        };
        Self {
            beam: Beam { direction, target },
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: bevy::render::color::Color::rgb_u8(0, 153, 255),
                    anchor: sprite_anchor,
                    ..Default::default()
                },
                transform: Transform {
                    scale: beam_scale(origin, direction, target).extend(1.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}

pub fn spawn_beams(
    anchor: &mut ChildBuilder,
    origin: BoardCoords,
    emitters: Emitters,
    board: &Board,
) {
    let mut beams = anchor.spawn((
        SpatialBundle {
            transform: Transform {
                translation: Vec2::ZERO.extend(Z_LAYER),
                ..Default::default()
            },
            ..Default::default()
        },
        AnimationBundle::new(anchor.parent_entity(), Z_LAYER),
    ));
    beams.with_children(|beams| {
        for direction in emitters.directions() {
            let target = board.find_beam_target(origin, direction);
            beams.spawn(BeamBundle::new(origin, direction, target));
        }
    });
}

fn retarget_beams(
    mut events: EventReader<RetargetBeams>,
    mut q_beam: Query<(Entity, &mut Beam, &mut Transform)>,
    q_parent: Query<&Parent>,
    q_origin: Query<&BoardCoordsHolder>,
    board: Res<BoardResource>,
) {
    if events.is_empty() {
        return;
    }
    events.clear();
    for (beam_id, mut beam, mut xform) in q_beam.iter_mut() {
        let origin = q_parent
            .iter_ancestors(beam_id)
            .find_map(|id| q_origin.get(id).ok())
            .unwrap()
            .0;
        let target = board.model.find_beam_target(origin, beam.direction);
        beam.target = target;
        xform.scale = beam_scale(origin, beam.direction, target).extend(1.0);
    }
}

fn beam_scale(origin: BoardCoords, direction: Direction, target: BeamTarget) -> Vec2 {
    let width = target.coords.col.abs_diff(origin.col) as f32;
    let height = target.coords.row.abs_diff(origin.row) as f32;
    let scale = match direction {
        Direction::Up | Direction::Down => Vec2::new(4.0, height * TILE_HEIGHT),
        Direction::Left | Direction::Right => Vec2::new(width * TILE_WIDTH, 4.0),
    };
    match target.kind {
        BeamTargetKind::Piece => scale,
        BeamTargetKind::Border => {
            scale
                + match direction {
                    Direction::Up => Vec2::new(0.0, BORDER_OFFSET_Y),
                    Direction::Left => Vec2::new(BORDER_OFFSET_X, 0.0),
                    Direction::Down => Vec2::new(0.0, -BORDER_OFFSET_Y),
                    Direction::Right => Vec2::new(-BORDER_OFFSET_X, 0.0),
                }
        }
    }
}

impl Plugin for BeamPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<RetargetBeams>()
            .add_systems(FixedPostUpdate, retarget_beams);
    }
}

const Z_LAYER: f32 = 1.0;
