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

use crate::model::{Board, BoardCoords, Emitters, Manipulator};

use super::animation::{AnimatedSpriteBundle, AnimationBundle, FadeOutAnimator};
use super::beam::{spawn_beams, HaloBundle};
use super::{BoardCoordsHolder, EngineCoords, GameAssets, SpriteSheet};

pub struct ManipulatorAssets {
    textures: EnumMap<Emitters, Handle<Image>>,
    halos: EnumMap<Emitters, SpriteSheet>,
    core: SpriteSheet,
}

#[derive(Bundle)]
struct ManipulatorBundle {
    coords: BoardCoordsHolder,
    sprite: SpriteBundle,
    animation: AnimationBundle,
}

impl ManipulatorAssets {
    pub fn load(server: &AssetServer, barrier: &Arc<()>) -> Self {
        let mut textures = EnumMap::default();
        let mut halos = EnumMap::default();
        for emitters in Emitters::iter() {
            let prefix = match emitters {
                Emitters::Left => "manipulator-l",
                Emitters::Right => "manipulator-r",
                Emitters::Up => "manipulator-u",
                Emitters::Down => "manipulator-d",
                Emitters::LeftUp => "manipulator-lu",
                Emitters::LeftDown => "manipulator-ld",
                Emitters::RightUp => "manipulator-ru",
                Emitters::RightDown => "manipulator-rd",
                Emitters::LeftRight => "manipulator-lr",
                Emitters::UpDown => "manipulator-ud",
            };
            textures[emitters] =
                server.load_acquire(format!("{}.png", prefix), Arc::clone(&barrier));
            halos[emitters] = SpriteSheet::new(
                server.load_acquire(format!("{}-halo.png", prefix), Arc::clone(&barrier)),
                UVec2::splat(39),
                48,
                server,
            );
        }

        let core = SpriteSheet::new(
            server.load_acquire("manipulator-core.png", Arc::clone(&barrier)),
            UVec2::splat(14),
            48,
            server,
        );

        Self {
            textures,
            halos,
            core,
        }
    }
}

impl ManipulatorBundle {
    fn new(coords: BoardCoords, manipulator: &Manipulator, assets: &ManipulatorAssets) -> Self {
        let coords = BoardCoordsHolder(coords);
        let texture = assets.textures[manipulator.emitters].clone();
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

pub fn spawn_manipulator(
    parent: &mut ChildBuilder,
    manipulator: &Manipulator,
    coords: BoardCoords,
    board: &Board,
    assets: &GameAssets,
) -> Entity {
    let mut anchor = parent.spawn(ManipulatorBundle::new(
        coords,
        manipulator,
        &assets.manipulators,
    ));
    anchor.with_children(|anchor| {
        anchor.spawn((
            BoardCoordsHolder(coords),
            AnimatedSpriteBundle::new(&assets.manipulators.core),
            FadeOutAnimator::default(),
        ));

        anchor.spawn(HaloBundle::new(
            coords,
            &assets.manipulators.halos[manipulator.emitters],
            REL_Z_LAYER_HALO,
        ));

        spawn_beams(anchor, coords, manipulator.emitters, board, &assets.beams);
    });
    anchor.id()
}

pub fn is_offset_inside_manipulator(offset: Vec2) -> bool {
    offset.length_squared() <= MANIPULATOR_SELECTION_RADIUS_SQUARED
}

const MANIPULATOR_SELECTION_RADIUS_SQUARED: f32 = 256.0;
const Z_LAYER: f32 = 2.0;
const REL_Z_LAYER_HALO: f32 = 1.0;
