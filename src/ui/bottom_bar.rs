use iced::widget::{button, container, row, text};
use iced::{Alignment, Element, Length, Border, Theme};

use crate::app::Message;
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

pub fn view() -> Element<'static, Message> {
    let left = row![
        button(text("\u{2302} Home").size(11)).padding([4, 8]).style(bar_btn_style).on_press(Message::ShowChat),
        button(text("\u{2699} Settings").size(11)).padding([4, 8]).style(bar_btn_style).on_press(Message::ShowSettings),
    ].spacing(4).align_y(Alignment::Center);

    let bar = row![
        left,
        iced::widget::Space::new().width(Length::Fill),
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
