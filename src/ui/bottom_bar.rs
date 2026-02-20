use iced::widget::{button, container, row, text};
use iced::{Alignment, Element, Length, Color, Border, Theme};

use crate::app::Message;

const BAR_BG: Color = Color::from_rgb8(0x10, 0x17, 0x20);
const TEXT_SEC: Color = Color::from_rgb8(0x8a, 0x90, 0x9a);
const TEXT_MUTED: Color = Color::from_rgb8(0x50, 0x5a, 0x66);
const DIVIDER: Color = Color::from_rgb8(0x1e, 0x28, 0x34);

fn bar_btn_style(_: &Theme, status: button::Status) -> button::Style {
    let fg = match status {
        button::Status::Hovered => TEXT_SEC,
        _ => TEXT_MUTED,
    };
    button::Style {
        background: Some(iced::Background::Color(Color::TRANSPARENT)),
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

    let right = row![
        button(text("\u{25A7} Left").size(11)).padding([4, 8]).style(bar_btn_style),
        button(text("\u{25A8} Right").size(11)).padding([4, 8]).style(bar_btn_style),
    ].spacing(4).align_y(Alignment::Center);

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
