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

#[derive(Component, Default)]
pub struct Focus(Option<BoardCoords>);

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
    pub fn spawn(parent: &mut ChildBuilder, assets: &FocusAssets) {
        let mut focus = parent.spawn(FocusBundle::default());
        focus.with_children(|focus| {
            for direction in Direction::iter() {
                focus.spawn(FocusArrowBundle::new(direction, assets));
            }
        });
    }

    pub fn get_coords(query: &Query<&Focus>) -> Option<BoardCoords> {
        query.single().0
    }

    pub fn update(
        coords: Option<BoardCoords>,
        directions: EnumSet<Direction>,
        q_focus: &mut Query<(&mut Focus, &mut Transform, &Children)>,
        q_arrow: &mut Query<(&FocusArrow, &mut Visibility)>,
    ) {
        let (mut focus, mut xform, children) = q_focus.single_mut();
        focus.0 = coords;
        if let Some(coords) = coords {
            xform.translation = coords.to_xy().extend(3.0);
        }
        for &child in children {
            let (arrow, mut child_visibility) = q_arrow.get_mut(child).unwrap();
            let show = coords.is_some() && directions.contains(arrow.0);
            *child_visibility = match show {
                false => Visibility::Hidden,
                true => Visibility::Visible,
            }
        }
    }
}

impl FocusAssets {
    pub fn load(server: &AssetServer) -> Self {
        let mut textures = HashMap::new();
        for direction in Direction::iter() {
            let path = match direction {
                Direction::Left => "focus-l.png",
                Direction::Right => "focus-r.png",
                Direction::Up => "focus-u.png",
                Direction::Down => "focus-d.png",
            };
            textures.insert(direction, server.load(path));
        }
        Self { textures }
    }
}

impl FocusArrowBundle {
    fn new(direction: Direction, assets: &FocusAssets) -> Self {
        let offset = match direction {
            Direction::Left => Vec2::new(-11.0, 0.0),
            Direction::Right => Vec2::new(11.0, 0.0),
            Direction::Up => Vec2::new(0.0, 11.0),
            Direction::Down => Vec2::new(0.0, -11.0),
        };
        Self {
            arrow: FocusArrow(direction),
            sprite: SpriteBundle {
                texture: assets.textures[&direction].clone(),
                visibility: Visibility::Hidden,
                transform: Transform {
                    translation: offset.extend(0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}
