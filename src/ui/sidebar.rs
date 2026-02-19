use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{Alignment, Element, Length, Color, Border, Theme};

use crate::app::{ChatApp, Message, View};

const BG: Color = Color::from_rgb8(0x12, 0x1a, 0x24);
const BG_HOVER: Color = Color::from_rgb8(0x1a, 0x24, 0x30);
const BG_ACTIVE: Color = Color::from_rgb8(0x1e, 0x28, 0x36);
const ACCENT: Color = Color::from_rgb8(0xc9, 0xa8, 0x4c);
const TEXT_HEAD: Color = Color::from_rgb8(0xe8, 0xe0, 0xd0);
const TEXT_SEC: Color = Color::from_rgb8(0x8a, 0x90, 0x9a);
const TEXT_MUTED: Color = Color::from_rgb8(0x50, 0x5a, 0x66);
const DIVIDER: Color = Color::from_rgb8(0x1e, 0x28, 0x34);

fn nav_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let bg = match (active, status) {
            (true, _) => BG_ACTIVE,
            (false, button::Status::Hovered) => BG_HOVER,
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: if active { ACCENT } else { TEXT_SEC },
            border: Border { radius: 6.0.into(), ..Default::default() },
            ..Default::default()
        }
    }
}

fn conv_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let bg = match (active, status) {
            (true, _) => BG_ACTIVE,
            (false, button::Status::Hovered) => BG_HOVER,
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: if active { TEXT_HEAD } else { TEXT_SEC },
            border: Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        }
    }
}

fn del_style(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::TRANSPARENT)),
        text_color: match status {
            button::Status::Hovered => Color::from_rgb8(0xda, 0x6b, 0x6b),
            _ => TEXT_MUTED,
        },
        ..Default::default()
    }
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let chat_active = matches!(app.view, View::Chat);
    let settings_active = matches!(app.view, View::Settings);

    // Workspace header
    let header = container(
        column![
            text("Workspace").size(11).color(TEXT_MUTED),
            row![
                text("rust-chat").size(16).color(TEXT_HEAD),
                text("  v0.1.0").size(11).color(TEXT_MUTED),
            ].align_y(Alignment::End),
        ].spacing(3)
    ).padding(iced::Padding { top: 24.0, right: 20.0, bottom: 16.0, left: 20.0 });

    // Navigation
    let nav_label = container(
        text("Navigation").size(11).color(TEXT_MUTED)
    ).padding(iced::Padding { top: 4.0, right: 20.0, bottom: 6.0, left: 20.0 });

    let nav = container(column![
        button(text("\u{2302}  Home").size(13))
            .on_press(Message::ShowChat)
            .width(Length::Fill).padding([8, 12])
            .style(nav_style(chat_active)),
        button(text("+  New Chat").size(13))
            .on_press(Message::NewConversation)
            .width(Length::Fill).padding([8, 12])
            .style(nav_style(false)),
        button(text("\u{2699}  Settings").size(13))
            .on_press(Message::ShowSettings)
            .width(Length::Fill).padding([8, 12])
            .style(nav_style(settings_active)),
    ].spacing(2)).padding([0, 10]);

    // History (conversations)
    let history_label = container(
        text("History").size(11).color(TEXT_MUTED)
    ).padding(iced::Padding { top: 16.0, right: 20.0, bottom: 6.0, left: 20.0 });

    let can_delete = app.conversations.len() > 1;
    let mut conv_list = Column::new().spacing(1).padding([0, 10]);
    for (i, conv) in app.conversations.iter().enumerate() {
        let is_active = i == app.active_conversation && chat_active;
        let title: String = conv.title.chars().take(22).collect();
        let title = if conv.title.len() > 22 { format!("{title}\u{2026}") } else { title };

        let content: Element<Message> = if can_delete {
            row![
                text(title).size(12).width(Length::Fill),
                button(text("\u{00D7}").size(11))
                    .on_press(Message::DeleteConversation(i))
                    .padding([2, 6]).style(del_style),
            ].align_y(Alignment::Center).into()
        } else {
            text(title).size(12).into()
        };

        conv_list = conv_list.push(
            button(content)
                .on_press(Message::SelectConversation(i))
                .width(Length::Fill).padding([6, 12])
                .style(conv_style(is_active)),
        );
    }

    // Status
    let status_text = if app.is_streaming { "Streaming..." } else { "All systems nominal" };
    let status = container(column![
        text("Status").size(11).color(TEXT_MUTED),
        text(status_text).size(11).color(if app.is_streaming { ACCENT } else { TEXT_SEC }),
        iced::widget::Space::new().height(4),
        row![
            text("Latency").size(10).color(TEXT_MUTED),
            iced::widget::Space::new().width(Length::Fill),
            text("--").size(10).color(TEXT_SEC).font(iced::Font::MONOSPACE),
        ],
    ].spacing(3)).padding([12, 20]);

    let content = column![
        header,
        nav_label,
        nav,
        history_label,
        scrollable(conv_list).height(Length::Fill),
        container(iced::widget::Space::new()).width(Length::Fill).height(1)
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(DIVIDER)),
                ..Default::default()
            }),
        status,
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
