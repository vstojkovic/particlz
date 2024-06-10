use bevy::app::{App, Startup};
use bevy::asset::AssetServer;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
use bevy::DefaultPlugins;
use model::Piece;

mod engine;
mod model;

use self::engine::animation::{
    Animation, AnimationFinished, AnimationPlugin, AnimationSet, StartAnimation,
};
use self::engine::beam::{BeamPlugin, BeamSet, MoveBeams, ResetBeams};
use self::engine::board::BoardResource;
use self::engine::focus::{get_focus, Focus, FocusPlugin, UpdateFocusEvent};
use self::engine::input::{InputPlugin, MoveManipulatorEvent, SelectManipulatorEvent};
use self::engine::{Assets, BoardCoordsHolder};
use self::model::{Board, Border, Emitters, Manipulator, Particle, Tile, TileKind, Tint};

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
        .add_plugins(InputPlugin)
        .add_plugins(AnimationPlugin)
        .add_plugins(FocusPlugin)
        .add_plugins(BeamPlugin)
        .insert_resource(BoardResource::new(board))
        .add_systems(Startup, (load_assets, setup_board).chain())
        .add_systems(
            FixedUpdate,
            (
                get_focus.pipe(select_manipulator),
                get_focus
                    .pipe(move_manipulator)
                    .before(AnimationSet)
                    .before(BeamSet),
                finish_move.after(AnimationSet),
            ),
        )
        .run();
}

fn load_assets(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(Assets::load(&server));
}

fn setup_board(mut commands: Commands, mut board: ResMut<BoardResource>, assets: Res<Assets>) {
    commands.spawn(Camera2dBundle::default());
    board.spawn(&mut commands, &assets);
}

fn select_manipulator(
    focus: In<Focus>,
    mut ev_select_manipulator: EventReader<SelectManipulatorEvent>,
    mut ev_update_focus: EventWriter<UpdateFocusEvent>,
    board: Res<BoardResource>,
) {
    let Some(event) = ev_select_manipulator.read().last() else {
        return;
    };
    let coords = focus.coords();
    let coords = match event {
        SelectManipulatorEvent::Previous => board.present.prev_manipulator(coords),
        SelectManipulatorEvent::Next => board.present.next_manipulator(coords),
        SelectManipulatorEvent::AtCoords(coords) => Some(*coords),
        SelectManipulatorEvent::Deselect => None,
    };
    let new_focus = coords
        .map(|coords| Focus::Selected(coords, board.present.compute_allowed_moves(coords)))
        .unwrap_or(Focus::None);
    ev_update_focus.send(UpdateFocusEvent(new_focus));
}

fn move_manipulator(
    focus: In<Focus>,
    mut ev_move_manipulator: EventReader<MoveManipulatorEvent>,
    mut ev_start_animation: EventWriter<StartAnimation>,
    mut ev_move_beams: EventWriter<MoveBeams>,
    mut ev_update_focus: EventWriter<UpdateFocusEvent>,
    mut board: ResMut<BoardResource>,
) {
    let Some(event) = ev_move_manipulator.read().last() else {
        return;
    };
    let Some(coords) = focus.coords() else {
        warn!("Received {:?} without a selected manipulator", event);
        return;
    };

    let direction = event.0;

    let to_coords = board.present.neighbor(coords, direction).unwrap();
    board.future.move_piece(coords, to_coords);
    board.future.retarget_beams();

    let anchor_id = *board.pieces.get(coords).unwrap();
    ev_start_animation.send(StartAnimation {
        anchor: anchor_id,
        animation: Animation::Movement(direction),
    });
    ev_move_beams.send(MoveBeams {
        anchor: anchor_id,
        direction,
    });
    ev_update_focus.send(UpdateFocusEvent(Focus::Busy));
}

fn finish_move(
    mut ev_animation: EventReader<AnimationFinished>,
    mut ev_retarget: EventWriter<ResetBeams>,
    mut ev_update_focus: EventWriter<UpdateFocusEvent>,
    mut board: ResMut<BoardResource>,
    mut anchor: Query<(&mut BoardCoordsHolder, &mut Transform), Without<Focus>>,
) {
    if ev_animation.is_empty() {
        return;
    }

    board.update_present();

    for event in ev_animation.read() {
        match event.animation {
            Animation::Idle => unreachable!(),
            Animation::Movement(direction) => {
                let (coords, _) = anchor.get_mut(event.anchor).unwrap();
                let from_coords = coords.0;
                let to_coords = board.present.neighbor(from_coords, direction).unwrap();
                board.move_piece(from_coords, to_coords, &mut anchor.transmute_lens().query());
                ev_update_focus.send(UpdateFocusEvent(Focus::Selected(
                    to_coords,
                    board.present.compute_allowed_moves(to_coords),
                )));
            }
        }
    }
    ev_retarget.send(ResetBeams);
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
