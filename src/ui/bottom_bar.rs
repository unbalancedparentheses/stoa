use iced::widget::{container, row, text};
use iced::{Alignment, Element, Length, Theme};

use crate::app::{ChatApp, Message};
use crate::theme::*;

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let cost_text = if app.session_cost > 0.001 {
        format!("${:.4}", app.session_cost)
    } else {
        String::new()
    };

    let bar = row![
        text(cost_text).size(10).color(TEXT_MUTED).font(iced::Font::MONOSPACE),
        iced::widget::Space::new().width(Length::Fill),
        text("Ctrl+K").size(9).color(TEXT_MUTED).font(iced::Font::MONOSPACE),
        text(" Search").size(9).color(TEXT_SEC),
        iced::widget::Space::new().width(16),
        text("Ctrl+P").size(9).color(TEXT_MUTED).font(iced::Font::MONOSPACE),
        text(" Commands").size(9).color(TEXT_SEC),
    ].align_y(Alignment::Center);

    container(
        iced::widget::column![
            container(iced::widget::Space::new()).width(Length::Fill).height(1)
                .style(|_: &Theme| container::Style {
                    background: Some(iced::Background::Color(DIVIDER)),
                    ..Default::default()
                }),
            container(bar).padding([5, 16]),
        ]
    )
    .width(Length::Fill)
    .style(|_: &Theme| container::Style {
        background: Some(iced::Background::Color(BAR_BG)),
        ..Default::default()
    })
    .into()
}
