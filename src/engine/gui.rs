use std::sync::Arc;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::{AssetsLoaded, GameAssets, GameState};

mod font;

use self::font::{EguiFontAsset, EguiFontAssetLoader};

pub struct GuiPlugin;

pub struct GuiAssets {
    main_font: Handle<EguiFontAsset>,
}

impl GuiAssets {
    pub fn load(server: &AssetServer, barrier: &Arc<()>) -> Self {
        Self {
            main_font: server.load_acquire("space-age.ttf", Arc::clone(&barrier)),
        }
    }
}

fn setup_gui_ctx(
    mut ev_loaded: EventReader<AssetsLoaded>,
    assets: Res<GameAssets>,
    font_data: Res<Assets<EguiFontAsset>>,
    mut egui_ctx: EguiContexts,
) {
    if ev_loaded.read().last().is_none() {
        return;
    }

    let main_font = font_data.get(&assets.gui.main_font).unwrap();

    let egui_ctx = egui_ctx.ctx_mut();

    let mut fonts = egui::FontDefinitions::empty();
    fonts
        .font_data
        .insert("Space Age".to_string(), main_font.data.clone());
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "Space Age".to_string());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "Space Age".to_string());
    egui_ctx.set_fonts(fonts);

    egui_ctx.style_mut(|style| {
        style
            .text_styles
            .entry(egui::TextStyle::Heading)
            .or_default()
            .size = 48.0;
        style
            .text_styles
            .entry(egui::TextStyle::Body)
            .or_default()
            .size = 20.0;
        style
            .text_styles
            .entry(egui::TextStyle::Button)
            .or_default()
            .size = 20.0;
    });
}

fn main_menu_ui(
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
        next_state.set(GameState::Playing);
    }

    if quit_clicked {
        exit.send(AppExit::Success);
    }
}

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<EguiFontAsset>()
            .init_asset_loader::<EguiFontAssetLoader>()
            .add_systems(Update, setup_gui_ctx.run_if(in_state(GameState::Init)))
            .add_systems(Update, main_menu_ui.run_if(in_state(GameState::MainMenu)));
    }
}
