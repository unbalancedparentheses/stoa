use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Element, Length, Color, Border, Theme};

use crate::app::{ChatApp, Message};

const MAIN_BG: Color = Color::from_rgb8(0x13, 0x1a, 0x24);
const INPUT_BG: Color = Color::from_rgb8(0x10, 0x17, 0x20);
const ACCENT: Color = Color::from_rgb8(0xc9, 0xa8, 0x4c);
const BORDER_DEFAULT: Color = Color::from_rgb8(0x2a, 0x3a, 0x4e);
const TEXT_PRIMARY: Color = Color::from_rgb8(0xe6, 0xed, 0xf3);
const TEXT_MUTED: Color = Color::from_rgb8(0x4a, 0x54, 0x60);

fn input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => ACCENT,
        _ => BORDER_DEFAULT,
    };
    text_input::Style {
        background: iced::Background::Color(INPUT_BG),
        border: Border {
            radius: 10.0.into(),
            width: 1.0,
            color: border_color,
        },
        icon: TEXT_MUTED,
        placeholder: TEXT_MUTED,
        value: TEXT_PRIMARY,
        selection: Color::from_rgb8(0x2a, 0x3e, 0x55),
    }
}

fn send_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => ACCENT,
        _ => Color::from_rgb8(0x3a, 0x50, 0x68),
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: TEXT_PRIMARY,
        border: Border {
            radius: 20.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn send_disabled_style(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(0x18, 0x22, 0x2e))),
        text_color: TEXT_MUTED,
        border: Border {
            radius: 20.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn icon_btn_style(_theme: &Theme, status: button::Status) -> button::Style {
    let fg = match status {
        button::Status::Hovered => ACCENT,
        _ => TEXT_MUTED,
    };
    button::Style {
        background: Some(iced::Background::Color(Color::TRANSPARENT)),
        text_color: fg,
        ..Default::default()
    }
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let input = text_input("Share a thought...", &app.input_value)
        .on_input(Message::InputChanged)
        .on_submit(Message::SendMessage)
        .padding([12, 16])
        .size(14)
        .style(input_style);

    let can_send = !app.input_value.trim().is_empty() && !app.is_streaming;
    let send = if can_send {
        button(
            container(text("\u{2191}").size(16))
                .align_x(Alignment::Center)
                .align_y(iced::alignment::Vertical::Center)
        )
            .on_press(Message::SendMessage)
            .width(36)
            .height(36)
            .style(send_style)
    } else {
        button(
            container(text("\u{2191}").size(16))
                .align_x(Alignment::Center)
                .align_y(iced::alignment::Vertical::Center)
        )
            .width(36)
            .height(36)
            .style(send_disabled_style)
    };

    // Icon buttons row (attachment, etc.)
    let attach_btn = button(text("\u{1F4CE}").size(14))
        .padding([4, 8])
        .style(icon_btn_style);

    let icons_row = row![attach_btn].spacing(8);

    let input_row = row![input, send]
        .spacing(8)
        .align_y(Alignment::Center);

    let bar = container(
        column![
            input_row,
            icons_row,
        ].spacing(6)
    );

    container(bar)
        .width(Length::Fill)
        .padding(iced::Padding { top: 12.0, right: 24.0, bottom: 16.0, left: 24.0 })
        .style(|_: &Theme| container::Style {
            background: Some(iced::Background::Color(MAIN_BG)),
            ..Default::default()
        })
        .into()
}
