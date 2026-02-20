use egui::{Align, Layout, RichText};

use crate::app::{ChatApp, View};
use crate::theme::*;

pub fn draw(ctx: &egui::Context, app: &mut ChatApp) {
    egui::TopBottomPanel::bottom("bottom_bar")
        .exact_height(32.0)
        .frame(egui::Frame::new().fill(BG_DARKEST))
        .show(ctx, |ui| {
            // Divider at top
            let rect = ui.cursor();
            ui.painter().rect_filled(
                egui::Rect::from_min_size(rect.min, egui::vec2(ui.available_width(), 1.0)),
                0.0,
                DIVIDER,
            );
            ui.add_space(1.0);

            ui.horizontal_centered(|ui| {
                ui.add_space(16.0);

                // Left buttons
                if ui
                    .add(
                        egui::Button::new(RichText::new("\u{2302} Home").size(11.0).color(TEXT_MUTED))
                            .fill(egui::Color32::TRANSPARENT),
                    )
                    .clicked()
                {
                    app.view = View::Chat;
                }
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("\u{2699} Settings").size(11.0).color(TEXT_MUTED),
                        )
                        .fill(egui::Color32::TRANSPARENT),
                    )
                    .clicked()
                {
                    app.view = View::Settings;
                    app.config_saved = false;
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(16.0);
                    ui.add(
                        egui::Button::new(
                            RichText::new("\u{25A8} Right").size(11.0).color(TEXT_MUTED),
                        )
                        .fill(egui::Color32::TRANSPARENT),
                    );
                    ui.add(
                        egui::Button::new(
                            RichText::new("\u{25A7} Left").size(11.0).color(TEXT_MUTED),
                        )
                        .fill(egui::Color32::TRANSPARENT),
                    );
                });
            });
        });
}
