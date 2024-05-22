use std::collections::HashMap;

use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::math::{Quat, Vec3};
use bevy::render::texture::Image;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use strum::IntoEnumIterator;

use crate::model::Border;

use super::{TILE_HEIGHT, TILE_WIDTH};

pub struct BorderAssets {
    textures: HashMap<Border, Handle<Image>>,
}

#[derive(Bundle)]
pub struct BorderBundle {
    sprite: SpriteBundle,
}

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Orientation {
    fn offset_x(self) -> f32 {
        match self {
            Self::Horizontal => 0.0,
            Self::Vertical => BORDER_OFFSET_X,
        }
    }

    fn offset_y(self) -> f32 {
        match self {
            Self::Horizontal => BORDER_OFFSET_Y,
            Self::Vertical => 0.0,
        }
    }
}

impl BorderAssets {
    pub fn load(server: &AssetServer) -> Self {
        let mut textures = HashMap::new();
        for kind in Border::iter() {
            let path = match kind {
                Border::Wall => "wall.png",
                Border::Window => "window.png",
            };
            textures.insert(kind, server.load(path));
        }
        Self { textures }
    }
}

impl BorderBundle {
    pub fn new(
        border: &Border,
        row: usize,
        col: usize,
        orientation: Orientation,
        assets: &BorderAssets,
    ) -> Self {
        let texture = assets.textures[border].clone();
        let x = TILE_WIDTH * col as f32 - orientation.offset_x();
        let y = TILE_HEIGHT * row as f32 - orientation.offset_y();
        let rotation = match orientation {
            Orientation::Horizontal => Quat::from_rotation_z(f32::to_radians(90.0)),
            Orientation::Vertical => Quat::IDENTITY,
        };
        Self {
            sprite: SpriteBundle {
                texture,
                transform: Transform {
                    translation: Vec3::new(x, -y, 2.0),
                    rotation,
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}

const BORDER_OFFSET_X: f32 = 22.0;
const BORDER_OFFSET_Y: f32 = 22.0;
