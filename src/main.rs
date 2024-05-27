use bevy::app::{App, Startup};
use bevy::asset::AssetServer;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
use bevy::DefaultPlugins;
use bevy_tweening::Animator;
use engine::focus::{get_focus, set_focus};

mod engine;
mod model;

use self::engine::animation::{
    set_animation, Animation, AnimationFinished, AnimationPlugin, AnimationSet,
};
use self::engine::board::BoardResource;
use self::engine::focus::{Focus, FocusArrow};
use self::engine::input::{InputPlugin, MoveManipulatorEvent, SelectManipulatorEvent};
use self::engine::{Assets, BoardCoords};
use self::model::{Board, Border, Emitters, Manipulator, Particle, Piece, Tile, TileKind, Tint};

fn main() {
    let board = make_test_board();
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
        .insert_resource(BoardResource::new(board))
        .add_systems(Startup, (load_assets, setup_board).chain())
        .add_systems(
            FixedUpdate,
            (
                select_manipulator,
                move_manipulator,
                finish_animation.after(AnimationSet),
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
            SelectManipulatorEvent::Previous => prev_manipulator(&board.model, coords),
            SelectManipulatorEvent::Next => next_manipulator(&board.model, coords),
            SelectManipulatorEvent::AtCoords(coords) => {
                if !board
                    .model
                    .compute_allowed_moves(coords.row, coords.col)
                    .is_empty()
                {
                    Some(*coords)
                } else {
                    None
                }
            }
            SelectManipulatorEvent::Deselect => None,
        };
        let new_focus = coords
            .map(|coords| {
                Focus::Selected(
                    coords,
                    board.model.compute_allowed_moves(coords.row, coords.col),
                )
            })
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
    mut events: EventReader<AnimationFinished>,
    mut anchor: Query<(&mut BoardCoords, &mut Transform), Without<Focus>>,
    mut focus: Query<(&mut Focus, &mut Transform, &Children)>,
    mut arrows: Query<(&FocusArrow, &mut Visibility)>,
    mut board: ResMut<BoardResource>,
) {
    if events.is_empty() {
        return;
    }
    for event in events.read() {
        match event.animation {
            Animation::Idle => unreachable!(),
            Animation::Movement(direction) => {
                let (coords, _) = anchor.get_mut(event.anchor).unwrap();
                let from_coords = *coords;
                let to_coords = coords.move_to(direction);
                board.move_piece(from_coords, to_coords, &mut anchor.transmute_lens().query());
                set_focus(
                    Focus::Selected(
                        to_coords,
                        board
                            .model
                            .compute_allowed_moves(to_coords.row, to_coords.col),
                    ),
                    &mut focus,
                    &mut arrows,
                );
            }
        }
    }
}

fn prev_manipulator(board: &Board, coords: Option<BoardCoords>) -> Option<BoardCoords> {
    // NOTE: An active board should never have 0 manipulators
    let mut coords = coords.unwrap_or_default();
    let mut remaining = board.rows * board.cols;
    while remaining > 0 {
        if coords.col > 0 {
            coords.col -= 1;
        } else {
            coords.col = board.cols - 1;
            if coords.row > 0 {
                coords.row -= 1;
            } else {
                coords.row = board.rows - 1;
            }
        }
        if let Some(Piece::Manipulator(_)) = board.get_piece(coords.row, coords.col) {
            if !board
                .compute_allowed_moves(coords.row, coords.col)
                .is_empty()
            {
                return Some(coords);
            }
        }
        remaining -= 1;
    }
    None
}

fn next_manipulator(board: &Board, coords: Option<BoardCoords>) -> Option<BoardCoords> {
    // NOTE: An active board should never have 0 manipulators
    let max_row = board.rows - 1;
    let max_col = board.cols - 1;
    let mut coords = coords.unwrap_or_else(|| BoardCoords::new(max_row, max_col));
    let mut remaining = board.rows * board.cols;
    while remaining > 0 {
        if coords.col < max_row {
            coords.col += 1;
        } else {
            coords.col = 0;
            if coords.row < max_row {
                coords.row += 1;
            } else {
                coords.row = 0;
            }
        }
        if let Some(Piece::Manipulator(_)) = board.get_piece(coords.row, coords.col) {
            if !board
                .compute_allowed_moves(coords.row, coords.col)
                .is_empty()
            {
                return Some(coords);
            }
        }
        remaining -= 1;
    }
    None
}

fn make_test_board() -> Board {
    let mut board = Board::new(5, 5);
    board.set_tile(0, 0, Tile::new(TileKind::Platform, Tint::White));
    board.set_tile(0, 1, Tile::new(TileKind::Platform, Tint::Green));
    board.set_tile(0, 2, Tile::new(TileKind::Platform, Tint::Yellow));
    board.set_tile(0, 3, Tile::new(TileKind::Platform, Tint::Red));
    for row in 1..=3 {
        for col in 0..=4 {
            board.set_tile(row, col, Tile::new(TileKind::Platform, Tint::White));
        }
    }
    board.set_tile(4, 4, Tile::new(TileKind::Collector, Tint::White));
    board.set_tile(4, 3, Tile::new(TileKind::Collector, Tint::Green));
    board.set_tile(4, 2, Tile::new(TileKind::Collector, Tint::Yellow));
    board.set_tile(4, 1, Tile::new(TileKind::Collector, Tint::Red));
    board.set_horz_border(0, 0, Border::Wall);
    board.set_horz_border(1, 0, Border::Wall);
    board.set_horz_border(4, 4, Border::Window);
    board.set_horz_border(5, 4, Border::Window);
    board.set_vert_border(0, 0, Border::Window);
    board.set_vert_border(4, 5, Border::Wall);
    board.set_piece(1, 1, Particle::new(Tint::Green));
    board.set_piece(1, 2, Particle::new(Tint::Yellow));
    board.set_piece(1, 3, Particle::new(Tint::Red));
    board.set_piece(2, 0, Manipulator::new(Emitters::Left));
    board.set_piece(2, 1, Manipulator::new(Emitters::Up));
    board.set_piece(2, 2, Manipulator::new(Emitters::Right));
    board.set_piece(2, 3, Manipulator::new(Emitters::Down));
    board.set_piece(2, 4, Manipulator::new(Emitters::LeftRight));
    board.set_piece(3, 0, Manipulator::new(Emitters::LeftUp));
    board.set_piece(3, 1, Manipulator::new(Emitters::LeftDown));
    board.set_piece(3, 2, Manipulator::new(Emitters::RightUp));
    board.set_piece(3, 3, Manipulator::new(Emitters::RightDown));
    board.set_piece(3, 4, Manipulator::new(Emitters::UpDown));
    board
}
