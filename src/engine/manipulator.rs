use std::collections::HashMap;

use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::math::Vec2;
use bevy::render::texture::Image;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use strum::IntoEnumIterator;

use crate::model::{Emitters, Manipulator};

use super::BoardCoords;

pub struct ManipulatorAssets {
    textures: HashMap<Emitters, Handle<Image>>,
}

#[derive(Bundle)]
pub struct ManipulatorBundle {
    coords: BoardCoords,
    sprite: SpriteBundle,
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

impl ManipulatorBundle {
    pub fn new(manipulator: &Manipulator, coords: BoardCoords, assets: &ManipulatorAssets) -> Self {
        let texture = assets.textures[&manipulator.emitters].clone();
        Self {
            coords,
            sprite: SpriteBundle {
                texture,
                transform: Transform {
                    translation: coords.to_xy().extend(2.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}

pub fn is_offset_inside_manipulator(offset: Vec2) -> bool {
    offset.length_squared() <= MANIPULATOR_SELECTION_RADIUS_SQUARED
}

const MANIPULATOR_SELECTION_RADIUS_SQUARED: f32 = 256.0;
