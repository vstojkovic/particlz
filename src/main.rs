use bevy::app::App;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin, WindowResolution};
use bevy::DefaultPlugins;
use bevy_egui::EguiPlugin;
use engine::gui::{WINDOW_HEIGHT, WINDOW_WIDTH};

mod engine;
mod model;

use self::engine::animation::{
    Animation, AnimationFinished, AnimationPlugin, AnimationSet, StartAnimation,
};
use self::engine::beam::{BeamPlugin, BeamSet, MoveBeams, ResetBeams};
use self::engine::focus::{get_focus, Focus, FocusPlugin, UpdateFocusEvent};
use self::engine::gui::{GuiPlugin, PlayLevel, UndoMoves};
use self::engine::input::{InputPlugin, InputSet, MoveManipulatorEvent, SelectManipulatorEvent};
use self::engine::level::{update_piece_coords, Campaign, Level};
use self::engine::particle::{collect_particles, ParticleCollected};
use self::engine::{
    AssetsLoaded, AssetsPlugin, GameAssets, GameState, GameplaySet, InLevel, InLevelSet, MainCamera,
};
use self::model::{Board, CampaignData, LevelCampaign, Piece, Tile, TileKind};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Particlz".into(),
                resolution: WindowResolution::new(WINDOW_WIDTH as _, WINDOW_HEIGHT as _),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .init_state::<GameState>()
        .add_computed_state::<InLevel>()
        .add_plugins(EguiPlugin)
        .add_plugins(GuiPlugin)
        .add_plugins(AssetsPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(AnimationPlugin)
        .add_plugins(FocusPlugin)
        .add_plugins(BeamPlugin)
        .add_event::<ParticleCollected>()
        .configure_sets(
            FixedPreUpdate,
            GameplaySet.run_if(in_state(GameState::Playing)),
        )
        .configure_sets(
            FixedUpdate,
            GameplaySet.run_if(in_state(GameState::Playing)),
        )
        .configure_sets(
            FixedPostUpdate,
            GameplaySet.run_if(in_state(GameState::Playing)),
        )
        .configure_sets(FixedPreUpdate, InLevelSet.run_if(in_state(InLevel)))
        .configure_sets(FixedUpdate, InLevelSet.run_if(in_state(InLevel)))
        .configure_sets(FixedPostUpdate, InLevelSet.run_if(in_state(InLevel)))
        .add_systems(Update, finish_init.run_if(in_state(GameState::Init)))
        .add_systems(
            Update,
            start_level.run_if(not(in_state(GameState::Playing))),
        )
        .add_systems(OnEnter(GameState::Playing), setup_board)
        .add_systems(
            FixedPreUpdate,
            undo_moves.in_set(InLevelSet).before(InputSet),
        )
        .add_systems(
            FixedUpdate,
            (
                get_focus.pipe(select_manipulator).in_set(GameplaySet),
                get_focus
                    .pipe(move_manipulator)
                    .before(AnimationSet)
                    .before(BeamSet)
                    .in_set(GameplaySet),
                get_focus
                    .pipe(finish_animation)
                    .after(AnimationSet)
                    .in_set(GameplaySet),
                update_piece_coords
                    .after(finish_animation)
                    .in_set(GameplaySet),
            ),
        )
        .add_systems(
            FixedPostUpdate,
            (
                check_game_over.in_set(GameplaySet),
                collect_particles.in_set(GameplaySet),
            ),
        )
        .add_systems(OnExit(InLevel), remove_level)
        .run();
}

fn finish_init(
    mut ev_loaded: EventReader<AssetsLoaded>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut ev_play: EventWriter<PlayLevel>,
) {
    if ev_loaded.read().last().is_none() {
        return;
    }

    let classic_campaign = LevelCampaign::from_static(CLASSIC_CAMPAIGN_DATA);
    commands.insert_resource(Campaign(classic_campaign));

    let mut camera = Camera2dBundle::default();
    camera.projection.viewport_origin = Vec2::new(0.0, 1.0);
    commands.spawn((camera, MainCamera));

    if let Some(code) = std::env::args().nth(1) {
        match Board::from_pbc1(&code) {
            Ok(board) => {
                ev_play.send(PlayLevel(board, Default::default()));
                return;
            }
            Err(err) => bevy::log::error!("Invalid custom level code: {}", err),
        }
    }
    next_state.set(GameState::MainMenu);
}

fn start_level(
    mut ev_play: EventReader<PlayLevel>,
    current_level: Option<ResMut<Level>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(PlayLevel(board, metadata)) = ev_play.read().last() else {
        return;
    };
    let new_level = Level::new(board.clone(), metadata.clone());
    if let Some(mut level) = current_level {
        level.despawn(&mut commands);
        *level = new_level;
    } else {
        commands.insert_resource(new_level);
    }
    next_state.set(GameState::Playing);
}

fn setup_board(
    mut commands: Commands,
    mut level: ResMut<Level>,
    assets: Res<GameAssets>,
    mut ev_retarget: EventWriter<ResetBeams>,
) {
    level.spawn(&mut commands, &assets);
    ev_retarget.send(ResetBeams);
}

fn select_manipulator(
    focus: In<Focus>,
    mut ev_select_manipulator: EventReader<SelectManipulatorEvent>,
    mut ev_update_focus: EventWriter<UpdateFocusEvent>,
    level: Res<Level>,
) {
    let Some(event) = ev_select_manipulator.read().last() else {
        return;
    };
    let coords = focus.coords(false);
    let coords = match event {
        SelectManipulatorEvent::Previous => level.present.prev_manipulator(coords),
        SelectManipulatorEvent::Next => level.present.next_manipulator(coords),
        SelectManipulatorEvent::AtCoords(coords) => Some(*coords),
        SelectManipulatorEvent::Deselect => None,
    };
    let new_focus = coords
        .map(|coords| Focus::Selected(coords, level.present.compute_allowed_moves(coords)))
        .unwrap_or(Focus::None);
    ev_update_focus.send(UpdateFocusEvent(new_focus));
}

fn move_manipulator(
    focus: In<Focus>,
    mut ev_move_manipulator: EventReader<MoveManipulatorEvent>,
    mut ev_start_animation: EventWriter<StartAnimation>,
    mut ev_move_beams: EventWriter<MoveBeams>,
    mut ev_update_focus: EventWriter<UpdateFocusEvent>,
    mut level: ResMut<Level>,
) {
    let Some(event) = ev_move_manipulator.read().last() else {
        return;
    };
    let Some(leader) = focus.coords(false) else {
        warn!("Received {:?} without a selected manipulator", event);
        return;
    };

    let direction = event.0;

    let move_set = level.present.compute_move_set(leader, direction);
    level.prepare_move(&move_set, direction);

    ev_start_animation.send(StartAnimation(
        Animation::Movement(direction),
        move_set.clone(),
    ));
    ev_move_beams.send(MoveBeams {
        move_set,
        direction,
    });
    ev_update_focus.send(UpdateFocusEvent(Focus::Busy(Some(leader))));
}

fn finish_animation(
    focus: In<Focus>,
    mut ev_animation_finished: EventReader<AnimationFinished>,
    mut ev_start_animation: EventWriter<StartAnimation>,
    mut ev_retarget: EventWriter<ResetBeams>,
    mut ev_update_focus: EventWriter<UpdateFocusEvent>,
    mut ev_collected: EventWriter<ParticleCollected>,
    mut level: ResMut<Level>,
    mut commands: Commands,
) {
    let Some(AnimationFinished(animation, pieces)) = ev_animation_finished.read().last() else {
        return;
    };

    level.update_present();

    match animation {
        Animation::Movement(direction) => {
            pieces.for_each(*direction, |from_coords| {
                let to_coords = level.present.neighbor(from_coords, *direction).unwrap();
                level.move_piece(from_coords, to_coords);
                if let Some(Piece::Particle(_)) = level.present.pieces.get(to_coords) {
                    if let Some(Tile {
                        kind: TileKind::Collector,
                        ..
                    }) = level.present.tiles.get(to_coords)
                    {
                        ev_collected.send(ParticleCollected(
                            level.pieces.get(to_coords).copied().unwrap(),
                        ));
                    }
                }
            });

            let focus_coords = level
                .present
                .neighbor(focus.coords(true).unwrap(), *direction)
                .unwrap();

            let unsupported = level.present.unsupported_pieces();
            if unsupported.is_empty() {
                ev_update_focus.send(UpdateFocusEvent(Focus::Selected(
                    focus_coords,
                    level.present.compute_allowed_moves(focus_coords),
                )));
            } else {
                ev_update_focus.send(UpdateFocusEvent(Focus::Busy(Some(focus_coords))));
                ev_start_animation.send(StartAnimation(Animation::FadeOut, unsupported));
            }
        }
        Animation::FadeOut => {
            let focus_coords = match focus.coords(true) {
                Some(coords) if !pieces.contains(coords) => Some(coords),
                _ => None,
            };
            level.remove_pieces(pieces, &mut commands);
            let new_focus = match focus_coords {
                Some(coords) => {
                    Focus::Selected(coords, level.present.compute_allowed_moves(coords))
                }
                None => Focus::None,
            };
            ev_update_focus.send(UpdateFocusEvent(new_focus));
        }
    }
    ev_retarget.send(ResetBeams);
}

fn check_game_over(level: Res<Level>, mut next_state: ResMut<NextState<GameState>>) {
    if level.progress.outcome.is_some() {
        next_state.set(GameState::GameOver);
    }
}

fn undo_moves(
    mut ev_undo: EventReader<UndoMoves>,
    mut level: ResMut<Level>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_retarget: EventWriter<ResetBeams>,
) {
    if ev_undo.is_empty() {
        return;
    }
    for undo in ev_undo.read() {
        match undo {
            UndoMoves::Last => level.undo(),
            UndoMoves::All => level.reset(),
        }
    }
    level.spawn(&mut commands, &assets);
    ev_retarget.send(ResetBeams);
}

fn remove_level(mut level: ResMut<Level>, mut commands: Commands) {
    level.despawn(&mut commands);
    commands.remove_resource::<Level>();
}

const CLASSIC_CAMPAIGN_DATA: CampaignData = &[
    ("eASY", &[
        ("Tutorial", ":PBC1:AapHrUCxAhxBEASxUBAEBQoMEARhjihQoEBQoECBI5BCEARBACAFAEFQokCBhYIgCAoER6AAsVAQBEHRIAiwUBAEABBisUMQFC5QugBBYKEgKBKELAbB/wE="),
        ("Experiment", ":PBC1:AaocQRMEUaBAgQIpgGFYngmCFACwLIIgBQAsiyBIAQDLIghSAMCyCIZJAQDLIggeoUEGAFgWQZACwINhgyAFoG0es0Hwfw=="),
        ("Teamwork", ":PBC1:AXpciRIlCIIgDsABSAEAAAyQAgAAwKMUBEEQBAAWCoIgCAIACwVBEAQBgIWCIAiCgQD8Hw=="),
        ("Roundabout", ":PBC1:AaocUYIgCIIgiBQAAABSGAAAgMFSIAAAQAo4RAAApAAKGAbAowSUAgAgBQAAgBQAoBSGwELBQAAA4P8="),
        ("Relay", ":PBC1:AZrcYShQoECBAgUKFEgBAAAgBQAAgBQAAACWIhiCIRiCGSDFEAzBEAyBFAAAAFIAAABYKAiCIAiCgfB/"),
        ("Occlusion", ":PBC1:AVoHrMABKHEAChcoUKDAUggxQNEgCIKlgiAIiwZBMMxSCDFA0SAIggcoGCAcoGgQBMH/AQ=="),
        ("Transfer", ":PBC1:AZlA4QIFChRgAWCKDhbwgIJszFjChCi+UBEWAVA8WGgoQ4MwUBzTYKGARQAUDRbicwgApmgGKH5QirBgAMWDICjCAh8="),
    ]),
    ("MedIUM", &[
        ("Mmmm, pi!", ":PBC1:AaocQRAEQRAEkQIAAEBqsCAPgjwYDCkgAAIAKRUCIIAGKWAAYAAAKWAQYBAAKSAFUgApAAAApAAAAPB/"),
        ("Milky Way", ":PBC1:AaqHrEQBgiAIgjgCKSAAAOQpAAEABCkACIAAKSAYZiAEQAoBBhsqAJAKgAAAsBAABACwFwAgAPAAAQAQpIP8Hw=="),
        ("Maze", ":PBC1:AartChQoUKBAgQIFeixUpEiRIkGRIkWCBYsUPeJBkSJFihRZKAiKBEWKFClSdMGuRYoULVKkSBAsGBQJijQpUiQoulCRIkWKFi8SFQkWLFJkgCA4JEWKxMkiRZgiRZgiRZiFgoGCIAiCIPg/"),
        ("Checkers", ":PBC1:AXdHjShAFCAOQCpAjsHwCCFAgCCVIkCAhTAIYgSpAAMhwEIIEGCYfw=="),
        ("Crowded", ":PBC1:AaocQTRo0KAF0eMBpBZLEmRZliUbJQAyAMlGWZhlGYBkowxIgiRJko0yIMmyLMNGGZAAyPApZUCSJFmGjTbJsiwLM+ADSpIkSZJtsk3+Dw=="),
        ("Juggle", ":PBC1:Aaq3rUCBAgUKFChQoEQqAAAgQCoAACBAKmAYhmGYAKkAgwDAMAM8QkMBGAQIkAoAAAiQChiGYRgmQCoAACDAXkEQBEEQBCv9Hw=="),
        ("I Kill You", ":PBC1:AaocQRAEQRDH4CikAADAYR1mIRYAAAYLsQAAACkAAACkUKTOASxShAK2KxIMUigIAo5AHKIgKBQMkFMAolVQaIiAAwAEQfTiAAAB"),
    ]),
    ("HArd", &[
        ("Lock", ":PBC1:AXqcBRYQhAUEQRApQAJIAGwFQABAM0wqz3PkOYAUgAAIgFQABAgCIDXkQEMOO9BwwwD/Bw=="),
        ("Delicate", ":PBC1:AZnFihUoUKBwgQLFFhq0AM/UKTxgsFhQiAWKFiqwEM8MgQGYPkUXZAEAKLpQWwyCIYDiCxUpyALFCwaLDRnUBYoOV2ChQgWKFC9SICj0Pw=="),
        ("Void", ":PBC1:AaqHjaAJgiAIwoMUwAIAkALAAgCTAgAgYJACAIABUgAOQDkASIEBQQBAigHABgCSAQCwALoEAAAL0f8B"),
        ("Nautilus", ":PBC1:AapnrQBBEARBEAYsJAAABKMhhbECAIIAKQQCBKMBSAEAAgApAIAgAFJAIEAwDpACRgoACIJUIAAABOOkRgoAAMD/AQ=="),
        ("Trapped", ":PBC1:AanlCIIoQBBEgYUABAAGepQAQQggWAgUOyxoKlgIFBuApYKFcIDYAAeUChYCxQZgqWAhUGwAlgoeJhAIBcAwCwEIAIT/Aw=="),
        ("Quadruped", ":PBC1:AaqHjiAIgiAIgkgBAIABkQIQAABSAADQBJEaEgDADoAUgOEQHlAUXQgAAARIASGAAOxSAAAwTPAABQACAPg/"),
        ("Rails", ":PBC1:AaoccRgIgiAIgkgBAAAgBQAAMEwKAAAAKRxwpg9ThgUeJTBHFAGKsEihOAZBgDZsCswRRYCARwoHHDFCHkiBYRiGwUHB/wE="),
    ]),
];
