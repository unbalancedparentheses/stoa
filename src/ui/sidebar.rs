use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{Alignment, Element, Length, Color, Border, Theme};

use crate::app::{ChatApp, Message, View};
use crate::model::Provider;

const BG: Color = Color::from_rgb8(0x0c, 0x12, 0x19);
const BG_HOVER: Color = Color::from_rgb8(0x14, 0x1c, 0x26);
const BG_ACTIVE: Color = Color::from_rgb8(0x16, 0x20, 0x2c);
const ACCENT: Color = Color::from_rgb8(0xc9, 0xa8, 0x4c);
const TEXT_PRIMARY: Color = Color::from_rgb8(0xe6, 0xed, 0xf3);
const TEXT_SECONDARY: Color = Color::from_rgb8(0x8b, 0x94, 0x9e);
const TEXT_MUTED: Color = Color::from_rgb8(0x4a, 0x54, 0x60);
const BORDER_SUBTLE: Color = Color::from_rgb8(0x1e, 0x2a, 0x38);

fn nav_item_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let bg = match (active, status) {
            (true, _) => BG_ACTIVE,
            (false, button::Status::Hovered) => BG_HOVER,
            _ => Color::TRANSPARENT,
        };
        let fg = if active { ACCENT } else { TEXT_SECONDARY };
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

fn conv_item_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let bg = match (active, status) {
            (true, _) => BG_ACTIVE,
            (false, button::Status::Hovered) => BG_HOVER,
            _ => Color::TRANSPARENT,
        };
        let fg = if active { TEXT_PRIMARY } else { TEXT_SECONDARY };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: fg,
            border: Border {
                radius: 6.0.into(),
                width: if active { 1.0 } else { 0.0 },
                color: if active { ACCENT } else { Color::TRANSPARENT },
            },
            ..Default::default()
        }
    }
}

fn delete_style(_theme: &Theme, status: button::Status) -> button::Style {
    let fg = match status {
        button::Status::Hovered => Color::from_rgb8(0xda, 0x6b, 0x6b),
        _ => TEXT_MUTED,
    };
    button::Style {
        background: Some(iced::Background::Color(Color::TRANSPARENT)),
        text_color: fg,
        ..Default::default()
    }
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    // Header: "Chats" + subtitle
    let header = container(
        column![
            text("Chats").size(20).color(TEXT_PRIMARY),
            text("Conversation history").size(12).color(TEXT_MUTED),
        ].spacing(4)
    ).padding(iced::Padding { top: 20.0, right: 16.0, bottom: 12.0, left: 16.0 });

    // Navigation section
    let nav_label = container(
        text("Navigation").size(11).color(TEXT_MUTED)
    ).padding(iced::Padding { top: 8.0, right: 16.0, bottom: 6.0, left: 16.0 });

    let settings_active = matches!(app.view, View::Settings);
    let chat_active = matches!(app.view, View::Chat);

    let new_chat_btn = button(
        text("+  New Chat").size(13)
    )
        .on_press(Message::NewConversation)
        .width(Length::Fill)
        .padding([8, 16])
        .style(nav_item_style(false));

    let chat_btn = button(
        text("\u{2302}  Chat").size(13)
    )
        .on_press(Message::ShowChat)
        .width(Length::Fill)
        .padding([8, 16])
        .style(nav_item_style(chat_active));

    let settings_btn = button(
        text("\u{2699}  Settings").size(13)
    )
        .on_press(Message::ShowSettings)
        .width(Length::Fill)
        .padding([8, 16])
        .style(nav_item_style(settings_active));

    let nav = container(
        column![new_chat_btn, chat_btn, settings_btn].spacing(2)
    ).padding([0, 8]);

    // History section
    let history_label = container(
        text("History").size(11).color(TEXT_MUTED)
    ).padding(iced::Padding { top: 16.0, right: 16.0, bottom: 6.0, left: 16.0 });

    let can_delete = app.conversations.len() > 1;
    let mut conv_list = Column::new().spacing(1).padding([0, 8]);
    for (i, conv) in app.conversations.iter().enumerate() {
        let is_active = i == app.active_conversation && chat_active;
        let title: String = conv.title.chars().take(24).collect();
        let truncated = if conv.title.len() > 24 {
            format!("{title}\u{2026}")
        } else {
            title
        };

        // Derive simple tags from title words
        let tags: String = conv.title
            .split_whitespace()
            .take(3)
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .join(", ");

        let title_col = column![
            text(truncated).size(13),
            text(tags).size(10).color(TEXT_MUTED),
        ].spacing(2);

        let content: Element<Message> = if can_delete {
            let del = button(text("\u{2715}").size(10))
                .on_press(Message::DeleteConversation(i))
                .padding([2, 6])
                .style(delete_style);
            row![
                title_col.width(Length::Fill),
                del,
            ].align_y(Alignment::Center).into()
        } else {
            title_col.into()
        };

        let btn = button(content)
            .on_press(Message::SelectConversation(i))
            .width(Length::Fill)
            .padding([8, 12])
            .style(conv_item_style(is_active));
        conv_list = conv_list.push(btn);
    }

    // Status bar at bottom
    let provider_name = match app.config.active_provider {
        Provider::OpenAI => "OpenAI",
        Provider::Anthropic => "Anthropic",
    };
    let model = &app.config.active_provider_config().model;
    let status_text = if app.is_streaming { "Streaming..." } else { "Ready" };

    let status_bar = container(
        column![
            text("Status").size(11).color(TEXT_MUTED),
            text(status_text).size(11).color(if app.is_streaming { ACCENT } else { TEXT_SECONDARY }),
            iced::widget::Space::new().height(4),
            row![
                text("Provider").size(10).color(TEXT_MUTED),
                iced::widget::Space::new().width(Length::Fill),
                text(provider_name).size(10).color(TEXT_SECONDARY).font(iced::Font::MONOSPACE),
            ],
            row![
                text("Model").size(10).color(TEXT_MUTED),
                iced::widget::Space::new().width(Length::Fill),
                text(model.clone()).size(10).color(TEXT_SECONDARY).font(iced::Font::MONOSPACE),
            ],
        ].spacing(3)
    ).padding([12, 16]);

    let content = column![
        header,
        nav_label,
        nav,
        history_label,
        scrollable(conv_list).height(Length::Fill),
        container(iced::widget::Space::new()).width(Length::Fill).height(1)
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(BORDER_SUBTLE)),
                ..Default::default()
            }),
        status_bar,
    ]
    .height(Length::Fill);

    container(content)
        .width(280)
        .height(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(BG)),
            ..Default::default()
        })
        .into()
}
