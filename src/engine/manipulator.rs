use std::collections::HashMap;

use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::math::Vec3;
use bevy::render::texture::Image;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use strum::IntoEnumIterator;

use crate::model::{Emitters, Manipulator};

use super::{TILE_HEIGHT, TILE_WIDTH};

pub struct ManipulatorAssets {
    textures: HashMap<Emitters, Handle<Image>>,
}

#[derive(Bundle)]
pub struct ManipulatorBundle {
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
    pub fn new(
        manipulator: &Manipulator,
        row: usize,
        col: usize,
        assets: &ManipulatorAssets,
    ) -> Self {
        let texture = assets.textures[&manipulator.emitters].clone();
        let x = TILE_WIDTH * col as f32;
        let y = TILE_HEIGHT * row as f32;
        Self {
            sprite: SpriteBundle {
                texture,
                transform: Transform {
                    translation: Vec3::new(x, -y, 2.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}
