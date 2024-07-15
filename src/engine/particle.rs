use std::collections::HashMap;
use std::sync::Arc;

use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::hierarchy::ChildBuilder;
use bevy::render::texture::Image;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use strum::IntoEnumIterator;

use crate::model::{BoardCoords, Particle, Tint};

use super::animation::AnimationBundle;
use super::{BoardCoordsHolder, EngineCoords};

pub struct ParticleAssets {
    textures: HashMap<Tint, Handle<Image>>,
}

#[derive(Bundle)]
struct ParticleBundle {
    coords: BoardCoordsHolder,
    sprite: SpriteBundle,
    animation: AnimationBundle,
}

impl ParticleAssets {
    pub fn load(server: &AssetServer, barrier: &Arc<()>) -> Self {
        let mut textures = HashMap::new();
        for tint in Tint::iter() {
            let path = match tint {
                Tint::White => continue,
                Tint::Green => "particle-green.png",
                Tint::Yellow => "particle-yellow.png",
                Tint::Red => "particle-red.png",
            };
            textures.insert(tint, server.load_acquire(path, Arc::clone(&barrier)));
        }
        Self { textures }
    }
}

impl ParticleBundle {
    fn new(coords: BoardCoords, particle: &Particle, assets: &ParticleAssets) -> Self {
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
            animation: AnimationBundle::default(),
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
        .spawn(ParticleBundle::new(coords, particle, assets))
        .id()
}

const Z_LAYER: f32 = 2.0;
