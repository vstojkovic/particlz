use bevy::app::Plugin;
use bevy::ecs::event::{Event, EventReader, EventWriter};
use bevy::ecs::query::With;
use bevy::ecs::schedule::SystemSet;
use bevy::ecs::system::{Local, Query, Res};
use bevy::input::keyboard::{KeyCode, KeyboardInput};
use bevy::input::mouse::{MouseButton, MouseButtonInput};
use bevy::input::{ButtonInput, ButtonState};
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::transform::components::{GlobalTransform, Transform};
use bevy::window::{PrimaryWindow, Window};

use crate::model::{BoardCoords, Direction, Piece};

use super::focus::{focus_direction_for_offset, get_focus, Focus};
use super::level::Level;
use super::manipulator::is_offset_inside_manipulator;

pub struct InputPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputSet;

#[derive(Debug, Event)]
pub enum SelectManipulatorEvent {
    Previous,
    Next,
    AtCoords(BoardCoords),
    Deselect,
}

#[derive(Debug, Event)]
pub struct MoveManipulatorEvent(pub Direction);

fn process_keyboard_input(
    In(focus): In<Focus>,
    mut keyboard_events: EventReader<KeyboardInput>,
    mut keyboard_input: Local<ButtonInput<KeyCode>>,
    mut ev_select_manipulator: EventWriter<SelectManipulatorEvent>,
    mut ev_move_manipulator: EventWriter<MoveManipulatorEvent>,
) {
    keyboard_input.clear();
    for event in keyboard_events.read() {
        match event.state {
            ButtonState::Pressed => keyboard_input.press(event.key_code),
            ButtonState::Released => keyboard_input.release(event.key_code),
        }
    }

    if let Focus::Busy(_) = focus {
        return;
    }

    if keyboard_input.any_just_pressed([KeyCode::KeyQ, KeyCode::PageUp]) {
        ev_select_manipulator.send(SelectManipulatorEvent::Previous);
    } else if keyboard_input.any_just_pressed([KeyCode::KeyE, KeyCode::PageDown]) {
        ev_select_manipulator.send(SelectManipulatorEvent::Next);
    }

    let Focus::Selected(_, directions) = focus else {
        return;
    };

    if keyboard_input.any_just_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        if directions.contains(Direction::Up) {
            ev_move_manipulator.send(MoveManipulatorEvent(Direction::Up));
        }
    } else if keyboard_input.any_just_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        if directions.contains(Direction::Left) {
            ev_move_manipulator.send(MoveManipulatorEvent(Direction::Left));
        }
    } else if keyboard_input.any_just_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        if directions.contains(Direction::Down) {
            ev_move_manipulator.send(MoveManipulatorEvent(Direction::Down));
        }
    } else if keyboard_input.any_just_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        if directions.contains(Direction::Right) {
            ev_move_manipulator.send(MoveManipulatorEvent(Direction::Right));
        }
    }
}

fn process_mouse_input(
    In(focus): In<Focus>,
    mut mouse_events: EventReader<MouseButtonInput>,
    mut mouse_input: Local<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    level: Res<Level>,
    q_xform: Query<&Transform>,
    mut ev_select_manipulator: EventWriter<SelectManipulatorEvent>,
    mut ev_move_manipulator: EventWriter<MoveManipulatorEvent>,
) {
    mouse_input.clear();
    for event in mouse_events.read() {
        match event.state {
            ButtonState::Pressed => mouse_input.press(event.button),
            ButtonState::Released => mouse_input.release(event.button),
        }
    }

    if let Focus::Busy(_) = focus {
        return;
    }

    if mouse_input.just_pressed(MouseButton::Left) {
        let (camera, xform) = camera.single();
        let window = window.single();
        let coords_and_offset = window
            .cursor_position()
            .and_then(|pos| camera.viewport_to_world_2d(xform, pos))
            .and_then(|pos| level.coords_at_pos(pos, &q_xform));
        if let Some((coords, offset)) = coords_and_offset {
            if let Focus::Selected(focus_coords, directions) = focus {
                if coords == focus_coords {
                    if let Some(direction) = focus_direction_for_offset(offset) {
                        if directions.contains(direction) {
                            ev_move_manipulator.send(MoveManipulatorEvent(direction));
                        }
                    }
                    return;
                }
            }
            if let Some(Piece::Manipulator(_)) = level.present.pieces.get(coords) {
                if is_offset_inside_manipulator(offset) {
                    ev_select_manipulator.send(SelectManipulatorEvent::AtCoords(coords));
                }
            } else {
                ev_select_manipulator.send(SelectManipulatorEvent::Deselect);
            }
        }
    }
}

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SelectManipulatorEvent>()
            .add_event::<MoveManipulatorEvent>()
            .add_systems(
                FixedPreUpdate,
                (
                    get_focus.pipe(process_keyboard_input),
                    get_focus.pipe(process_mouse_input),
                )
                    .in_set(InputSet),
            );
    }
}
