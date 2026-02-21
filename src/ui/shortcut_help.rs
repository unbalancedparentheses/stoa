use iced::widget::{column, container, row, scrollable, text};
use iced::{Alignment, Border, Element, Length, Theme};

use crate::app::{ChatApp, Message};
use crate::commands;
use crate::theme::*;

fn modal_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(CARD_BG)),
        border: Border { radius: 12.0.into(), width: 1.0, color: BORDER_DEFAULT },
        ..Default::default()
    }
}

fn overlay_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(OVERLAY_BG)),
        ..Default::default()
    }
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let mut rows = column![
        text("Keyboard Shortcuts").size(14).color(TEXT_HEAD),
        text("Press Esc to close").size(11).color(TEXT_MUTED),
        iced::widget::Space::new().height(8),
    ]
    .spacing(6);

    for (binding, label) in commands::shortcut_rows(&app.config.keybindings) {
        rows = rows.push(
            row![
                text(binding).size(11).color(TEXT_MUTED).font(iced::Font::MONOSPACE),
                iced::widget::Space::new().width(Length::Fill),
                text(label).size(11).color(TEXT_SEC),
            ]
            .align_y(Alignment::Center)
        );
    }

    let modal = container(scrollable(rows)).width(460).padding(18).style(modal_style);

    container(modal)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(overlay_style)
        .into()
}
