use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::model::Board;

use super::PlayLevel;

pub(super) fn classic_level_select_ui(
    mut egui_ctx: EguiContexts,
    mut ev_play: EventWriter<PlayLevel>,
) {
    fn add_button(ui: &mut egui::Ui, idx: usize) -> egui::Response {
        ui.vertical_centered(|ui| {
            ui.add(egui::Button::new((idx + 1).to_string()).min_size(egui::Vec2::new(60.0, 0.0)))
        })
        .inner
    }

    let mut selected_level = None;

    egui::CentralPanel::default()
        .frame(egui::Frame::none().inner_margin(10.0))
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("SeLeCT A LeVeL");
                ui.group(|ui| {
                    ui.label("eASY");
                    ui.columns(7, |ui| {
                        for level_idx in 0..=6 {
                            if add_button(&mut ui[level_idx], level_idx).clicked() {
                                selected_level = Some(level_idx);
                            }
                        }
                    });
                });
                ui.add_space(20.0);
                ui.group(|ui| {
                    ui.label("MedIUM");
                    ui.columns(7, |ui| {
                        for level_idx in 7..=13 {
                            if add_button(&mut ui[level_idx - 7], level_idx).clicked() {
                                selected_level = Some(level_idx);
                            }
                        }
                    });
                });
                ui.add_space(20.0);
                ui.group(|ui| {
                    ui.label("HArd");
                    ui.columns(7, |ui| {
                        for level_idx in 14..=20 {
                            if add_button(&mut ui[level_idx - 14], level_idx).clicked() {
                                selected_level = Some(level_idx);
                            }
                        }
                    });
                });
            });
        });
    if let Some(level_idx) = selected_level {
        let board = Board::from_pbc1(CLASSIC_CAMPAIGN[level_idx]).unwrap();
        ev_play.send(PlayLevel(board));
    }
}

const CLASSIC_CAMPAIGN: [&'static str; 21] = [
    ":PBC1:AapHrUCxAhxBEASxUBAEBQoMEARhjihQoEBQoECBI5BCEARBACAFAEFQokCBhYIgCAoER6AAsVAQBEHRIAiwUBAEABBisUMQFC5QugBBYKEgKBKELAbB/wE=",
    ":PBC1:AaocQRMEUaBAgQIpgGFYngmCFACwLIIgBQAsiyBIAQDLIghSAMCyCIZJAQDLIggeoUEGAFgWQZACwINhgyAFoG0es0Hwfw==",
    ":PBC1:AXpciRIlCIIgDsABSAEAAAyQAgAAwKMUBEEQBAAWCoIgCAIACwVBEAQBgIWCIAiCgQD8Hw==",
    ":PBC1:AaocUYIgCIIgiBQAAABSGAAAgMFSIAAAQAo4RAAApAAKGAbAowSUAgAgBQAAgBQAoBSGwELBQAAA4P8=",
    ":PBC1:AZrcYShQoECBAgUKFEgBAAAgBQAAgBQAAACWIhiCIRiCGSDFEAzBEAyBFAAAAFIAAABYKAiCIAiCgfB/",
    ":PBC1:AVoHrMABKHEAChcoUKDAUggxQNEgCIKlgiAIiwZBMMxSCDFA0SAIggcoGCAcoGgQBMH/AQ==",
    ":PBC1:AZlA4QIFChRgAWCKDhbwgIJszFjChCi+UBEWAVA8WGgoQ4MwUBzTYKGARQAUDRbicwgApmgGKH5QirBgAMWDICjCAh8=",
    ":PBC1:AaocQRAEQRAEkQIAAEBqsCAPgjwYDCkgAAIAKRUCIIAGKWAAYAAAKWAQYBAAKSAFUgApAAAApAAAAPB/",
    ":PBC1:AaqHrEQBgiAIgjgCKSAAAOQpAAEABCkACIAAKSAYZiAEQAoBBhsqAJAKgAAAsBAABACwFwAgAPAAAQAQpIP8Hw==",
    ":PBC1:AartChQoUKBAgQIFeixUpEiRIkGRIkWCBYsUPeJBkSJFihRZKAiKBEWKFClSdMGuRYoULVKkSBAsGBQJijQpUiQoulCRIkWKFi8SFQkWLFJkgCA4JEWKxMkiRZgiRZgiRZiFgoGCIAiCIPg/",
    ":PBC1:AXdHjShAFCAOQCpAjsHwCCFAgCCVIkCAhTAIYgSpAAMhwEIIEGCYfw==",
    ":PBC1:AaocQTRo0KAF0eMBpBZLEmRZliUbJQAyAMlGWZhlGYBkowxIgiRJko0yIMmyLMNGGZAAyPApZUCSJFmGjTbJsiwLM+ADSpIkSZJtsk3+Dw==",
    ":PBC1:Aaq3rUCBAgUKFChQoEQqAAAgQCoAACBAKmAYhmGYAKkAgwDAMAM8QkMBGAQIkAoAAAiQChiGYRgmQCoAACDAXkEQBEEQBCv9Hw==",
    ":PBC1:AaocQRAEQRDH4CikAADAYR1mIRYAAAYLsQAAACkAAACkUKTOASxShAK2KxIMUigIAo5AHKIgKBQMkFMAolVQaIiAAwAEQfTiAAAB",
    ":PBC1:AXqcBRYQhAUEQRApQAJIAGwFQABAM0wqz3PkOYAUgAAIgFQABAgCIDXkQEMOO9BwwwD/Bw==",
    ":PBC1:AZnFihUoUKBwgQLFFhq0AM/UKTxgsFhQiAWKFiqwEM8MgQGYPkUXZAEAKLpQWwyCIYDiCxUpyALFCwaLDRnUBYoOV2ChQgWKFC9SICj0Pw==",
    ":PBC1:AaqHjaAJgiAIwoMUwAIAkALAAgCTAgAgYJACAIABUgAOQDkASIEBQQBAigHABgCSAQCwALoEAAAL0f8B",
    ":PBC1:AapnrQBBEARBEAYsJAAABKMhhbECAIIAKQQCBKMBSAEAAgApAIAgAFJAIEAwDpACRgoACIJUIAAABOOkRgoAAMD/AQ==",
    ":PBC1:AanlCIIoQBBEgYUABAAGepQAQQggWAgUOyxoKlgIFBuApYKFcIDYAAeUChYCxQZgqWAhUGwAlgoeJhAIBcAwCwEIAIT/Aw==",
    ":PBC1:AaqHjiAIgiAIgkgBAIABkQIQAABSAADQBJEaEgDADoAUgOEQHlAUXQgAAARIASGAAOxSAAAwTPAABQACAPg/",
    ":PBC1:AaoccRgIgiAIgkgBAAAgBQAAMEwKAAAAKRxwpg9ThgUeJTBHFAGKsEihOAZBgDZsCswRRYCARwoHHDFCHkiBYRiGwUHB/wE=",
];
