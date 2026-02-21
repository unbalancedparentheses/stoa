use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Border, Theme};

use crate::app::{ChatApp, Message};
use crate::model::Provider;
use crate::shortcuts;
use crate::theme::*;

fn field_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => ACCENT(),
        _ => BORDER_DEFAULT(),
    };
    text_input::Style {
        background: iced::Background::Color(INPUT_BG()),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: border_color,
        },
        icon: TEXT_MUTED(),
        placeholder: TEXT_MUTED(),
        value: TEXT_HEAD(),
        selection: SELECTION(),
    }
}

fn tab_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let bg = match (active, status) {
            (true, _) => TAB_ACTIVE_BG(),
            (false, button::Status::Hovered) => TAB_HOVER_BG(),
            _ => iced::Color::TRANSPARENT,
        };
        let fg = if active { ACCENT() } else { TEXT_SEC() };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: fg,
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

fn chip_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let bg = match (active, status) {
            (true, _) => CHIP_ACTIVE_BG(),
            (false, button::Status::Hovered) => BG_HOVER(),
            _ => iced::Color::TRANSPARENT,
        };
        let fg = if active { ACCENT() } else { TEXT_SEC() };
        let bc = if active { ACCENT() } else { BORDER_DEFAULT() };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: fg,
            border: Border {
                radius: 14.0.into(),
                width: 1.0,
                color: bc,
            },
            ..Default::default()
        }
    }
}

fn save_style(saved: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        if saved {
            button::Style {
                background: Some(iced::Background::Color(SAVED_BG())),
                text_color: SUCCESS(),
                border: Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: SAVED_BORDER(),
                },
                ..Default::default()
            }
        } else {
            let bg = match status {
                button::Status::Hovered => CHIP_ACTIVE_BG(),
                _ => CARD_BG(),
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: ACCENT(),
                border: Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: BORDER_DEFAULT(),
                },
                ..Default::default()
            }
        }
    }
}

fn card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(CARD_BG())),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: BORDER_SUBTLE(),
        },
        ..Default::default()
    }
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let config = &app.config;
    let active = config.active_provider_config();

    // Header
    let header = container(
        row![
            text("Settings").size(FONT_H1).color(TEXT_HEAD()),
        ]
    )
    .width(Length::Fill)
    .padding([14, 24])
    .style(|_: &Theme| container::Style {
        background: Some(iced::Background::Color(HEADER_BG())),
        ..Default::default()
    });

    // Theme picker
    let mut theme_chips = iced::widget::Row::new().spacing(6);
    for name in ThemeName::all() {
        let is_active = *name == app.config.theme;
        theme_chips = theme_chips.push(
            button(text(name.label()).size(FONT_SMALL))
                .on_press(Message::SetTheme(*name))
                .padding([6, 16])
                .style(chip_style(is_active)),
        );
    }
    let theme_section = container(
        column![
            text("Theme").size(FONT_SMALL).color(TEXT_MUTED()),
            theme_chips,
        ].spacing(10)
    )
    .padding(16)
    .width(Length::Fill)
    .style(card_style);

    // Provider section
    let openai_btn = button(
        container(text("OpenAI").size(FONT_BODY)).width(Length::Fill).align_x(Alignment::Center)
    )
        .on_press(Message::SetProvider(Provider::OpenAI))
        .width(Length::Fill)
        .padding([10, 16])
        .style(tab_style(config.active_provider == Provider::OpenAI));

    let anthropic_btn = button(
        container(text("Anthropic").size(FONT_BODY)).width(Length::Fill).align_x(Alignment::Center)
    )
        .on_press(Message::SetProvider(Provider::Anthropic))
        .width(Length::Fill)
        .padding([10, 16])
        .style(tab_style(config.active_provider == Provider::Anthropic));

    let ollama_btn = button(
        container(text("Ollama").size(FONT_BODY)).width(Length::Fill).align_x(Alignment::Center)
    )
        .on_press(Message::SetProvider(Provider::Ollama))
        .width(Length::Fill)
        .padding([10, 16])
        .style(tab_style(config.active_provider == Provider::Ollama));

    let openrouter_btn = button(
        container(text("OpenRouter").size(FONT_BODY)).width(Length::Fill).align_x(Alignment::Center)
    )
        .on_press(Message::SetProvider(Provider::OpenRouter))
        .width(Length::Fill)
        .padding([10, 16])
        .style(tab_style(config.active_provider == Provider::OpenRouter));

    let provider_toggle = container(
        row![openai_btn, anthropic_btn, ollama_btn, openrouter_btn].spacing(4)
    )
    .padding(4)
    .width(Length::Fill)
    .style(|_: &Theme| container::Style {
        background: Some(iced::Background::Color(INPUT_BG())),
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    // Model presets
    let presets: Vec<&str> = match config.active_provider {
        Provider::OpenAI => vec!["GPT-5", "GPT-4.1", "o3", "o4-mini"],
        Provider::Anthropic => vec!["Opus", "Sonnet", "Haiku"],
        Provider::Ollama | Provider::OpenRouter => Vec::new(),
    };

    let mut chips = iced::widget::Row::new().spacing(6);
    for p in presets {
        let is_active = is_preset_active(config, p);
        chips = chips.push(
            button(text(p).size(FONT_SMALL))
                .on_press(Message::ApplyPreset(p.to_string()))
                .padding([6, 16])
                .style(chip_style(is_active)),
        );
    }

    let model_section = container(
        column![
            text("Model").size(FONT_SMALL).color(TEXT_MUTED()),
            chips,
        ].spacing(10)
    )
    .padding(16)
    .width(Length::Fill)
    .style(card_style);

    // Ollama discovered models info
    let mut ollama_info = iced::widget::Column::new();
    if config.active_provider == Provider::Ollama {
        let model_count = app.config.ollama_models.len();
        let models_text = if model_count > 0 {
            format!("Discovered {} local model(s): {}", model_count, app.config.ollama_models.join(", "))
        } else {
            "No Ollama models found. Is Ollama running?".to_string()
        };
        ollama_info = ollama_info.push(
            container(
                column![
                    row![
                        text("Local Models").size(FONT_SMALL).color(TEXT_MUTED()),
                        iced::widget::Space::new().width(Length::Fill),
                        button(text("\u{21BB} Refresh").size(FONT_SMALL))
                            .on_press(Message::RefreshOllamaModels)
                            .padding([4, 10])
                            .style(chip_style(false)),
                    ].align_y(Alignment::Center),
                    text(models_text).size(FONT_SMALL).color(TEXT_SEC()),
                ].spacing(8)
            ).padding(16).width(Length::Fill).style(card_style)
        );
        ollama_info = ollama_info.push(iced::widget::Space::new().height(12));
    }

    // Connection fields
    let fields_section = container(
        column![
            text("Connection").size(FONT_SMALL).color(TEXT_MUTED()),
            labeled_field("API URL", text_input("https://api.openai.com/v1/chat/completions", &active.api_url)
                .on_input(Message::SetApiUrl)
                .padding([10, 14])
                .size(FONT_BODY)
                .style(field_style)),
            labeled_field("API Key", text_input("sk-...", &active.api_key)
                .on_input(Message::SetApiKey)
                .padding([10, 14])
                .size(FONT_BODY)
                .secure(true)
                .style(field_style)),
            labeled_field("Model ID", text_input("gpt-4.1", &active.model)
                .on_input(Message::SetModel)
                .padding([10, 14])
                .size(FONT_BODY)
                .style(field_style)),
        ].spacing(12)
    )
    .padding(16)
    .width(Length::Fill)
    .style(card_style);

    // Generation settings
    let generation_section = container(
        column![
            text("Generation").size(FONT_SMALL).color(TEXT_MUTED()),
            labeled_field("Temperature", text_input("0.7", &config.temperature)
                .on_input(Message::SetTemperature)
                .padding([10, 14])
                .size(FONT_BODY)
                .style(field_style)),
            labeled_field("Max Tokens", text_input("4096", &config.max_tokens)
                .on_input(Message::SetMaxTokens)
                .padding([10, 14])
                .size(FONT_BODY)
                .style(field_style)),
        ].spacing(12)
    )
    .padding(16)
    .width(Length::Fill)
    .style(card_style);

    // System prompt
    let system_prompt_section = container(
        column![
            text("System Prompt").size(FONT_SMALL).color(TEXT_MUTED()),
            text_input("You are a helpful assistant...", &config.system_prompt)
                .on_input(Message::SetSystemPrompt)
                .padding([10, 14])
                .size(FONT_BODY)
                .style(field_style),
        ].spacing(8)
    )
    .padding(16)
    .width(Length::Fill)
    .style(card_style);

    // Keybindings
    let mut keybinding_fields = column![text("Keybindings").size(FONT_SMALL).color(TEXT_MUTED())].spacing(10);
    for spec in shortcuts::specs() {
        let action = spec.action;
        keybinding_fields = keybinding_fields.push(
            column![
                text(spec.label).size(FONT_SMALL).color(TEXT_MUTED()),
                text_input(crate::shortcuts::default_binding(action), config.keybindings.get(action))
                    .on_input(move |v| Message::SetKeybinding(action, v))
                    .padding([10, 14])
                    .size(FONT_BODY)
                    .style(field_style),
                text(spec.description).size(FONT_CAPTION).color(TEXT_MUTED()),
            ]
            .spacing(4)
        );
    }
    let conflicts = config.keybindings.conflicts();
    if !conflicts.is_empty() {
        let mut conflict_col = iced::widget::Column::new().spacing(4);
        conflict_col = conflict_col.push(
            text("Shortcut conflicts detected").size(FONT_SMALL).color(DANGER())
        );
        for (binding, actions) in conflicts {
            let labels = actions.into_iter()
                .map(shortcuts::action_label)
                .collect::<Vec<_>>()
                .join(", ");
            conflict_col = conflict_col.push(
                text(format!("{binding} -> {labels}")).size(FONT_CAPTION).color(TEXT_MUTED())
            );
        }
        keybinding_fields = keybinding_fields.push(
            container(conflict_col)
                .padding(10)
                .style(|_: &Theme| container::Style {
                    background: Some(iced::Background::Color(CONFLICT_BG())),
                    border: Border { radius: 6.0.into(), width: 1.0, color: DANGER() },
                    ..Default::default()
                })
        );
    }
    keybinding_fields = keybinding_fields.push(
        row![
            button(text("Reset to defaults").size(FONT_SMALL))
                .on_press(Message::ResetKeybindings)
                .padding([6, 12])
                .style(chip_style(false)),
            iced::widget::Space::new().width(Length::Fill),
        ]
        .align_y(Alignment::Center)
    );
    keybinding_fields = keybinding_fields.push(
        row![
            text("Key Event Debug Log").size(FONT_SMALL).color(TEXT_MUTED()),
            iced::widget::Space::new().width(Length::Fill),
            button(text(if config.debug_key_events { "Enabled" } else { "Disabled" }).size(FONT_SMALL))
                .on_press(Message::SetDebugKeyEvents(!config.debug_key_events))
                .padding([6, 12])
                .style(debug_toggle_style(config.debug_key_events)),
        ]
        .align_y(Alignment::Center)
    );
    keybinding_fields = keybinding_fields.push(
        text("Logs to ~/.config/stoa/key-events.log (local only, no telemetry).").size(FONT_CAPTION).color(TEXT_MUTED())
    );
    let keybindings_section = container(keybinding_fields)
        .padding(16)
        .width(Length::Fill)
        .style(card_style);

    // Save
    let save_label = if app.config_saved { "\u{2713} Saved" } else { "Save" };
    let save_btn = button(
        container(text(save_label).size(FONT_BODY)).width(Length::Fill).align_x(Alignment::Center)
    )
        .on_press(Message::SaveConfig)
        .width(Length::Fill)
        .padding([10, 20])
        .style(save_style(app.config_saved));

    let content = column![
        theme_section,
        provider_toggle,
        model_section,
        ollama_info,
        fields_section,
        generation_section,
        system_prompt_section,
        keybindings_section,
        save_btn,
    ]
    .spacing(12)
    .padding([20, 32])
    .max_width(520);

    column![
        header,
        container(iced::widget::Space::new()).width(Length::Fill).height(1)
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(BORDER_SUBTLE())),
                ..Default::default()
            }),
        container(
            scrollable(
                container(content).width(Length::Fill)
            )
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &Theme| container::Style {
            background: Some(iced::Background::Color(MAIN_BG())),
            ..Default::default()
        }),
    ].into()
}

fn labeled_field<'a>(label: &str, field: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    column![
        text(label.to_string()).size(FONT_SMALL).color(TEXT_MUTED()),
        field.into(),
    ]
    .spacing(4)
    .into()
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

fn debug_toggle_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let bg = match (active, status) {
            (true, _) => DEBUG_ACTIVE_BG(),
            (false, button::Status::Hovered) => BG_HOVER(),
            _ => CARD_BG(),
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: if active { SUCCESS() } else { TEXT_SEC() },
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: if active { DEBUG_ACTIVE_BORDER() } else { BORDER_DEFAULT() },
            },
            ..Default::default()
        }
    }
}
