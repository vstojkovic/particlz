use std::collections::HashMap;

use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::hierarchy::{BuildChildren, ChildBuilder};
use bevy::math::Vec2;
use bevy::prelude::SpatialBundle;
use bevy::render::texture::Image;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use strum::IntoEnumIterator;

use crate::model::{Board, Emitters, Manipulator};

use super::animation::{AnimationAnchorBundle, AnimationBundle};
use super::beam::spawn_beams;
use super::BoardCoords;

pub struct ManipulatorAssets {
    textures: HashMap<Emitters, Handle<Image>>,
}

#[derive(Bundle)]
struct ManipulatorAnchorBundle {
    coords: BoardCoords,
    spatial: SpatialBundle,
    animation: AnimationAnchorBundle,
}

#[derive(Bundle)]
struct ManipulatorBundle {
    sprite: SpriteBundle,
    animation: AnimationBundle,
}

impl ManipulatorAssets {
    pub fn load(server: &AssetServer) -> Self {
        let mut textures = HashMap::new();
        for emitters in Emitters::iter() {
            let path = match emitters {
                Emitters::Left => "manipulator-l.png",
                Emitters::Right => "manipulator-r.png",
                Emitters::Up => "manipulator-u.png",
                Emitters::Down => "manipulator-d.png",
                Emitters::LeftUp => "manipulator-lu.png",
                Emitters::LeftDown => "manipulator-ld.png",
                Emitters::RightUp => "manipulator-ru.png",
                Emitters::RightDown => "manipulator-rd.png",
                Emitters::LeftRight => "manipulator-lr.png",
                Emitters::UpDown => "manipulator-ud.png",
            };
            textures.insert(emitters, server.load(path));
        }
        Self { textures }
    }
}

impl ManipulatorAnchorBundle {
    fn new(coords: BoardCoords) -> Self {
        Self {
            coords,
            spatial: SpatialBundle {
                transform: Transform {
                    translation: coords.to_xy().extend(0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            animation: AnimationAnchorBundle::new(),
        }
    }
}

impl ManipulatorBundle {
    fn new(manipulator: &Manipulator, anchor: Entity, assets: &ManipulatorAssets) -> Self {
        let texture = assets.textures[&manipulator.emitters].clone();
        Self {
            sprite: SpriteBundle {
                texture,
                transform: Transform {
                    translation: Vec2::ZERO.extend(Z_LAYER),
                    ..Default::default()
                },
                ..Default::default()
            },
            animation: AnimationBundle::new(anchor, Z_LAYER),
        }
    }
}

pub fn spawn_manipulator(
    parent: &mut ChildBuilder,
    manipulator: &Manipulator,
    coords: BoardCoords,
    board: &Board,
    assets: &ManipulatorAssets,
) -> Entity {
    let mut anchor = parent.spawn(ManipulatorAnchorBundle::new(coords));
    anchor.with_children(|anchor| {
        anchor.spawn(ManipulatorBundle::new(
            manipulator,
            anchor.parent_entity(),
            assets,
        ));
        spawn_beams(anchor, coords, manipulator.emitters, board)
    });
    anchor.id()
}

pub fn is_offset_inside_manipulator(offset: Vec2) -> bool {
    offset.length_squared() <= MANIPULATOR_SELECTION_RADIUS_SQUARED
}

const MANIPULATOR_SELECTION_RADIUS_SQUARED: f32 = 256.0;
const Z_LAYER: f32 = 2.0;
