use bevy::app::App;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
use bevy::DefaultPlugins;
use bevy_egui::EguiPlugin;

mod engine;
mod model;

use self::engine::animation::{
    Animation, AnimationFinished, AnimationPlugin, AnimationSet, StartAnimation,
};
use self::engine::beam::{BeamPlugin, BeamSet, MoveBeams, ResetBeams};
use self::engine::focus::{get_focus, Focus, FocusPlugin, UpdateFocusEvent};
use self::engine::gui::{GuiPlugin, PlayLevel};
use self::engine::input::{InputPlugin, MoveManipulatorEvent, SelectManipulatorEvent};
use self::engine::level::{update_piece_coords, Level};
use self::engine::particle::{collect_particles, ParticleCollected};
use self::engine::{
    AssetsLoaded, AssetsPlugin, GameAssets, GameState, GameplaySet, InLevel, InLevelSet,
};
use self::model::{Board, Piece, Tile, TileKind};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Particlz".into(),
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

    let mut camera = Camera2dBundle::default();
    camera.projection.viewport_origin = Vec2::new(0.0, 1.0);
    commands.spawn(camera);

    if let Some(code) = std::env::args().nth(1) {
        match Board::from_pbc1(&code) {
            Ok(board) => {
                ev_play.send(PlayLevel(board));
                return;
            }
            Err(err) => bevy::log::error!("Invalid custom level code: {}", err),
        }
    }
    next_state.set(GameState::MainMenu);
}

fn start_level(
    mut ev_play: EventReader<PlayLevel>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(level_event) = ev_play.read().last() else {
        return;
    };
    commands.insert_resource(Level::new(level_event.0.clone()));
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
    level.future.move_pieces(&move_set, direction);
    level.future.retarget_beams();

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

fn remove_level(mut level: ResMut<Level>, mut commands: Commands) {
    level.despawn(&mut commands);
    commands.remove_resource::<Level>();
}
