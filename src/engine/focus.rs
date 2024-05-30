use std::collections::HashMap;

use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::system::Query;
use bevy::hierarchy::{BuildChildren, ChildBuilder, Children};
use bevy::math::Vec2;
use bevy::prelude::SpatialBundle;
use bevy::render::texture::Image;
use bevy::render::view::Visibility;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use enumset::EnumSet;
use strum::IntoEnumIterator;

use super::{BoardCoords, Direction};

#[derive(Component, Debug, Clone)]
pub enum Focus {
    None,
    Selected(BoardCoords, EnumSet<Direction>),
    Busy,
}

impl Default for Focus {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Component)]
pub struct FocusArrow(Direction);

pub struct FocusAssets {
    textures: HashMap<Direction, Handle<Image>>,
}

#[derive(Bundle, Default)]
struct FocusBundle {
    focus: Focus,
    spatial: SpatialBundle,
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
        let mut textures = HashMap::new();
        for direction in Direction::iter() {
            let path = match direction {
                Direction::Up => "focus-u.png",
                Direction::Left => "focus-l.png",
                Direction::Down => "focus-d.png",
                Direction::Right => "focus-r.png",
            };
            textures.insert(direction, server.load(path));
        }
        Self { textures }
    }
}

impl FocusArrowBundle {
    fn new(direction: Direction, assets: &FocusAssets) -> Self {
        Self {
            arrow: FocusArrow(direction),
            sprite: SpriteBundle {
                texture: assets.textures[&direction].clone(),
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
    let mut focus = parent.spawn(FocusBundle::default());
    focus.with_children(|focus| {
        for direction in Direction::iter() {
            focus.spawn(FocusArrowBundle::new(direction, assets));
        }
    });
}

pub fn get_focus<'q>(query: &'q Query<&Focus>) -> &'q Focus {
    &*query.single()
}

pub fn set_focus(
    value: Focus,
    q_focus: &mut Query<(&mut Focus, &mut Transform, &Children)>,
    q_arrow: &mut Query<(&FocusArrow, &mut Visibility)>,
) {
    let (mut focus, mut xform, children) = q_focus.single_mut();
    let (coords, directions) = match &value {
        Focus::Selected(coords, directions) => (Some(coords), Some(directions)),
        _ => (None, None),
    };
    if let Some(coords) = coords {
        xform.translation = coords.to_xy().extend(Z_LAYER);
    }
    for &child in children {
        let (arrow, mut child_visibility) = q_arrow.get_mut(child).unwrap();
        let show = directions
            .map(|directions| directions.contains(arrow.0))
            .unwrap_or(false);
        *child_visibility = match show {
            false => Visibility::Hidden,
            true => Visibility::Visible,
        }
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

const ARROW_HALF_SIZE: Vec2 = Vec2::new(7.0, 7.0);
const Z_LAYER: f32 = 3.0;
