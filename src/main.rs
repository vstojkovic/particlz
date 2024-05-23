use bevy::app::{App, Startup};
use bevy::asset::AssetServer;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window, WindowPlugin};
use bevy::DefaultPlugins;
use enumset::EnumSet;

mod engine;
mod model;

use self::engine::board::BoardResource;
use self::engine::focus::{Focus, FocusArrow};
use self::engine::manipulator::is_offset_inside_manipulator;
use self::engine::{Assets, BoardCoords};
use self::model::{Board, Border, Emitters, Manipulator, Particle, Piece, Tile, TileKind, Tint};

#[derive(Event)]
enum SelectManipulatorEvent {
    Previous,
    Next,
    AtCoords(BoardCoords),
    Deselect,
}

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
        .insert_resource(BoardResource::new(board))
        .add_event::<SelectManipulatorEvent>()
        .add_systems(Startup, (load_assets, setup_board).chain())
        .add_systems(
            FixedPreUpdate,
            (process_keyboard_input, process_mouse_input),
        )
        .add_systems(FixedUpdate, select_manipulator)
        .run();
}

fn load_assets(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(Assets::load(&server));
}

fn setup_board(mut commands: Commands, mut board: ResMut<BoardResource>, assets: Res<Assets>) {
    commands.spawn(Camera2dBundle::default());
    board.spawn(&mut commands, &assets);
}

fn process_keyboard_input(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut keyboard_input: Local<ButtonInput<KeyCode>>,
    mut ev_select_manipulator: EventWriter<SelectManipulatorEvent>,
) {
    keyboard_input.clear();
    for event in keyboard_events.read() {
        match event.state {
            ButtonState::Pressed => keyboard_input.press(event.key_code),
            ButtonState::Released => keyboard_input.release(event.key_code),
        }
    }

    if keyboard_input.any_just_pressed([KeyCode::KeyQ, KeyCode::PageUp]) {
        ev_select_manipulator.send(SelectManipulatorEvent::Previous);
    } else if keyboard_input.any_just_pressed([KeyCode::KeyE, KeyCode::PageDown]) {
        ev_select_manipulator.send(SelectManipulatorEvent::Next);
    }
}

fn process_mouse_input(
    mut mouse_events: EventReader<MouseButtonInput>,
    mut mouse_input: Local<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    board: Res<BoardResource>,
    q_xform: Query<&Transform>,
    mut ev_select_manipulator: EventWriter<SelectManipulatorEvent>,
) {
    mouse_input.clear();
    for event in mouse_events.read() {
        match event.state {
            ButtonState::Pressed => mouse_input.press(event.button),
            ButtonState::Released => mouse_input.release(event.button),
        }
    }

    if mouse_input.just_pressed(MouseButton::Left) {
        let (camera, xform) = camera.single();
        let window = window.single();
        let coords_and_offset = window
            .cursor_position()
            .and_then(|pos| camera.viewport_to_world_2d(xform, pos))
            .and_then(|pos| board.coords_at_pos(pos, &q_xform));
        if let Some((coords, offset)) = coords_and_offset {
            if let Some(Piece::Manipulator(_)) = board.board.get_piece(coords.row, coords.col) {
                if is_offset_inside_manipulator(offset) {
                    ev_select_manipulator.send(SelectManipulatorEvent::AtCoords(coords));
                }
            } else {
                ev_select_manipulator.send(SelectManipulatorEvent::Deselect);
            }
        }
    }
}

fn select_manipulator(
    mut events: EventReader<SelectManipulatorEvent>,
    board: Res<BoardResource>,
    mut focus: Query<(&mut Focus, &mut Transform, &Children)>,
    mut arrows: Query<(&FocusArrow, &mut Visibility)>,
) {
    for event in events.read() {
        let coords = Focus::get_coords(&focus.transmute_lens().query());
        let coords = match event {
            SelectManipulatorEvent::Previous => Some(prev_manipulator(&board.board, coords)),
            SelectManipulatorEvent::Next => Some(next_manipulator(&board.board, coords)),
            SelectManipulatorEvent::AtCoords(coords) => Some(*coords),
            SelectManipulatorEvent::Deselect => None,
        };
        Focus::update(coords, EnumSet::all(), &mut focus, &mut arrows);
    }
}

fn prev_manipulator(board: &Board, coords: Option<BoardCoords>) -> BoardCoords {
    // NOTE: An active board should never have 0 manipulators
    let mut coords = coords.unwrap_or_default();
    loop {
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
            return coords;
        }
    }
}

fn next_manipulator(board: &Board, coords: Option<BoardCoords>) -> BoardCoords {
    // NOTE: An active board should never have 0 manipulators
    let max_row = board.rows - 1;
    let max_col = board.cols - 1;
    let mut coords = coords.unwrap_or_else(|| BoardCoords::new(max_row, max_col));
    loop {
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
            return coords;
        }
    }
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
