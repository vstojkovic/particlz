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
use bevy::render::color::Color;
use bevy::render::view::Visibility;
use bevy::sprite::{Anchor, Sprite, SpriteBundle};
use bevy::transform::components::Transform;
use bevy_tweening::lens::{SpriteColorLens, TransformScaleLens};
use bevy_tweening::{Animator, AnimatorState, Delay, EaseFunction, Tween};

use crate::model::{BeamTarget, BeamTargetKind, Board, BoardCoords, Direction, Emitters, Piece};

use super::animation::AnimationBundle;
use super::board::BoardResource;
use super::border::{BORDER_OFFSET_X, BORDER_OFFSET_Y};
use super::{BoardCoordsHolder, MOVE_DURATION, TILE_HEIGHT, TILE_WIDTH};

pub struct BeamPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BeamSet;

#[derive(Component, Debug)]
pub struct Beam {
    direction: Direction,
    target: BeamTarget,
    group: BeamGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BeamGroup {
    Present,
    Future,
}

#[derive(Bundle)]
pub struct BeamBundle {
    beam: Beam,
    sprite: SpriteBundle,
    xform_animator: Animator<Transform>,
    alpha_animator: Animator<Sprite>,
}

#[derive(Event)]
pub struct MoveBeams {
    pub anchor: Entity,
    pub direction: Direction,
}

#[derive(Event)]
pub struct ResetBeams;

impl BeamBundle {
    fn new(
        origin: BoardCoords,
        direction: Direction,
        target: BeamTarget,
        group: BeamGroup,
    ) -> Self {
        let sprite_anchor = match direction {
            Direction::Up => Anchor::BottomCenter,
            Direction::Left => Anchor::CenterRight,
            Direction::Down => Anchor::TopCenter,
            Direction::Right => Anchor::CenterLeft,
        };

        let mut xform_animator = Animator::new(Delay::new(Duration::from_nanos(1)));
        xform_animator.state = AnimatorState::Paused;

        let alpha_tween = Tween::new(
            EaseFunction::SineInOut,
            MOVE_DURATION * 3 / 5,
            SpriteColorLens {
                start: beam_color(group.alpha()),
                end: beam_color(1.0 - group.alpha()),
            },
        );
        let alpha_tween = Delay::new(MOVE_DURATION * 2 / 5).then(alpha_tween);
        let mut alpha_animator = Animator::new(alpha_tween);
        alpha_animator.state = AnimatorState::Paused;

        Self {
            beam: Beam {
                direction,
                target,
                group,
            },
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: beam_color(group.alpha()),
                    anchor: sprite_anchor,
                    ..Default::default()
                },
                transform: Transform {
                    scale: beam_scale(origin, direction, target).extend(1.0),
                    ..Default::default()
                },
                visibility: group.visibility(),
                ..Default::default()
            },
            xform_animator,
            alpha_animator,
        }
    }
}

impl BeamGroup {
    fn visibility(self) -> Visibility {
        match self {
            Self::Present => Visibility::Inherited,
            Self::Future => Visibility::Hidden,
        }
    }

    fn alpha(self) -> f32 {
        match self {
            Self::Present => 1.0,
            Self::Future => 0.0,
        }
    }
}

pub fn spawn_beams(
    anchor: &mut ChildBuilder,
    origin: BoardCoords,
    emitters: Emitters,
    board: &Board,
) {
    spawn_beam_group(anchor, origin, emitters, board, BeamGroup::Future);
    spawn_beam_group(anchor, origin, emitters, board, BeamGroup::Present);
}

fn spawn_beam_group(
    anchor: &mut ChildBuilder,
    origin: BoardCoords,
    emitters: Emitters,
    board: &Board,
    group: BeamGroup,
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
            beams.spawn(BeamBundle::new(origin, direction, target, group));
        }
    });
}

fn move_beams(
    mut events: EventReader<MoveBeams>,
    board: Res<BoardResource>,
    q_children: Query<&Children>,
    mut q_beam: Query<(
        &mut Beam,
        &mut Transform,
        &mut Visibility,
        &mut Sprite,
        &mut Animator<Transform>,
        &mut Animator<Sprite>,
    )>,
) {
    enum BeamChange {
        None,
        Resize,
        Crossfade,
    }

    let Some(event) = events.read().last() else {
        return;
    };
    for (coords, piece) in board.present.iter_pieces() {
        let Piece::Manipulator(_) = piece else {
            continue;
        };
        let anchor = board.get_piece(coords).unwrap();
        let future_origin = match anchor == event.anchor {
            false => coords,
            true => board.present.neighbor(coords, event.direction).unwrap(),
        };
        for child in q_children.iter_descendants(anchor) {
            let Ok((mut beam, mut xform, mut visibility, mut sprite, mut xform_animator, mut alpha_animator)) =
                q_beam.get_mut(child)
            else {
                continue;
            };

            beam.target = board.future.find_beam_target(future_origin, beam.direction);

            let future_scale = beam_scale(future_origin, beam.direction, beam.target);
            let beam_change = if future_scale == xform.scale.truncate() {
                BeamChange::None
            } else if beam.direction.orientation() == event.direction.orientation() {
                BeamChange::Resize
            } else {
                BeamChange::Crossfade
            };

            match beam_change {
                BeamChange::None => (),
                BeamChange::Resize => {
                    if let BeamGroup::Present = beam.group {
                        let tween = Tween::new(
                            EaseFunction::SineInOut,
                            MOVE_DURATION,
                            TransformScaleLens {
                                start: xform.scale,
                                end: future_scale.extend(1.0),
                            },
                        );
                        xform_animator.set_tweenable(tween);
                        xform_animator.state = AnimatorState::Playing;
                    }
                }
                BeamChange::Crossfade => {
                    let present_len = xform.scale.truncate().length_squared();
                    let future_len = future_scale.length_squared();
                    let future_grows = future_len > present_len;
                    let is_future = beam.group == BeamGroup::Future;
                    if is_future {
                        xform.scale = future_scale.extend(1.0);
                        *visibility = Visibility::Inherited;
                    }
                    if future_grows == is_future {
                        alpha_animator.tweenable_mut().rewind();
                        alpha_animator.state = AnimatorState::Playing;
                    } else {
                        sprite.color = beam_color(1.0);
                    }
                }
            }
        }
    }
}

fn reset_beams(
    mut events: EventReader<ResetBeams>,
    mut q_beam: Query<(
        Entity,
        &Beam,
        &mut Sprite,
        &mut Transform,
        &mut Visibility,
        &mut Animator<Sprite>,
    )>,
    q_parent: Query<&Parent>,
    q_origin: Query<&BoardCoordsHolder>,
) {
    if events.is_empty() {
        return;
    }
    events.clear();
    for (beam_id, beam, mut sprite, mut xform, mut visibility, mut alpha_animator) in
        q_beam.iter_mut()
    {
        let origin = q_parent
            .iter_ancestors(beam_id)
            .find_map(|id| q_origin.get(id).ok())
            .unwrap()
            .0;
        xform.scale = beam_scale(origin, beam.direction, beam.target).extend(1.0);
        *visibility = beam.group.visibility();
        alpha_animator.stop();
        sprite.color = beam_color(beam.group.alpha());
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

fn beam_color(alpha: f32) -> Color {
    Color::rgb_u8(0, 153, 255).with_a(alpha)
}

impl Plugin for BeamPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<MoveBeams>()
            .add_event::<ResetBeams>()
            .add_systems(FixedUpdate, move_beams.in_set(BeamSet))
            .add_systems(FixedPostUpdate, reset_beams.in_set(BeamSet));
    }
}

const Z_LAYER: f32 = 1.0;
