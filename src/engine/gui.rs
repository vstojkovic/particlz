use std::sync::Arc;

use bevy::prelude::*;
use bevy_egui::egui::FontFamily;
use bevy_egui::{egui, EguiContexts};

use crate::model::Board;

use super::focus::get_focus;
use super::{AssetsLoaded, GameAssets, GameState, InLevel};

mod classic_campaign;
mod font;
mod game_over;
mod in_game;
mod main_menu;

use self::classic_campaign::classic_level_select_ui;
use self::font::{EguiFontAsset, EguiFontAssetLoader};
use self::game_over::game_over_ui;
use self::in_game::in_game_ui;
use self::main_menu::main_menu_ui;

pub struct GuiPlugin;

pub struct GuiAssets {
    main_font: Handle<EguiFontAsset>,
    msg_font: Handle<EguiFontAsset>,
}

#[derive(Event)]
pub struct PlayLevel(pub Board);

#[derive(Event)]
pub enum UndoMoves {
    Last,
    All,
}

impl GuiAssets {
    pub fn load(server: &AssetServer, barrier: &Arc<()>) -> Self {
        Self {
            main_font: server.load_acquire("space-age.ttf", Arc::clone(&barrier)),
            msg_font: server.load_acquire("hall-fetica-decompose.ttf", Arc::clone(&barrier)),
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
    let msg_font = font_data.get(&assets.gui.msg_font).unwrap();

    let egui_ctx = egui_ctx.ctx_mut();

    let mut fonts = egui::FontDefinitions::empty();
    fonts
        .font_data
        .insert("Space Age".to_string(), main_font.data.clone());
    fonts
        .font_data
        .insert("Hall Fetica Decompose".to_string(), msg_font.data.clone());
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "Space Age".to_string());
    fonts
        .families
        .entry(egui::FontFamily::Name("main".into()))
        .or_default()
        .insert(0, "Space Age".to_string());
    fonts
        .families
        .entry(egui::FontFamily::Name("message".into()))
        .or_default()
        .insert(0, "Hall Fetica Decompose".to_string());
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

        let entry = style.text_styles.entry(egui::TextStyle::Small).or_default();
        entry.family = FontFamily::Name("message".into());
        entry.size = 20.0;
    });
}

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<EguiFontAsset>()
            .init_asset_loader::<EguiFontAssetLoader>()
            .add_event::<PlayLevel>()
            .add_event::<UndoMoves>()
            .add_systems(Update, setup_gui_ctx.run_if(in_state(GameState::Init)))
            .add_systems(Update, main_menu_ui.run_if(in_state(GameState::MainMenu)))
            .add_systems(
                Update,
                classic_level_select_ui.run_if(in_state(GameState::ClassicLevelSelect)),
            )
            .add_systems(Update, get_focus.pipe(in_game_ui).run_if(in_state(InLevel)))
            .add_systems(Update, game_over_ui.run_if(in_state(GameState::GameOver)));
    }
}
