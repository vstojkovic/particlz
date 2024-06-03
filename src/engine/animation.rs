use std::time::Duration;

use bevy::app::{FixedUpdate, Plugin};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::{Event, EventReader, EventWriter};
use bevy::ecs::schedule::{IntoSystemConfigs, SystemSet};
use bevy::ecs::system::Query;
use bevy::hierarchy::Children;
use bevy::math::Vec2;
use bevy::transform::components::Transform;
use bevy_tweening::lens::TransformPositionLens;
use bevy_tweening::{
    component_animator_system, Animator, EaseFunction, EaseMethod, Sequence, Tween, TweenCompleted,
    Tweenable,
};
use strum::{EnumCount, IntoEnumIterator};

use crate::model::Direction;

use super::{TILE_HEIGHT, TILE_WIDTH};

pub struct AnimationPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnimationSet;

#[derive(Component, Debug, Clone, Copy)]
pub enum Animation {
    Idle,
    Movement(Direction),
}

#[derive(Event, Debug)]
pub struct StartAnimation {
    pub anchor: Entity,
    pub animation: Animation,
}

#[derive(Event, Debug)]
pub struct AnimationFinished {
    pub anchor: Entity,
    pub animation: Animation,
}

#[derive(Bundle)]
pub(super) struct AnimationAnchorBundle {
    animation: Animation,
}

#[derive(Bundle)]
pub(super) struct AnimationBundle {
    xform_animator: Animator<Transform>,
}

impl Animation {
    fn start(self, animator: &mut Animator<Transform>) {
        let tweenable = animator.tweenable_mut();
        match self {
            Self::Idle => tweenable.set_progress(1.0),
            Self::Movement(direction) => {
                tweenable.set_elapsed(Duration::from_millis(500 * direction as isize as u64))
            }
        }
    }
}

impl AnimationAnchorBundle {
    pub fn new() -> Self {
        Self {
            animation: Animation::Idle,
        }
    }
}

impl AnimationBundle {
    pub fn new(anchor: Entity, z: f32) -> Self {
        let mut sequence = Sequence::with_capacity(Direction::COUNT);
        for direction in Direction::iter() {
            let end = match direction {
                Direction::Up => Vec2::new(0.0, TILE_HEIGHT),
                Direction::Left => Vec2::new(-TILE_WIDTH, 0.0),
                Direction::Down => Vec2::new(0.0, -TILE_HEIGHT),
                Direction::Right => Vec2::new(TILE_WIDTH, 0.0),
            };
            let tween = Tween::new(
                EaseFunction::SineInOut,
                Duration::from_millis(500),
                TransformPositionLens {
                    start: Vec2::ZERO.extend(z),
                    end: end.extend(z),
                },
            );
            let tween = tween.with_completed_event(anchor.to_bits());
            sequence = sequence.then(tween);
        }
        sequence = sequence.then(Tween::new(
            EaseMethod::Linear,
            Duration::from_nanos(1),
            TransformPositionLens {
                start: Vec2::ZERO.extend(z),
                end: Vec2::ZERO.extend(z),
            },
        ));
        sequence.set_progress(1.0);
        let xform_animator = Animator::new(sequence);
        Self { xform_animator }
    }
}

fn set_animation(
    anchor: Entity,
    animation: Animation,
    q_anchor: &mut Query<(&mut Animation, &Children)>,
    q_animator: &mut Query<&mut Animator<Transform>>,
) {
    let (mut anchor_animation, children) = q_anchor.get_mut(anchor).unwrap();
    *anchor_animation = animation;
    for &child in children.iter() {
        animation.start(&mut *q_animator.get_mut(child).unwrap());
    }
}

fn start_animation(
    mut ev_start_animation: EventReader<StartAnimation>,
    mut q_anchor: Query<(&mut Animation, &Children)>,
    mut q_animator: Query<&mut Animator<Transform>>,
) {
    for StartAnimation { anchor, animation } in ev_start_animation.read() {
        set_animation(*anchor, *animation, &mut q_anchor, &mut q_animator);
    }
}

fn finish_animation(
    mut ev_tweens: EventReader<TweenCompleted>,
    mut ev_animation: EventWriter<AnimationFinished>,
    mut q_anchor: Query<(&mut Animation, &Children)>,
    mut q_animator: Query<&mut Animator<Transform>>,
) {
    for event in ev_tweens.read() {
        let anchor = Entity::from_bits(event.user_data);
        let (&animation, _) = q_anchor.get(anchor).unwrap();
        if let Animation::Idle = animation {
            // we've already processed this anchor
            continue;
        }
        set_animation(anchor, Animation::Idle, &mut q_anchor, &mut q_animator);
        ev_animation.send(AnimationFinished { anchor, animation });
    }
}

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let tween_system = component_animator_system::<Transform>;
        app.add_event::<TweenCompleted>()
            .add_event::<StartAnimation>()
            .add_event::<AnimationFinished>()
            .add_systems(
                FixedUpdate,
                (
                    start_animation.in_set(AnimationSet),
                    tween_system.after(start_animation),
                    finish_animation.after(tween_system).in_set(AnimationSet),
                ),
            );
    }
}
