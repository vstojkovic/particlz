use std::collections::HashMap;

use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::math::Vec3;
use bevy::render::texture::Image;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use strum::IntoEnumIterator;

use crate::model::{Particle, Tint};

use super::{TILE_HEIGHT, TILE_WIDTH};

pub struct ParticleAssets {
    textures: HashMap<Tint, Handle<Image>>,
}

#[derive(Bundle)]
pub struct ParticleBundle {
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
    pub fn new(particle: &Particle, row: usize, col: usize, assets: &ParticleAssets) -> Self {
        let texture = assets.textures[&particle.tint].clone();
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
