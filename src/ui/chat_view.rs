use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{Alignment, Element, Length, Color, Border, Theme};

use crate::app::{ChatApp, Message};
use crate::model::Role;

const MAIN_BG: Color = Color::from_rgb8(0x13, 0x1a, 0x24);
const TEXT_PRIMARY: Color = Color::from_rgb8(0xe6, 0xed, 0xf3);
const TEXT_SECONDARY: Color = Color::from_rgb8(0x8b, 0x94, 0x9e);
const TEXT_MUTED: Color = Color::from_rgb8(0x4a, 0x54, 0x60);
const ACCENT: Color = Color::from_rgb8(0xc9, 0xa8, 0x4c);
const USER_BUBBLE: Color = Color::from_rgb8(0x1a, 0x25, 0x35);
const ASST_BUBBLE: Color = Color::from_rgb8(0x15, 0x1d, 0x28);
const HEADER_BG: Color = Color::from_rgb8(0x11, 0x19, 0x22);
const BORDER_SUBTLE: Color = Color::from_rgb8(0x1e, 0x2a, 0x38);

fn user_bubble_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(USER_BUBBLE)),
        border: Border {
            radius: 12.0.into(),
            width: 1.0,
            color: Color::from_rgb8(0x22, 0x30, 0x42),
        },
        ..Default::default()
    }
}

fn assistant_bubble_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(ASST_BUBBLE)),
        border: Border {
            radius: 12.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn error_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(0x2a, 0x18, 0x18))),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: Color::from_rgb8(0x44, 0x22, 0x22),
        },
        ..Default::default()
    }
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let conv = &app.conversations[app.active_conversation];

    // Chat header
    let title = conv.title.clone();
    let model = &app.config.active_provider_config().model;
    let chat_header = container(
        row![
            text(title).size(15).color(TEXT_PRIMARY),
            iced::widget::Space::new().width(Length::Fill),
            text(model.clone()).size(11).color(TEXT_MUTED).font(iced::Font::MONOSPACE),
        ].align_y(Alignment::Center)
    )
    .width(Length::Fill)
    .padding([14, 24])
    .style(|_: &Theme| container::Style {
        background: Some(iced::Background::Color(HEADER_BG)),
        ..Default::default()
    });

    let mut messages = Column::new().spacing(16).padding([20, 32]);

    if conv.messages.is_empty() && !app.is_streaming {
        let empty_state = container(
            column![
                text("rust-chat").size(28).color(TEXT_PRIMARY),
                iced::widget::Space::new().height(4),
                text("Share a thought...").size(14).color(TEXT_MUTED),
                iced::widget::Space::new().height(12),
                text(format!("Model: {model}")).size(12).color(TEXT_MUTED).font(iced::Font::MONOSPACE),
            ]
            .spacing(0)
            .align_x(Alignment::Start)
        )
        .padding([60, 0]);

        messages = messages.push(empty_state);
    }

    let mut prev_role: Option<&Role> = None;
    for msg in &conv.messages {
        let same_role = prev_role == Some(&msg.role);

        let content_text = text(msg.content.clone())
            .size(14)
            .line_height(1.6)
            .color(TEXT_PRIMARY);

        let bubble = container(content_text)
            .padding([10, 14])
            .max_width(720);

        let aligned: Element<Message> = match msg.role {
            Role::User => {
                let mut col = Column::new().spacing(4);
                if !same_role {
                    col = col.push(
                        container(
                            text("You").size(11).color(TEXT_SECONDARY)
                        )
                        .width(Length::Fill)
                        .align_x(Alignment::End)
                    );
                }
                col = col.push(
                    container(bubble.style(user_bubble_style))
                        .width(Length::Fill)
                        .align_x(Alignment::End)
                );
                container(col).width(Length::Fill).into()
            }
            Role::Assistant => {
                let styled = bubble.style(assistant_bubble_style);
                let mut col = Column::new().spacing(4);
                if !same_role {
                    col = col.push(
                        text("Assistant").size(11).color(TEXT_SECONDARY)
                    );
                }
                col = col.push(styled);
                if msg.streaming {
                    col = col.push(
                        text("\u{2022}\u{2022}\u{2022}").size(14).color(ACCENT)
                    );
                }
                // Model indicator under assistant messages
                if !msg.streaming {
                    col = col.push(
                        text(format!("{model} \u{00B7} just now")).size(10).color(TEXT_MUTED)
                    );
                }
                container(col).width(Length::Fill).into()
            }
        };

        messages = messages.push(aligned);
        prev_role = Some(&msg.role);
    }

    if app.is_streaming && app.current_response.is_empty() {
        messages = messages.push(
            column![
                text("Assistant").size(11).color(TEXT_SECONDARY),
                container(
                    text("\u{2022}\u{2022}\u{2022}").size(16).color(ACCENT)
                )
                .padding([10, 14])
                .style(assistant_bubble_style),
                text(format!("{model} \u{00B7} just now")).size(10).color(TEXT_MUTED),
            ].spacing(4)
        );
    }

    if let Some(ref err) = app.error_message {
        let dismiss = button(text("\u{2715}").size(11))
            .on_press(Message::DismissError)
            .padding([2, 8])
            .style(|_: &Theme, status: button::Status| {
                let fg = match status {
                    button::Status::Hovered => Color::from_rgb8(0xda, 0x6b, 0x6b),
                    _ => Color::from_rgb8(0x88, 0x55, 0x55),
                };
                button::Style {
                    background: Some(iced::Background::Color(Color::TRANSPARENT)),
                    text_color: fg,
                    ..Default::default()
                }
            });
        messages = messages.push(
            container(
                row![
                    text(err.to_string()).size(13).color(Color::from_rgb8(0xda, 0x6b, 0x6b)),
                    iced::widget::Space::new().width(Length::Fill),
                    dismiss,
                ].align_y(Alignment::Center)
            )
            .width(Length::Fill)
            .padding([10, 14])
            .style(error_style),
        );
    }

    column![
        chat_header,
        container(iced::widget::Space::new()).width(Length::Fill).height(1)
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(BORDER_SUBTLE)),
                ..Default::default()
            }),
        container(
            scrollable(messages)
                .height(Length::Fill)
                .anchor_bottom()
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &Theme| container::Style {
            background: Some(iced::Background::Color(MAIN_BG)),
            ..Default::default()
        }),
    ].into()
}
