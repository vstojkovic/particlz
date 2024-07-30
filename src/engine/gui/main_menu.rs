use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::engine::GameState;

pub(super) fn main_menu_ui(
    mut egui_ctx: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    let mut play_clicked = false;
    let mut quit_clicked = false;

    egui::CentralPanel::default()
        .frame(egui::Frame::none().inner_margin(10.0))
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("pArTICLZ");
                play_clicked = ui.button("pLAY").clicked();
                quit_clicked = ui.button("QUIT").clicked();
            });
        });

    if play_clicked {
        next_state.set(GameState::ClassicLevelSelect);
    }

    if quit_clicked {
        exit.send(AppExit::Success);
    }
}
