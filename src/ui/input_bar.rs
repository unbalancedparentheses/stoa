use iced::widget::{button, container, row, text, text_input, Column};
use iced::{Alignment, Element, Length, Color, Border, Theme};

use crate::app::{ChatApp, Message};
use crate::config::AppConfig;
use crate::theme::*;

fn input_style(_: &Theme, status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: iced::Background::Color(INPUT_BG),
        border: Border {
            radius: 24.0.into(),
            width: 1.0,
            color: match status {
                text_input::Status::Focused { .. } => ACCENT_DIM,
                _ => BORDER_DEFAULT,
            },
        },
        icon: TEXT_MUTED,
        placeholder: TEXT_MUTED,
        value: TEXT_HEAD,
        selection: SELECTION,
    }
}

fn send_style(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(match status {
            button::Status::Hovered => ACCENT,
            _ => ACCENT_DIM,
        })),
        text_color: TEXT_HEAD,
        border: Border { radius: 18.0.into(), ..Default::default() },
        ..Default::default()
    }
}

fn send_disabled_style(_: &Theme, _: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(0x1a, 0x22, 0x2e))),
        text_color: TEXT_MUTED,
        border: Border { radius: 18.0.into(), ..Default::default() },
        ..Default::default()
    }
}

fn stop_style(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(match status {
            button::Status::Hovered => DANGER,
            _ => Color::from_rgb8(0x8a, 0x3a, 0x3a),
        })),
        text_color: TEXT_HEAD,
        border: Border { radius: 18.0.into(), ..Default::default() },
        ..Default::default()
    }
}

fn chip_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let bg = if active {
            ACCENT_DIM
        } else {
            match status {
                button::Status::Hovered => BG_HOVER,
                _ => BG_ACTIVE,
            }
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: if active { ACCENT } else { TEXT_SEC },
            border: Border {
                radius: 12.0.into(),
                width: 1.0,
                color: if active { ACCENT_DIM } else { BORDER_DEFAULT },
            },
            ..Default::default()
        }
    }
}

fn run_n_style(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(match status {
            button::Status::Hovered => Color::from_rgb8(0x2a, 0x7a, 0x5a),
            _ => Color::from_rgb8(0x1e, 0x5a, 0x42),
        })),
        text_color: TEXT_HEAD,
        border: Border { radius: 18.0.into(), ..Default::default() },
        ..Default::default()
    }
}

pub fn short_model_name(model_id: &str) -> &str {
    for (display, id) in AppConfig::available_models() {
        if id == model_id {
            return display;
        }
    }
    // Fallback: last segment
    model_id.split('/').last().unwrap_or(model_id)
}

pub fn provider_icon(model_id: &str) -> &'static str {
    if model_id.contains("claude") || model_id.contains("haiku") || model_id.contains("sonnet") || model_id.contains("opus") {
        "\u{2726}" // diamond / anthropic
    } else {
        "\u{25CF}" // circle / openai
    }
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let input = text_input("Share a thought...", &app.input_value)
        .on_input(Message::InputChanged)
        .on_submit(Message::SendMessage)
        .padding([12, 20])
        .size(14)
        .style(input_style);

    let conv_streaming = app.is_active_conv_streaming();
    let conv_stream_count = app.conv_stream_count(&app.conversations[app.active_conversation].id);
    let selected_count = app.selected_models.len();

    let action_btn: Element<'_, Message> = if conv_streaming {
        let label = if conv_stream_count > 1 { "Stop All" } else { "\u{25A0}" };
        button(
            container(text(label).size(if conv_stream_count > 1 { 11 } else { 14 }))
                .align_x(Alignment::Center)
                .align_y(iced::alignment::Vertical::Center)
        ).on_press(Message::StopStreaming).width(if conv_stream_count > 1 { 72 } else { 36 }).height(36).style(stop_style).into()
    } else {
        let can_send = !app.input_value.trim().is_empty();
        if can_send {
            button(
                container(text("\u{2191}").size(14))
                    .align_x(Alignment::Center)
                    .align_y(iced::alignment::Vertical::Center)
            ).on_press(Message::SendMessage).width(36).height(36).style(send_style).into()
        } else {
            button(
                container(text("\u{2191}").size(14))
                    .align_x(Alignment::Center)
                    .align_y(iced::alignment::Vertical::Center)
            ).width(36).height(36).style(send_disabled_style).into()
        }
    };

    // Model selector chip
    let icon = provider_icon(&app.selected_model);
    let name = short_model_name(&app.selected_model);
    let model_chip = button(
        text(format!("{icon} {name}")).size(11)
    )
    .on_press(Message::ToggleModelPicker)
    .padding([4, 10])
    .style(chip_style(app.model_picker_open));

    // Attach file button
    let attach_btn = button(
        container(text("+").size(14))
            .align_x(Alignment::Center)
            .align_y(iced::alignment::Vertical::Center)
    ).on_press(Message::AttachFile).width(36).height(36).style(|_: &Theme, status: button::Status| {
        button::Style {
            background: Some(iced::Background::Color(match status {
                button::Status::Hovered => BG_HOVER,
                _ => BG_ACTIVE,
            })),
            text_color: TEXT_SEC,
            border: Border { radius: 18.0.into(), width: 1.0, color: BORDER_DEFAULT },
            ..Default::default()
        }
    });

    let mut input_row = row![
        model_chip,
        attach_btn,
        input,
    ].spacing(10).align_y(Alignment::Center);

    // "Run N" button when multiple models selected and not streaming
    if selected_count > 1 && !conv_streaming && !app.input_value.trim().is_empty() {
        let models: Vec<String> = app.selected_models.iter().cloned().collect();
        input_row = input_row.push(
            button(
                container(text(format!("Run {selected_count}")).size(11))
                    .align_x(Alignment::Center)
                    .align_y(iced::alignment::Vertical::Center)
            ).on_press(Message::SendToModels(models)).width(64).height(36).style(run_n_style)
        );
    }

    input_row = input_row.push(action_btn);

    let mut content = Column::new();

    // Show attached file indicator
    if let Some(ref filename) = app.attached_filename {
        content = content.push(
            container(
                row![
                    text(format!("\u{1F4CE} {filename}")).size(11).color(ACCENT),
                    iced::widget::Space::new().width(8),
                    text(format!("({} chars)", app.attached_file.as_ref().map(|f| f.len()).unwrap_or(0))).size(10).color(TEXT_MUTED),
                ].align_y(Alignment::Center)
            ).padding(iced::Padding { top: 0.0, right: 28.0, bottom: 4.0, left: 28.0 })
        );
    }

    // Model picker dropdown (above input row)
    if app.model_picker_open {
        let mut picker_row = iced::widget::Row::new().spacing(6);
        for (display, model_id) in AppConfig::available_models() {
            let is_selected_multi = app.selected_models.contains(model_id);
            let is_current = model_id == app.selected_model;
            let icon = provider_icon(model_id);
            let check = if is_selected_multi { "\u{2611}" } else { "\u{2610}" };

            // Left part: checkbox for multi-select
            picker_row = picker_row.push(
                button(text(format!("{check} {icon} {display}")).size(11))
                    .on_press(Message::ToggleModelSelection(model_id.to_string()))
                    .padding([4, 10])
                    .style(chip_style(is_current || is_selected_multi))
            );
        }
        // "Run All" chip
        picker_row = picker_row.push(
            button(text("Run All").size(11))
                .on_press(Message::SendToAll)
                .padding([4, 10])
                .style(run_n_style)
        );
        content = content.push(
            container(picker_row)
                .padding(iced::Padding { top: 0.0, right: 28.0, bottom: 8.0, left: 28.0 })
        );

        // Second row: single-click to set primary model
        let mut select_row = iced::widget::Row::new().spacing(6);
        select_row = select_row.push(text("Primary:").size(10).color(TEXT_MUTED));
        for (display, model_id) in AppConfig::available_models() {
            let is_current = model_id == app.selected_model;
            let icon = provider_icon(model_id);
            select_row = select_row.push(
                button(text(format!("{icon} {display}")).size(10))
                    .on_press(Message::SelectModel(model_id.to_string()))
                    .padding([3, 8])
                    .style(chip_style(is_current))
            );
        }
        content = content.push(
            container(select_row)
                .padding(iced::Padding { top: 0.0, right: 28.0, bottom: 8.0, left: 28.0 })
        );
    }

    content = content.push(container(input_row));

    container(content)
        .width(Length::Fill)
        .padding(iced::Padding { top: 12.0, right: 28.0, bottom: 20.0, left: 28.0 })
        .style(|_: &Theme| container::Style {
            background: Some(iced::Background::Color(MAIN_BG)),
            ..Default::default()
        })
        .into()
}
