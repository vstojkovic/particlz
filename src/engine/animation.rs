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
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnimationSet;

#[derive(Debug, Clone)]
pub struct Movement {
    pub leader: BoardCoords,
    pub direction: Direction,
}

#[derive(Resource, Debug)]
pub struct AnimationState {
    animation: Animation,
    pieces: GridSet,
    played_duration: Duration,
    total_duration: Duration,
}

#[derive(Resource, Debug, Default)]
pub struct AnimationStateHolder(Option<AnimationState>);

#[derive(Event, Debug)]
pub struct StartAnimation(pub Animation, pub GridSet);

#[derive(Event, Debug)]
pub struct AnimationFinished(pub Animation, pub GridSet);

#[derive(Component, Default)]
pub struct MovementAnimator {
    is_moving: bool,
    start: Vec2,
    end: Vec2,
}

#[derive(Bundle, Default)]
pub struct AnimationBundle {
    animator: MovementAnimator,
}

fn start_animation(
    mut ev_start_animation: EventReader<StartAnimation>,
    mut state: ResMut<AnimationStateHolder>,
    mut q_animator: Query<(&BoardCoordsHolder, &mut MovementAnimator)>,
) {
    let Some(StartAnimation(animation, pieces)) = ev_start_animation.read().last() else {
        return;
    };
    let total_duration = match animation {
        Animation::Movement(_) => MOVE_DURATION,
    };
    state.0 = Some(AnimationState {
        animation: animation.clone(),
        pieces: pieces.clone(),
        played_duration: Duration::ZERO,
        total_duration,
    });
    match animation {
        Animation::Movement(movement) => {
            for (coords, mut animator) in q_animator.iter_mut() {
                if !pieces.contains(coords.0) {
                    continue;
                }
                animator.start = coords.to_xy();
                animator.end = animator.start + movement.direction.delta();
                animator.is_moving = true;
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

    state.played_duration += time.delta();
    let finished = state.played_duration >= state.total_duration;
    if finished {
        state.played_duration = state.total_duration;
    }
    let progress = state.played_duration.as_secs_f32() / state.total_duration.as_secs_f32();

    for (mut animator, mut xform) in q_animator.iter_mut() {
        if !animator.is_moving {
            continue;
        }
        let z_layer = xform.translation.z;
        let position = animator.start.lerp(animator.end, progress.sine_in_out());
        xform.translation = position.extend(z_layer);
        animator.is_moving = !finished;
    }

    if finished {
        let state = state_holder.0.take().unwrap();
        ev_animation_finished.send(AnimationFinished(state.animation, state.pieces));
    }
}

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AnimationStateHolder::default())
            .add_event::<StartAnimation>()
            .add_event::<AnimationFinished>()
            .add_systems(
                FixedUpdate,
                (start_animation, animate_movement)
                    .chain()
                    .in_set(AnimationSet),
            );
    }
}
