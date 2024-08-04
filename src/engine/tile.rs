use std::sync::Arc;

use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::hierarchy::{BuildChildren, ChildBuilder};
use bevy::prelude::*;
use bevy::render::texture::Image;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use enum_map::EnumMap;
use strum::IntoEnumIterator;

use crate::model::{BoardCoords, Tile, TileKind, Tint};

use super::animation::AnimatedSpriteBundle;
use super::{BoardCoordsHolder, EngineCoords, SpriteSheet};

pub struct TileAssets {
    textures: EnumMap<TileKind, EnumMap<Tint, Handle<Image>>>,
    collector_pulse: SpriteSheet,
}

#[derive(Bundle)]
struct TileBundle {
    coords: BoardCoordsHolder,
    sprite: SpriteBundle,
}

impl TileAssets {
    pub fn load(server: &AssetServer, barrier: &Arc<()>) -> Self {
        let mut textures = EnumMap::<TileKind, EnumMap<Tint, Handle<Image>>>::default();
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
                textures[kind][tint] = server.load_acquire(
                    format!("{}-{}.png", kind_part, tint_part),
                    Arc::clone(&barrier),
                );
            }
        }

        let texture = server.load_acquire("collector-pulse.png", Arc::clone(&barrier));
        let collector_pulse = SpriteSheet::new(texture, UVec2::splat(20), 48, server);

        Self {
            textures,
            collector_pulse,
        }
    }
}

impl TileBundle {
    fn new(tile: &Tile, coords: BoardCoords, assets: &TileAssets) -> Self {
        let coords = BoardCoordsHolder(coords);
        let texture = assets.textures[tile.kind][tile.tint].clone();
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
    let mut tile_entity = parent.spawn(TileBundle::new(tile, coords, assets));
    if tile.kind == TileKind::Collector {
        tile_entity.with_children(|parent| {
            let sprite = SpriteBundle {
                transform: Transform {
                    translation: Vec2::ZERO.extend(REL_Z_LAYER_PULSE),
                    ..Default::default()
                },
                ..Default::default()
            };
            parent.spawn(AnimatedSpriteBundle::with_defaults(
                &assets.collector_pulse,
                sprite,
            ));
        });
    }
    tile_entity.id()
}

const Z_LAYER: f32 = 0.0;
const REL_Z_LAYER_PULSE: f32 = 1.0;
