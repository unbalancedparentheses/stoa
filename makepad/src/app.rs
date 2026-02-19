use makepad_widgets::*;

use crate::config::AppConfig;
use crate::model::{Conversation, Provider};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use crate::widgets::ConvList;
    use crate::widgets::MsgList;

    // ── Morphe colour constants ──
    WINDOW_BG      = #111922
    SIDEBAR_BG     = #121a24
    MAIN_BG        = #161e2a
    HEADER_BG      = #141c26
    BAR_BG         = #101720
    DIVIDER_C      = #1e2834
    TEXT_PRIMARY    = #e8e0d0
    TEXT_SECONDARY  = #8a909a
    TEXT_MUTED      = #505a66
    ACCENT          = #c9a84c
    ACCENT_DIM      = #6a5e3a
    GREEN_C         = #3fb88c
    CARD_BG         = #1a2430
    INPUT_BG        = #121a24
    BORDER_DEFAULT  = #3a4a5a
    BORDER_SUBTLE   = #1e2834

    // ── Preset chip button ──
    PresetChip = <Button> {
        width: Fit, height: Fit,
        padding: {top: 6, bottom: 6, left: 16, right: 16},
        draw_bg: {
            color: #0000,
            border_radius: 14.0,
            border_size: 1.0,
            border_color_1: (BORDER_DEFAULT),
            border_color_2: (BORDER_DEFAULT),
        }
        draw_text: {
            color: (TEXT_SECONDARY),
            text_style: { font_size: 9.0 }
        }
        text: ""
    }

    // ═══════════════════════════════════
    //  Main application
    // ═══════════════════════════════════
    App = {{App}} {
        ui: <Window> {
            window: { inner_size: vec2(1200, 800) },
            pass: { clear_color: (WINDOW_BG) },

            body = {
                flow: Down,
                width: Fill, height: Fill,

                main_row = <View> {
                    width: Fill, height: Fill,
                    flow: Right,

                    // ─── Sidebar ───
                    sidebar = <View> {
                        width: 260, height: Fill,
                        flow: Down,
                        show_bg: true,
                        draw_bg: { color: (SIDEBAR_BG) }

                        sidebar_header = <View> {
                            width: Fill, height: Fit,
                            flow: Right,
                            padding: {top: 24, right: 20, bottom: 16, left: 20},
                            align: {y: 0.5},

                            header_titles = <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 3,
                                <Label> {
                                    width: Fit, height: Fit,
                                    draw_text: {
                                        color: (TEXT_PRIMARY),
                                        text_style: { font_size: 11.0 }
                                    }
                                    text: "Chats"
                                }
                                <Label> {
                                    width: Fit, height: Fit,
                                    draw_text: {
                                        color: (TEXT_MUTED),
                                        text_style: { font_size: 8.0 }
                                    }
                                    text: "Recent conversations"
                                }
                            }
                            new_chat_btn = <Button> {
                                width: 28, height: 28,
                                draw_bg: {
                                    color: #2a3440,
                                    border_radius: 14.0,
                                    border_size: 0.0,
                                }
                                draw_text: {
                                    color: (TEXT_PRIMARY),
                                    text_style: { font_size: 10.0 }
                                }
                                text: "+"
                            }
                        }

                        <Label> {
                            width: Fit, height: Fit,
                            margin: {top: 8, left: 20, bottom: 6},
                            draw_text: {
                                color: (TEXT_MUTED),
                                text_style: { font_size: 8.0 }
                            }
                            text: "Today"
                        }

                        conversation_list = <ConvList> {}
                    }

                    // ─── Left divider ───
                    <View> {
                        width: 1, height: Fill,
                        show_bg: true,
                        draw_bg: { color: (DIVIDER_C) }
                    }

                    // ─── Centre panel ───
                    centre_panel = <View> {
                        width: Fill, height: Fill,
                        flow: Overlay,

                        // Chat view (shown by default)
                        chat_view = <View> {
                            width: Fill, height: Fill,
                            flow: Down,

                            // Toolbar
                            chat_toolbar = <View> {
                                width: Fill, height: Fit,
                                flow: Right,
                                padding: {top: 10, bottom: 10, left: 20, right: 20},
                                align: {y: 0.5},
                                show_bg: true,
                                draw_bg: { color: (HEADER_BG) }

                                hamburger_btn = <Button> {
                                    width: Fit, height: Fit,
                                    padding: {top: 4, bottom: 4, left: 8, right: 8},
                                    draw_bg: { color: #0000, border_size: 0.0 }
                                    draw_text: {
                                        color: (TEXT_MUTED),
                                        text_style: { font_size: 10.0 }
                                    }
                                    text: "="
                                }
                                <View> { width: Fill, height: 1 }
                                model_label = <Label> {
                                    width: Fit, height: Fit,
                                    draw_text: {
                                        color: (TEXT_MUTED),
                                        text_style: { font_size: 8.0 }
                                    }
                                    text: ""
                                }
                                gear_btn = <Button> {
                                    width: Fit, height: Fit,
                                    padding: {top: 4, bottom: 4, left: 6, right: 6},
                                    draw_bg: { color: #0000, border_size: 0.0 }
                                    draw_text: {
                                        color: (TEXT_MUTED),
                                        text_style: { font_size: 10.0 }
                                    }
                                    text: "@"
                                }
                                delete_btn = <Button> {
                                    width: Fit, height: Fit,
                                    padding: {top: 4, bottom: 4, left: 6, right: 6},
                                    draw_bg: { color: #0000, border_size: 0.0 }
                                    draw_text: {
                                        color: (TEXT_MUTED),
                                        text_style: { font_size: 10.0 }
                                    }
                                    text: "x"
                                }
                            }

                            <View> {
                                width: Fill, height: 1,
                                show_bg: true,
                                draw_bg: { color: (DIVIDER_C) }
                            }

                            // Message list
                            message_area = <View> {
                                width: Fill, height: Fill,
                                show_bg: true,
                                draw_bg: { color: (MAIN_BG) }
                                message_list = <MsgList> {}
                            }

                            // ── Input bar ──
                            input_area = <View> {
                                width: Fill, height: Fit,
                                padding: {top: 12, bottom: 20, left: 28, right: 28},
                                show_bg: true,
                                draw_bg: { color: (MAIN_BG) }

                                input_card = <RoundedView> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    show_bg: true,
                                    draw_bg: {
                                        color: (INPUT_BG),
                                        border_radius: 12.0,
                                        border_color: (ACCENT_DIM),
                                        border_size: 1.0,
                                    }

                                    message_input = <TextInput> {
                                        width: Fill, height: Fit,
                                        padding: {top: 10, bottom: 10, left: 16, right: 16},
                                        empty_text: "Type a message...",
                                        draw_bg: {
                                            color: #0000,
                                            border_size: 0.0,
                                        }
                                        draw_text: {
                                            color: (TEXT_PRIMARY),
                                            text_style: { font_size: 10.0 }
                                        }
                                        draw_cursor: { color: (TEXT_PRIMARY) }
                                        draw_selection: { color: #2a3e55 }
                                    }

                                    input_buttons = <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        padding: {top: 0, bottom: 4, left: 4, right: 4},
                                        align: {y: 0.5},

                                        plus_btn = <Button> {
                                            width: 32, height: 32,
                                            draw_bg: {
                                                color: #0000,
                                                border_radius: 14.0,
                                                border_size: 0.0,
                                            }
                                            draw_text: {
                                                color: (TEXT_MUTED),
                                                text_style: { font_size: 12.0 }
                                            }
                                            text: "+"
                                        }
                                        <View> { width: Fill, height: 1 }
                                        send_btn = <Button> {
                                            width: 32, height: 32,
                                            draw_bg: {
                                                color: (ACCENT_DIM),
                                                border_radius: 14.0,
                                                border_size: 0.0,
                                            }
                                            draw_text: {
                                                color: (TEXT_PRIMARY),
                                                text_style: { font_size: 10.0 }
                                            }
                                            text: "^"
                                        }
                                    }
                                }
                            }
                        }

                        // Settings view (hidden by default)
                        settings_view = <View> {
                            width: Fill, height: Fill,
                            flow: Down,
                            visible: false,

                            settings_header = <View> {
                                width: Fill, height: Fit,
                                padding: {top: 14, bottom: 14, left: 24, right: 24},
                                show_bg: true,
                                draw_bg: { color: (HEADER_BG) }
                                <Label> {
                                    width: Fit, height: Fit,
                                    draw_text: {
                                        color: (TEXT_PRIMARY),
                                        text_style: { font_size: 10.5 }
                                    }
                                    text: "Settings"
                                }
                            }
                            <View> {
                                width: Fill, height: 1,
                                show_bg: true,
                                draw_bg: { color: (BORDER_SUBTLE) }
                            }

                            settings_scroll = <View> {
                                width: Fill, height: Fill,
                                flow: Down,
                                show_bg: true,
                                draw_bg: { color: (MAIN_BG) }
                                padding: {top: 20, bottom: 20, left: 32, right: 32},
                                spacing: 12,

                                // Provider toggle
                                provider_toggle = <RoundedView> {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    padding: 4,
                                    spacing: 4,
                                    show_bg: true,
                                    draw_bg: {
                                        color: (INPUT_BG),
                                        border_radius: 8.0,
                                    }
                                    openai_btn = <Button> {
                                        width: Fill, height: Fit,
                                        padding: {top: 10, bottom: 10, left: 16, right: 16},
                                        draw_bg: { color: #0000, border_radius: 6.0, border_size: 0.0 }
                                        draw_text: {
                                            color: (TEXT_SECONDARY),
                                            text_style: { font_size: 9.5 }
                                        }
                                        text: "OpenAI"
                                    }
                                    anthropic_btn = <Button> {
                                        width: Fill, height: Fit,
                                        padding: {top: 10, bottom: 10, left: 16, right: 16},
                                        draw_bg: { color: #0000, border_radius: 6.0, border_size: 0.0 }
                                        draw_text: {
                                            color: (TEXT_SECONDARY),
                                            text_style: { font_size: 9.5 }
                                        }
                                        text: "Anthropic"
                                    }
                                }

                                // Model presets card
                                model_card = <RoundedView> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    padding: 16,
                                    spacing: 10,
                                    show_bg: true,
                                    draw_bg: {
                                        color: (CARD_BG),
                                        border_radius: 8.0,
                                        border_color: (BORDER_SUBTLE),
                                        border_size: 1.0,
                                    }
                                    <Label> {
                                        width: Fit, height: Fit,
                                        draw_text: {
                                            color: (TEXT_MUTED),
                                            text_style: { font_size: 8.5 }
                                        }
                                        text: "Model"
                                    }
                                    preset_row = <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        spacing: 6,
                                        preset_0 = <PresetChip> {}
                                        preset_1 = <PresetChip> {}
                                        preset_2 = <PresetChip> {}
                                        preset_3 = <PresetChip> {}
                                    }
                                }

                                // Connection fields card
                                connection_card = <RoundedView> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    padding: 16,
                                    spacing: 12,
                                    show_bg: true,
                                    draw_bg: {
                                        color: (CARD_BG),
                                        border_radius: 8.0,
                                        border_color: (BORDER_SUBTLE),
                                        border_size: 1.0,
                                    }
                                    <Label> {
                                        width: Fit, height: Fit,
                                        draw_text: {
                                            color: (TEXT_MUTED),
                                            text_style: { font_size: 8.5 }
                                        }
                                        text: "Connection"
                                    }
                                    // API URL
                                    <Label> {
                                        width: Fit, height: Fit,
                                        draw_text: { color: (TEXT_MUTED), text_style: { font_size: 8.0 } }
                                        text: "API URL"
                                    }
                                    api_url_input = <TextInput> {
                                        width: Fill, height: Fit,
                                        padding: {top: 10, bottom: 10, left: 14, right: 14},
                                        empty_text: "https://api.openai.com/v1/chat/completions",
                                        draw_bg: {
                                            color: (INPUT_BG),
                                        }
                                        draw_text: { color: (TEXT_PRIMARY), text_style: { font_size: 9.5 } }
                                        draw_cursor: { color: (TEXT_PRIMARY) }
                                        draw_selection: { color: #2a3e55 }
                                    }
                                    // API Key
                                    <Label> {
                                        width: Fit, height: Fit,
                                        draw_text: { color: (TEXT_MUTED), text_style: { font_size: 8.0 } }
                                        text: "API Key"
                                    }
                                    api_key_input = <TextInput> {
                                        width: Fill, height: Fit,
                                        padding: {top: 10, bottom: 10, left: 14, right: 14},
                                        empty_text: "sk-...",
                                        draw_bg: {
                                            color: (INPUT_BG),
                                        }
                                        draw_text: { color: (TEXT_PRIMARY), text_style: { font_size: 9.5 } }
                                        draw_cursor: { color: (TEXT_PRIMARY) }
                                        draw_selection: { color: #2a3e55 }
                                    }
                                    // Model ID
                                    <Label> {
                                        width: Fit, height: Fit,
                                        draw_text: { color: (TEXT_MUTED), text_style: { font_size: 8.0 } }
                                        text: "Model ID"
                                    }
                                    model_id_input = <TextInput> {
                                        width: Fill, height: Fit,
                                        padding: {top: 10, bottom: 10, left: 14, right: 14},
                                        empty_text: "gpt-4.1",
                                        draw_bg: {
                                            color: (INPUT_BG),
                                        }
                                        draw_text: { color: (TEXT_PRIMARY), text_style: { font_size: 9.5 } }
                                        draw_cursor: { color: (TEXT_PRIMARY) }
                                        draw_selection: { color: #2a3e55 }
                                    }
                                }

                                // Save button
                                save_btn = <Button> {
                                    width: Fill, height: Fit,
                                    padding: {top: 10, bottom: 10, left: 20, right: 20},
                                    draw_bg: {
                                        color: (CARD_BG),
                                        border_radius: 8.0,
                                        border_size: 1.0,
                                        border_color_1: (BORDER_DEFAULT),
                                        border_color_2: (BORDER_DEFAULT),
                                    }
                                    draw_text: {
                                        color: (ACCENT),
                                        text_style: { font_size: 9.5 }
                                    }
                                    text: "Save"
                                }
                            }
                        }
                    }

                    // ─── Right divider ───
                    <View> {
                        width: 1, height: Fill,
                        show_bg: true,
                        draw_bg: { color: (DIVIDER_C) }
                    }

                    // ─── Right panel ───
                    right_panel = <View> {
                        width: 260, height: Fill,
                        flow: Down,
                        show_bg: true,
                        draw_bg: { color: (SIDEBAR_BG) }
                        <View> {
                            width: Fill, height: Fit,
                            flow: Down,
                            padding: {top: 24, right: 20, bottom: 16, left: 20},
                            spacing: 12,
                            <Label> {
                                width: Fit, height: Fit,
                                draw_text: {
                                    color: (TEXT_PRIMARY),
                                    text_style: { font_size: 11.0 }
                                }
                                text: "Inspector"
                            }
                            <Label> {
                                width: Fit, height: Fit,
                                draw_text: {
                                    color: (TEXT_MUTED),
                                    text_style: { font_size: 9.5 }
                                }
                                text: "Coming soon..."
                            }
                        }
                    }
                }

                // ── Bottom bar ──
                bottom_bar = <View> {
                    width: Fill, height: Fit,
                    flow: Down,
                    show_bg: true,
                    draw_bg: { color: (BAR_BG) }
                    <View> {
                        width: Fill, height: 1,
                        show_bg: true,
                        draw_bg: { color: (DIVIDER_C) }
                    }
                    bar_content = <View> {
                        width: Fill, height: Fit,
                        flow: Right,
                        padding: {top: 6, bottom: 6, left: 16, right: 16},
                        align: {y: 0.5},
                        <Label> {
                            width: Fit, height: Fit,
                            draw_text: { color: (GREEN_C), text_style: { font_size: 7.5 } }
                            text: "RPS: --"
                        }
                        <Label> {
                            width: Fit, height: Fit,
                            draw_text: { color: (TEXT_MUTED), text_style: { font_size: 7.5 } }
                            text: " | "
                        }
                        <Label> {
                            width: Fit, height: Fit,
                            draw_text: { color: (GREEN_C), text_style: { font_size: 7.5 } }
                            text: "RPM: --"
                        }
                        <Label> {
                            width: Fit, height: Fit,
                            draw_text: { color: (TEXT_MUTED), text_style: { font_size: 7.5 } }
                            text: " | "
                        }
                        <Label> {
                            width: Fit, height: Fit,
                            draw_text: { color: (GREEN_C), text_style: { font_size: 7.5 } }
                            text: "R: --"
                        }
                        <View> { width: Fill, height: 1 }
                        <Label> {
                            width: Fit, height: Fit,
                            draw_text: { color: (ACCENT), text_style: { font_size: 9.0 } }
                            text: "*"
                        }
                        bottom_model_label = <Label> {
                            width: Fit, height: Fit,
                            margin: {left: 4},
                            draw_text: { color: (TEXT_SECONDARY), text_style: { font_size: 8.0 } }
                            text: ""
                        }
                        <View> { width: Fill, height: 1 }
                        <Label> {
                            width: Fit, height: Fit,
                            draw_text: { color: (TEXT_MUTED), text_style: { font_size: 9.0 } }
                            text: "@ i"
                        }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────
//  Shared app data (passed through Scope)
// ─────────────────────────────────────────────

pub struct AppData {
    pub conversations: Vec<Conversation>,
    pub active_conversation: usize,
    pub is_streaming: bool,
    pub current_response: String,
    pub config: AppConfig,
    pub show_settings: bool,
    pub error_message: Option<String>,
    pub config_saved: bool,
    pub sse_buffer: String,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            conversations: vec![Conversation::new()],
            active_conversation: 0,
            is_streaming: false,
            current_response: String::new(),
            config: AppConfig::default(),
            show_settings: false,
            error_message: None,
            config_saved: false,
            sse_buffer: String::new(),
        }
    }
}

// ─────────────────────────────────────────────
//  App struct
// ─────────────────────────────────────────────

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    data: AppData,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
        crate::widgets::live_design(cx);
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        // Handle startup
        if let Event::Startup = event {
            self.data.config = AppConfig::load();
            self.data.conversations = Conversation::load_all();
            if self.data.conversations.is_empty() {
                self.data.conversations.push(Conversation::new());
            }
            self.update_model_labels(cx);
            self.update_settings_ui(cx);
        }

        // Propagate to UI widgets first (generates actions)
        let mut scope = Scope::with_data(&mut self.data);
        self.ui.handle_event(cx, event, &mut scope);

        // Then dispatch to WidgetMatchEvent (processes actions + HTTP events)
        self.widget_match_event(cx, event, &mut Scope::empty());
    }
}

impl WidgetMatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // ── New chat button ──
        if self.ui.button(id!(new_chat_btn)).clicked(actions) {
            self.data.conversations.push(Conversation::new());
            self.data.active_conversation = self.data.conversations.len() - 1;
            self.data.show_settings = false;
            self.switch_view(cx);
            self.ui.redraw(cx);
        }

        // ── Conversation list clicks ──
        let conv_list = self.ui.portal_list(id!(list));
        for (item_id, item) in conv_list.items_with_actions(actions) {
            if item.button(id!(ConvItem)).clicked(actions) {
                if item_id < self.data.conversations.len() {
                    self.data.active_conversation = item_id;
                    self.data.show_settings = false;
                    self.switch_view(cx);
                    self.ui.redraw(cx);
                }
            }
        }

        // ── Send message ──
        let send_clicked = self.ui.button(id!(send_btn)).clicked(actions);
        let return_pressed = self.ui.text_input(id!(message_input)).returned(actions).is_some();
        if send_clicked || return_pressed {
            self.send_message(cx);
        }

        // ── Gear button → settings ──
        if self.ui.button(id!(gear_btn)).clicked(actions) {
            self.data.show_settings = true;
            self.data.config_saved = false;
            self.switch_view(cx);
            self.update_settings_ui(cx);
            self.ui.redraw(cx);
        }

        // ── Delete conversation ──
        if self.ui.button(id!(delete_btn)).clicked(actions) {
            if self.data.conversations.len() > 1 {
                let conv = &self.data.conversations[self.data.active_conversation];
                conv.delete();
                self.data.conversations.remove(self.data.active_conversation);
                if self.data.active_conversation >= self.data.conversations.len() {
                    self.data.active_conversation = self.data.conversations.len() - 1;
                }
                self.ui.redraw(cx);
            }
        }

        // ── Dismiss error ──
        if self.ui.button(id!(dismiss_btn)).clicked(actions) {
            self.data.error_message = None;
            self.ui.redraw(cx);
        }

        // ── Provider toggle ──
        if self.ui.button(id!(openai_btn)).clicked(actions) {
            self.data.config.active_provider = Provider::OpenAI;
            self.data.config_saved = false;
            self.update_settings_ui(cx);
            self.update_model_labels(cx);
            self.ui.redraw(cx);
        }
        if self.ui.button(id!(anthropic_btn)).clicked(actions) {
            self.data.config.active_provider = Provider::Anthropic;
            self.data.config_saved = false;
            self.update_settings_ui(cx);
            self.update_model_labels(cx);
            self.ui.redraw(cx);
        }

        // ── Presets ──
        self.handle_preset_clicks(cx, actions);

        // ── Save config ──
        if self.ui.button(id!(save_btn)).clicked(actions) {
            // Read current values from text inputs
            let url = self.ui.text_input(id!(api_url_input)).text();
            let key = self.ui.text_input(id!(api_key_input)).text();
            let model = self.ui.text_input(id!(model_id_input)).text();
            if !url.is_empty() {
                self.data.config.active_provider_config_mut().api_url = url;
            }
            if !key.is_empty() {
                self.data.config.active_provider_config_mut().api_key = key;
            }
            if !model.is_empty() {
                self.data.config.active_provider_config_mut().model = model;
            }
            self.data.config.save();
            self.data.config_saved = true;
            self.ui.button(id!(save_btn)).apply_over(cx, live! {
                draw_bg: { color: (Vec4::from_u32(0x142a1eff)), border_color_1: (Vec4::from_u32(0x24503aff)), border_color_2: (Vec4::from_u32(0x24503aff)) }
                draw_text: { color: (Vec4::from_u32(0x3fb88cff)) }
            });
            self.ui.button(id!(save_btn)).set_text(cx, "Saved");
            self.update_model_labels(cx);
            self.ui.redraw(cx);
        }
    }

    fn handle_http_stream(&mut self, cx: &mut Cx, _request_id: LiveId, response: &HttpResponse, _scope: &mut Scope) {
        if let Some(data) = response.get_string_body() {
            self.data.sse_buffer.push_str(&data);
            self.process_sse_buffer();
            if !self.data.current_response.is_empty() {
                if let Some(conv) = self.data.conversations.get_mut(self.data.active_conversation) {
                    conv.update_assistant_streaming(&self.data.current_response);
                }
            }
            self.ui.redraw(cx);
        }
    }

    fn handle_http_stream_complete(&mut self, cx: &mut Cx, _request_id: LiveId, _response: &HttpResponse, _scope: &mut Scope) {
        if !self.data.sse_buffer.is_empty() {
            self.process_sse_buffer();
        }
        self.data.is_streaming = false;
        let response = self.data.current_response.clone();
        if let Some(conv) = self.data.conversations.get_mut(self.data.active_conversation) {
            conv.finalize_assistant_message(&response);
            conv.save();
        }
        self.data.current_response.clear();
        self.data.sse_buffer.clear();
        self.ui.redraw(cx);
    }

    fn handle_http_request_error(&mut self, cx: &mut Cx, _request_id: LiveId, err: &HttpError, _scope: &mut Scope) {
        self.data.is_streaming = false;
        self.data.error_message = Some(format!("Request error: {}", err.message));
        self.data.current_response.clear();
        self.data.sse_buffer.clear();
        self.ui.redraw(cx);
    }

    fn handle_http_response(&mut self, cx: &mut Cx, _request_id: LiveId, response: &HttpResponse, _scope: &mut Scope) {
        if response.status_code != 200 {
            self.data.is_streaming = false;
            let body = response
                .get_string_body()
                .unwrap_or_else(|| format!("HTTP {}", response.status_code));
            self.data.error_message = Some(body);
            self.data.current_response.clear();
            self.data.sse_buffer.clear();
            self.ui.redraw(cx);
        }
    }
}

impl App {
    fn send_message(&mut self, cx: &mut Cx) {
        let text = self.ui.text_input(id!(message_input)).text();
        if text.trim().is_empty() || self.data.is_streaming {
            return;
        }

        self.ui.text_input(id!(message_input)).set_text(cx, "");
        self.data.error_message = None;

        let conv = &mut self.data.conversations[self.data.active_conversation];
        conv.add_user_message(&text);

        self.data.is_streaming = true;
        self.data.current_response.clear();
        self.data.sse_buffer.clear();

        let provider_config = self.data.config.active_provider_config().clone();
        let messages = conv.messages.clone();

        crate::api::start_completion(cx, &provider_config, &messages);
        self.ui.redraw(cx);
    }

    fn process_sse_buffer(&mut self) {
        while let Some(pos) = self.data.sse_buffer.find("\n\n") {
            let event_str = self.data.sse_buffer[..pos].to_string();
            self.data.sse_buffer = self.data.sse_buffer[pos + 2..].to_string();

            for line in event_str.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        continue;
                    }
                    self.parse_sse_data(data);
                }
            }
        }
    }

    fn parse_sse_data(&mut self, data: &str) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
            // OpenAI format
            if let Some(content) = parsed["choices"][0]["delta"]["content"].as_str() {
                if !content.is_empty() {
                    self.data.current_response.push_str(content);
                }
                return;
            }
            // Anthropic format
            if parsed["type"].as_str() == Some("content_block_delta") {
                if let Some(text) = parsed["delta"]["text"].as_str() {
                    if !text.is_empty() {
                        self.data.current_response.push_str(text);
                    }
                }
            }
        }
    }

    fn switch_view(&mut self, cx: &mut Cx) {
        self.ui.view(id!(chat_view)).set_visible(cx, !self.data.show_settings);
        self.ui.view(id!(settings_view)).set_visible(cx, self.data.show_settings);
    }

    fn update_model_labels(&mut self, cx: &mut Cx) {
        let model = self.data.config.active_provider_config().model.clone();
        self.ui.label(id!(model_label)).set_text(cx, &model);
        self.ui.label(id!(bottom_model_label)).set_text(cx, &format!(" {}", model));
    }

    fn update_settings_ui(&mut self, cx: &mut Cx) {
        let is_openai = self.data.config.active_provider == Provider::OpenAI;

        // Provider toggle
        if is_openai {
            self.ui.button(id!(openai_btn)).apply_over(cx, live! {
                draw_bg: { color: (Vec4::from_u32(0x1e2a38ff)) }
                draw_text: { color: (Vec4::from_u32(0xc9a84cff)) }
            });
            self.ui.button(id!(anthropic_btn)).apply_over(cx, live! {
                draw_bg: { color: (Vec4::from_u32(0x00000000)) }
                draw_text: { color: (Vec4::from_u32(0x8a909aff)) }
            });
        } else {
            self.ui.button(id!(openai_btn)).apply_over(cx, live! {
                draw_bg: { color: (Vec4::from_u32(0x00000000)) }
                draw_text: { color: (Vec4::from_u32(0x8a909aff)) }
            });
            self.ui.button(id!(anthropic_btn)).apply_over(cx, live! {
                draw_bg: { color: (Vec4::from_u32(0x1e2a38ff)) }
                draw_text: { color: (Vec4::from_u32(0xc9a84cff)) }
            });
        }

        // Presets
        let presets: Vec<&str> = if is_openai {
            vec!["GPT-5", "GPT-4.1", "o3", "o4-mini"]
        } else {
            vec!["Opus", "Sonnet", "Haiku", ""]
        };
        let preset_ids = [id!(preset_0), id!(preset_1), id!(preset_2), id!(preset_3)];
        for (i, pid) in preset_ids.iter().enumerate() {
            let btn = self.ui.button(*pid);
            if i < presets.len() && !presets[i].is_empty() {
                btn.set_text(cx, presets[i]);
                btn.set_visible(cx, true);
                let is_active = is_preset_active(&self.data.config, presets[i]);
                if is_active {
                    btn.apply_over(cx, live! {
                        draw_bg: { color: (Vec4::from_u32(0x2a2414ff)), border_color_1: (Vec4::from_u32(0xc9a84cff)), border_color_2: (Vec4::from_u32(0xc9a84cff)) }
                        draw_text: { color: (Vec4::from_u32(0xc9a84cff)) }
                    });
                } else {
                    btn.apply_over(cx, live! {
                        draw_bg: { color: (Vec4::from_u32(0x00000000)), border_color_1: (Vec4::from_u32(0x3a4a5aff)), border_color_2: (Vec4::from_u32(0x3a4a5aff)) }
                        draw_text: { color: (Vec4::from_u32(0x8a909aff)) }
                    });
                }
            } else {
                btn.set_visible(cx, false);
            }
        }

        // Connection fields
        let active_cfg = self.data.config.active_provider_config().clone();
        self.ui.text_input(id!(api_url_input)).set_text(cx, &active_cfg.api_url);
        self.ui.text_input(id!(api_key_input)).set_text(cx, &active_cfg.api_key);
        self.ui.text_input(id!(model_id_input)).set_text(cx, &active_cfg.model);

        // Save button
        if !self.data.config_saved {
            self.ui.button(id!(save_btn)).set_text(cx, "Save");
            self.ui.button(id!(save_btn)).apply_over(cx, live! {
                draw_bg: { color: (Vec4::from_u32(0x1a2430ff)), border_color_1: (Vec4::from_u32(0x3a4a5aff)), border_color_2: (Vec4::from_u32(0x3a4a5aff)) }
                draw_text: { color: (Vec4::from_u32(0xc9a84cff)) }
            });
        }
    }

    fn handle_preset_clicks(&mut self, cx: &mut Cx, actions: &Actions) {
        let preset_ids = [id!(preset_0), id!(preset_1), id!(preset_2), id!(preset_3)];
        let is_openai = self.data.config.active_provider == Provider::OpenAI;
        let presets: Vec<&str> = if is_openai {
            vec!["GPT-5", "GPT-4.1", "o3", "o4-mini"]
        } else {
            vec!["Opus", "Sonnet", "Haiku"]
        };

        for (i, pid) in preset_ids.iter().enumerate() {
            if i < presets.len() && self.ui.button(*pid).clicked(actions) {
                self.data.config.apply_preset(presets[i]);
                self.data.config_saved = false;
                self.update_settings_ui(cx);
                self.update_model_labels(cx);
                self.ui.redraw(cx);
                break;
            }
        }
    }
}

fn is_preset_active(config: &AppConfig, preset: &str) -> bool {
    let model = &config.active_provider_config().model;
    match preset {
        "GPT-5" => model == "gpt-5",
        "GPT-4.1" => model == "gpt-4.1",
        "o3" => model == "o3",
        "o4-mini" => model == "o4-mini",
        "Opus" => model.contains("opus"),
        "Sonnet" => model.contains("sonnet"),
        "Haiku" => model.contains("haiku"),
        _ => false,
    }
}
