use iced::widget::{button, column, container, scrollable, text, text_input, Column};
use iced::{Alignment, Element, Length, Border, Theme};

use crate::app::{ChatApp, Message};
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

fn result_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let bg = match (active, status) {
            (true, _) => BG_ACTIVE,
            (false, button::Status::Hovered) => BG_HOVER,
            _ => iced::Color::TRANSPARENT,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: if active { TEXT_HEAD } else { TEXT_SEC },
            border: Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        }
    }
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let input = text_input("Search conversations...", &app.quick_switcher_query)
        .on_input(Message::QuickSwitcherQueryChanged)
        .id("quick-switcher-input")
        .size(14)
        .padding([10, 16])
        .style(input_style);

    let query = app.quick_switcher_query.to_lowercase();
    let mut results = Column::new().spacing(2);
    let mut count = 0;
    for (i, conv) in app.conversations.iter().enumerate() {
        if count >= 12 { break; }
        if !query.is_empty() {
            let title_match = conv.title.to_lowercase().contains(&query);
            let tag_match = conv.tags.iter().any(|t| t.to_lowercase().contains(&query));
            let content_match = conv.messages.iter().any(|m| m.content.to_lowercase().contains(&query));
            if !title_match && !tag_match && !content_match { continue; }
        }

        let pin_icon = if conv.pinned { "\u{25B6} " } else { "" };
        let msg_count = conv.messages.iter().filter(|m| !m.streaming).count();
        let tags_str = if conv.tags.is_empty() { String::new() } else { format!("  [{}]", conv.tags.join(", ")) };
        let label = format!("{pin_icon}{}{tags_str}  ({msg_count} msgs)", conv.title);
        let is_active = i == app.active_conversation;

        results = results.push(
            button(text(label).size(13))
                .on_press(Message::QuickSwitcherSelect(i))
                .width(Length::Fill)
                .padding([8, 16])
                .style(result_style(is_active))
        );
        count += 1;
    }

    let modal = container(
        column![
            text("Quick Switcher").size(12).color(TEXT_MUTED),
            input,
            scrollable(results).height(300),
        ].spacing(8)
    ).width(500).padding(16).style(modal_style);

    container(modal)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(overlay_style)
        .into()
}
