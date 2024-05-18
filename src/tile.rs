use std::collections::HashMap;

use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::math::Vec3;
use bevy::render::texture::Image;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::Tint;

pub struct Tile {
    pub kind: TileKind,
    pub tint: Tint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum TileKind {
    Platform,
    Collector,
}

pub struct TileAssets {
    textures: HashMap<(TileKind, Tint), Handle<Image>>,
}

#[derive(Bundle)]
pub struct TileBundle {
    sprite: SpriteBundle,
}

impl Tile {
    pub fn new(kind: TileKind, tint: Tint) -> Self {
        Self { kind, tint }
    }
}

impl TileAssets {
    pub fn load(server: &AssetServer) -> Self {
        let mut textures = HashMap::new();
        for kind in TileKind::iter() {
            let kind_part = match kind {
                TileKind::Platform => "platform",
                TileKind::Collector => "collector",
            };
            for tint in Tint::iter() {
                let tint_part = match tint {
                    Tint::White => "white",
                    Tint::Green => "green",
                    Tint::Yellow => "yellow",
                    Tint::Red => "red",
                };
                textures.insert(
                    (kind, tint),
                    server.load(format!("{}-{}.png", kind_part, tint_part)),
                );
            }
        }
        Self { textures }
    }
}

impl TileBundle {
    pub fn new(tile: &Tile, row: usize, col: usize, assets: &TileAssets) -> Self {
        let texture = assets.textures[&(tile.kind, tile.tint)].clone();
        let x = TILE_WIDTH * col as f32;
        let y = TILE_HEIGHT * row as f32;
        Self {
            sprite: SpriteBundle {
                texture,
                transform: Transform {
                    translation: Vec3::new(x, -y, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}

pub const TILE_WIDTH: f32 = 45.0;
pub const TILE_HEIGHT: f32 = 45.0;
