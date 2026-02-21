use iced::widget::{button, column, container, scrollable, text, text_input, row, Column};
use iced::{Alignment, Element, Length, Border, Theme};

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

fn input_style(_: &Theme, status: iced::widget::text_input::Status) -> iced::widget::text_input::Style {
    iced::widget::text_input::Style {
        background: iced::Background::Color(INPUT_BG),
        border: Border { radius: 8.0.into(), width: 1.0, color: match status {
            iced::widget::text_input::Status::Focused { .. } => ACCENT,
            _ => BORDER_DEFAULT,
        }},
        icon: TEXT_MUTED,
        placeholder: TEXT_MUTED,
        value: TEXT_HEAD,
        selection: SELECTION,
    }
}

fn cmd_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let bg = match (active, status) {
            (true, _) => BG_ACTIVE,
            (false, button::Status::Hovered) => BG_HOVER,
            _ => iced::Color::TRANSPARENT,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: if active { TEXT_HEAD } else {
                match status {
                    button::Status::Hovered => TEXT_HEAD,
                    _ => TEXT_SEC,
                }
            },
            border: Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        }
    }
}

pub fn filtered_commands(app: &ChatApp) -> Vec<commands::CommandEntry> {
    commands::filtered_commands(&app.command_palette_query, &app.config.keybindings)
}

fn shortcuts_hint() -> Element<'static, Message> {
    row![
        text("↑↓").size(10).color(TEXT_MUTED).font(iced::Font::MONOSPACE),
        text("navigate").size(10).color(TEXT_MUTED),
        text("Enter").size(10).color(TEXT_MUTED).font(iced::Font::MONOSPACE),
        text("run").size(10).color(TEXT_MUTED),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let input = text_input("Type a command...", &app.command_palette_query)
        .on_input(Message::CommandPaletteQueryChanged)
        .on_submit(Message::CommandPaletteExecuteSelected)
        .id("command-palette-input")
        .size(14)
        .padding([10, 16])
        .style(input_style);

    let commands = filtered_commands(app);
    let mut results = Column::new().spacing(2);

    for (index, cmd) in commands.iter().enumerate() {
        let is_selected = index == app.command_palette_selected.min(commands.len().saturating_sub(1));
        let mut cmd_row = row![
            text(cmd.label).size(13).color(TEXT_HEAD),
            iced::widget::Space::new().width(8),
            text(cmd.description).size(11).color(TEXT_MUTED),
            iced::widget::Space::new().width(Length::Fill),
        ].spacing(4).align_y(Alignment::Center);

        if !cmd.shortcut.is_empty() {
            cmd_row = cmd_row.push(
                text(cmd.shortcut.clone()).size(10).color(TEXT_MUTED).font(iced::Font::MONOSPACE)
            );
        }

        results = results.push(
            button(cmd_row)
                .on_press(cmd.message.clone())
                .width(Length::Fill)
                .padding([8, 16])
                .style(cmd_style(is_selected))
        );
    }

    let modal = container(
        column![
            row![
                text("Command Palette").size(12).color(TEXT_MUTED),
                iced::widget::Space::new().width(Length::Fill),
                shortcuts_hint(),
            ].align_y(Alignment::Center),
            input,
            scrollable(results).height(300),
        ].spacing(8)
    ).width(550).padding(16).style(modal_style);

    container(modal)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(overlay_style)
        .into()
}
