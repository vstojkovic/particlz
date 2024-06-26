use bevy::app::{App, Startup};
use bevy::asset::AssetServer;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
use bevy::DefaultPlugins;
use engine::beam::BeamSet;

mod engine;
mod model;

use self::engine::animation::{
    Animation, AnimationFinished, AnimationPlugin, AnimationSet, StartAnimation,
};
use self::engine::beam::{BeamPlugin, MoveBeams, ResetBeams};
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

    let anchor_id = board.get_piece(coords).unwrap();
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
    let mut board = Board::new(5, 5);
    board.set_tile((0, 0).into(), Tile::new(TileKind::Platform, Tint::White));
    board.set_tile((0, 1).into(), Tile::new(TileKind::Platform, Tint::Green));
    board.set_tile((0, 2).into(), Tile::new(TileKind::Platform, Tint::Yellow));
    board.set_tile((0, 3).into(), Tile::new(TileKind::Platform, Tint::Red));
    for row in 1..=3 {
        for col in 0..=4 {
            board.set_tile(
                (row, col).into(),
                Tile::new(TileKind::Platform, Tint::White),
            );
        }
    }
    board.set_tile((4, 4).into(), Tile::new(TileKind::Collector, Tint::White));
    board.set_tile((4, 3).into(), Tile::new(TileKind::Collector, Tint::Green));
    board.set_tile((4, 2).into(), Tile::new(TileKind::Collector, Tint::Yellow));
    board.set_tile((4, 1).into(), Tile::new(TileKind::Collector, Tint::Red));
    board.set_horz_border((0, 0).into(), Border::Wall);
    board.set_horz_border((1, 0).into(), Border::Wall);
    board.set_horz_border((4, 4).into(), Border::Window);
    board.set_horz_border((5, 4).into(), Border::Window);
    board.set_vert_border((0, 0).into(), Border::Window);
    board.set_vert_border((4, 5).into(), Border::Wall);
    board.set_piece((1, 1).into(), Particle::new(Tint::Green));
    board.set_piece((1, 2).into(), Particle::new(Tint::Yellow));
    board.set_piece((1, 3).into(), Particle::new(Tint::Red));
    board.set_piece((2, 0).into(), Manipulator::new(Emitters::Left));
    board.set_piece((2, 1).into(), Manipulator::new(Emitters::Up));
    board.set_piece((2, 2).into(), Manipulator::new(Emitters::Right));
    board.set_piece((2, 3).into(), Manipulator::new(Emitters::Down));
    board.set_piece((2, 4).into(), Manipulator::new(Emitters::LeftRight));
    board.set_piece((3, 0).into(), Manipulator::new(Emitters::LeftUp));
    board.set_piece((3, 1).into(), Manipulator::new(Emitters::LeftDown));
    board.set_piece((3, 2).into(), Manipulator::new(Emitters::RightUp));
    board.set_piece((3, 3).into(), Manipulator::new(Emitters::RightDown));
    board.set_piece((3, 4).into(), Manipulator::new(Emitters::UpDown));
    board
}
