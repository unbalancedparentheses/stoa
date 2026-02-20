use egui::{Align, Layout, RichText};

use crate::app::ChatApp;
use crate::model::Provider;
use crate::theme::*;

pub fn draw(ctx: &egui::Context, app: &ChatApp) {
    egui::SidePanel::right("right_panel")
        .exact_width(260.0)
        .frame(egui::Frame::new().fill(BG_SIDEBAR))
        .show(ctx, |ui| {
            let provider_name = match app.config.active_provider {
                Provider::OpenAI => "OpenAI",
                Provider::Anthropic => "Anthropic",
            };
            let model = &app.config.active_provider_config().model;
            let conv = &app.conversations[app.active_conversation];
            let msg_count = conv.messages.len();
            let conv_count = app.conversations.len();

            ui.add_space(24.0);

            // Highlights header
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("Highlights").size(16.0).color(TEXT_HEAD));
                    ui.label(
                        RichText::new("Newest updates").size(11.0).color(TEXT_MUTED),
                    );
                });
            });

            ui.add_space(16.0);
            divider(ui);

            // System section
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("System").size(11.0).color(TEXT_MUTED));
                    ui.add_space(8.0);
                    info_row(ui, "\u{25CB}", "Provider", provider_name);
                    info_row(ui, "\u{25CB}", "Model", model);
                    info_row(
                        ui,
                        "\u{25CB}",
                        "Status",
                        if app.is_streaming { "Streaming" } else { "Ready" },
                    );
                });
            });

            ui.add_space(12.0);
            divider(ui);

            // Resources section
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("Resources").size(11.0).color(TEXT_MUTED));
                    ui.add_space(8.0);
                    info_row(ui, "\u{25CB}", "Conversations", &conv_count.to_string());
                    info_row(ui, "\u{25CB}", "Messages", &msg_count.to_string());
                    info_row(ui, "\u{25CB}", "Version", "v0.1.0");
                });
            });

            // Shortcuts at bottom
            ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.vertical(|ui| {
                        shortcut_row(ui, "Ctrl+N", "New chat");
                        shortcut_row(ui, "Enter", "Send message");
                        ui.add_space(8.0);
                        ui.label(RichText::new("Shortcuts").size(12.0).color(TEXT_HEAD));
                    });
                });
                divider(ui);
            });
        });
}

fn info_row(ui: &mut egui::Ui, icon: &str, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(icon).size(12.0).color(TEXT_MUTED));
        ui.label(RichText::new(format!("  {label}")).size(12.0).color(TEXT_SEC));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(20.0);
            ui.label(RichText::new(value).size(11.0).color(TEXT_SEC).monospace());
        });
    });
    ui.add_space(6.0);
}

fn shortcut_row(ui: &mut egui::Ui, key: &str, desc: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(key).size(10.0).color(TEXT_MUTED).monospace());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(20.0);
            ui.label(RichText::new(desc).size(10.0).color(TEXT_SEC));
        });
    });
}

fn divider(ui: &mut egui::Ui) {
    let rect = ui.cursor();
    ui.painter().rect_filled(
        egui::Rect::from_min_size(rect.min, egui::vec2(ui.available_width(), 1.0)),
        0.0,
        DIVIDER,
    );
    ui.add_space(1.0);
}
