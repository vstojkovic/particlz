use std::sync::Arc;

use bevy::prelude::*;
use enum_map::{Enum, EnumMap};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::GameAssets;

pub struct AudioPlugin;

pub struct AudioAssets {
    sfx: EnumMap<PlaySfx, Handle<AudioSource>>,
    tunes: EnumMap<PlayTune, Handle<AudioSource>>,
}

#[derive(Event, Debug, Clone, Copy, Enum, EnumIter)]
pub enum PlaySfx {
    Focus,
    Collect,
    Fade,
    Win,
    Lose,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq, Enum, EnumIter)]
pub enum PlayTune {
    Menu,
    Easy,
    Medium,
    Hard,
}

#[derive(Component)]
struct TuneHolder(Option<PlayTune>);

#[derive(Bundle)]
struct TuneHolderBundle {
    holder: TuneHolder,
    settings: PlaybackSettings,
}

impl AudioAssets {
    pub fn load(server: &AssetServer, barrier: &Arc<()>) -> Self {
        let mut sfx = EnumMap::default();
        for effect in PlaySfx::iter() {
            let suffix = match effect {
                PlaySfx::Focus => "focus",
                PlaySfx::Collect => "collect",
                PlaySfx::Fade => "fade",
                PlaySfx::Win => "win",
                PlaySfx::Lose => "lose",
            };
            let path = format!("sfx-{}.ogg", suffix);
            sfx[effect] = server.load_acquire(path, Arc::clone(&barrier));
        }

        let mut tunes = EnumMap::default();
        for tune in PlayTune::iter() {
            let suffix = match tune {
                PlayTune::Menu => "menu",
                PlayTune::Easy => "easy",
                PlayTune::Medium => "medium",
                PlayTune::Hard => "hard",
            };
            let path = format!("tune-{}.ogg", suffix);
            tunes[tune] = server.load_acquire(path, Arc::clone(&barrier));
        }

        Self { sfx, tunes }
    }
}

fn spawn_tune_holder(mut commands: Commands) {
    commands.spawn(TuneHolderBundle {
        holder: TuneHolder(None),
        settings: PlaybackSettings::LOOP,
    });
}

fn play_sfx(mut ev_sfx: EventReader<PlaySfx>, assets: Res<GameAssets>, mut commands: Commands) {
    for &effect in ev_sfx.read() {
        commands.spawn(AudioBundle {
            source: assets.audio.sfx[effect].clone(),
            settings: PlaybackSettings::DESPAWN,
            ..Default::default()
        });
    }
}

fn play_tune(
    mut ev_tune: EventReader<PlayTune>,
    mut q_holder: Query<(Entity, &mut TuneHolder)>,
    assets: Res<GameAssets>,
    mut commands: Commands,
) {
    let Some(&tune) = ev_tune.read().last() else {
        return;
    };

    let (entity, mut holder) = q_holder.single_mut();
    if holder.0 == Some(tune) {
        return;
    }
    holder.0 = Some(tune);
    commands
        .entity(entity)
        .remove::<AudioSink>()
        .remove::<Handle<AudioSource>>()
        .insert(assets.audio.tunes[tune].clone());
}

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaySfx>()
            .add_event::<PlayTune>()
            .add_systems(Startup, spawn_tune_holder)
            .add_systems(PostUpdate, play_sfx)
            .add_systems(PostUpdate, play_tune);
    }
}
