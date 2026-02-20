use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{Alignment, Element, Length, Color, Border, Theme};

use crate::app::{ChatApp, Message};
use crate::config::AppConfig;
use crate::model::Role;
use crate::theme::*;
use crate::ui::input_bar::{short_model_name, provider_icon};
use crate::ui::markdown;

fn user_bubble(_: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(USER_BG)),
        border: Border { radius: 12.0.into(), ..Default::default() },
        ..Default::default()
    }
}

fn special_user_bubble(_: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(USER_BG)),
        border: Border {
            radius: 12.0.into(),
            width: 1.0,
            color: ACCENT_DIM,
        },
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

fn review_btn_style(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(match status {
            button::Status::Hovered => BG_HOVER,
            _ => Color::TRANSPARENT,
        })),
        text_color: match status {
            button::Status::Hovered => ACCENT,
            _ => TEXT_MUTED,
        },
        border: Border { radius: 4.0.into(), ..Default::default() },
        ..Default::default()
    }
}

fn review_chip_style(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(match status {
            button::Status::Hovered => ACCENT_DIM,
            _ => BG_ACTIVE,
        })),
        text_color: match status {
            button::Status::Hovered => ACCENT,
            _ => TEXT_SEC,
        },
        border: Border {
            radius: 10.0.into(),
            width: 1.0,
            color: BORDER_DEFAULT,
        },
        ..Default::default()
    }
}

fn error_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(ERROR_BG)),
        border: Border { radius: 8.0.into(), width: 1.0, color: ERROR_BORDER },
        ..Default::default()
    }
}

fn is_special_user_message(content: &str) -> bool {
    content.starts_with("[Review request]") || content.starts_with("[Analyze conversation]")
}

fn special_label(content: &str) -> &str {
    if content.starts_with("[Review request]") {
        "Review Request"
    } else if content.starts_with("[Analyze conversation]") {
        "Analyze Conversation"
    } else {
        "You"
    }
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let conv = &app.conversations[app.active_conversation];
    let streaming_model_display = app.streaming_model.as_deref().unwrap_or(&app.selected_model);

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
        let model = short_model_name(&app.selected_model);
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
    let last_assistant_idx = conv.messages.iter().rposition(|m| m.role == Role::Assistant && !m.streaming);
    for (i, msg) in conv.messages.iter().enumerate() {
        let same_role = prev_role == Some(&msg.role);

        match msg.role {
            Role::User => {
                let mut col = Column::new().spacing(4);
                let is_special = is_special_user_message(&msg.content);
                let label = special_label(&msg.content);

                if !same_role || is_special {
                    let label_color = if is_special { ACCENT } else { TEXT_SEC };
                    col = col.push(
                        container(text(label).size(11).color(label_color))
                            .width(Length::Fill).align_x(Alignment::End)
                    );
                }

                let bubble_style: fn(&Theme) -> container::Style = if is_special {
                    special_user_bubble
                } else {
                    user_bubble
                };

                col = col.push(
                    container(
                        container(
                            text(msg.content.clone()).size(15).color(TEXT_HEAD)
                        ).padding([12, 16]).max_width(600).style(bubble_style)
                    ).width(Length::Fill).align_x(Alignment::End)
                );
                // Delete button for user messages
                col = col.push(
                    container(
                        button(text("\u{00D7}").size(12)).padding([2, 6]).style(action_btn_style)
                            .on_press(Message::DeleteMessage(i))
                    ).width(Length::Fill).align_x(Alignment::End)
                );
                messages = messages.push(container(col).width(Length::Fill));
            }
            Role::Assistant => {
                let mut col = Column::new().spacing(6);

                // Show model name per message
                let model_label = match &msg.model {
                    Some(m) => {
                        let icon = provider_icon(m);
                        let name = short_model_name(m);
                        format!("{icon} {name}")
                    }
                    None => format!("({}) reading {msg_count} messages", short_model_name(&app.selected_model)),
                };

                if !same_role || msg.model.is_some() {
                    col = col.push(
                        text(model_label).size(11).color(TEXT_MUTED)
                    );
                }

                // Render markdown for assistant messages
                if msg.streaming {
                    col = col.push(
                        text(msg.content.clone()).size(14).line_height(1.7).color(TEXT_BODY)
                    );
                    col = col.push(text("\u{2022}\u{2022}\u{2022}").size(14).color(ACCENT));
                } else {
                    col = col.push(markdown::render_markdown(&msg.content));
                }

                // Action row
                if !msg.streaming {
                    let mut actions = row![
                        button(text("\u{2398}").size(12)).padding([2, 6]).style(action_btn_style)
                            .on_press(Message::CopyToClipboard(msg.content.clone())),
                    ].spacing(4);

                    // Retry on last assistant message
                    if last_assistant_idx == Some(i) && !app.is_streaming {
                        actions = actions.push(
                            button(text("\u{21BB}").size(12)).padding([2, 6]).style(action_btn_style)
                                .on_press(Message::RetryMessage)
                        );
                    }

                    // Review button
                    if !app.is_streaming {
                        actions = actions.push(
                            button(text("Review").size(11)).padding([2, 8]).style(review_btn_style)
                                .on_press(Message::ShowReviewPicker(i))
                        );
                    }

                    // Delete
                    actions = actions.push(
                        button(text("\u{00D7}").size(12)).padding([2, 6]).style(action_btn_style)
                            .on_press(Message::DeleteMessage(i))
                    );

                    col = col.push(actions);

                    // Inline review picker
                    if app.review_picker == Some(i) {
                        let current_model = msg.model.as_deref().unwrap_or("");
                        let mut picker = iced::widget::Row::new().spacing(6);
                        picker = picker.push(text("Review with:").size(11).color(TEXT_MUTED));
                        for (display, model_id) in AppConfig::available_models() {
                            if model_id == current_model {
                                continue; // exclude current model
                            }
                            let icon = provider_icon(model_id);
                            picker = picker.push(
                                button(text(format!("{icon} {display}")).size(10))
                                    .on_press(Message::ReviewWith(model_id.to_string()))
                                    .padding([3, 8])
                                    .style(review_chip_style)
                            );
                        }
                        picker = picker.push(
                            button(text("\u{00D7}").size(11)).padding([2, 6]).style(action_btn_style)
                                .on_press(Message::DismissReviewPicker)
                        );
                        col = col.push(container(picker).padding([4, 0]));
                    }
                }

                messages = messages.push(container(col).width(Length::Fill));
            }
        }
        prev_role = Some(&msg.role);
    }

    if app.is_streaming && app.current_response.is_empty() {
        let icon = provider_icon(streaming_model_display);
        let name = short_model_name(streaming_model_display);
        messages = messages.push(column![
            text(format!("{icon} {name} reading {msg_count} messages")).size(11).color(TEXT_MUTED),
            text("\u{2022}\u{2022}\u{2022}").size(16).color(ACCENT),
        ].spacing(6));
    }

    if let Some(ref err) = app.error_message {
        let dismiss = button(text("\u{00D7}").size(12))
            .on_press(Message::DismissError).padding([2, 8])
            .style(|_: &Theme, status: button::Status| button::Style {
                background: Some(iced::Background::Color(Color::TRANSPARENT)),
                text_color: match status {
                    button::Status::Hovered => DANGER,
                    _ => ERROR_MUTED,
                },
                ..Default::default()
            });
        messages = messages.push(
            container(
                row![
                    text(err.to_string()).size(13).color(DANGER),
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
