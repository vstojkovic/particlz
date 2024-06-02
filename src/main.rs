use bevy::app::{App, Startup};
use bevy::asset::AssetServer;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
use bevy::DefaultPlugins;
use bevy_tweening::Animator;

mod engine;
mod model;

use self::engine::animation::{
    set_animation, Animation, AnimationFinished, AnimationPlugin, AnimationSet,
};
use self::engine::beam::{retarget_beams, RetargetBeams};
use self::engine::board::BoardResource;
use self::engine::focus::{get_focus, set_focus, Focus, FocusArrow};
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
        .add_event::<RetargetBeams>()
        .insert_resource(BoardResource::new(board))
        .add_systems(Startup, (load_assets, setup_board).chain())
        .add_systems(
            FixedUpdate,
            (
                select_manipulator,
                move_manipulator,
                finish_animation.after(AnimationSet),
                retarget_beams.after(finish_animation),
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
    mut events: EventReader<SelectManipulatorEvent>,
    board: Res<BoardResource>,
    mut focus: Query<(&mut Focus, &mut Transform, &Children)>,
    mut arrows: Query<(&FocusArrow, &mut Visibility)>,
) {
    for event in events.read() {
        let coords = get_focus(&focus.transmute_lens().query()).coords();
        let coords = match event {
            SelectManipulatorEvent::Previous => board.model.prev_manipulator(coords),
            SelectManipulatorEvent::Next => board.model.next_manipulator(coords),
            SelectManipulatorEvent::AtCoords(coords) => {
                if !board.model.compute_allowed_moves(*coords).is_empty() {
                    Some(*coords)
                } else {
                    None
                }
            }
            SelectManipulatorEvent::Deselect => None,
        };
        let new_focus = coords
            .map(|coords| Focus::Selected(coords, board.model.compute_allowed_moves(coords)))
            .unwrap_or(Focus::None);
        set_focus(new_focus, &mut focus, &mut arrows);
    }
}

fn move_manipulator(
    mut events: EventReader<MoveManipulatorEvent>,
    board: Res<BoardResource>,
    mut anchor: Query<(&mut Animation, &Children)>,
    mut animator: Query<&mut Animator<Transform>>,
    mut focus: Query<(&mut Focus, &mut Transform, &Children)>,
    mut arrows: Query<(&FocusArrow, &mut Visibility)>,
) {
    if events.is_empty() {
        return;
    }
    let Some(coords) = get_focus(&focus.transmute_lens().query()).coords() else {
        for event in events.read() {
            warn!("Received {:?} without a selected manipulator", event);
        }
        return;
    };
    let event = events.read().last().unwrap();
    let anchor_id = board.get_piece(coords).unwrap();
    set_animation(
        anchor_id,
        Animation::Movement(event.0),
        &mut anchor,
        &mut animator,
    );
    set_focus(Focus::Busy, &mut focus, &mut arrows);
}

fn finish_animation(
    mut ev_animation: EventReader<AnimationFinished>,
    mut ev_retarget: EventWriter<RetargetBeams>,
    mut anchor: Query<(&mut BoardCoordsHolder, &mut Transform), Without<Focus>>,
    mut focus: Query<(&mut Focus, &mut Transform, &Children)>,
    mut arrows: Query<(&FocusArrow, &mut Visibility)>,
    mut board: ResMut<BoardResource>,
) {
    if ev_animation.is_empty() {
        return;
    }
    for event in ev_animation.read() {
        match event.animation {
            Animation::Idle => unreachable!(),
            Animation::Movement(direction) => {
                let (coords, _) = anchor.get_mut(event.anchor).unwrap();
                let from_coords = coords.0;
                let to_coords = board.model.neighbor(from_coords, direction).unwrap();
                board.move_piece(from_coords, to_coords, &mut anchor.transmute_lens().query());
                set_focus(
                    Focus::Selected(to_coords, board.model.compute_allowed_moves(to_coords)),
                    &mut focus,
                    &mut arrows,
                );
            }
        }
    }
    ev_retarget.send(RetargetBeams);
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
