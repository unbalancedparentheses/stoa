use iced::widget::{column, container, row, scrollable, text, Column};
use iced::{Alignment, Element, Length, Border, Theme};
use std::collections::HashMap;

use crate::app::{ChatApp, Message};
use crate::model::Role;
use crate::theme::*;
use crate::ui::input_bar::{short_model_name, provider_icon};

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

struct ModelStats {
    messages: u32,
    total_tokens: u32,
    total_cost: f64,
    thumbs_up: u32,
    thumbs_down: u32,
    total_latency_ms: u64,
    latency_count: u32,
}

pub fn view(app: &ChatApp) -> Element<'_, Message> {
    let header = container(
        row![text("Analytics").size(15).color(TEXT_HEAD)].align_y(Alignment::Center)
    ).width(Length::Fill).padding([14, 28]).style(|_: &Theme| container::Style {
        background: Some(iced::Background::Color(HEADER_BG)),
        ..Default::default()
    });

    // Aggregate stats across all conversations
    let mut model_stats: HashMap<String, ModelStats> = HashMap::new();
    let mut total_conversations = 0;
    let mut total_messages = 0u32;
    let mut total_cost = 0.0f64;

    for conv in &app.conversations {
        total_conversations += 1;
        for msg in &conv.messages {
            if msg.streaming { continue; }
            total_messages += 1;

            if msg.role == Role::Assistant {
                if let Some(model_id) = &msg.model {
                    let stats = model_stats.entry(model_id.clone()).or_insert(ModelStats {
                        messages: 0, total_tokens: 0, total_cost: 0.0,
                        thumbs_up: 0, thumbs_down: 0, total_latency_ms: 0, latency_count: 0,
                    });
                    stats.messages += 1;
                    let tokens = msg.token_count.unwrap_or_else(|| crate::cost::estimate_tokens(&msg.content));
                    stats.total_tokens += tokens;
                    let cost = crate::cost::message_cost(model_id, &msg.role, tokens);
                    stats.total_cost += cost;
                    total_cost += cost;
                    if msg.rating > 0 { stats.thumbs_up += 1; }
                    if msg.rating < 0 { stats.thumbs_down += 1; }
                    if let Some(lat) = msg.latency_ms {
                        stats.total_latency_ms += lat;
                        stats.latency_count += 1;
                    }
                }
            }
        }
    }

    // Overview card
    let overview = container(column![
        text("Overview").size(13).color(TEXT_HEAD),
        iced::widget::Space::new().height(8),
        stat_row("Conversations", total_conversations.to_string()),
        stat_row("Messages", total_messages.to_string()),
        stat_row("Total Est. Cost", format!("${:.4}", total_cost)),
        stat_row("Session Cost", format!("${:.4}", app.session_cost)),
        stat_row("Models Used", model_stats.len().to_string()),
        stat_row("Ollama Models", app.config.ollama_models.len().to_string()),
    ].spacing(6)).padding(16).width(Length::Fill).style(card_style);

    // Per-model cards
    let mut model_cards = Column::new().spacing(12);
    let mut sorted_models: Vec<_> = model_stats.iter().collect();
    sorted_models.sort_by(|a, b| b.1.messages.cmp(&a.1.messages));

    for (model_id, stats) in &sorted_models {
        let icon = provider_icon(model_id);
        let name = short_model_name(model_id);
        let avg_latency = if stats.latency_count > 0 {
            format!("{} ms", stats.total_latency_ms / stats.latency_count as u64)
        } else {
            "--".to_string()
        };
        let total_rated = stats.thumbs_up + stats.thumbs_down;
        let win_rate = if total_rated > 0 {
            format!("{:.0}%", (stats.thumbs_up as f64 / total_rated as f64) * 100.0)
        } else {
            "--".to_string()
        };

        let card = container(column![
            row![
                text(format!("{icon} {name}")).size(13).color(TEXT_HEAD),
                iced::widget::Space::new().width(Length::Fill),
                text(format!("{} responses", stats.messages)).size(11).color(TEXT_MUTED),
            ].align_y(Alignment::Center),
            iced::widget::Space::new().height(6),
            stat_row("Tokens", format!("{}", stats.total_tokens)),
            stat_row("Est. Cost", format!("${:.4}", stats.total_cost)),
            stat_row("Avg Latency", avg_latency),
            stat_row("Thumbs Up", format!("{}", stats.thumbs_up)),
            stat_row("Thumbs Down", format!("{}", stats.thumbs_down)),
            stat_row("Approval Rate", win_rate),
        ].spacing(4)).padding(16).width(Length::Fill).style(card_style);

        model_cards = model_cards.push(card);
    }

    if sorted_models.is_empty() {
        model_cards = model_cards.push(
            text("No model data yet. Start chatting to see analytics.").size(13).color(TEXT_MUTED)
        );
    }

    let content = column![
        text("Per-Model Statistics").size(13).color(TEXT_HEAD),
        model_cards,
    ].spacing(12);

    let body = column![
        overview,
        iced::widget::Space::new().height(16),
        content,
    ].spacing(12).padding([20, 32]).max_width(600);

    column![
        header,
        container(iced::widget::Space::new()).width(Length::Fill).height(1)
            .style(|_: &Theme| container::Style { background: Some(iced::Background::Color(BORDER_SUBTLE)), ..Default::default() }),
        container(scrollable(container(body).width(Length::Fill)))
            .width(Length::Fill).height(Length::Fill)
            .style(|_: &Theme| container::Style { background: Some(iced::Background::Color(MAIN_BG)), ..Default::default() }),
    ].into()
}
