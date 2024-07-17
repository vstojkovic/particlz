use bevy::app::{App, AppExit};
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
use bevy::DefaultPlugins;
use bevy_egui::EguiPlugin;
use engine::gui::GuiPlugin;
use engine::AssetsPlugin;

mod engine;
mod model;

use self::engine::animation::{
    Animation, AnimationFinished, AnimationPlugin, AnimationSet, StartAnimation,
};
use self::engine::beam::{BeamPlugin, BeamSet, MoveBeams, ResetBeams};
use self::engine::focus::{get_focus, Focus, FocusPlugin, UpdateFocusEvent};
use self::engine::input::{InputPlugin, MoveManipulatorEvent, SelectManipulatorEvent};
use self::engine::level::Level;
use self::engine::{AssetsLoaded, BoardCoordsHolder, GameAssets, GameState, GameplaySet};
use self::model::{
    Board, Border, Emitters, LevelOutcome, Manipulator, Particle, Piece, Tile, TileKind, Tint,
};

fn main() {
    let board = if let Some(code) = std::env::args().nth(1) {
        Board::from_pbc1(&code).unwrap()
    } else {
        make_test_board()
    };
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Particlz".into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .init_state::<GameState>()
        .add_plugins(EguiPlugin)
        .add_plugins(GuiPlugin)
        .add_plugins(AssetsPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(AnimationPlugin)
        .add_plugins(FocusPlugin)
        .add_plugins(BeamPlugin)
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
        .insert_resource(Level::new(board))
        .add_systems(Update, finish_init.run_if(in_state(GameState::Init)))
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
            ),
        )
        .add_systems(FixedPostUpdate, check_game_over.in_set(GameplaySet))
        .add_systems(OnEnter(GameState::GameOver), game_over)
        .run();
}

fn finish_init(
    mut ev_loaded: EventReader<AssetsLoaded>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if ev_loaded.read().last().is_some() {
        next_state.set(GameState::MainMenu);
    }
}

fn setup_board(mut commands: Commands, mut level: ResMut<Level>, assets: Res<GameAssets>) {
    commands.spawn(Camera2dBundle::default());
    level.spawn(&mut commands, &assets);
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
    mut level: ResMut<Level>,
    mut commands: Commands,
    mut q_piece: Query<(&mut BoardCoordsHolder, &mut Transform), Without<Focus>>,
) {
    let Some(AnimationFinished(animation, pieces)) = ev_animation_finished.read().last() else {
        return;
    };

    level.update_present();

    match animation {
        Animation::Movement(direction) => {
            level.move_pieces(pieces, *direction, &mut q_piece.transmute_lens().query());

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

fn game_over(level: Res<Level>, mut exit: EventWriter<AppExit>) {
    let outcome_text = match level.progress.outcome.unwrap() {
        LevelOutcome::NoManipulatorsLeft => "you have no manipulators left",
        LevelOutcome::ParticleLost => "you lost a particle",
        LevelOutcome::Victory => "you beat the level",
    };
    bevy::log::info!("Game over: {}", outcome_text);
    exit.send(AppExit::Success);
}

fn make_test_board() -> Board {
    fn add_tile(board: &mut Board, row: usize, col: usize, kind: TileKind, tint: Tint) {
        board.tiles.set((row, col).into(), Tile::new(kind, tint));
    }

    fn add_horz_border(board: &mut Board, row: usize, col: usize, border: Border) {
        board.horz_borders.set((row, col).into(), border);
    }

    fn add_vert_border(board: &mut Board, row: usize, col: usize, border: Border) {
        board.vert_borders.set((row, col).into(), border);
    }

    fn add_piece<P: Into<Option<Piece>>>(board: &mut Board, row: usize, col: usize, piece: P) {
        board.pieces.set((row, col).into(), piece.into());
    }

    let mut board = Board::new(5, 5);
    add_tile(&mut board, 0, 0, TileKind::Platform, Tint::White);
    add_tile(&mut board, 0, 1, TileKind::Platform, Tint::Green);
    add_tile(&mut board, 0, 2, TileKind::Platform, Tint::Yellow);
    add_tile(&mut board, 0, 3, TileKind::Platform, Tint::Red);
    for row in 1..=3 {
        for col in 0..=4 {
            add_tile(&mut board, row, col, TileKind::Platform, Tint::White);
        }
    }
    add_tile(&mut board, 4, 4, TileKind::Collector, Tint::White);
    add_tile(&mut board, 4, 3, TileKind::Collector, Tint::Green);
    add_tile(&mut board, 4, 2, TileKind::Collector, Tint::Yellow);
    add_tile(&mut board, 4, 1, TileKind::Collector, Tint::Red);
    add_horz_border(&mut board, 0, 0, Border::Wall);
    add_horz_border(&mut board, 1, 0, Border::Wall);
    add_horz_border(&mut board, 4, 4, Border::Window);
    add_horz_border(&mut board, 5, 4, Border::Window);
    add_vert_border(&mut board, 0, 0, Border::Window);
    add_vert_border(&mut board, 4, 5, Border::Wall);
    add_piece(&mut board, 1, 1, Particle::new(Tint::Green));
    add_piece(&mut board, 1, 2, Particle::new(Tint::Yellow));
    add_piece(&mut board, 1, 3, Particle::new(Tint::Red));
    add_piece(&mut board, 2, 0, Manipulator::new(Emitters::Left));
    add_piece(&mut board, 2, 1, Manipulator::new(Emitters::Up));
    add_piece(&mut board, 2, 2, Manipulator::new(Emitters::Right));
    add_piece(&mut board, 2, 3, Manipulator::new(Emitters::Down));
    add_piece(&mut board, 2, 4, Manipulator::new(Emitters::LeftRight));
    add_piece(&mut board, 3, 0, Manipulator::new(Emitters::LeftUp));
    add_piece(&mut board, 3, 1, Manipulator::new(Emitters::LeftDown));
    add_piece(&mut board, 3, 2, Manipulator::new(Emitters::RightUp));
    add_piece(&mut board, 3, 3, Manipulator::new(Emitters::RightDown));
    add_piece(&mut board, 3, 4, Manipulator::new(Emitters::UpDown));
    board.retarget_beams();
    board
}
