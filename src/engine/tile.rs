use std::collections::HashMap;

use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::render::texture::Image;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use strum::IntoEnumIterator;

use crate::model::{Tile, TileKind, Tint};

use super::BoardCoords;

pub struct TileAssets {
    textures: HashMap<(TileKind, Tint), Handle<Image>>,
}

#[derive(Bundle)]
pub struct TileBundle {
    coords: BoardCoords,
    sprite: SpriteBundle,
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
    pub fn new(tile: &Tile, coords: BoardCoords, assets: &TileAssets) -> Self {
        let texture = assets.textures[&(tile.kind, tile.tint)].clone();
        Self {
            coords,
            sprite: SpriteBundle {
                texture,
                transform: Transform {
                    translation: coords.to_xy().extend(0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}
