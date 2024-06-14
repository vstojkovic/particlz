use std::collections::HashMap;

use bevy::asset::{AssetServer, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::hierarchy::{BuildChildren, ChildBuilder};
use bevy::math::Vec2;
use bevy::prelude::SpatialBundle;
use bevy::render::texture::Image;
use bevy::sprite::SpriteBundle;
use bevy::transform::components::Transform;
use strum::IntoEnumIterator;

use crate::model::{BoardCoords, Particle, Tint};

use super::animation::{AnimationAnchorBundle, AnimationBundle};
use super::{BoardCoordsHolder, EngineCoords};

pub struct ParticleAssets {
    textures: HashMap<Tint, Handle<Image>>,
}

#[derive(Bundle)]
struct ParticleAnchorBundle {
    coords: BoardCoordsHolder,
    spatial: SpatialBundle,
    animation: AnimationAnchorBundle,
}

#[derive(Bundle)]
struct ParticleBundle {
    sprite: SpriteBundle,
    animation: AnimationBundle,
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

impl ParticleAnchorBundle {
    fn new(coords: BoardCoords) -> Self {
        let coords = BoardCoordsHolder(coords);
        Self {
            coords,
            spatial: SpatialBundle {
                transform: Transform {
                    translation: coords.to_xy().extend(0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            animation: AnimationAnchorBundle::new(),
        }
    }
}

impl ParticleBundle {
    fn new(particle: &Particle, anchor: Entity, assets: &ParticleAssets) -> Self {
        let texture = assets.textures[&particle.tint].clone();
        Self {
            sprite: SpriteBundle {
                texture,
                transform: Transform {
                    translation: Vec2::ZERO.extend(Z_LAYER),
                    ..Default::default()
                },
                ..Default::default()
            },
            animation: AnimationBundle::new(anchor, Z_LAYER),
        }
    }
}

pub fn spawn_particle(
    parent: &mut ChildBuilder,
    particle: &Particle,
    coords: BoardCoords,
    assets: &ParticleAssets,
) -> Entity {
    let mut anchor = parent.spawn(ParticleAnchorBundle::new(coords));
    anchor.with_children(|anchor| {
        anchor.spawn(ParticleBundle::new(
            particle,
            anchor.parent_entity(),
            assets,
        ));
    });
    anchor.id()
}

const Z_LAYER: f32 = 2.0;
