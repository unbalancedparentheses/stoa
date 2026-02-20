use egui::{Align, Layout, RichText, ScrollArea};

use crate::app::ChatApp;
use crate::model::Role;
use crate::theme::*;

pub fn draw(ui: &mut egui::Ui, app: &mut ChatApp) {
    let conv = &app.conversations[app.active_conversation];
    let model = app.config.active_provider_config().model.clone();

    // Header
    ui.horizontal(|ui| {
        let header_rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(
            egui::Rect::from_min_size(header_rect.min, egui::vec2(header_rect.width(), 44.0)),
            0.0,
            BG_HEADER,
        );
        ui.add_space(28.0);
        ui.add_sized(
            [0.0, 44.0],
            egui::Label::new(RichText::new("Home").size(15.0).color(TEXT_HEAD)),
        );
    });

    // Divider
    let rect = ui.cursor();
    ui.painter().rect_filled(
        egui::Rect::from_min_size(rect.min, egui::vec2(ui.available_width(), 1.0)),
        0.0,
        DIVIDER,
    );
    ui.add_space(1.0);

    // Messages area (fill remaining minus input bar height)
    let available = ui.available_height() - 70.0;
    ScrollArea::vertical()
        .stick_to_bottom(true)
        .max_height(available)
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.add_space(24.0);
            ui.horizontal(|ui| {
                ui.add_space(36.0);
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width() - 72.0);

                    if conv.messages.is_empty() && !app.is_streaming {
                        ui.add_space(48.0);
                        ui.label(RichText::new("rust-chat").size(24.0).color(TEXT_HEAD));
                        ui.label(
                            RichText::new("Share a thought to get started.")
                                .size(13.0)
                                .color(TEXT_MUTED),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!("Model: {model}"))
                                .size(11.0)
                                .color(TEXT_MUTED)
                                .monospace(),
                        );
                        return;
                    }

                    let msg_count = conv.messages.len();
                    let mut prev_role: Option<&Role> = None;

                    for msg in &conv.messages {
                        let same_role = prev_role == Some(&msg.role);

                        match msg.role {
                            Role::User => {
                                if !same_role {
                                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                        ui.label(
                                            RichText::new("You").size(11.0).color(TEXT_SEC),
                                        );
                                    });
                                    ui.add_space(4.0);
                                }
                                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                    egui::Frame::new()
                                        .fill(BG_USER_BUBBLE)
                                        .corner_radius(12.0)
                                        .inner_margin(egui::Margin::symmetric(16, 12))
                                        .show(ui, |ui| {
                                            ui.set_max_width(600.0);
                                            ui.label(
                                                RichText::new(&msg.content)
                                                    .size(15.0)
                                                    .color(TEXT_HEAD),
                                            );
                                        });
                                });
                            }
                            Role::Assistant => {
                                if !same_role {
                                    ui.label(
                                        RichText::new(format!(
                                            "({model}) reading {msg_count} messages"
                                        ))
                                        .size(11.0)
                                        .color(TEXT_MUTED),
                                    );
                                    ui.add_space(6.0);
                                }
                                ui.label(
                                    RichText::new(&msg.content)
                                        .size(14.0)
                                        .color(TEXT_BODY)
                                        .line_height(Some(20.0)),
                                );

                                if msg.streaming {
                                    ui.label(
                                        RichText::new("\u{2022}\u{2022}\u{2022}")
                                            .size(14.0)
                                            .color(ACCENT),
                                    );
                                }

                                if !msg.streaming {
                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        for icon in
                                            ["\u{2398}", "\u{25B3}", "\u{25BD}", "\u{2026}"]
                                        {
                                            ui.add(
                                                egui::Button::new(
                                                    RichText::new(icon)
                                                        .size(12.0)
                                                        .color(TEXT_MUTED),
                                                )
                                                .fill(egui::Color32::TRANSPARENT),
                                            );
                                        }
                                    });
                                }
                            }
                        }

                        ui.add_space(20.0);
                        prev_role = Some(&msg.role);
                    }

                    // Streaming placeholder when no content yet
                    if app.is_streaming && app.current_response.is_empty() {
                        ui.label(
                            RichText::new(format!("({model}) reading {msg_count} messages"))
                                .size(11.0)
                                .color(TEXT_MUTED),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("\u{2022}\u{2022}\u{2022}")
                                .size(16.0)
                                .color(ACCENT),
                        );
                    }

                    // Error banner
                    if let Some(ref err) = app.error_message {
                        ui.add_space(8.0);
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgb(0x2a, 0x18, 0x18))
                            .corner_radius(8.0)
                            .stroke(egui::Stroke::new(
                                1.0,
                                egui::Color32::from_rgb(0x44, 0x22, 0x22),
                            ))
                            .inner_margin(egui::Margin::symmetric(14, 10))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(err).size(13.0).color(DANGER));
                                    ui.with_layout(
                                        Layout::right_to_left(Align::Center),
                                        |ui| {
                                            if ui
                                                .add(
                                                    egui::Button::new(
                                                        RichText::new("\u{00D7}")
                                                            .size(12.0)
                                                            .color(egui::Color32::from_rgb(
                                                                0x88, 0x55, 0x55,
                                                            )),
                                                    )
                                                    .fill(egui::Color32::TRANSPARENT),
                                                )
                                                .clicked()
                                            {
                                                // Handled outside
                                            }
                                        },
                                    );
                                });
                            });
                    }
                });
            });
            ui.add_space(24.0);
        });
}
