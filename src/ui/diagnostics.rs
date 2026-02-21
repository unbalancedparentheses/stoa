use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Border, Element, Length, Theme};

use crate::app::{ChatApp, Message};
use crate::theme::*;

fn card_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(CARD_BG)),
        border: Border { radius: 8.0.into(), width: 1.0, color: BORDER_SUBTLE },
        ..Default::default()
    }
}

fn stat_row<'a>(label: &'a str, value: String) -> Element<'a, Message> {
    row![
        text(label).size(12).color(TEXT_SEC),
        iced::widget::Space::new().width(Length::Fill),
        text(value).size(12).color(TEXT_HEAD).font(iced::Font::MONOSPACE),
    ].align_y(Alignment::Center).into()
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let header = container(
        row![text("Diagnostics").size(15).color(TEXT_HEAD)].align_y(Alignment::Center)
    ).width(Length::Fill).padding([14, 28]).style(|_: &Theme| container::Style {
        background: Some(iced::Background::Color(HEADER_BG)),
        ..Default::default()
    });

    let focus = container(column![
        text("Focus State").size(13).color(TEXT_HEAD),
        iced::widget::Space::new().height(8),
        stat_row("Startup focus attempts", app.startup_focus_attempts.to_string()),
        stat_row("Startup focus successes", app.startup_focus_successes.to_string()),
        stat_row("Platform", if cfg!(target_os = "macos") { "macOS".to_string() } else { "non-macOS".to_string() }),
    ].spacing(6)).padding(16).width(Length::Fill).style(card_style);

    let key_log_path = crate::shortcuts::key_log_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "<unavailable>".to_string());

    let keyboard = container(column![
        text("Keyboard State").size(13).color(TEXT_HEAD),
        iced::widget::Space::new().height(8),
        stat_row("Debug key log", if app.config.debug_key_events { "enabled".into() } else { "disabled".into() }),
        stat_row("Key log path", key_log_path),
        stat_row(
            "Last shortcut event",
            app.last_shortcut_event.clone().unwrap_or_else(|| "<none>".to_string()),
        ),
    ].spacing(6)).padding(16).width(Length::Fill).style(card_style);

    let mut bindings_col = column![
        text("Active Keybindings").size(13).color(TEXT_HEAD),
        iced::widget::Space::new().height(8),
    ].spacing(6);
    for spec in crate::shortcuts::specs() {
        bindings_col = bindings_col.push(
            stat_row(crate::shortcuts::action_label(spec.action), app.config.keybindings.get(spec.action).to_string())
        );
    }
    let bindings = container(bindings_col).padding(16).width(Length::Fill).style(card_style);

    let run = button(text("Run Diagnostics").size(12))
        .on_press(Message::RunDiagnostics)
        .padding([8, 14])
        .style(|_: &Theme, status: button::Status| button::Style {
            background: Some(iced::Background::Color(match status {
                button::Status::Hovered => BG_HOVER,
                _ => BG_ACTIVE,
            })),
            text_color: TEXT_HEAD,
            border: Border { radius: 8.0.into(), width: 1.0, color: BORDER_DEFAULT },
            ..Default::default()
        });

    let run_card = container(column![
        row![run, iced::widget::Space::new().width(Length::Fill)].align_y(Alignment::Center),
        text(
            app.diagnostics_last_run
                .as_ref()
                .map(|s| format!("Last run: {s}"))
                .unwrap_or_else(|| "Not run yet this session.".to_string())
        )
        .size(11)
        .color(TEXT_MUTED),
    ].spacing(8)).padding(16).width(Length::Fill).style(card_style);

    let body = column![run_card, focus, keyboard, bindings]
        .spacing(12)
        .padding([20, 32])
        .max_width(760);

    column![
        header,
        container(iced::widget::Space::new()).width(Length::Fill).height(1)
            .style(|_: &Theme| container::Style { background: Some(iced::Background::Color(BORDER_SUBTLE)), ..Default::default() }),
        container(scrollable(container(body).width(Length::Fill)))
            .width(Length::Fill).height(Length::Fill)
            .style(|_: &Theme| container::Style { background: Some(iced::Background::Color(MAIN_BG)), ..Default::default() }),
    ].into()
}
