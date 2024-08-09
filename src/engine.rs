//! Engine-specific game data and logic

use std::sync::{Arc, Once, Weak};
use std::time::Duration;

use bevy::asset::AssetServer;
use bevy::ecs::component::Component;
use bevy::ecs::system::{EntityCommands, Resource};
use bevy::math::Vec2;
use bevy::prelude::*;

pub mod animation;
pub mod beam;
pub mod border;
pub mod focus;
pub mod gui;
pub mod input;
pub mod level;
pub mod manipulator;
pub mod particle;
pub mod tile;

use crate::model::{BoardCoords, Direction};

use self::beam::BeamAssets;
use self::border::BorderAssets;
use self::focus::FocusAssets;
use self::gui::GuiAssets;
use self::manipulator::ManipulatorAssets;
use self::particle::ParticleAssets;
use self::tile::TileAssets;

const TILE_WIDTH: f32 = 45.0;
const TILE_HEIGHT: f32 = 45.0;
const COORDS_ORIGIN_OFFSET: Vec2 = Vec2 { x: 22.5, y: -22.5 };
const MOVE_DURATION: Duration = Duration::from_millis(500);

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Init,
    MainMenu,
    ClassicLevelSelect,
    Playing,
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InLevel;

impl ComputedStates for InLevel {
    type SourceStates = GameState;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        match sources {
            GameState::Playing | GameState::GameOver => Some(Self),
            _ => None,
        }
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InLevelSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameplaySet;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BoardCoordsHolder(pub BoardCoords);

pub struct AssetsPlugin;

#[derive(Resource)]
pub struct GameAssets {
    load_barrier: Weak<()>,
    event_trigger: Once,
    gui: GuiAssets,
    tiles: TileAssets,
    borders: BorderAssets,
    particles: ParticleAssets,
    manipulators: ManipulatorAssets,
    beams: BeamAssets,
    focus: FocusAssets,
}

#[derive(Event, Debug)]
pub struct AssetsLoaded;

impl GameAssets {
    pub fn load(server: &AssetServer) -> Self {
        let load_barrier = Arc::new(());
        Self {
            load_barrier: Arc::downgrade(&load_barrier),
            event_trigger: Once::new(),
            gui: GuiAssets::load(server, &load_barrier),
            tiles: TileAssets::load(server, &load_barrier),
            borders: BorderAssets::load(server, &load_barrier),
            particles: ParticleAssets::load(server, &load_barrier),
            manipulators: ManipulatorAssets::load(server, &load_barrier),
            beams: BeamAssets::load(server, &load_barrier),
            focus: FocusAssets::load(server, &load_barrier),
        }
    }

    fn ready(&self) -> bool {
        self.load_barrier.strong_count() == 0
    }
}

fn load_assets(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(GameAssets::load(&server));
}

fn monitor_load(assets: Res<GameAssets>, mut ev_loaded: EventWriter<AssetsLoaded>) {
    if assets.ready() {
        assets.event_trigger.call_once(|| {
            ev_loaded.send(AssetsLoaded);
        });
    }
}

#[derive(Debug, Default)]
pub struct SpriteSheet {
    texture: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    frames: usize,
}

impl SpriteSheet {
    fn new(texture: Handle<Image>, tile_size: UVec2, frames: usize, server: &AssetServer) -> Self {
        let layout = TextureAtlasLayout::from_grid(tile_size, 1, frames as _, None, None);
        let layout = server.add(layout);
        Self {
            texture,
            layout,
            frames,
        }
    }
}

trait Mutable: Sized {
    fn mutate(mut self, mutator: &impl Fn(&mut Self)) -> Self {
        mutator(&mut self);
        self
    }
}

impl Mutable for EntityCommands<'_> {}

trait EngineCoords: Sized {
    fn from_xy(pos: Vec2) -> Option<Self>;
    fn to_xy(self) -> Vec2;
}

impl EngineCoords for BoardCoords {
    fn from_xy(pos: Vec2) -> Option<Self> {
        let pos = pos - COORDS_ORIGIN_OFFSET + Vec2::new(TILE_WIDTH, -TILE_HEIGHT) / 2.0;
        if pos.x < 0.0 || pos.y > 0.0 {
            return None;
        }
        let row = (-pos.y / TILE_HEIGHT).trunc() as usize;
        let col = (pos.x / TILE_WIDTH).trunc() as usize;
        Some(Self::new(row, col))
    }

    fn to_xy(self) -> Vec2 {
        Vec2 {
            x: (self.col as f32) * TILE_WIDTH,
            y: -(self.row as f32) * TILE_HEIGHT,
        } + COORDS_ORIGIN_OFFSET
    }
}

impl EngineCoords for BoardCoordsHolder {
    fn from_xy(pos: Vec2) -> Option<Self> {
        BoardCoords::from_xy(pos).map(|coords| Self(coords))
    }

    fn to_xy(self) -> Vec2 {
        self.0.to_xy()
    }
}

trait EngineDirection {
    fn delta(self) -> Vec2;
}

impl EngineDirection for Direction {
    fn delta(self) -> Vec2 {
        match self {
            Self::Up => Vec2::new(0.0, TILE_HEIGHT),
            Self::Left => Vec2::new(-TILE_WIDTH, 0.0),
            Self::Down => Vec2::new(0.0, -TILE_HEIGHT),
            Self::Right => Vec2::new(TILE_WIDTH, 0.0),
        }
    }
}

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AssetsLoaded>()
            .add_systems(Startup, load_assets)
            .add_systems(PreUpdate, monitor_load.run_if(in_state(GameState::Init)));
    }
}
