use iced::widget::{button, column, container, row, text, Column};
use iced::{Element, Length, Theme, Border};

use crate::app::{ChatApp, Message};
use crate::commands;
use crate::theme::*;
use crate::ui::input_bar::{short_model_name, provider_icon};

fn info_row<'a>(icon: &'a str, label: &'a str, value: String) -> Element<'a, Message> {
    row![
        text(icon).size(12).color(TEXT_MUTED),
        text(format!("  {label}")).size(12).color(TEXT_SEC),
        iced::widget::Space::new().width(Length::Fill),
        text(value).size(11).color(TEXT_SEC).font(iced::Font::MONOSPACE),
    ].align_y(iced::Alignment::Center).into()
}

fn stream_card_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(BG_ACTIVE)),
        border: Border { radius: 6.0.into(), width: 1.0, color: BORDER_DEFAULT },
        ..Default::default()
    }
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let icon = provider_icon(&app.selected_model);
    let provider_name = if app.selected_model.contains("claude") || app.selected_model.contains("haiku") || app.selected_model.contains("sonnet") || app.selected_model.contains("opus") {
        "Anthropic"
    } else if app.config.ollama_models.contains(&app.selected_model) {
        "Ollama"
    } else {
        "OpenAI"
    };
    let model = short_model_name(&app.selected_model);
    let conv = &app.conversations[app.active_conversation];
    let msg_count = conv.messages.len();
    let conv_count = app.conversations.len();

    let header = container(column![
        text("Highlights").size(16).color(TEXT_HEAD),
        text("Newest updates").size(11).color(TEXT_MUTED),
    ].spacing(3)).padding(iced::Padding { top: 24.0, right: 20.0, bottom: 16.0, left: 20.0 });

    let mut content_col = column![
        header,
        container(iced::widget::Space::new()).width(Length::Fill).height(1)
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(DIVIDER)),
                ..Default::default()
            }),
    ];

    // Active streams section
    if app.is_streaming() {
        let mut streams_col = Column::new().spacing(8);
        streams_col = streams_col.push(text("Active Streams").size(11).color(TEXT_MUTED));
        streams_col = streams_col.push(iced::widget::Space::new().height(4));

        for (_id, stream) in &app.active_streams {
            let s_icon = provider_icon(&stream.model);
            let s_name = short_model_name(&stream.model);
            let elapsed = stream.stream_start.elapsed().as_secs();
            let chars = stream.current_response.len();
            let status = if !stream.first_token_received { "connecting...".to_string() } else { format!("{chars} chars, {elapsed}s") };
            let conv_title: String = app.conv_index_by_id(&stream.conversation_id)
                .and_then(|ci| app.conversations.get(ci))
                .map(|c| c.title.chars().take(14).collect())
                .unwrap_or_else(|| "?".to_string());

            let card = container(column![
                row![
                    text(format!("{s_icon} {s_name}")).size(11).color(TEXT_HEAD),
                    iced::widget::Space::new().width(Length::Fill),
                    text("\u{25CF}").size(8).color(ACCENT),
                ].align_y(iced::Alignment::Center),
                text(status).size(10).color(TEXT_SEC).font(iced::Font::MONOSPACE),
                text(conv_title).size(9).color(TEXT_MUTED),
            ].spacing(3)).padding([8, 10]).style(stream_card_style);

            streams_col = streams_col.push(card);
        }

        content_col = content_col.push(container(streams_col).padding([12, 20]));
        content_col = content_col.push(
            container(iced::widget::Space::new()).width(Length::Fill).height(1)
                .style(|_: &Theme| container::Style { background: Some(iced::Background::Color(DIVIDER)), ..Default::default() })
        );
    }

    // System section
    let is_streaming = app.is_streaming();
    let status_val = if is_streaming { format!("{} active", app.active_streams.len()) } else { "Ready".into() };
    let system = container(column![
        text("System").size(11).color(TEXT_MUTED),
        iced::widget::Space::new().height(8),
        info_row("\u{25CB}", "Provider", provider_name.to_string()),
        info_row("\u{25CB}", "Model", format!("{icon} {model}")),
        info_row("\u{25CB}", "Status", status_val),
    ].spacing(6)).padding([12, 20]);
    content_col = content_col.push(system);

    content_col = content_col.push(
        container(iced::widget::Space::new()).width(Length::Fill).height(1)
            .style(|_: &Theme| container::Style { background: Some(iced::Background::Color(DIVIDER)), ..Default::default() })
    );

    // Resources section with cost
    let conv_cost = crate::cost::conversation_cost(&conv.messages);
    let cost_str = if conv_cost > 0.0001 { format!("${:.4}", conv_cost) } else { "Free".to_string() };
    let resources = container(column![
        text("Resources").size(11).color(TEXT_MUTED),
        iced::widget::Space::new().height(8),
        info_row("\u{25CB}", "Conversations", conv_count.to_string()),
        info_row("\u{25CB}", "Messages", msg_count.to_string()),
        info_row("\u{25CB}", "Est. Cost", cost_str),
        info_row("\u{25CB}", "Ollama", format!("{} models", app.config.ollama_models.len())),
    ].spacing(6)).padding([12, 20]);
    content_col = content_col.push(resources);

    content_col = content_col.push(iced::widget::Space::new().height(Length::Fill));

    // Shortcuts section
    content_col = content_col.push(
        container(iced::widget::Space::new()).width(Length::Fill).height(1)
            .style(|_: &Theme| container::Style { background: Some(iced::Background::Color(DIVIDER)), ..Default::default() })
    );
    let mut shortcut_col = column![
        row![
            text("Shortcuts").size(12).color(TEXT_HEAD),
            iced::widget::Space::new().width(Length::Fill),
            button(text("?").size(11).color(TEXT_MUTED))
                .on_press(Message::ToggleShortcutHelp)
                .padding([2, 8])
                .style(|_: &Theme, status: button::Status| button::Style {
                    background: Some(iced::Background::Color(match status {
                        button::Status::Hovered => BG_HOVER,
                        _ => BG_ACTIVE,
                    })),
                    text_color: TEXT_MUTED,
                    border: Border { radius: 8.0.into(), width: 1.0, color: BORDER_DEFAULT },
                    ..Default::default()
                }),
        ].align_y(iced::Alignment::Center),
        iced::widget::Space::new().height(8),
    ]
    .spacing(6);
    for (binding, label) in commands::shortcut_rows(&app.config.keybindings) {
        shortcut_col = shortcut_col.push(
            row![
                text(binding).size(10).color(TEXT_MUTED).font(iced::Font::MONOSPACE),
                iced::widget::Space::new().width(Length::Fill),
                text(label).size(10).color(TEXT_SEC),
            ]
        );
    }
    let shortcuts = container(shortcut_col).padding([12, 20]);
    content_col = content_col.push(shortcuts);

    let content_col = content_col.height(Length::Fill);

    container(content_col)
        .width(260)
        .height(Length::Fill)
        .style(|_: &Theme| container::Style { background: Some(iced::Background::Color(BG)), ..Default::default() })
        .into()
}
