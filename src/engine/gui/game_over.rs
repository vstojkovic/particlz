use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::engine::gui::UndoMoves;
use crate::engine::level::Level;
use crate::engine::GameState;
use crate::model::LevelOutcome;

pub(super) fn game_over_ui(
    mut egui_ctx: EguiContexts,
    level: Res<Level>,
    mut ev_undo: EventWriter<UndoMoves>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    fn add_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
        ui.vertical_centered(|ui| {
            ui.add(egui::Button::new(text).min_size(egui::Vec2::new(100.0, 0.0)))
        })
        .inner
    }

    let outcome = level.progress.outcome.unwrap();

    let (title, color) = match outcome {
        LevelOutcome::Victory => ("LeVeL pASSed", egui::Color32::from_rgb(0x00, 0x98, 0xfe)),
        _ => ("LeVeL FAILed", egui::Color32::from_rgb(0xfe, 0x98, 0x98)),
    };
    let title = egui::RichText::new(title)
        .text_style(egui::TextStyle::Body)
        .color(color);

    egui::Window::new(title)
        .resizable(false)
        .movable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::splat(0.0))
        .min_width(360.0)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                let message = match outcome {
                    LevelOutcome::NoManipulatorsLeft => "You have no manipulators left",
                    LevelOutcome::ParticleLost => "You lost one of the particles",
                    LevelOutcome::Victory => "Congratulations!",
                };
                let message = egui::RichText::new(message).text_style(egui::TextStyle::Small);
                ui.label(message);
                ui.columns(3, |ui| {
                    if let LevelOutcome::Victory = outcome {
                        add_button(&mut ui[0], "nexT");
                    } else {
                        if add_button(&mut ui[0], "UndO").clicked() {
                            ev_undo.send(UndoMoves::Last);
                            next_state.set(GameState::Playing);
                        }
                    }
                    if add_button(&mut ui[1], "repLAy").clicked() {
                        ev_undo.send(UndoMoves::All);
                        next_state.set(GameState::Playing);
                    }
                    if add_button(&mut ui[2], "MenU").clicked() {
                        next_state.set(GameState::MainMenu);
                    }
                });
            });
        });
}
