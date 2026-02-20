use iced::widget::{button, container, row, text};
use iced::{Alignment, Element, Length, Border, Theme};

use crate::app::{ChatApp, Message};
use crate::theme::*;

fn bar_btn_style(_: &Theme, status: button::Status) -> button::Style {
    let fg = match status {
        button::Status::Hovered => TEXT_SEC,
        _ => TEXT_MUTED,
    };
    button::Style {
        background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
        text_color: fg,
        border: Border { radius: 4.0.into(), ..Default::default() },
        ..Default::default()
    }
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let left = row![
        button(text("\u{2302} Home").size(11)).padding([4, 8]).style(bar_btn_style).on_press(Message::ShowChat),
        button(text("\u{2699} Settings").size(11)).padding([4, 8]).style(bar_btn_style).on_press(Message::ShowSettings),
    ].spacing(4).align_y(Alignment::Center);

    let cost_text = if app.session_cost > 0.001 {
        format!("Session: ${:.4}", app.session_cost)
    } else {
        String::new()
    };

    let right = row![
        text(cost_text).size(10).color(TEXT_MUTED).font(iced::Font::MONOSPACE),
        iced::widget::Space::new().width(8),
        text("Ctrl+K").size(10).color(TEXT_MUTED).font(iced::Font::MONOSPACE),
        iced::widget::Space::new().width(4),
        text("Search").size(10).color(TEXT_SEC),
        iced::widget::Space::new().width(12),
        text("Ctrl+P").size(10).color(TEXT_MUTED).font(iced::Font::MONOSPACE),
        iced::widget::Space::new().width(4),
        text("Commands").size(10).color(TEXT_SEC),
    ].align_y(Alignment::Center);

    let bar = row![
        left,
        iced::widget::Space::new().width(Length::Fill),
        right,
    ].align_y(Alignment::Center);

    container(
        iced::widget::column![
            container(iced::widget::Space::new()).width(Length::Fill).height(1)
                .style(|_: &Theme| container::Style {
                    background: Some(iced::Background::Color(DIVIDER)),
                    ..Default::default()
                }),
            container(bar).padding([6, 16]),
        ]
    )
    .width(Length::Fill)
    .style(|_: &Theme| container::Style {
        background: Some(iced::Background::Color(BAR_BG)),
        ..Default::default()
    })
    .into()
}
