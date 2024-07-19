use std::sync::Arc;

use bevy::asset::AssetServer;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::hierarchy::ChildBuilder;
use bevy::prelude::*;
use enum_map::EnumMap;
use strum::IntoEnumIterator;

use crate::model::{BoardCoords, Particle, Tint};

use super::animation::{AnimatedSpriteBundle, AnimationBundle, FadeOutAnimator};
use super::beam::HaloBundle;
use super::{BoardCoordsHolder, EngineCoords, SpriteSheet};

pub struct ParticleAssets {
    sheets: EnumMap<Tint, ParticleSheets>,
    halo: SpriteSheet,
}

#[derive(Debug, Default)]
struct ParticleSheets {
    core: SpriteSheet,
    corona: SpriteSheet,
}

#[derive(Bundle)]
struct ParticleBundle {
    coords: BoardCoordsHolder,
    sprite: AnimatedSpriteBundle,
    animation: AnimationBundle,
}

#[derive(Component)]
pub struct Corona;

#[derive(Event)]
pub struct ParticleCollected(pub Entity);

impl ParticleAssets {
    pub fn load(server: &AssetServer, barrier: &Arc<()>) -> Self {
        let mut sheets = EnumMap::default();
        for tint in Tint::iter() {
            let prefix = match tint {
                Tint::White => continue,
                Tint::Green => "particle-green",
                Tint::Yellow => "particle-yellow",
                Tint::Red => "particle-red",
            };
            let core = server.load_acquire(format!("{}-core.png", prefix), Arc::clone(&barrier));
            let corona =
                server.load_acquire(format!("{}-corona.png", prefix), Arc::clone(&barrier));
            sheets[tint] = ParticleSheets {
                core: SpriteSheet::new(core, UVec2::splat(34), 96, server),
                corona: SpriteSheet::new(corona, UVec2::splat(34), 96, server),
            };
        }

        let halo = SpriteSheet::new(
            server.load_acquire("particle-halo.png", Arc::clone(&barrier)),
            UVec2::splat(37),
            48,
            server,
        );

        Self { sheets, halo }
    }
}

impl ParticleBundle {
    fn new(coords: BoardCoords, particle: &Particle, assets: &ParticleAssets) -> Self {
        let coords = BoardCoordsHolder(coords);
        let sheets = &assets.sheets[particle.tint];
        let sprite = SpriteBundle {
            transform: Transform {
                translation: coords.to_xy().extend(Z_LAYER),
                ..Default::default()
            },
            ..Default::default()
        };
        Self {
            coords,
            sprite: AnimatedSpriteBundle::with_defaults(&sheets.core, sprite),
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
    let mut anchor = parent.spawn(ParticleBundle::new(coords, particle, assets));
    anchor.with_children(|anchor| {
        let sprite = SpriteBundle {
            transform: Transform {
                translation: Vec2::ZERO.extend(REL_Z_LAYER_CORONA),
                ..Default::default()
            },
            ..Default::default()
        };
        anchor.spawn((
            Corona,
            BoardCoordsHolder(coords),
            AnimatedSpriteBundle::with_defaults(&assets.sheets[particle.tint].corona, sprite),
            FadeOutAnimator::default(),
        ));

        anchor.spawn(HaloBundle::new(coords, &assets.halo, REL_Z_LAYER_HALO));
    });
    anchor.id()
}

pub fn collect_particles(
    mut ev_collected: EventReader<ParticleCollected>,
    q_children: Query<&Children>,
    mut q_corona: Query<&mut Visibility, With<Corona>>,
) {
    for &ParticleCollected(anchor) in ev_collected.read() {
        for &child in q_children.get(anchor).unwrap().iter() {
            if let Ok(mut visibility) = q_corona.get_mut(child) {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

const Z_LAYER: f32 = 2.0;
const REL_Z_LAYER_CORONA: f32 = 1.0;
const REL_Z_LAYER_HALO: f32 = 2.0;
