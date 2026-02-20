use iced::widget::{button, column, container, rich_text, row, scrollable, span, text, Column, Row};
use iced::{Alignment, Element, Length, Color, Border, Theme};

use crate::app::{ChatApp, Message};
use crate::config::AppConfig;
use crate::model::{ChatMessage, Role};
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
        border: Border { radius: 12.0.into(), width: 1.0, color: ACCENT_DIM },
        ..Default::default()
    }
}

fn action_btn_style(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::TRANSPARENT)),
        text_color: match status { button::Status::Hovered => TEXT_SEC, _ => TEXT_MUTED },
        ..Default::default()
    }
}

fn review_btn_style(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(match status { button::Status::Hovered => BG_HOVER, _ => Color::TRANSPARENT })),
        text_color: match status { button::Status::Hovered => ACCENT, _ => TEXT_MUTED },
        border: Border { radius: 4.0.into(), ..Default::default() },
        ..Default::default()
    }
}

fn review_chip_style(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(match status { button::Status::Hovered => ACCENT_DIM, _ => BG_ACTIVE })),
        text_color: match status { button::Status::Hovered => ACCENT, _ => TEXT_SEC },
        border: Border { radius: 10.0.into(), width: 1.0, color: BORDER_DEFAULT },
        ..Default::default()
    }
}

fn stop_stream_style(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(match status { button::Status::Hovered => DANGER, _ => Color::from_rgb8(0x8a, 0x3a, 0x3a) })),
        text_color: TEXT_HEAD,
        border: Border { radius: 10.0.into(), ..Default::default() },
        ..Default::default()
    }
}

fn diff_btn_style(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(match status { button::Status::Hovered => ACCENT_DIM, _ => BG_ACTIVE })),
        text_color: match status { button::Status::Hovered => ACCENT, _ => TEXT_SEC },
        border: Border { radius: 10.0.into(), width: 1.0, color: BORDER_DEFAULT },
        ..Default::default()
    }
}

fn comparison_col_style(_: &Theme) -> container::Style {
    container::Style {
        border: Border { radius: 8.0.into(), width: 1.0, color: BORDER_SUBTLE },
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
    if content.starts_with("[Review request]") { "Review Request" }
    else if content.starts_with("[Analyze conversation]") { "Analyze Conversation" }
    else { "You" }
}

/// Group messages into display groups for comparison mode.
enum DisplayGroup {
    User(usize),
    Assistants(Vec<usize>), // indices of consecutive assistant messages
}

fn group_messages(messages: &[ChatMessage]) -> Vec<DisplayGroup> {
    let mut groups = Vec::new();
    let mut i = 0;
    while i < messages.len() {
        if messages[i].role == Role::User {
            groups.push(DisplayGroup::User(i));
            i += 1;
        } else {
            let start = i;
            while i < messages.len() && messages[i].role == Role::Assistant {
                i += 1;
            }
            groups.push(DisplayGroup::Assistants((start..i).collect()));
        }
    }
    groups
}

fn render_assistant_message<'a>(
    app: &'a ChatApp,
    msg: &'a ChatMessage,
    i: usize,
    is_streaming: bool,
    last_assistant_idx: Option<usize>,
    show_model: bool,
    compact: bool,
) -> Column<'a, Message> {
    let msg_count = app.conversations[app.active_conversation].messages.len();
    let mut col = Column::new().spacing(6);

    if show_model {
        let model_label = match &msg.model {
            Some(m) => { let icon = provider_icon(m); let name = short_model_name(m); format!("{icon} {name}") }
            None => format!("({}) reading {msg_count} messages", short_model_name(&app.selected_model)),
        };
        col = col.push(text(model_label).size(11).color(TEXT_MUTED));
    }

    if msg.streaming {
        if msg.content.is_empty() {
            col = col.push(text("\u{2022}\u{2022}\u{2022}").size(14).color(ACCENT));
        } else {
            // Streaming markdown rendering
            col = col.push(markdown::render_markdown(&msg.content));
            col = col.push(text("\u{2022}\u{2022}\u{2022}").size(14).color(ACCENT));
        }
        if let Some((&stream_id, _)) = app.active_streams.iter().find(|(_, s)| s.message_index == i) {
            col = col.push(button(text("Stop").size(10)).padding([3, 8]).on_press(Message::StopStream(stream_id)).style(stop_stream_style));
        }
    } else {
        col = col.push(markdown::render_markdown(&msg.content));
    }

    if !msg.streaming {
        // Thumbs up/down + action buttons
        let up_color = if msg.rating > 0 { SUCCESS } else { TEXT_MUTED };
        let down_color = if msg.rating < 0 { DANGER } else { TEXT_MUTED };

        let mut actions = row![
            button(text("\u{25B2}").size(11).color(up_color)).padding([2, 5])
                .style(action_btn_style).on_press(Message::RateMessage(i, 1)),
            button(text("\u{25BC}").size(11).color(down_color)).padding([2, 5])
                .style(action_btn_style).on_press(Message::RateMessage(i, -1)),
            button(text("\u{2398}").size(12)).padding([2, 6]).style(action_btn_style)
                .on_press(Message::CopyToClipboard(msg.content.clone())),
            button(text("\u{2442}").size(12)).padding([2, 6]).style(action_btn_style)
                .on_press(Message::ForkConversation(i)),
        ].spacing(4);

        if last_assistant_idx == Some(i) && !is_streaming {
            actions = actions.push(button(text("\u{21BB}").size(12)).padding([2, 6]).style(action_btn_style).on_press(Message::RetryMessage));
        }
        if !is_streaming && !compact {
            actions = actions.push(button(text("Review").size(11)).padding([2, 8]).style(review_btn_style).on_press(Message::ShowReviewPicker(i)));
        }
        actions = actions.push(button(text("\u{00D7}").size(12)).padding([2, 6]).style(action_btn_style).on_press(Message::DeleteMessage(i)));
        col = col.push(actions);

        // Cost + latency info line
        let mut info_parts = Vec::new();
        if let Some(model_id) = &msg.model {
            let tokens = msg.token_count.unwrap_or_else(|| crate::cost::estimate_tokens(&msg.content));
            let cost = crate::cost::message_cost(model_id, &msg.role, tokens);
            if cost > 0.0 {
                info_parts.push(format!("~${:.4}", cost));
            }
            info_parts.push(format!("{} tok", tokens));
        }
        if let Some(lat) = msg.latency_ms {
            info_parts.push(format!("{lat} ms"));
        }
        if !info_parts.is_empty() {
            col = col.push(text(info_parts.join(" \u{00B7} ")).size(9).color(TEXT_MUTED));
        }

        // Review picker (only in non-compact mode)
        if !compact && app.review_picker == Some(i) {
            let current_model = msg.model.as_deref().unwrap_or("");
            let mut picker = iced::widget::Row::new().spacing(6);
            picker = picker.push(text("Review with:").size(11).color(TEXT_MUTED));
            for (display, model_id) in AppConfig::available_models() {
                if model_id == current_model { continue; }
                let icon = provider_icon(model_id);
                picker = picker.push(button(text(format!("{icon} {display}")).size(10)).on_press(Message::ReviewWith(model_id.to_string())).padding([3, 8]).style(review_chip_style));
            }
            picker = picker.push(button(text("\u{00D7}").size(11)).padding([2, 6]).style(action_btn_style).on_press(Message::DismissReviewPicker));
            col = col.push(container(picker).padding([4, 0]));
        }
    }

    col
}

fn render_diff_panel<'a>(app: &'a ChatApp) -> Option<Element<'a, Message>> {
    let (idx_a, idx_b) = app.diff_active?;
    let conv = &app.conversations[app.active_conversation];
    let msg_a = conv.messages.get(idx_a)?;
    let msg_b = conv.messages.get(idx_b)?;

    let segments = crate::diff::word_diff(&msg_a.content, &msg_b.content);
    let agreement = crate::diff::agreement_percentage(&msg_a.content, &msg_b.content);

    let model_a = msg_a.model.as_deref().map(short_model_name).unwrap_or("A");
    let model_b = msg_b.model.as_deref().map(short_model_name).unwrap_or("B");

    let mut spans_vec: Vec<iced::widget::text::Span<'a>> = Vec::new();
    for seg in &segments {
        match seg {
            crate::diff::DiffSegment::Common(t) => {
                spans_vec.push(span(format!("{t} ")).color(TEXT_BODY).size(13));
            }
            crate::diff::DiffSegment::OnlyA(t) => {
                spans_vec.push(span(format!("[{t}] ")).color(DIFF_A_TEXT).size(13));
            }
            crate::diff::DiffSegment::OnlyB(t) => {
                spans_vec.push(span(format!("[{t}] ")).color(DIFF_B_TEXT).size(13));
            }
        }
    }

    let diff_text = rich_text(spans_vec).wrapping(iced::widget::text::Wrapping::Word);

    let header = row![
        text(format!("Diff: {model_a} vs {model_b}")).size(12).color(TEXT_HEAD),
        iced::widget::Space::new().width(12),
        text(format!("{:.0}% agreement", agreement)).size(11).color(ACCENT),
        iced::widget::Space::new().width(Length::Fill),
        container(
            row![
                text(format!("{model_a} only")).size(10).color(DIFF_A_TEXT),
                iced::widget::Space::new().width(12),
                text(format!("{model_b} only")).size(10).color(DIFF_B_TEXT),
            ].spacing(4)
        ),
        iced::widget::Space::new().width(12),
        button(text("\u{00D7}").size(12)).padding([2, 6]).style(action_btn_style).on_press(Message::DismissDiff),
    ].align_y(Alignment::Center);

    let panel = container(
        column![header, diff_text].spacing(8)
    ).padding([12, 16]).width(Length::Fill).style(|_: &Theme| container::Style {
        background: Some(iced::Background::Color(CARD_BG)),
        border: Border { radius: 8.0.into(), width: 1.0, color: BORDER_DEFAULT },
        ..Default::default()
    });

    Some(panel.into())
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let conv = &app.conversations[app.active_conversation];
    let is_streaming = app.is_active_conv_streaming();

    // Header
    let compare_label = if app.comparison_mode { "Stack" } else { "Compare" };
    let compare_btn = button(text(compare_label).size(11))
        .on_press(Message::ToggleComparisonMode)
        .padding([4, 10])
        .style(|_: &Theme, status: button::Status| button::Style {
            background: Some(iced::Background::Color(match status { button::Status::Hovered => BG_HOVER, _ => BG_ACTIVE })),
            text_color: if app.comparison_mode { ACCENT } else { TEXT_SEC },
            border: Border { radius: 10.0.into(), width: 1.0, color: if app.comparison_mode { ACCENT_DIM } else { BORDER_DEFAULT } },
            ..Default::default()
        });

    let has_sys_prompt = !conv.system_prompt.is_empty();
    let sys_prompt_btn = button(
        text(if has_sys_prompt { "Sys \u{2713}" } else { "Sys" }).size(11)
    ).on_press(Message::ToggleConvSystemPrompt).padding([4, 10]).style(move |_: &Theme, status: button::Status| button::Style {
        background: Some(iced::Background::Color(match status { button::Status::Hovered => BG_HOVER, _ => BG_ACTIVE })),
        text_color: if has_sys_prompt { SUCCESS } else { TEXT_SEC },
        border: Border { radius: 10.0.into(), width: 1.0, color: if has_sys_prompt { Color::from_rgb8(0x24, 0x50, 0x3a) } else { BORDER_DEFAULT } },
        ..Default::default()
    });

    let chat_header = container(
        row![
            text("Home").size(15).color(TEXT_HEAD),
            iced::widget::Space::new().width(Length::Fill),
            sys_prompt_btn,
            iced::widget::Space::new().width(8),
            compare_btn,
        ].align_y(Alignment::Center)
    ).width(Length::Fill).padding([14, 28]).style(|_: &Theme| container::Style {
        background: Some(iced::Background::Color(HEADER_BG)),
        ..Default::default()
    });

    let mut messages_col = Column::new().spacing(20).padding([24, 36]);

    // System prompt editor
    if app.conv_system_prompt_open {
        let sys_input = iced::widget::text_input("System prompt for this conversation...", &app.conv_system_prompt_value)
            .on_input(Message::ConvSystemPromptChanged)
            .on_submit(Message::SaveConvSystemPrompt)
            .size(13).padding([10, 14])
            .style(|_: &Theme, status: iced::widget::text_input::Status| iced::widget::text_input::Style {
                background: iced::Background::Color(INPUT_BG),
                border: Border { radius: 8.0.into(), width: 1.0, color: match status {
                    iced::widget::text_input::Status::Focused { .. } => ACCENT,
                    _ => BORDER_DEFAULT,
                }},
                icon: TEXT_MUTED, placeholder: TEXT_MUTED, value: TEXT_HEAD, selection: SELECTION,
            });
        let save_btn = button(text("Save").size(11)).padding([6, 14]).on_press(Message::SaveConvSystemPrompt)
            .style(|_: &Theme, status: button::Status| button::Style {
                background: Some(iced::Background::Color(match status { button::Status::Hovered => ACCENT, _ => ACCENT_DIM })),
                text_color: TEXT_HEAD,
                border: Border { radius: 8.0.into(), ..Default::default() },
                ..Default::default()
            });
        messages_col = messages_col.push(
            container(column![
                text("System Prompt (this conversation)").size(11).color(TEXT_MUTED),
                row![sys_input, iced::widget::Space::new().width(8), save_btn].align_y(Alignment::Center),
            ].spacing(6)).padding(iced::Padding { top: 0.0, right: 0.0, bottom: 8.0, left: 0.0 })
        );
    }

    if conv.messages.is_empty() && !is_streaming {
        let model = short_model_name(&app.selected_model);
        messages_col = messages_col.push(
            container(column![
                text("Stoa").size(24).color(TEXT_HEAD),
                text("Share a thought to get started.").size(13).color(TEXT_MUTED),
                iced::widget::Space::new().height(8),
                text(format!("Model: {model}")).size(11).color(TEXT_MUTED).font(iced::Font::MONOSPACE),
            ].spacing(4)).padding([48, 0])
        );
    }

    let last_assistant_idx = conv.messages.iter().rposition(|m| m.role == Role::Assistant && !m.streaming);
    let groups = group_messages(&conv.messages);

    for group in &groups {
        match group {
            DisplayGroup::User(i) => {
                let msg = &conv.messages[*i];
                let mut col = Column::new().spacing(4);
                let is_special = is_special_user_message(&msg.content);
                let label = special_label(&msg.content);
                let label_color = if is_special { ACCENT } else { TEXT_SEC };
                col = col.push(container(text(label).size(11).color(label_color)).width(Length::Fill).align_x(Alignment::End));

                let bubble_style: fn(&Theme) -> container::Style = if is_special { special_user_bubble } else { user_bubble };
                col = col.push(
                    container(container(text(msg.content.clone()).size(15).color(TEXT_HEAD)).padding([12, 16]).max_width(600).style(bubble_style))
                        .width(Length::Fill).align_x(Alignment::End)
                );
                col = col.push(container(button(text("\u{00D7}").size(12)).padding([2, 6]).style(action_btn_style).on_press(Message::DeleteMessage(*i))).width(Length::Fill).align_x(Alignment::End));
                messages_col = messages_col.push(container(col).width(Length::Fill));
            }
            DisplayGroup::Assistants(indices) => {
                let use_comparison = app.comparison_mode && indices.len() >= 2;

                if use_comparison {
                    // Side-by-side rendering
                    let mut cols_row = Row::new().spacing(12);
                    for &i in indices {
                        let msg = &conv.messages[i];
                        let col = render_assistant_message(app, msg, i, is_streaming, last_assistant_idx, true, true);
                        cols_row = cols_row.push(
                            container(col).width(Length::Fill).padding([8, 10]).style(comparison_col_style)
                        );
                    }
                    messages_col = messages_col.push(container(cols_row).width(Length::Fill));

                    // Diff button for exactly 2 messages
                    if indices.len() == 2 {
                        let is_diff_active = app.diff_active == Some((indices[0], indices[1]));
                        let diff_label = if is_diff_active { "Hide Diff" } else { "Show Diff" };
                        let model_a = conv.messages[indices[0]].model.as_deref().map(short_model_name).unwrap_or("A");
                        let model_b = conv.messages[indices[1]].model.as_deref().map(short_model_name).unwrap_or("B");
                        let btn_msg = if is_diff_active { Message::DismissDiff } else { Message::ShowDiff(indices[0], indices[1]) };

                        messages_col = messages_col.push(
                            row![
                                button(text(format!("{diff_label} ({model_a} vs {model_b})")).size(10))
                                    .padding([4, 10]).style(diff_btn_style).on_press(btn_msg),
                            ]
                        );
                    }

                    // Diff panel
                    if let Some(panel) = render_diff_panel(app) {
                        messages_col = messages_col.push(panel);
                    }
                } else {
                    // Stacked rendering (default)
                    let mut prev_was_assistant = false;
                    for &i in indices {
                        let msg = &conv.messages[i];
                        let show_model = !prev_was_assistant || msg.model.is_some();
                        let col = render_assistant_message(app, msg, i, is_streaming, last_assistant_idx, show_model, false);
                        messages_col = messages_col.push(container(col).width(Length::Fill));
                        prev_was_assistant = true;
                    }
                }
            }
        }
    }

    if let Some(ref err) = app.error_message {
        let dismiss = button(text("\u{00D7}").size(12)).on_press(Message::DismissError).padding([2, 8])
            .style(|_: &Theme, status: button::Status| button::Style {
                background: Some(iced::Background::Color(Color::TRANSPARENT)),
                text_color: match status { button::Status::Hovered => DANGER, _ => ERROR_MUTED },
                ..Default::default()
            });
        messages_col = messages_col.push(
            container(row![
                text(err.to_string()).size(13).color(DANGER),
                iced::widget::Space::new().width(Length::Fill),
                dismiss,
            ].align_y(Alignment::Center)).width(Length::Fill).padding([10, 14]).style(error_style)
        );
    }

    column![
        chat_header,
        container(iced::widget::Space::new()).width(Length::Fill).height(1)
            .style(|_: &Theme| container::Style { background: Some(iced::Background::Color(DIVIDER)), ..Default::default() }),
        container(scrollable(messages_col).height(Length::Fill).anchor_bottom())
            .width(Length::Fill).height(Length::Fill)
            .style(|_: &Theme| container::Style { background: Some(iced::Background::Color(MAIN_BG)), ..Default::default() }),
    ].into()
}
