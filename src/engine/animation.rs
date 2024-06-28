use std::time::Duration;

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::transform::components::Transform;
use interpolation::Ease;

use crate::model::{BoardCoords, Direction, GridSet};

use super::{BoardCoordsHolder, EngineCoords, EngineDirection, MOVE_DURATION};

pub struct AnimationPlugin;

#[derive(Debug, Clone)]
pub enum Animation {
    Movement(Movement),
    FadeOut,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnimationSet;

#[derive(Debug, Clone)]
pub struct Movement {
    pub leader: BoardCoords,
    pub direction: Direction,
}

#[derive(Resource, Debug, Default)]
struct AnimationStateHolder(Option<AnimationState>);

#[derive(Debug)]
struct AnimationState {
    animation: Animation,
    pieces: GridSet,
    played_duration: Duration,
    total_duration: Duration,
}

#[derive(Event, Debug)]
pub struct StartAnimation(pub Animation, pub GridSet);

#[derive(Event, Debug)]
pub struct AnimationFinished(pub Animation, pub GridSet);

#[derive(Component, Default)]
struct MovementAnimator {
    is_moving: bool,
    start: Vec2,
    end: Vec2,
}

#[derive(Component, Default)]
pub struct FadeOutAnimator {
    is_fading: bool,
}

#[derive(Bundle, Default)]
pub struct AnimationBundle {
    mover: MovementAnimator,
    fader: FadeOutAnimator,
}

impl AnimationState {
    fn progress(&self) -> f32 {
        self.played_duration.as_secs_f32() / self.total_duration.as_secs_f32()
    }

    fn is_finished(&self) -> bool {
        self.played_duration >= self.total_duration
    }

    fn tick(&mut self, delta: Duration) {
        self.played_duration = std::cmp::min(self.played_duration + delta, self.total_duration);
    }
}

fn start_animation(
    mut ev_start_animation: EventReader<StartAnimation>,
    mut state: ResMut<AnimationStateHolder>,
    mut q_mover: Query<(&BoardCoordsHolder, &mut MovementAnimator)>,
    mut q_fader: Query<(&BoardCoordsHolder, &mut FadeOutAnimator)>,
) {
    let Some(StartAnimation(animation, pieces)) = ev_start_animation.read().last() else {
        return;
    };
    let total_duration = match animation {
        Animation::Movement(_) => MOVE_DURATION,
        Animation::FadeOut => MOVE_DURATION,
    };
    state.0 = Some(AnimationState {
        animation: animation.clone(),
        pieces: pieces.clone(),
        played_duration: Duration::ZERO,
        total_duration,
    });
    match animation {
        Animation::Movement(movement) => {
            for (coords, mut animator) in q_mover.iter_mut() {
                if !pieces.contains(coords.0) {
                    continue;
                }
                animator.start = coords.to_xy();
                animator.end = animator.start + movement.direction.delta();
                animator.is_moving = true;
            }
        }
        Animation::FadeOut => {
            for (coords, mut animator) in q_fader.iter_mut() {
                if !pieces.contains(coords.0) {
                    continue;
                }
                animator.is_fading = true;
            }
        }
    }
}

fn animate_movement(
    mut ev_animation_finished: EventWriter<AnimationFinished>,
    time: Res<Time>,
    mut state_holder: ResMut<AnimationStateHolder>,
    mut q_animator: Query<(&mut MovementAnimator, &mut Transform)>,
) {
    let Some(state) = state_holder.0.as_mut() else {
        return;
    };
    let Animation::Movement(_) = state.animation else {
        return;
    };

    state.tick(time.delta());

    for (mut animator, mut xform) in q_animator.iter_mut() {
        if !animator.is_moving {
            continue;
        }
        let z_layer = xform.translation.z;
        let position = animator
            .start
            .lerp(animator.end, state.progress().sine_in_out());
        xform.translation = position.extend(z_layer);
        animator.is_moving = !state.is_finished();
    }

    if state.is_finished() {
        let state = state_holder.0.take().unwrap();
        ev_animation_finished.send(AnimationFinished(state.animation, state.pieces));
    }
}

fn animate_fade_out(
    mut ev_animation_finished: EventWriter<AnimationFinished>,
    time: Res<Time>,
    mut state_holder: ResMut<AnimationStateHolder>,
    mut q_animator: Query<(&mut FadeOutAnimator, &mut Sprite)>,
) {
    let Some(state) = state_holder.0.as_mut() else {
        return;
    };
    let Animation::FadeOut = state.animation else {
        return;
    };

    state.tick(time.delta());

    for (mut animator, mut sprite) in q_animator.iter_mut() {
        if !animator.is_fading {
            continue;
        }
        let alpha = 1.0.lerp(0.0, state.progress().sine_in_out());
        sprite.color = sprite.color.with_a(alpha);
        animator.is_fading = !state.is_finished();
    }

    if state.is_finished() {
        let state = state_holder.0.take().unwrap();
        ev_animation_finished.send(AnimationFinished(state.animation, state.pieces));
    }
}

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AnimationStateHolder::default())
            .add_event::<StartAnimation>()
            .add_event::<AnimationFinished>()
            .add_systems(FixedUpdate, start_animation.in_set(AnimationSet))
            .add_systems(
                FixedUpdate,
                animate_movement.after(start_animation).in_set(AnimationSet),
            )
            .add_systems(
                FixedUpdate,
                animate_fade_out.after(start_animation).in_set(AnimationSet),
            );
    }
}
