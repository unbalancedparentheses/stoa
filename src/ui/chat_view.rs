use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{Alignment, Element, Length, Color, Border, Theme};

use crate::app::{ChatApp, Message};
use crate::model::Role;

const MAIN_BG: Color = Color::from_rgb8(0x16, 0x1e, 0x2a);
const HEADER_BG: Color = Color::from_rgb8(0x14, 0x1c, 0x26);
const TEXT_HEAD: Color = Color::from_rgb8(0xe8, 0xe0, 0xd0);
const TEXT_BODY: Color = Color::from_rgb8(0xd0, 0xc8, 0xb8);
const TEXT_SEC: Color = Color::from_rgb8(0x8a, 0x90, 0x9a);
const TEXT_MUTED: Color = Color::from_rgb8(0x50, 0x5a, 0x66);
const ACCENT: Color = Color::from_rgb8(0xc9, 0xa8, 0x4c);
const DIVIDER: Color = Color::from_rgb8(0x1e, 0x28, 0x34);
const USER_BG: Color = Color::from_rgb8(0x1e, 0x28, 0x36);

fn user_bubble(_: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(USER_BG)),
        border: Border { radius: 12.0.into(), ..Default::default() },
        ..Default::default()
    }
}

fn action_btn_style(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::TRANSPARENT)),
        text_color: match status {
            button::Status::Hovered => TEXT_SEC,
            _ => TEXT_MUTED,
        },
        ..Default::default()
    }
}

fn error_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(0x2a, 0x18, 0x18))),
        border: Border { radius: 8.0.into(), width: 1.0, color: Color::from_rgb8(0x44, 0x22, 0x22) },
        ..Default::default()
    }
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let conv = &app.conversations[app.active_conversation];
    let model = &app.config.active_provider_config().model;

    // Header
    let chat_header = container(
        row![
            text("Home").size(15).color(TEXT_HEAD),
        ].align_y(Alignment::Center)
    )
    .width(Length::Fill)
    .padding([14, 28])
    .style(|_: &Theme| container::Style {
        background: Some(iced::Background::Color(HEADER_BG)),
        ..Default::default()
    });

    let mut messages = Column::new().spacing(20).padding([24, 36]);

    if conv.messages.is_empty() && !app.is_streaming {
        messages = messages.push(
            container(
                column![
                    text("rust-chat").size(24).color(TEXT_HEAD),
                    text("Share a thought to get started.").size(13).color(TEXT_MUTED),
                    iced::widget::Space::new().height(8),
                    text(format!("Model: {model}")).size(11).color(TEXT_MUTED).font(iced::Font::MONOSPACE),
                ].spacing(4)
            ).padding([48, 0])
        );
    }

    let msg_count = conv.messages.len();
    let mut prev_role: Option<&Role> = None;
    for msg in &conv.messages {
        let same_role = prev_role == Some(&msg.role);

        match msg.role {
            Role::User => {
                let mut col = Column::new().spacing(4);
                if !same_role {
                    col = col.push(
                        container(text("You").size(11).color(TEXT_SEC))
                            .width(Length::Fill).align_x(Alignment::End)
                    );
                }
                col = col.push(
                    container(
                        container(
                            text(msg.content.clone()).size(15).color(TEXT_HEAD)
                        ).padding([12, 16]).max_width(600).style(user_bubble)
                    ).width(Length::Fill).align_x(Alignment::End)
                );
                messages = messages.push(container(col).width(Length::Fill));
            }
            Role::Assistant => {
                let mut col = Column::new().spacing(6);

                // Model info line (like Noēma)
                if !same_role {
                    col = col.push(
                        text(format!("({model}) reading {msg_count} messages"))
                            .size(11).color(TEXT_MUTED)
                    );
                }

                // Plain text (no bubble, like Noēma)
                col = col.push(
                    text(msg.content.clone()).size(14).line_height(1.7).color(TEXT_BODY)
                );

                if msg.streaming {
                    col = col.push(text("\u{2022}\u{2022}\u{2022}").size(14).color(ACCENT));
                }

                // Action row (copy, thumbs up, thumbs down, ...)
                if !msg.streaming {
                    col = col.push(
                        row![
                            button(text("\u{2398}").size(12)).padding([2, 6]).style(action_btn_style),
                            button(text("\u{25B3}").size(12)).padding([2, 6]).style(action_btn_style),
                            button(text("\u{25BD}").size(12)).padding([2, 6]).style(action_btn_style),
                            button(text("\u{2026}").size(12)).padding([2, 6]).style(action_btn_style),
                        ].spacing(4)
                    );
                }

                messages = messages.push(container(col).width(Length::Fill));
            }
        }
        prev_role = Some(&msg.role);
    }

    if app.is_streaming && app.current_response.is_empty() {
        messages = messages.push(column![
            text(format!("({model}) reading {msg_count} messages")).size(11).color(TEXT_MUTED),
            text("\u{2022}\u{2022}\u{2022}").size(16).color(ACCENT),
        ].spacing(6));
    }

    if let Some(ref err) = app.error_message {
        let dismiss = button(text("\u{00D7}").size(12))
            .on_press(Message::DismissError).padding([2, 8])
            .style(|_: &Theme, status: button::Status| button::Style {
                background: Some(iced::Background::Color(Color::TRANSPARENT)),
                text_color: match status {
                    button::Status::Hovered => Color::from_rgb8(0xda, 0x6b, 0x6b),
                    _ => Color::from_rgb8(0x88, 0x55, 0x55),
                },
                ..Default::default()
            });
        messages = messages.push(
            container(
                row![
                    text(err.to_string()).size(13).color(Color::from_rgb8(0xda, 0x6b, 0x6b)),
                    iced::widget::Space::new().width(Length::Fill),
                    dismiss,
                ].align_y(Alignment::Center)
            ).width(Length::Fill).padding([10, 14]).style(error_style)
        );
    }

    column![
        chat_header,
        container(iced::widget::Space::new()).width(Length::Fill).height(1)
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(DIVIDER)),
                ..Default::default()
            }),
        container(scrollable(messages).height(Length::Fill).anchor_bottom())
            .width(Length::Fill).height(Length::Fill)
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(MAIN_BG)),
                ..Default::default()
            }),
    ].into()
}
