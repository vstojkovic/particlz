use std::collections::HashMap;

use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::hierarchy::ChildBuilder;
use bevy::render::texture::Image;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use strum::IntoEnumIterator;

use crate::model::{BoardCoords, Particle, Tint};

use super::{BoardCoordsHolder, EngineCoords};

pub struct ParticleAssets {
    textures: HashMap<Tint, Handle<Image>>,
}

#[derive(Bundle)]
struct ParticleBundle {
    coords: BoardCoordsHolder,
    sprite: SpriteBundle,
}

impl ParticleAssets {
    pub fn load(server: &AssetServer) -> Self {
        let mut textures = HashMap::new();
        for tint in Tint::iter() {
            let path = match tint {
                Tint::White => continue,
                Tint::Green => "particle-green.png",
                Tint::Yellow => "particle-yellow.png",
                Tint::Red => "particle-red.png",
            };
            textures.insert(tint, server.load(path));
        }
        Self { textures }
    }
}

impl ParticleBundle {
    fn new(particle: &Particle, coords: BoardCoords, assets: &ParticleAssets) -> Self {
        let coords = BoardCoordsHolder(coords);
        let texture = assets.textures[&particle.tint].clone();
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

pub fn spawn_particle(
    parent: &mut ChildBuilder,
    particle: &Particle,
    coords: BoardCoords,
    assets: &ParticleAssets,
) -> Entity {
    parent
        .spawn(ParticleBundle::new(particle, coords, assets))
        .id()
}

const Z_LAYER: f32 = 2.0;
