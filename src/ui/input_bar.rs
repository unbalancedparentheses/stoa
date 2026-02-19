use iced::widget::{button, container, row, text, text_input};
use iced::{Alignment, Element, Length, Color, Border, Theme};

use crate::app::{ChatApp, Message};

const MAIN_BG: Color = Color::from_rgb8(0x16, 0x1e, 0x2a);
const INPUT_BG: Color = Color::from_rgb8(0x12, 0x1a, 0x24);
const ACCENT: Color = Color::from_rgb8(0xc9, 0xa8, 0x4c);
const ACCENT_DIM: Color = Color::from_rgb8(0x6a, 0x5e, 0x3a);
const BORDER_DEFAULT: Color = Color::from_rgb8(0x3a, 0x4a, 0x5a);
const TEXT_HEAD: Color = Color::from_rgb8(0xe8, 0xe0, 0xd0);
const TEXT_MUTED: Color = Color::from_rgb8(0x50, 0x5a, 0x66);

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
        selection: Color::from_rgb8(0x2a, 0x3e, 0x55),
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

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let input = text_input("Share a thought...", &app.input_value)
        .on_input(Message::InputChanged)
        .on_submit(Message::SendMessage)
        .padding([12, 20])
        .size(14)
        .style(input_style);

    let can_send = !app.input_value.trim().is_empty() && !app.is_streaming;
    let send = if can_send {
        button(
            container(text("\u{2191}").size(14))
                .align_x(Alignment::Center)
                .align_y(iced::alignment::Vertical::Center)
        ).on_press(Message::SendMessage).width(36).height(36).style(send_style)
    } else {
        button(
            container(text("\u{2191}").size(14))
                .align_x(Alignment::Center)
                .align_y(iced::alignment::Vertical::Center)
        ).width(36).height(36).style(send_disabled_style)
    };

    let bar = container(
        row![
            text("+").size(16).color(TEXT_MUTED),
            input,
            send,
        ].spacing(10).align_y(Alignment::Center)
    );

    container(bar)
        .width(Length::Fill)
        .padding(iced::Padding { top: 12.0, right: 28.0, bottom: 20.0, left: 28.0 })
        .style(|_: &Theme| container::Style {
            background: Some(iced::Background::Color(MAIN_BG)),
            ..Default::default()
        })
        .into()
}
