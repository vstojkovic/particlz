use std::collections::HashMap;

use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::hierarchy::ChildBuilder;
use bevy::render::texture::Image;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use strum::IntoEnumIterator;

use crate::model::{BoardCoords, Tile, TileKind, Tint};

use super::{BoardCoordsHolder, EngineCoords};

pub struct TileAssets {
    textures: HashMap<(TileKind, Tint), Handle<Image>>,
}

#[derive(Bundle)]
struct TileBundle {
    coords: BoardCoordsHolder,
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
    fn new(tile: &Tile, coords: BoardCoords, assets: &TileAssets) -> Self {
        let coords = BoardCoordsHolder(coords);
        let texture = assets.textures[&(tile.kind, tile.tint)].clone();
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
        }
    }
}

pub fn spawn_tile(
    parent: &mut ChildBuilder,
    tile: &Tile,
    coords: BoardCoords,
    assets: &TileAssets,
) -> Entity {
    parent.spawn(TileBundle::new(tile, coords, assets)).id()
}

const Z_LAYER: f32 = 0.0;
