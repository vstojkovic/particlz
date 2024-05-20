use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::math::Vec3;
use bevy::render::texture::Image;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use bevy::utils::HashMap;
use strum::IntoEnumIterator;

use crate::tile::{TILE_HEIGHT, TILE_WIDTH};
use crate::Tint;

pub struct Particle {
    pub tint: Tint,
}

pub struct ParticleAssets {
    textures: HashMap<Tint, Handle<Image>>,
}

#[derive(Bundle)]
pub struct ParticleBundle {
    sprite: SpriteBundle,
}

impl Particle {
    pub fn new(tint: Tint) -> Self {
        assert!(tint != Tint::White);
        Self { tint }
    }
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
