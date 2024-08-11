use std::sync::Arc;

use bevy::prelude::*;
use enum_map::{Enum, EnumMap};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::GameAssets;

pub struct AudioPlugin;

pub struct AudioAssets {
    sfx: EnumMap<PlaySfx, Handle<AudioSource>>,
}

#[derive(Event, Debug, Clone, Copy, Enum, EnumIter)]
pub enum PlaySfx {
    Focus,
    Collect,
    Fade,
    Win,
    Lose,
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
        Self { sfx }
    }
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

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaySfx>().add_systems(PostUpdate, play_sfx);
    }
}
