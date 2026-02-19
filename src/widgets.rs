use makepad_widgets::*;

use crate::app::AppData;
use crate::model::Role;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    // ── Conversation list widget ──
    pub ConvList = {{ConvList}} {
        width: Fill, height: Fill,
        list = <PortalList> {
            width: Fill, height: Fill,
            scroll_bar: { bar_size: 0.0 }
            ConvItem = <Button> {
                width: Fill, height: Fit,
                padding: {top: 8, bottom: 8, left: 12, right: 12},
                margin: {left: 10, right: 10},
                draw_bg: {
                    color: #0000,
                    border_radius: 6.0,
                    border_size: 0.0,
                }
                draw_text: {
                    color: #8a909a,
                    text_style: { font_size: 9.0 }
                }
                text: "New Chat"
            }
        }
    }

    // ── Message list widget ──
    pub MsgList = {{MsgList}} {
        width: Fill, height: Fill,
        list = <PortalList> {
            width: Fill, height: Fill,
            padding: {top: 24, bottom: 24, left: 36, right: 36},
            spacing: 20,
            scroll_bar: { bar_size: 0.0 }

            UserMsg = <View> {
                width: Fill, height: Fit,
                flow: Down,
                spacing: 4,
                user_label = <Label> {
                    width: Fill, height: Fit,
                    align: {x: 1.0},
                    draw_text: {
                        color: #8a909a,
                        text_style: { font_size: 8.0 }
                    }
                    text: "You"
                }
                bubble_wrap = <View> {
                    width: Fill, height: Fit,
                    align: {x: 1.0},
                    bubble = <RoundedView> {
                        width: Fit, height: Fit,
                        padding: {top: 12, bottom: 12, left: 16, right: 16},
                        show_bg: true,
                        draw_bg: {
                            color: #1e2836,
                            border_radius: 12.0,
                        }
                        content = <Label> {
                            width: Fit, height: Fit,
                            draw_text: {
                                color: #e8e0d0,
                                text_style: { font_size: 10.5 }
                            }
                            text: ""
                        }
                    }
                }
            }

            AssistantMsg = <View> {
                width: Fill, height: Fit,
                flow: Down,
                spacing: 6,
                content = <Label> {
                    width: Fill, height: Fit,
                    draw_text: {
                        color: #d0c8b8,
                        text_style: { font_size: 10.0, line_spacing: 1.7 }
                    }
                    text: ""
                }
                streaming_dots = <Label> {
                    width: Fit, height: Fit,
                    draw_text: {
                        color: #c9a84c,
                        text_style: { font_size: 10.0 }
                    }
                    text: "..."
                    visible: false,
                }
                model_tag = <Label> {
                    width: Fit, height: Fit,
                    draw_text: {
                        color: #505a66,
                        text_style: { font_size: 7.5 }
                    }
                    text: ""
                }
            }

            ErrorMsg = <RoundedView> {
                width: Fill, height: Fit,
                padding: {top: 10, bottom: 10, left: 14, right: 14},
                show_bg: true,
                draw_bg: {
                    color: #2a1818,
                    border_radius: 8.0,
                    border_color: #442222,
                    border_size: 1.0,
                }
                flow: Right,
                spacing: 8,
                align: {y: 0.5},
                error_text = <Label> {
                    width: Fill, height: Fit,
                    draw_text: {
                        color: #da6b6b,
                        text_style: { font_size: 9.5 }
                    }
                    text: ""
                }
                dismiss_btn = <Button> {
                    width: Fit, height: Fit,
                    padding: {top: 2, bottom: 2, left: 8, right: 8},
                    draw_bg: { color: #0000 }
                    draw_text: {
                        color: #885555,
                        text_style: { font_size: 9.0 }
                    }
                    text: "x"
                }
            }

            EmptyMsg = <View> {
                width: Fill, height: Fit,
                flow: Down,
                spacing: 4,
                padding: {top: 48},
                <Label> {
                    width: Fit, height: Fit,
                    draw_text: {
                        color: #e8e0d0,
                        text_style: { font_size: 16.0 }
                    }
                    text: "rust-chat"
                }
                <Label> {
                    width: Fit, height: Fit,
                    draw_text: {
                        color: #505a66,
                        text_style: { font_size: 9.5 }
                    }
                    text: "Share a thought to get started."
                }
            }
        }
    }
}

// ─────────────────────────────────────────────
//  ConvList – conversation sidebar widget
// ─────────────────────────────────────────────

#[derive(Live, LiveHook, Widget)]
pub struct ConvList {
    #[deref]
    view: View,
}

impl Widget for ConvList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                let data = scope.data.get::<AppData>();
                let (convs, active, show_settings) = match data {
                    Some(d) => (&d.conversations, d.active_conversation, d.show_settings),
                    None => return DrawStep::done(),
                };

                list.set_item_range(cx, 0, convs.len());

                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id >= convs.len() {
                        continue;
                    }
                    let item_widget = list.item(cx, item_id, live_id!(ConvItem));

                    let conv = &convs[item_id];
                    let title: String = conv.title.chars().take(24).collect();
                    let title = if conv.title.len() > 24 {
                        format!("{title}\u{2026}")
                    } else {
                        title
                    };

                    let is_active = item_id == active && !show_settings;
                    let btn = item_widget.button(id!(ConvItem));
                    btn.set_text(cx, &format!("\u{25A1} {}", title));

                    if is_active {
                        btn.apply_over(cx, live! {
                            draw_bg: { color: (Vec4::from_u32(0x1e2836ff)) }
                            draw_text: { color: (Vec4::from_u32(0xe8e0d0ff)) }
                        });
                    } else {
                        btn.apply_over(cx, live! {
                            draw_bg: { color: (Vec4::from_u32(0x00000000)) }
                            draw_text: { color: (Vec4::from_u32(0x8a909aff)) }
                        });
                    }

                    item_widget.draw_all(cx, &mut Scope::empty());
                }
            }
        }
        DrawStep::done()
    }
}

// ─────────────────────────────────────────────
//  MsgList – chat messages widget
// ─────────────────────────────────────────────

#[derive(Live, LiveHook, Widget)]
pub struct MsgList {
    #[deref]
    view: View,
}

impl Widget for MsgList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                let data = scope.data.get::<AppData>();
                let data = match data {
                    Some(d) => d,
                    None => return DrawStep::done(),
                };

                let conv = &data.conversations[data.active_conversation];
                let model_name = data.config.active_provider_config().model.clone();
                let msg_count = conv.messages.len();
                let has_empty = msg_count == 0 && !data.is_streaming;
                let has_waiting_dots = data.is_streaming && data.current_response.is_empty();
                let has_error = data.error_message.is_some();

                let mut total = msg_count;
                if has_empty { total += 1; }
                if has_waiting_dots { total += 1; }
                if has_error { total += 1; }

                list.set_item_range(cx, 0, total);

                let _is_streaming = data.is_streaming;

                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id >= total {
                        continue;
                    }

                    // Empty state
                    if has_empty && item_id == 0 {
                        let w = list.item(cx, item_id, live_id!(EmptyMsg));
                        w.draw_all(cx, &mut Scope::empty());
                        continue;
                    }

                    let msg_idx = if has_empty { item_id - 1 } else { item_id };

                    if msg_idx < msg_count {
                        let msg = &conv.messages[msg_idx];
                        match msg.role {
                            Role::User => {
                                let w = list.item(cx, item_id, live_id!(UserMsg));
                                w.label(id!(content)).set_text(cx, &msg.content);
                                w.draw_all(cx, &mut Scope::empty());
                            }
                            Role::Assistant => {
                                let w = list.item(cx, item_id, live_id!(AssistantMsg));
                                w.label(id!(content)).set_text(cx, &msg.content);
                                w.label(id!(streaming_dots)).set_visible(cx, msg.streaming);
                                w.label(id!(model_tag)).set_text(cx, &model_name);
                                if !msg.streaming {
                                    w.label(id!(model_tag)).set_visible(cx, true);
                                } else {
                                    w.label(id!(model_tag)).set_visible(cx, false);
                                }
                                w.draw_all(cx, &mut Scope::empty());
                            }
                        }
                        continue;
                    }

                    // Extra items after messages
                    let extra_idx = msg_idx - msg_count;
                    if has_waiting_dots && extra_idx == 0 {
                        let w = list.item(cx, item_id, live_id!(AssistantMsg));
                        w.label(id!(content)).set_text(cx, "");
                        w.label(id!(streaming_dots)).set_visible(cx, true);
                        w.label(id!(model_tag)).set_text(cx, "");
                        w.label(id!(model_tag)).set_visible(cx, false);
                        w.draw_all(cx, &mut Scope::empty());
                    } else if has_error {
                        let w = list.item(cx, item_id, live_id!(ErrorMsg));
                        if let Some(ref err) = data.error_message {
                            w.label(id!(error_text)).set_text(cx, err);
                        }
                        w.draw_all(cx, &mut Scope::empty());
                    }
                }
            }
        }
        DrawStep::done()
    }
}
