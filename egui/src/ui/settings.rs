use egui::{RichText, ScrollArea};

use crate::app::ChatApp;
use crate::model::Provider;
use crate::theme::*;

pub fn draw(ui: &mut egui::Ui, app: &mut ChatApp) {
    // Header
    ui.horizontal(|ui| {
        let header_rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(
            egui::Rect::from_min_size(header_rect.min, egui::vec2(header_rect.width(), 44.0)),
            0.0,
            BG_HEADER,
        );
        ui.add_space(24.0);
        ui.add_sized(
            [0.0, 44.0],
            egui::Label::new(RichText::new("Settings").size(15.0).color(TEXT_HEAD)),
        );
    });

    // Divider
    let rect = ui.cursor();
    ui.painter().rect_filled(
        egui::Rect::from_min_size(rect.min, egui::vec2(ui.available_width(), 1.0)),
        0.0,
        BORDER_SUBTLE,
    );
    ui.add_space(1.0);

    ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.add_space(20.0);

            let max_width = 520.0_f32.min(ui.available_width() - 64.0);
            ui.horizontal(|ui| {
                ui.add_space(32.0);
                ui.vertical(|ui| {
                    ui.set_max_width(max_width);

                    // Provider toggle
                    egui::Frame::new()
                        .fill(BG_INPUT)
                        .corner_radius(8.0)
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let half = (ui.available_width() - 4.0) / 2.0;

                                let openai_active = app.config.active_provider == Provider::OpenAI;
                                let openai_bg = if openai_active {
                                    egui::Color32::from_rgb(0x1e, 0x2a, 0x38)
                                } else {
                                    egui::Color32::TRANSPARENT
                                };
                                let openai_text = if openai_active { ACCENT } else { TEXT_SEC };

                                if ui
                                    .add_sized(
                                        [half, 32.0],
                                        egui::Button::new(
                                            RichText::new("OpenAI")
                                                .size(13.0)
                                                .color(openai_text),
                                        )
                                        .fill(openai_bg)
                                        .corner_radius(6.0),
                                    )
                                    .clicked()
                                {
                                    app.config.active_provider = Provider::OpenAI;
                                    app.config_saved = false;
                                }

                                let anthropic_active =
                                    app.config.active_provider == Provider::Anthropic;
                                let anthropic_bg = if anthropic_active {
                                    egui::Color32::from_rgb(0x1e, 0x2a, 0x38)
                                } else {
                                    egui::Color32::TRANSPARENT
                                };
                                let anthropic_text =
                                    if anthropic_active { ACCENT } else { TEXT_SEC };

                                if ui
                                    .add_sized(
                                        [half, 32.0],
                                        egui::Button::new(
                                            RichText::new("Anthropic")
                                                .size(13.0)
                                                .color(anthropic_text),
                                        )
                                        .fill(anthropic_bg)
                                        .corner_radius(6.0),
                                    )
                                    .clicked()
                                {
                                    app.config.active_provider = Provider::Anthropic;
                                    app.config_saved = false;
                                }
                            });
                        });

                    ui.add_space(12.0);

                    // Model presets card
                    egui::Frame::new()
                        .fill(BG_CARD)
                        .corner_radius(8.0)
                        .stroke(egui::Stroke::new(1.0, BORDER_SUBTLE))
                        .inner_margin(16.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Model").size(12.0).color(TEXT_MUTED));
                            ui.add_space(10.0);

                            let presets = match app.config.active_provider {
                                Provider::OpenAI => vec!["GPT-5", "GPT-4.1", "o3", "o4-mini"],
                                Provider::Anthropic => vec!["Opus", "Sonnet", "Haiku"],
                            };

                            ui.horizontal_wrapped(|ui| {
                                for p in presets {
                                    let is_active = is_preset_active(&app.config, p);
                                    let bg = if is_active {
                                        egui::Color32::from_rgb(0x2a, 0x24, 0x14)
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    };
                                    let fg = if is_active { ACCENT } else { TEXT_SEC };
                                    let border = if is_active { ACCENT } else { BORDER_DEFAULT };

                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new(p).size(12.0).color(fg),
                                            )
                                            .fill(bg)
                                            .corner_radius(14.0)
                                            .stroke(egui::Stroke::new(1.0, border))
                                            .min_size(egui::vec2(0.0, 28.0)),
                                        )
                                        .clicked()
                                    {
                                        app.config.apply_preset(p);
                                        app.config_saved = false;
                                    }
                                }
                            });
                        });

                    ui.add_space(12.0);

                    // Connection fields card
                    egui::Frame::new()
                        .fill(BG_CARD)
                        .corner_radius(8.0)
                        .stroke(egui::Stroke::new(1.0, BORDER_SUBTLE))
                        .inner_margin(16.0)
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("Connection").size(12.0).color(TEXT_MUTED),
                            );
                            ui.add_space(12.0);

                            // API URL
                            ui.label(RichText::new("API URL").size(11.0).color(TEXT_MUTED));
                            ui.add_space(4.0);
                            let url = &mut app.config.active_provider_config_mut().api_url;
                            if ui
                                .add(
                                    egui::TextEdit::singleline(url)
                                        .hint_text("https://api.openai.com/v1/chat/completions")
                                        .desired_width(f32::INFINITY),
                                )
                                .changed()
                            {
                                app.config_saved = false;
                            }

                            ui.add_space(12.0);

                            // API Key
                            ui.label(RichText::new("API Key").size(11.0).color(TEXT_MUTED));
                            ui.add_space(4.0);
                            let key = &mut app.config.active_provider_config_mut().api_key;
                            if ui
                                .add(
                                    egui::TextEdit::singleline(key)
                                        .hint_text("sk-...")
                                        .password(true)
                                        .desired_width(f32::INFINITY),
                                )
                                .changed()
                            {
                                app.config_saved = false;
                            }

                            ui.add_space(12.0);

                            // Model ID
                            ui.label(RichText::new("Model ID").size(11.0).color(TEXT_MUTED));
                            ui.add_space(4.0);
                            let model_id = &mut app.config.active_provider_config_mut().model;
                            if ui
                                .add(
                                    egui::TextEdit::singleline(model_id)
                                        .hint_text("gpt-4.1")
                                        .desired_width(f32::INFINITY),
                                )
                                .changed()
                            {
                                app.config_saved = false;
                            }
                        });

                    ui.add_space(12.0);

                    // Save button
                    let (save_bg, save_text, save_border) = if app.config_saved {
                        (
                            egui::Color32::from_rgb(0x14, 0x2a, 0x1e),
                            SUCCESS,
                            egui::Color32::from_rgb(0x24, 0x50, 0x3a),
                        )
                    } else {
                        (BG_CARD, ACCENT, BORDER_DEFAULT)
                    };

                    let save_label = if app.config_saved {
                        "\u{2713} Saved"
                    } else {
                        "Save"
                    };

                    if ui
                        .add_sized(
                            [ui.available_width(), 36.0],
                            egui::Button::new(
                                RichText::new(save_label).size(13.0).color(save_text),
                            )
                            .fill(save_bg)
                            .corner_radius(8.0)
                            .stroke(egui::Stroke::new(1.0, save_border)),
                        )
                        .clicked()
                    {
                        app.config.save();
                        app.config_saved = true;
                    }

                    ui.add_space(20.0);
                });
            });
        });
}

fn is_preset_active(config: &crate::config::AppConfig, preset: &str) -> bool {
    let model = &config.active_provider_config().model;
    match preset {
        "GPT-5" => model == "gpt-5",
        "GPT-4.1" => model == "gpt-4.1",
        "o3" => model == "o3",
        "o4-mini" => model == "o4-mini",
        "Opus" => model.contains("opus"),
        "Sonnet" => model.contains("sonnet"),
        "Haiku" => model.contains("haiku"),
        _ => false,
    }
}
