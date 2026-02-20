use iced::widget::{column, container, row, text};
use iced::{Element, Length, Theme};

use crate::app::{ChatApp, Message};
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

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let icon = provider_icon(&app.selected_model);
    let provider_name = if app.selected_model.contains("claude") || app.selected_model.contains("haiku") || app.selected_model.contains("sonnet") || app.selected_model.contains("opus") {
        "Anthropic"
    } else {
        "OpenAI"
    };
    let model = short_model_name(&app.selected_model);
    let conv = &app.conversations[app.active_conversation];
    let msg_count = conv.messages.len();
    let conv_count = app.conversations.len();

    // Highlights header
    let header = container(column![
        text("Highlights").size(16).color(TEXT_HEAD),
        text("Newest updates").size(11).color(TEXT_MUTED),
    ].spacing(3)).padding(iced::Padding { top: 24.0, right: 20.0, bottom: 16.0, left: 20.0 });

    // System section
    let system = container(column![
        text("System").size(11).color(TEXT_MUTED),
        iced::widget::Space::new().height(8),
        info_row("\u{25CB}", "Provider", provider_name.to_string()),
        info_row("\u{25CB}", "Model", format!("{icon} {model}")),
        info_row("\u{25CB}", "Status", if app.is_streaming { "Streaming".into() } else { "Ready".into() }),
    ].spacing(6)).padding([12, 20]);

    // Resources section
    let resources = container(column![
        text("Resources").size(11).color(TEXT_MUTED),
        iced::widget::Space::new().height(8),
        info_row("\u{25CB}", "Conversations", conv_count.to_string()),
        info_row("\u{25CB}", "Messages", msg_count.to_string()),
        info_row("\u{25CB}", "Version", "v0.1.0".to_string()),
    ].spacing(6)).padding([12, 20]);

    // Shortcuts section
    let shortcuts = container(column![
        text("Shortcuts").size(12).color(TEXT_HEAD),
        iced::widget::Space::new().height(8),
        row![
            text("Enter").size(10).color(TEXT_MUTED).font(iced::Font::MONOSPACE),
            iced::widget::Space::new().width(Length::Fill),
            text("Send message").size(10).color(TEXT_SEC),
        ],
        row![
            text("Ctrl+N").size(10).color(TEXT_MUTED).font(iced::Font::MONOSPACE),
            iced::widget::Space::new().width(Length::Fill),
            text("New chat").size(10).color(TEXT_SEC),
        ],
    ].spacing(6)).padding([12, 20]);

    let content = column![
        header,
        container(iced::widget::Space::new()).width(Length::Fill).height(1)
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(DIVIDER)),
                ..Default::default()
            }),
        system,
        container(iced::widget::Space::new()).width(Length::Fill).height(1)
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(DIVIDER)),
                ..Default::default()
            }),
        resources,
        iced::widget::Space::new().height(Length::Fill),
        container(iced::widget::Space::new()).width(Length::Fill).height(1)
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(DIVIDER)),
                ..Default::default()
            }),
        shortcuts,
    ].height(Length::Fill);

    container(content)
        .width(260)
        .height(Length::Fill)
        .style(|_: &Theme| container::Style {
            background: Some(iced::Background::Color(BG)),
            ..Default::default()
        })
        .into()
}
