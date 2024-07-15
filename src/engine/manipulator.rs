use std::collections::HashMap;
use std::sync::Arc;

use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::hierarchy::{BuildChildren, ChildBuilder};
use bevy::math::Vec2;
use bevy::render::texture::Image;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use strum::IntoEnumIterator;

use crate::model::{Board, BoardCoords, Emitters, Manipulator};

use super::animation::AnimationBundle;
use super::beam::spawn_beams;
use super::{BoardCoordsHolder, EngineCoords};

pub struct ManipulatorAssets {
    textures: HashMap<Emitters, Handle<Image>>,
}

#[derive(Bundle)]
struct ManipulatorBundle {
    coords: BoardCoordsHolder,
    sprite: SpriteBundle,
    animation: AnimationBundle,
}

impl ManipulatorAssets {
    pub fn load(server: &AssetServer, barrier: &Arc<()>) -> Self {
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
            textures.insert(emitters, server.load_acquire(path, Arc::clone(&barrier)));
        }
        Self { textures }
    }
}

impl ManipulatorBundle {
    fn new(coords: BoardCoords, manipulator: &Manipulator, assets: &ManipulatorAssets) -> Self {
        let coords = BoardCoordsHolder(coords);
        let texture = assets.textures[&manipulator.emitters].clone();
        Self {
            coords,
            sprite: SpriteBundle {
                texture,
                transform: Transform {
                    translation: coords.to_xy().extend(Z_LAYER),
                    ..Default::default()
                },
                ..Default::default()
            },
            animation: AnimationBundle::default(),
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
    let mut parent = parent.spawn(ManipulatorBundle::new(coords, manipulator, assets));
    parent.with_children(|parent| spawn_beams(parent, coords, manipulator.emitters, board));
    parent.id()
}

pub fn is_offset_inside_manipulator(offset: Vec2) -> bool {
    offset.length_squared() <= MANIPULATOR_SELECTION_RADIUS_SQUARED
}

const MANIPULATOR_SELECTION_RADIUS_SQUARED: f32 = 256.0;
const Z_LAYER: f32 = 2.0;
