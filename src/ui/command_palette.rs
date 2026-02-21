use iced::widget::{button, column, container, scrollable, text, text_input, row, Column};
use iced::{Alignment, Element, Length, Border, Theme};

use crate::app::{ChatApp, Message};
use crate::theme::*;

struct Command {
    label: &'static str,
    description: &'static str,
    shortcut: &'static str,
    message: Message,
}

fn all_commands() -> Vec<Command> {
    vec![
        Command { label: "New Chat", description: "Create a new conversation", shortcut: "Cmd/Ctrl+N", message: Message::NewConversation },
        Command { label: "Settings", description: "Open settings", shortcut: "Cmd/Ctrl+,", message: Message::ShowSettings },
        Command { label: "Home", description: "Go to chat view", shortcut: "", message: Message::ShowChat },
        Command { label: "Send to All", description: "Send to all available models", shortcut: "Cmd/Ctrl+Shift+Enter", message: Message::SendToAll },
        Command { label: "Toggle Comparison", description: "Switch comparison mode on/off", shortcut: "", message: Message::ToggleComparisonMode },
        Command { label: "Export Markdown", description: "Copy conversation as Markdown", shortcut: "Cmd/Ctrl+E", message: Message::ExportMarkdown },
        Command { label: "Export HTML", description: "Copy conversation as styled HTML", shortcut: "", message: Message::ExportHtml },
        Command { label: "Export JSON", description: "Copy conversation as JSON", shortcut: "", message: Message::ExportJson },
        Command { label: "Import ChatGPT", description: "Import from ChatGPT export file", shortcut: "", message: Message::ImportChatGpt },
        Command { label: "Web Search", description: "Search web for current input", shortcut: "", message: Message::WebSearch },
        Command { label: "Refresh Ollama", description: "Re-scan local Ollama models", shortcut: "", message: Message::RefreshOllamaModels },
        Command { label: "Analytics", description: "View model stats and ratings", shortcut: "", message: Message::ShowAnalytics },
        Command { label: "Quick Switcher", description: "Search conversations", shortcut: "Cmd/Ctrl+K", message: Message::ToggleQuickSwitcher },
    ]
}

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

fn cmd_style(_: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => BG_ACTIVE,
        _ => iced::Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: match status {
            button::Status::Hovered => TEXT_HEAD,
            _ => TEXT_SEC,
        },
        border: Border { radius: 4.0.into(), ..Default::default() },
        ..Default::default()
    }
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let input = text_input("Type a command...", &app.command_palette_query)
        .on_input(Message::CommandPaletteQueryChanged)
        .id("command-palette-input")
        .size(14)
        .padding([10, 16])
        .style(input_style);

    let query = app.command_palette_query.to_lowercase();
    let mut results = Column::new().spacing(2);

    for cmd in all_commands() {
        if !query.is_empty() {
            let label_match = cmd.label.to_lowercase().contains(&query);
            let desc_match = cmd.description.to_lowercase().contains(&query);
            if !label_match && !desc_match { continue; }
        }

        let mut cmd_row = row![
            text(cmd.label).size(13).color(TEXT_HEAD),
            iced::widget::Space::new().width(8),
            text(cmd.description).size(11).color(TEXT_MUTED),
            iced::widget::Space::new().width(Length::Fill),
        ].spacing(4).align_y(Alignment::Center);

        if !cmd.shortcut.is_empty() {
            cmd_row = cmd_row.push(
                text(cmd.shortcut).size(10).color(TEXT_MUTED).font(iced::Font::MONOSPACE)
            );
        }

        results = results.push(
            button(cmd_row)
                .on_press(cmd.message.clone())
                .width(Length::Fill)
                .padding([8, 16])
                .style(cmd_style)
        );
    }

    let modal = container(
        column![
            text("Command Palette").size(12).color(TEXT_MUTED),
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
