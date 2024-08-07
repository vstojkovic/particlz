use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::texture::BevyDefault;
use bevy::render::view::RenderLayers;
use bevy_egui::{egui, EguiContexts, EguiUserTextures};

use crate::engine::border::{spawn_horz_border, spawn_vert_border};
use crate::engine::level::{spawn_board, Campaign};
use crate::engine::manipulator::spawn_manipulator;
use crate::engine::particle::spawn_particle;
use crate::engine::tile::spawn_tile;
use crate::engine::GameAssets;
use crate::model::{Board, Piece};

use super::{PlayLevel, WINDOW_WIDTH};

#[derive(Resource)]
pub struct LevelPreview {
    level_idx: Option<usize>,
    board: Entity,
    image: Handle<Image>,
}

pub(super) fn init_level_preview(
    assets: Res<AssetServer>,
    mut commands: Commands,
    mut egui_user_textures: ResMut<EguiUserTextures>,
) {
    let size = Extent3d {
        width: PREVIEW_WIDTH,
        height: PREVIEW_HEIGHT,
        ..Default::default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..Default::default()
    };
    image.resize(size);
    let image = assets.add(image);

    commands.insert_resource(LevelPreview {
        level_idx: None,
        board: Entity::PLACEHOLDER,
        image: image.clone(),
    });
    egui_user_textures.add_image(image.clone_weak());

    let layer = RenderLayers::layer(1);
    let mut camera = Camera2dBundle {
        camera: Camera {
            order: -1,
            target: RenderTarget::Image(image.clone_weak()),
            ..Default::default()
        },
        ..Default::default()
    };
    camera.projection.viewport_origin = Vec2::new(0.0, 1.0);
    camera.projection.scale = PREVIEW_SCALE_FACTOR;
    commands.spawn(camera).insert(layer);
}

pub(super) fn classic_level_select_ui(
    mut egui_ctx: EguiContexts,
    campaign: Res<Campaign>,
    assets: Res<GameAssets>,
    mut preview: ResMut<LevelPreview>,
    mut commands: Commands,
    mut ev_play: EventWriter<PlayLevel>,
) {
    fn add_button(ui: &mut egui::Ui, idx: usize) -> egui::Response {
        ui.vertical_centered(|ui| {
            ui.add(egui::Button::new((idx + 1).to_string()).min_size(egui::Vec2::new(60.0, 0.0)))
        })
        .inner
    }

    let preview_image_id = egui_ctx.image_id(&preview.image).unwrap();

    let mut preview_level = None;
    let mut selected_level = None;

    egui::SidePanel::left("selection")
        .exact_width(SELECTION_PANEL_WIDTH as _)
        .frame(egui::Frame::none().inner_margin(10.0))
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("SeLeCT A LeVeL");
                for tier in campaign.tiers.iter() {
                    ui.group(|ui| {
                        ui.label(&tier.name);
                        ui.columns(tier.levels.len(), |ui| {
                            for (col, &level_idx) in tier.levels.iter().enumerate() {
                                let btn_state = add_button(&mut ui[col], level_idx);
                                if btn_state.hovered() {
                                    preview_level = Some(level_idx);
                                }
                                if btn_state.clicked() {
                                    selected_level = Some(level_idx);
                                }
                            }
                        })
                    });
                    ui.add_space(20.0);
                }
            });
        });

    if preview.level_idx != preview_level {
        if preview.level_idx.is_some() {
            commands.entity(preview.board).despawn_recursive();
        }
        if let Some(level_idx) = preview_level {
            let board = &campaign.levels[level_idx].board;
            preview.board = spawn_preview(board, &assets, &mut commands);
        } else {
            preview.board = Entity::PLACEHOLDER;
        }
        preview.level_idx = preview_level;
    }

    egui::SidePanel::right("preview")
        .resizable(false)
        .exact_width(PREVIEW_PANEL_WIDTH as _)
        .frame(egui::Frame::none().inner_margin(10.0))
        .show(egui_ctx.ctx_mut(), |ui| {
            if let Some(level_idx) = preview_level {
                ui.vertical_centered(|ui| {
                    ui.label(&campaign.levels[level_idx].name);
                    ui.add_space(30.0);
                    ui.image(egui::load::SizedTexture::new(
                        preview_image_id,
                        egui::vec2(PREVIEW_WIDTH as _, PREVIEW_HEIGHT as _),
                    ));
                });
            }
        });

    if let Some(level_idx) = selected_level {
        let board = campaign.levels[level_idx].board.clone();
        let metadata = campaign.metadata(level_idx);
        ev_play.send(PlayLevel(board, metadata));
    }
}

pub(super) fn clean_up_level_preview(mut preview: ResMut<LevelPreview>, mut commands: Commands) {
    if preview.level_idx.take().is_some() {
        commands.entity(preview.board).despawn_recursive();
        preview.board = Entity::PLACEHOLDER;
    }
}

fn spawn_preview(board: &Board, assets: &GameAssets, commands: &mut Commands) -> Entity {
    let layer = RenderLayers::layer(1);
    let mutator = |cmds: &mut EntityCommands| {
        cmds.insert(layer.clone());
    };

    let mut parent = spawn_board(commands, &mutator);
    parent.insert(layer.clone());

    parent.with_children(|parent| {
        for (coords, tile) in board.tiles.iter() {
            spawn_tile(parent, tile, coords, &assets.tiles, &mutator);
        }
        for (coords, border) in board.horz_borders.iter() {
            spawn_horz_border(parent, border, coords, &assets.borders, &mutator);
        }
        for (coords, border) in board.vert_borders.iter() {
            spawn_vert_border(parent, border, coords, &assets.borders, &mutator);
        }
        for (coords, piece) in board.pieces.iter() {
            match piece {
                Piece::Particle(particle) => {
                    spawn_particle(parent, particle, coords, &assets.particles, &mutator)
                }
                Piece::Manipulator(manipulator) => {
                    spawn_manipulator(parent, manipulator, coords, &board, &assets, &mutator)
                }
            };
        }
    });

    parent.id()
}

const PREVIEW_WIDTH: u32 = 240;
const PREVIEW_HEIGHT: u32 = 240;
const PREVIEW_SCALE_FACTOR: f32 = 2.0625;
const PREVIEW_PANEL_WIDTH: u32 = 300;
const SELECTION_PANEL_WIDTH: u32 = WINDOW_WIDTH - PREVIEW_PANEL_WIDTH;
