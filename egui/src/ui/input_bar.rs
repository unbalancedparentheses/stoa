use egui::{Align, Layout, RichText};

use crate::app::ChatApp;
use crate::theme::*;

pub fn draw(ui: &mut egui::Ui, app: &mut ChatApp, ctx: &egui::Context) {
    ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            ui.add_space(28.0);

            // Plus icon
            ui.label(RichText::new("+").size(16.0).color(TEXT_MUTED));
            ui.add_space(10.0);

            // Text input
            let response = ui.add_sized(
                [ui.available_width() - 66.0, 36.0],
                egui::TextEdit::singleline(&mut app.input_value)
                    .hint_text("Share a thought...")
                    .font(egui::TextStyle::Body)
                    .margin(egui::Margin::symmetric(20, 10)),
            );

            // Enter to send
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                app.send_message(ctx);
                response.request_focus();
            }

            ui.add_space(10.0);

            // Send button
            let can_send = !app.input_value.trim().is_empty() && !app.is_streaming;
            let (bg, text_color) = if can_send {
                (ACCENT_DIM, TEXT_HEAD)
            } else {
                (egui::Color32::from_rgb(0x1a, 0x22, 0x2e), TEXT_MUTED)
            };

            let send_btn = egui::Button::new(
                RichText::new("\u{2191}").size(14.0).color(text_color),
            )
            .fill(bg)
            .corner_radius(18.0)
            .min_size(egui::vec2(36.0, 36.0));

            if ui.add(send_btn).clicked() && can_send {
                app.send_message(ctx);
            }

            ui.add_space(28.0);
        });
        ui.add_space(12.0);
    });
}
