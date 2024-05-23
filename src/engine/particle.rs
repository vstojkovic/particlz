use std::collections::HashMap;

use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::render::texture::Image;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use strum::IntoEnumIterator;

use crate::model::{Particle, Tint};

use super::BoardCoords;

pub struct ParticleAssets {
    textures: HashMap<Tint, Handle<Image>>,
}

#[derive(Bundle)]
pub struct ParticleBundle {
    coords: BoardCoords,
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
    pub fn new(particle: &Particle, coords: BoardCoords, assets: &ParticleAssets) -> Self {
        let texture = assets.textures[&particle.tint].clone();
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
