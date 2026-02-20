use egui::{Color32, CornerRadius, Stroke, Visuals};

// Morphe palette
pub const BG_DARKEST: Color32 = Color32::from_rgb(0x10, 0x17, 0x20);
pub const BG_SIDEBAR: Color32 = Color32::from_rgb(0x12, 0x1a, 0x24);
pub const BG_MAIN: Color32 = Color32::from_rgb(0x16, 0x1e, 0x2a);
pub const BG_HEADER: Color32 = Color32::from_rgb(0x14, 0x1c, 0x26);
pub const BG_CARD: Color32 = Color32::from_rgb(0x1a, 0x24, 0x30);
pub const BG_INPUT: Color32 = Color32::from_rgb(0x12, 0x1a, 0x24);
pub const BG_HOVER: Color32 = Color32::from_rgb(0x1a, 0x24, 0x30);
pub const BG_ACTIVE: Color32 = Color32::from_rgb(0x1e, 0x28, 0x36);
pub const BG_USER_BUBBLE: Color32 = Color32::from_rgb(0x1e, 0x28, 0x36);

pub const ACCENT: Color32 = Color32::from_rgb(0xc9, 0xa8, 0x4c);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(0x6a, 0x5e, 0x3a);
#[allow(dead_code)]
pub const PRIMARY: Color32 = Color32::from_rgb(0x4a, 0x9e, 0xc9);
pub const SUCCESS: Color32 = Color32::from_rgb(0x3f, 0xb8, 0x8c);
pub const DANGER: Color32 = Color32::from_rgb(0xda, 0x6b, 0x6b);

pub const TEXT_HEAD: Color32 = Color32::from_rgb(0xe8, 0xe0, 0xd0);
pub const TEXT_BODY: Color32 = Color32::from_rgb(0xd0, 0xc8, 0xb8);
pub const TEXT_SEC: Color32 = Color32::from_rgb(0x8a, 0x90, 0x9a);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x50, 0x5a, 0x66);

pub const DIVIDER: Color32 = Color32::from_rgb(0x1e, 0x28, 0x34);
pub const BORDER_DEFAULT: Color32 = Color32::from_rgb(0x3a, 0x4a, 0x5a);
pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(0x1e, 0x28, 0x34);

pub fn apply_morphe_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    visuals.override_text_color = Some(TEXT_HEAD);
    visuals.panel_fill = BG_MAIN;
    visuals.window_fill = BG_MAIN;
    visuals.extreme_bg_color = BG_INPUT;
    visuals.faint_bg_color = BG_SIDEBAR;

    visuals.widgets.noninteractive.bg_fill = BG_CARD;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_SEC);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER_SUBTLE);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);

    visuals.widgets.inactive.bg_fill = BG_INPUT;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_SEC);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER_DEFAULT);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(6);

    visuals.widgets.hovered.bg_fill = BG_HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_HEAD);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT_DIM);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(6);

    visuals.widgets.active.bg_fill = BG_ACTIVE;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.active.corner_radius = CornerRadius::same(6);

    visuals.widgets.open.bg_fill = BG_ACTIVE;
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.open.corner_radius = CornerRadius::same(6);

    visuals.selection.bg_fill = Color32::from_rgb(0x2a, 0x3e, 0x55);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);

    visuals.window_corner_radius = CornerRadius::same(8);
    visuals.window_stroke = Stroke::new(1.0, BORDER_SUBTLE);

    ctx.set_visuals(visuals);
}
