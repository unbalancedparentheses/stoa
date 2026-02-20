use egui::{Align, Layout, RichText, ScrollArea};

use crate::app::{ChatApp, View};
use crate::theme::*;

pub fn draw(ctx: &egui::Context, app: &mut ChatApp) {
    egui::SidePanel::left("sidebar")
        .exact_width(260.0)
        .frame(egui::Frame::new().fill(BG_SIDEBAR))
        .show(ctx, |ui| {
            ui.add_space(24.0);

            // Workspace header
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("Workspace").size(11.0).color(TEXT_MUTED));
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("rust-chat").size(16.0).color(TEXT_HEAD));
                        ui.label(RichText::new("v0.1.0").size(11.0).color(TEXT_MUTED));
                    });
                });
            });

            ui.add_space(16.0);

            // Navigation
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.label(RichText::new("Navigation").size(11.0).color(TEXT_MUTED));
            });
            ui.add_space(6.0);

            ui.vertical(|ui| {
                let chat_active = app.view == View::Chat;
                let settings_active = app.view == View::Settings;

                nav_button(ui, "\u{2302}  Home", chat_active, || {
                    app.view = View::Chat;
                });
                nav_button(ui, "+  New Chat", false, || {
                    app.conversations.push(crate::model::Conversation::new());
                    app.active_conversation = app.conversations.len() - 1;
                    app.view = View::Chat;
                });
                nav_button(ui, "\u{2699}  Settings", settings_active, || {
                    app.view = View::Settings;
                    app.config_saved = false;
                });
            });

            ui.add_space(16.0);

            // History
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.label(RichText::new("History").size(11.0).color(TEXT_MUTED));
            });
            ui.add_space(6.0);

            let can_delete = app.conversations.len() > 1;
            let chat_active = app.view == View::Chat;

            let mut action: Option<ConvAction> = None;

            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for i in 0..app.conversations.len() {
                        let is_active = i == app.active_conversation && chat_active;
                        let title: String = app.conversations[i].title.chars().take(22).collect();
                        let title = if app.conversations[i].title.len() > 22 {
                            format!("{title}\u{2026}")
                        } else {
                            title
                        };

                        ui.horizontal(|ui| {
                            ui.add_space(10.0);

                            let bg = if is_active { BG_ACTIVE } else { BG_SIDEBAR };
                            let text_color = if is_active { TEXT_HEAD } else { TEXT_SEC };

                            let button = egui::Button::new(
                                RichText::new(&title).size(12.0).color(text_color),
                            )
                            .fill(bg)
                            .corner_radius(4.0)
                            .min_size(egui::vec2(
                                ui.available_width() - if can_delete { 30.0 } else { 10.0 },
                                0.0,
                            ));

                            if ui.add(button).clicked() {
                                action = Some(ConvAction::Select(i));
                            }

                            if can_delete {
                                let del = egui::Button::new(
                                    RichText::new("\u{00D7}").size(11.0).color(TEXT_MUTED),
                                )
                                .fill(egui::Color32::TRANSPARENT);
                                if ui.add(del).clicked() {
                                    action = Some(ConvAction::Delete(i));
                                }
                            }
                        });
                        ui.add_space(1.0);
                    }
                });

            if let Some(a) = action {
                match a {
                    ConvAction::Select(i) => {
                        app.active_conversation = i;
                        app.view = View::Chat;
                    }
                    ConvAction::Delete(i) => {
                        if app.conversations.len() > 1 {
                            app.conversations[i].delete();
                            app.conversations.remove(i);
                            if app.active_conversation >= app.conversations.len() {
                                app.active_conversation = app.conversations.len() - 1;
                            } else if app.active_conversation > i {
                                app.active_conversation -= 1;
                            }
                        }
                    }
                }
            }

            // Status at bottom
            ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Latency").size(10.0).color(TEXT_MUTED));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(
                                    RichText::new("--")
                                        .size(10.0)
                                        .color(TEXT_SEC)
                                        .monospace(),
                                );
                            });
                        });
                        let status_text = if app.is_streaming {
                            "Streaming..."
                        } else {
                            "All systems nominal"
                        };
                        let status_color = if app.is_streaming { ACCENT } else { TEXT_SEC };
                        ui.label(RichText::new(status_text).size(11.0).color(status_color));
                        ui.label(RichText::new("Status").size(11.0).color(TEXT_MUTED));
                    });
                });
                // Divider
                ui.add_space(4.0);
                let rect = ui.available_rect_before_wrap();
                let line_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.left(), rect.bottom() - 1.0),
                    egui::vec2(rect.width(), 1.0),
                );
                ui.painter().rect_filled(line_rect, 0.0, DIVIDER);
            });
        });
}

enum ConvAction {
    Select(usize),
    Delete(usize),
}

fn nav_button(ui: &mut egui::Ui, label: &str, active: bool, mut on_click: impl FnMut()) {
    ui.horizontal(|ui| {
        ui.add_space(10.0);

        let bg = if active { BG_ACTIVE } else { BG_SIDEBAR };
        let text_color = if active { ACCENT } else { TEXT_SEC };

        let button = egui::Button::new(RichText::new(label).size(13.0).color(text_color))
            .fill(bg)
            .corner_radius(6.0)
            .min_size(egui::vec2(ui.available_width() - 10.0, 0.0));

        if ui.add(button).clicked() {
            on_click();
        }
    });
    ui.add_space(2.0);
}
