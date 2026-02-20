use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::{Alignment, Border, Element, Length, Color, Theme};

use crate::app::{ChatApp, Message, View};
use crate::config::AppConfig;
use crate::theme::*;
use crate::ui::input_bar::provider_icon;

fn nav_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let bg = match (active, status) {
            (true, _) => BG_ACTIVE,
            (false, button::Status::Hovered) => BG_HOVER,
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: if active { ACCENT } else { TEXT_SEC },
            border: Border { radius: 6.0.into(), ..Default::default() },
            ..Default::default()
        }
    }
}

fn conv_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let bg = match (active, status) {
            (true, _) => BG_ACTIVE,
            (false, button::Status::Hovered) => BG_HOVER,
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: if active { TEXT_HEAD } else { TEXT_SEC },
            border: Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        }
    }
}

fn del_style(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::TRANSPARENT)),
        text_color: match status {
            button::Status::Hovered => DANGER,
            _ => TEXT_MUTED,
        },
        ..Default::default()
    }
}

fn edit_style(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::TRANSPARENT)),
        text_color: match status {
            button::Status::Hovered => ACCENT,
            _ => TEXT_MUTED,
        },
        ..Default::default()
    }
}

fn analyze_style(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::TRANSPARENT)),
        text_color: match status {
            button::Status::Hovered => ACCENT,
            _ => TEXT_MUTED,
        },
        ..Default::default()
    }
}

fn analyze_chip_style(_: &Theme, status: button::Status) -> button::Style {
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

fn rename_input_style(_: &Theme, status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: iced::Background::Color(INPUT_BG),
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: match status {
                text_input::Status::Focused { .. } => ACCENT,
                _ => BORDER_DEFAULT,
            },
        },
        icon: TEXT_MUTED,
        placeholder: TEXT_MUTED,
        value: TEXT_HEAD,
        selection: SELECTION,
    }
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let chat_active = matches!(app.view, View::Chat);
    let settings_active = matches!(app.view, View::Settings);
    let analytics_active = matches!(app.view, View::Analytics);

    // Workspace header
    let header = container(
        column![
            text("Workspace").size(11).color(TEXT_MUTED),
            row![
                text("Stoa").size(16).color(TEXT_HEAD),
                text("  v0.1.0").size(11).color(TEXT_MUTED),
            ].align_y(Alignment::End),
        ].spacing(3)
    ).padding(iced::Padding { top: 24.0, right: 20.0, bottom: 16.0, left: 20.0 });

    // Navigation
    let nav_label = container(
        text("Navigation").size(11).color(TEXT_MUTED)
    ).padding(iced::Padding { top: 4.0, right: 20.0, bottom: 6.0, left: 20.0 });

    let nav = container(column![
        button(text("\u{2302}  Home").size(13))
            .on_press(Message::ShowChat)
            .width(Length::Fill).padding([8, 12])
            .style(nav_style(chat_active)),
        button(text("+  New Chat").size(13))
            .on_press(Message::NewConversation)
            .width(Length::Fill).padding([8, 12])
            .style(nav_style(false)),
        button(text("\u{2261}  Analytics").size(13))
            .on_press(Message::ShowAnalytics)
            .width(Length::Fill).padding([8, 12])
            .style(nav_style(analytics_active)),
        button(text("\u{2699}  Settings").size(13))
            .on_press(Message::ShowSettings)
            .width(Length::Fill).padding([8, 12])
            .style(nav_style(settings_active)),
    ].spacing(2)).padding([0, 10]);

    // Search
    let search_input = text_input("\u{1F50D} Search...", &app.sidebar_search_query)
        .on_input(Message::SidebarSearchChanged)
        .size(11)
        .padding([5, 10])
        .style(rename_input_style);
    let search_section = container(search_input).padding(iced::Padding { top: 12.0, right: 14.0, bottom: 4.0, left: 14.0 });

    // History (conversations)
    let history_label = container(
        text("History").size(11).color(TEXT_MUTED)
    ).padding(iced::Padding { top: 8.0, right: 20.0, bottom: 6.0, left: 20.0 });

    let can_delete = app.conversations.len() > 1;
    let mut conv_list = Column::new().spacing(1).padding([0, 10]);
    for (i, conv) in app.conversations.iter().enumerate() {
        // Filter by search results if active
        if let Some(ref results) = app.sidebar_search_results {
            if !results.contains(&conv.id) { continue; }
        }

        let is_active = i == app.active_conversation && chat_active;

        // Inline rename mode
        if app.renaming_conversation == Some(i) {
            let input = text_input("", &app.rename_value)
                .on_input(Message::RenameChanged)
                .on_submit(Message::FinishRename)
                .id("rename-input")
                .size(12)
                .padding([6, 12])
                .style(rename_input_style);
            conv_list = conv_list.push(input);
            continue;
        }

        let has_streams = app.conv_has_streams(&conv.id);
        let msg_count = conv.messages.iter().filter(|m| !m.streaming).count();

        let max_chars = if is_active { 14 } else { 20 };
        let title: String = conv.title.chars().take(max_chars).collect();
        let title = if conv.title.len() > max_chars { format!("{title}\u{2026}") } else { title };

        let mut action_row = iced::widget::Row::new().spacing(0).align_y(Alignment::Center);

        // Pin icon
        if conv.pinned {
            action_row = action_row.push(text("\u{25B6} ").size(8).color(ACCENT));
        }

        // Streaming indicator dot
        if has_streams {
            action_row = action_row.push(text("\u{25CF} ").size(8).color(ACCENT));
        }

        action_row = action_row.push(text(title).size(12).width(Length::Fill));

        // Message count badge
        if msg_count > 0 {
            action_row = action_row.push(text(format!("{msg_count}")).size(9).color(TEXT_MUTED));
            action_row = action_row.push(iced::widget::Space::new().width(4));
        }

        // Only show action icons on the active conversation
        if is_active {
            // Pin toggle
            action_row = action_row.push(
                button(text(if conv.pinned { "\u{25B6}" } else { "\u{25B7}" }).size(10))
                    .on_press(Message::TogglePin(i))
                    .padding([2, 4]).style(edit_style),
            );
            action_row = action_row.push(
                button(text("\u{1F50D}").size(10))
                    .on_press(Message::AnalyzeConversation(i))
                    .padding([2, 4]).style(analyze_style),
            );
            action_row = action_row.push(
                button(text("\u{270E}").size(11))
                    .on_press(Message::StartRename(i))
                    .padding([2, 4]).style(edit_style),
            );
            if can_delete {
                action_row = action_row.push(
                    button(text("\u{00D7}").size(11))
                        .on_press(Message::DeleteConversation(i))
                        .padding([2, 6]).style(del_style),
                );
            }
        }

        conv_list = conv_list.push(
            button(action_row)
                .on_press(Message::SelectConversation(i))
                .width(Length::Fill).padding([6, 12])
                .style(conv_style(is_active)),
        );

        // Show tags for active conversation
        if is_active && !conv.tags.is_empty() {
            let mut tag_row = iced::widget::Row::new().spacing(4);
            for tag in &conv.tags {
                tag_row = tag_row.push(
                    button(text(format!("#{tag}")).size(9))
                        .on_press(Message::RemoveTag(tag.clone()))
                        .padding([2, 6])
                        .style(analyze_chip_style)
                );
            }
            conv_list = conv_list.push(
                container(tag_row).padding(iced::Padding { top: 2.0, right: 0.0, bottom: 4.0, left: 12.0 })
            );
        }

        // Tag input for active conversation
        if is_active && app.tag_input_open {
            let tag_input = text_input("Add tag...", &app.tag_input_value)
                .on_input(Message::TagInputChanged)
                .on_submit(Message::SubmitTag)
                .size(10)
                .padding([4, 8])
                .style(rename_input_style);
            conv_list = conv_list.push(
                container(tag_input).padding(iced::Padding { top: 2.0, right: 0.0, bottom: 4.0, left: 12.0 })
            );
        }

        // + tag button for active conversation
        if is_active && !app.tag_input_open {
            conv_list = conv_list.push(
                container(
                    button(text("+ tag").size(9)).padding([2, 6]).style(analyze_style)
                        .on_press(Message::ToggleTagInput)
                ).padding(iced::Padding { top: 0.0, right: 0.0, bottom: 2.0, left: 12.0 })
            );
        }
    }

    // Status with latency
    let is_streaming = app.is_streaming();
    let stream_count = app.active_streams.len();
    let status_text = if stream_count > 1 {
        format!("Streaming {stream_count} models...")
    } else if is_streaming {
        "Streaming...".to_string()
    } else {
        "All systems nominal".to_string()
    };
    let latency_text = match app.last_latency_ms {
        Some(ms) => format!("{ms} ms"),
        None => "--".to_string(),
    };
    let status = container(column![
        text("Status").size(11).color(TEXT_MUTED),
        text(status_text).size(11).color(if is_streaming { ACCENT } else { TEXT_SEC }),
        iced::widget::Space::new().height(4),
        row![
            text("Latency").size(10).color(TEXT_MUTED),
            iced::widget::Space::new().width(Length::Fill),
            text(latency_text).size(10).color(TEXT_SEC).font(iced::Font::MONOSPACE),
        ],
    ].spacing(3)).padding([12, 20]);

    let mut content = column![
        header,
        nav_label,
        nav,
        search_section,
        history_label,
        scrollable(conv_list).height(Length::Fill),
    ];

    // Inline analyze model picker
    if let Some(idx) = app.analyze_source_conversation {
        let title: String = app.conversations.get(idx)
            .map(|c| c.title.chars().take(18).collect())
            .unwrap_or_default();
        let mut picker_col = Column::new().spacing(6);
        picker_col = picker_col.push(
            text(format!("Analyze \"{title}\u{2026}\"")).size(11).color(TEXT_HEAD)
        );
        let models = AppConfig::available_models();
        for chunk in models.chunks(2) {
            let mut chip_row = iced::widget::Row::new().spacing(6);
            for (display, model_id) in chunk {
                let icon = provider_icon(model_id);
                chip_row = chip_row.push(
                    button(text(format!("{icon} {display}")).size(11))
                        .on_press(Message::AnalyzeWith(model_id.to_string()))
                        .padding([5, 10])
                        .style(analyze_chip_style)
                );
            }
            picker_col = picker_col.push(chip_row);
        }
        picker_col = picker_col.push(
            button(text("Cancel").size(11)).padding([4, 8]).style(del_style)
                .on_press(Message::DismissAnalyzePicker)
        );
        content = content.push(
            container(picker_col).padding([10, 16])
                .style(|_: &Theme| container::Style {
                    background: Some(iced::Background::Color(BG_ACTIVE)),
                    border: Border {
                        radius: 0.0.into(),
                        width: 0.0,
                        color: BORDER_DEFAULT,
                    },
                    ..Default::default()
                })
        );
    }

    content = content.push(
        container(iced::widget::Space::new()).width(Length::Fill).height(1)
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(DIVIDER)),
                ..Default::default()
            })
    );
    content = content.push(status);
    let content = content.height(Length::Fill);

    container(content)
        .width(260)
        .height(Length::Fill)
        .style(|_: &Theme| container::Style {
            background: Some(iced::Background::Color(BG)),
            ..Default::default()
        })
        .into()
}
