use std::time::Duration;

use bevy::app::{FixedPostUpdate, FixedUpdate, Plugin};
use bevy::color::Color;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::event::{Event, EventReader};
use bevy::ecs::schedule::{IntoSystemConfigs, SystemSet};
use bevy::ecs::system::{Query, Res};
use bevy::hierarchy::{ChildBuilder, Children, Parent};
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::render::view::Visibility;
use bevy::sprite::{Anchor, Sprite, SpriteBundle};
use bevy::time::Time;
use bevy::transform::components::Transform;
use interpolation::{Ease, Lerp};

use crate::model::{
    BeamTarget, BeamTargetKind, Board, BoardCoords, Direction, Emitters, GridSet, Piece,
};

use super::animation::FadeOutAnimator;
use super::border::{BORDER_OFFSET_X, BORDER_OFFSET_Y};
use super::level::Level;
use super::{BoardCoordsHolder, GameplaySet, MOVE_DURATION, TILE_HEIGHT, TILE_WIDTH};

pub struct BeamPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BeamSet;

#[derive(Component, Debug)]
pub struct Beam {
    direction: Direction,
    group: BeamGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BeamGroup {
    Present,
    Future,
}

#[derive(Component, Debug, Default)]
struct BeamAnimator {
    animation: BeamAnimation,
    played_duration: Duration,
    total_duration: Duration,
}

#[derive(Debug)]
enum BeamAnimation {
    None,
    Resize { start: Vec2, end: Vec2 },
    Fade { start: f32, end: f32 },
}

impl Default for BeamAnimation {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Bundle)]
pub struct BeamBundle {
    beam: Beam,
    coords: BoardCoordsHolder,
    sprite: SpriteBundle,
    animator: BeamAnimator,
    fader: FadeOutAnimator,
}

#[derive(Event)]
pub struct MoveBeams {
    pub move_set: GridSet,
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

        Self {
            beam: Beam { direction, group },
            coords: BoardCoordsHolder(origin),
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: beam_color(group.alpha()),
                    anchor: sprite_anchor,
                    ..Default::default()
                },
                transform: Transform {
                    translation: Vec2::ZERO.extend(REL_Z_LAYER),
                    scale: beam_scale(origin, direction, target).extend(1.0),
                    ..Default::default()
                },
                visibility: group.visibility(),
                ..Default::default()
            },
            animator: BeamAnimator::default(),
            fader: FadeOutAnimator::default(),
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

impl BeamAnimator {
    fn start_animation(&mut self, animation: BeamAnimation) {
        self.animation = animation;
        self.played_duration = Duration::ZERO;
        self.total_duration = MOVE_DURATION;
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
    let manipulator = board.pieces.get(origin).unwrap().as_manipulator().unwrap();
    for direction in emitters.directions() {
        let target = manipulator.target(direction).unwrap();
        anchor.spawn(BeamBundle::new(origin, direction, target, group));
    }
}

fn move_beams(
    mut events: EventReader<MoveBeams>,
    level: Res<Level>,
    q_children: Query<&Children>,
    mut q_beam: Query<(
        &Beam,
        &mut Transform,
        &mut Visibility,
        &mut Sprite,
        &mut BeamAnimator,
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
    for (coords, piece) in level.present.pieces.iter() {
        let Piece::Manipulator(_) = piece else {
            continue;
        };
        let anchor = *level.pieces.get(coords).unwrap();
        let future_origin = match event.move_set.contains(coords) {
            false => coords,
            true => level.present.neighbor(coords, event.direction).unwrap(),
        };
        for &child in q_children.get(anchor).unwrap().iter() {
            let Ok((beam, mut xform, mut visibility, mut sprite, mut animator)) =
                q_beam.get_mut(child)
            else {
                continue;
            };

            let target = level
                .future
                .pieces
                .get(future_origin)
                .unwrap()
                .as_manipulator()
                .unwrap()
                .target(beam.direction)
                .unwrap();
            let present_scale = xform.scale.truncate();
            let future_scale = beam_scale(future_origin, beam.direction, target);
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
                        animator.start_animation(BeamAnimation::Resize {
                            start: present_scale,
                            end: future_scale,
                        });
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
                        animator.start_animation(BeamAnimation::Fade {
                            start: beam.group.alpha(),
                            end: 1.0 - beam.group.alpha(),
                        });
                    } else {
                        sprite.color = beam_color(1.0);
                    }
                }
            }
        }
    }
}

fn animate_beams(
    time: Res<Time>,
    mut q_beam: Query<(&mut BeamAnimator, &mut Transform, &mut Sprite)>,
) {
    for (mut animator, mut xform, mut sprite) in q_beam.iter_mut() {
        if let BeamAnimation::None = animator.animation {
            continue;
        }
        animator.played_duration += time.delta();
        let finished = animator.played_duration >= animator.total_duration;
        if finished {
            animator.played_duration = animator.total_duration;
        }
        let progress =
            animator.played_duration.as_secs_f32() / animator.total_duration.as_secs_f32();
        match &animator.animation {
            BeamAnimation::None => unreachable!(),
            BeamAnimation::Resize { start, end } => {
                xform.scale = start.lerp(*end, progress.sine_in_out()).extend(1.0);
            }
            BeamAnimation::Fade { start, end } => {
                let progress = (progress - 0.4).clamp(0.0, 1.0) / 0.6;
                let alpha = start.lerp(end, &progress.sine_in_out());
                sprite.color = beam_color(alpha);
            }
        }
        if finished {
            animator.animation = BeamAnimation::None;
        }
    }
}

fn reset_beams(
    mut events: EventReader<ResetBeams>,
    level: Res<Level>,
    mut q_beam: Query<(
        &Beam,
        &mut BoardCoordsHolder,
        &mut Sprite,
        &mut Transform,
        &mut Visibility,
        &Parent,
    )>,
    q_origin: Query<&BoardCoordsHolder, Without<Beam>>,
) {
    if events.is_empty() {
        return;
    }
    events.clear();
    for (beam, mut coords, mut sprite, mut xform, mut visibility, parent) in q_beam.iter_mut() {
        let origin = q_origin.get(parent.get()).unwrap().0;
        coords.0 = origin;

        let target = level
            .present
            .pieces
            .get(origin)
            .unwrap()
            .as_manipulator()
            .unwrap()
            .target(beam.direction)
            .unwrap();
        xform.scale = beam_scale(origin, beam.direction, target).extend(1.0);
        *visibility = beam.group.visibility();
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
    Color::srgb_u8(0, 153, 255).with_alpha(alpha)
}

impl Plugin for BeamPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<MoveBeams>()
            .add_event::<ResetBeams>()
            .configure_sets(FixedUpdate, BeamSet.in_set(GameplaySet))
            .configure_sets(FixedPostUpdate, BeamSet.in_set(GameplaySet))
            .add_systems(
                FixedUpdate,
                (move_beams, animate_beams).chain().in_set(BeamSet),
            )
            .add_systems(FixedPostUpdate, reset_beams.in_set(BeamSet));
    }
}

const REL_Z_LAYER: f32 = -1.0;
