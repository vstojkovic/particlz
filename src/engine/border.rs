use std::collections::HashMap;

use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::hierarchy::ChildBuilder;
use bevy::math::{Quat, Vec2};
use bevy::render::texture::Image;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use strum::IntoEnumIterator;

use crate::model::Border;

use super::BoardCoords;

pub struct BorderAssets {
    textures: HashMap<Border, Handle<Image>>,
}

#[derive(Bundle)]
struct BorderBundle {
    coords: BoardCoords,
    sprite: SpriteBundle,
}

#[derive(Debug, Clone, Copy)]
enum Orientation {
    Horizontal,
    Vertical,
}

impl Orientation {
    fn offset(self) -> Vec2 {
        match self {
            Self::Horizontal => Vec2::new(0.0, -BORDER_OFFSET_Y),
            Self::Vertical => Vec2::new(BORDER_OFFSET_X, 0.0),
        }
    }

    fn rotation(self) -> Quat {
        match self {
            Orientation::Horizontal => Quat::from_rotation_z(f32::to_radians(90.0)),
            Orientation::Vertical => Quat::IDENTITY,
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
    fn new(
        border: &Border,
        coords: BoardCoords,
        orientation: Orientation,
        assets: &BorderAssets,
    ) -> Self {
        let texture = assets.textures[border].clone();
        Self {
            coords,
            sprite: SpriteBundle {
                texture,
                transform: Transform {
                    translation: (coords.to_xy() - orientation.offset()).extend(2.0),
                    rotation: orientation.rotation(),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}

pub fn spawn_horz_border(
    parent: &mut ChildBuilder,
    border: &Border,
    coords: BoardCoords,
    assets: &BorderAssets,
) -> Entity {
    parent
        .spawn(BorderBundle::new(
            border,
            coords,
            Orientation::Horizontal,
            assets,
        ))
        .id()
}

pub fn spawn_vert_border(
    parent: &mut ChildBuilder,
    border: &Border,
    coords: BoardCoords,
    assets: &BorderAssets,
) -> Entity {
    parent
        .spawn(BorderBundle::new(
            border,
            coords,
            Orientation::Vertical,
            assets,
        ))
        .id()
}

const BORDER_OFFSET_X: f32 = 22.0;
const BORDER_OFFSET_Y: f32 = 22.0;
