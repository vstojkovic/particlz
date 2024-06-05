use std::time::Duration;

use bevy::app::{FixedPostUpdate, FixedUpdate, Plugin};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::{Event, EventReader};
use bevy::ecs::schedule::{IntoSystemConfigs, SystemSet};
use bevy::ecs::system::{Query, Res};
use bevy::hierarchy::{BuildChildren, ChildBuilder, Children, HierarchyQueryExt, Parent};
use bevy::math::Vec2;
use bevy::prelude::SpatialBundle;
use bevy::sprite::{Anchor, Sprite, SpriteBundle};
use bevy::transform::components::Transform;
use bevy_tweening::lens::TransformScaleLens;
use bevy_tweening::{Animator, AnimatorState, Delay, EaseFunction, Tween};

use crate::engine::{TILE_HEIGHT, TILE_WIDTH};
use crate::model::{
    BeamTarget, BeamTargetKind, Board, BoardCoords, Direction, Emitters, Orientation,
};

use super::animation::AnimationBundle;
use super::board::BoardResource;
use super::border::{BORDER_OFFSET_X, BORDER_OFFSET_Y};
use super::BoardCoordsHolder;

pub struct BeamPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BeamSet;

#[derive(Component, Debug)]
pub struct Beam {
    direction: Direction,
    target: BeamTarget,
}

#[derive(Bundle)]
pub struct BeamBundle {
    beam: Beam,
    sprite: SpriteBundle,
    xform_animator: Animator<Transform>,
}

#[derive(Event)]
pub struct MoveBeams {
    pub anchor: Entity,
    pub direction: Direction,
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
        let tween = Delay::new(Duration::from_nanos(1));
        let mut xform_animator = Animator::new(tween);
        xform_animator.state = AnimatorState::Paused;
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
            xform_animator,
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

fn move_beams(
    mut events: EventReader<MoveBeams>,
    q_children: Query<&Children>,
    mut q_beam: Query<(&mut Beam, &mut Transform, &mut Animator<Transform>)>,
) {
    for event in events.read() {
        for child in q_children.iter_descendants(event.anchor) {
            let Ok((beam, xform, mut xform_animator)) = q_beam.get_mut(child) else {
                continue;
            };
            if beam.direction.orientation() != event.direction.orientation() {
                continue;
            }
            let mut delta = match event.direction.orientation() {
                Orientation::Horizontal => Vec2::new(TILE_WIDTH, 0.0),
                Orientation::Vertical => Vec2::new(0.0, TILE_HEIGHT),
            };
            if beam.direction == event.direction {
                delta = -delta;
            }
            let start = xform.scale.truncate();
            let end = start + delta;
            let tween = Tween::new(
                EaseFunction::SineInOut,
                Duration::from_millis(500),
                TransformScaleLens {
                    start: start.extend(1.0),
                    end: end.extend(1.0),
                },
            );
            xform_animator.set_tweenable(tween);
            xform_animator.state = AnimatorState::Playing;
        }
    }
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
        app.add_event::<MoveBeams>()
            .add_event::<RetargetBeams>()
            .add_systems(FixedUpdate, move_beams.in_set(BeamSet))
            .add_systems(FixedPostUpdate, retarget_beams.in_set(BeamSet));
    }
}

const Z_LAYER: f32 = 1.0;
