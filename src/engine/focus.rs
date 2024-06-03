use std::collections::HashMap;

use bevy::app::Plugin;
use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::event::{Event, EventReader};
use bevy::ecs::query::Without;
use bevy::ecs::system::Query;
use bevy::hierarchy::{BuildChildren, ChildBuilder, Children};
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::render::texture::Image;
use bevy::render::view::Visibility;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use enumset::EnumSet;
use strum::IntoEnumIterator;

use crate::model::{BoardCoords, Direction};

use super::EngineCoords;

pub struct FocusPlugin;

#[derive(Component, Debug, Clone)]
pub enum Focus {
    None,
    Selected(BoardCoords, EnumSet<Direction>),
    Busy,
}

#[derive(Event, Debug)]
pub struct UpdateFocusEvent(pub Focus);

#[derive(Component)]
pub struct FocusArrow(Direction);

pub struct FocusAssets {
    texture: Handle<Image>,
    arrow_textures: HashMap<Direction, Handle<Image>>,
}

#[derive(Bundle)]
struct FocusBundle {
    focus: Focus,
    sprite: SpriteBundle,
}

#[derive(Bundle)]
struct FocusArrowBundle {
    arrow: FocusArrow,
    sprite: SpriteBundle,
}

impl Focus {
    pub fn coords(&self) -> Option<BoardCoords> {
        match self {
            Focus::Selected(coords, _) => Some(*coords),
            _ => None,
        }
    }
}

impl FocusAssets {
    pub fn load(server: &AssetServer) -> Self {
        let texture = server.load("focus.png");
        let mut arrow_textures = HashMap::new();
        for direction in Direction::iter() {
            let path = match direction {
                Direction::Up => "focus-u.png",
                Direction::Left => "focus-l.png",
                Direction::Down => "focus-d.png",
                Direction::Right => "focus-r.png",
            };
            arrow_textures.insert(direction, server.load(path));
        }
        Self {
            texture,
            arrow_textures,
        }
    }
}

impl FocusBundle {
    fn new(assets: &FocusAssets) -> Self {
        Self {
            focus: Focus::None,
            sprite: SpriteBundle {
                texture: assets.texture.clone(),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
        }
    }
}

impl FocusArrowBundle {
    fn new(direction: Direction, assets: &FocusAssets) -> Self {
        Self {
            arrow: FocusArrow(direction),
            sprite: SpriteBundle {
                texture: assets.arrow_textures[&direction].clone(),
                visibility: Visibility::Hidden,
                transform: Transform {
                    translation: direction_offset(direction).extend(0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}

pub fn spawn_focus(parent: &mut ChildBuilder, assets: &FocusAssets) {
    let mut focus = parent.spawn(FocusBundle::new(assets));
    focus.with_children(|focus| {
        for direction in Direction::iter() {
            focus.spawn(FocusArrowBundle::new(direction, assets));
        }
    });
}

pub fn get_focus(query: Query<&Focus>) -> Focus {
    query.single().clone()
}

pub fn update_focus(
    mut events: EventReader<UpdateFocusEvent>,
    mut q_focus: Query<(&mut Focus, &mut Transform, &mut Visibility, &Children)>,
    mut q_arrow: Query<(&FocusArrow, &mut Visibility), Without<Focus>>,
) {
    let Some(event) = events.read().last() else {
        return;
    };
    let value = event.0.clone();
    let (mut focus, mut xform, mut visibility, children) = q_focus.single_mut();
    if let Focus::Selected(coords, directions) = &value {
        xform.translation = coords.to_xy().extend(Z_LAYER);
        *visibility = Visibility::Inherited;
        for &child in children {
            let (arrow, mut child_visibility) = q_arrow.get_mut(child).unwrap();
            *child_visibility = match directions.contains(arrow.0) {
                false => Visibility::Hidden,
                true => Visibility::Inherited,
            }
        }
    } else {
        *visibility = Visibility::Hidden;
    }
    *focus = value;
}

pub fn focus_direction_for_offset(offset: Vec2) -> Option<Direction> {
    for direction in Direction::iter() {
        if (offset - direction_offset(direction))
            .abs()
            .cmple(ARROW_HALF_SIZE)
            .all()
        {
            return Some(direction);
        }
    }
    None
}

fn direction_offset(direction: Direction) -> Vec2 {
    match direction {
        Direction::Up => Vec2::new(0.0, 11.0),
        Direction::Left => Vec2::new(-11.0, 0.0),
        Direction::Down => Vec2::new(0.0, -11.0),
        Direction::Right => Vec2::new(11.0, 0.0),
    }
}

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateFocusEvent>()
            .add_systems(FixedPostUpdate, update_focus);
    }
}

const ARROW_HALF_SIZE: Vec2 = Vec2::new(7.0, 7.0);
const Z_LAYER: f32 = 3.0;
