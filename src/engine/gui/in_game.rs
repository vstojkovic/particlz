use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::engine::focus::Focus;
use crate::engine::level::Level;
use crate::engine::GameState;

use super::UndoMoves;

pub(super) fn in_game_ui(
    focus: In<Focus>,
    state: Res<State<GameState>>,
    level: Res<Level>,
    mut egui_ctx: EguiContexts,
    mut ev_undo: EventWriter<UndoMoves>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let enabled = match state.get() {
        GameState::Playing => true,
        _ => false,
    };
    let undo_enabled = enabled
        && level.can_undo()
        && match &*focus {
            Focus::Busy(_) => false,
            _ => true,
        };
    egui::SidePanel::right("in_game_ui")
        .resizable(false)
        .exact_width(IN_GAME_PANEL_WIDTH as _)
        .frame(egui::Frame::none().inner_margin(10.0))
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                if let Some(name) = level.metadata.name.as_ref() {
                    ui.label(name);
                    ui.add_space(20.0);
                }
                if ui
                    .add_enabled(undo_enabled, egui::Button::new("UndO"))
                    .clicked()
                {
                    ev_undo.send(UndoMoves::Last);
                }
                if ui
                    .add_enabled(undo_enabled, egui::Button::new("reSeT"))
                    .clicked()
                {
                    ev_undo.send(UndoMoves::All);
                }
                if ui.add_enabled(enabled, egui::Button::new("MenU")).clicked() {
                    next_state.set(GameState::MainMenu);
                }
            });
        });
}

pub const IN_GAME_PANEL_WIDTH: u32 = 200;
