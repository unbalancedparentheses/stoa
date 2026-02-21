#![allow(non_snake_case)]

use iced::Color;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

// ── Font size constants ──────────────────────────────────────────────
pub const FONT_TITLE: f32 = 26.0;
pub const FONT_H1: f32 = 17.0;
pub const FONT_BODY: f32 = 15.0;
pub const FONT_SMALL: f32 = 13.0;
pub const FONT_CAPTION: f32 = 12.0;
pub const FONT_MICRO: f32 = 11.0;
pub const FONT_BADGE: f32 = 10.0;

// Markdown heading sizes
pub const FONT_MD_H1: f32 = 26.0;
pub const FONT_MD_H2: f32 = 22.0;
pub const FONT_MD_H3: f32 = 19.0;
pub const FONT_MD_H4: f32 = 17.0;

// ── Theme colors struct ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ThemeColors {
    // Backgrounds
    pub bg: Color,
    pub main_bg: Color,
    pub header_bg: Color,
    pub bar_bg: Color,
    pub card_bg: Color,
    pub bg_hover: Color,
    pub bg_active: Color,
    pub user_bg: Color,
    pub input_bg: Color,
    pub code_bg: Color,
    // Text
    pub text_head: Color,
    pub text_body: Color,
    pub text_sec: Color,
    pub text_muted: Color,
    // Accent
    pub accent: Color,
    pub accent_dim: Color,
    // Borders
    pub border_default: Color,
    pub border_subtle: Color,
    pub divider: Color,
    // Semantic
    pub danger: Color,
    pub success: Color,
    // Selection
    pub selection: Color,
    // Error
    pub error_bg: Color,
    pub error_border: Color,
    pub error_muted: Color,
    // Diff
    pub diff_a_bg: Color,
    pub diff_b_bg: Color,
    pub diff_a_text: Color,
    pub diff_b_text: Color,
    // Overlay
    pub overlay_bg: Color,
    // Tag
    pub tag_bg: Color,
    // Derived (inline colors from UI)
    pub separator: Color,
    pub tab_active_bg: Color,
    pub tab_hover_bg: Color,
    pub chip_active_bg: Color,
    pub saved_bg: Color,
    pub saved_border: Color,
    pub stop_btn_bg: Color,
    pub send_disabled_bg: Color,
    pub run_n_bg: Color,
    pub run_n_hover: Color,
    pub conflict_bg: Color,
    pub debug_active_bg: Color,
    pub debug_active_border: Color,
}

// ── Theme name enum ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeName {
    FinancialTimes,
    StoaDark,
    SolarizedLight,
    SolarizedDark,
    Nord,
}

impl Default for ThemeName {
    fn default() -> Self {
        ThemeName::FinancialTimes
    }
}

impl ThemeName {
    pub fn all() -> &'static [ThemeName] {
        &[
            ThemeName::FinancialTimes,
            ThemeName::StoaDark,
            ThemeName::SolarizedLight,
            ThemeName::SolarizedDark,
            ThemeName::Nord,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            ThemeName::FinancialTimes => "Financial Times",
            ThemeName::StoaDark => "Stoa Dark",
            ThemeName::SolarizedLight => "Solarized Light",
            ThemeName::SolarizedDark => "Solarized Dark",
            ThemeName::Nord => "Nord",
        }
    }

    pub fn colors(&self) -> ThemeColors {
        match self {
            ThemeName::FinancialTimes => ft_theme(),
            ThemeName::StoaDark => stoa_dark_theme(),
            ThemeName::SolarizedLight => solarized_light_theme(),
            ThemeName::SolarizedDark => solarized_dark_theme(),
            ThemeName::Nord => nord_theme(),
        }
    }

    pub fn iced_palette(&self) -> iced::theme::Palette {
        let c = self.colors();
        let warning = match self {
            ThemeName::FinancialTimes => Color::from_rgb8(0xCC, 0x84, 0x00),
            ThemeName::StoaDark => Color::from_rgb8(0xd0, 0xa0, 0x50),
            ThemeName::SolarizedLight | ThemeName::SolarizedDark => Color::from_rgb8(0xB5, 0x89, 0x00),
            ThemeName::Nord => Color::from_rgb8(0xEB, 0xCB, 0x8B),
        };
        iced::theme::Palette {
            background: c.bg,
            text: c.text_head,
            primary: c.accent,
            success: c.success,
            warning,
            danger: c.danger,
        }
    }
}

// ── Theme presets ────────────────────────────────────────────────────

fn ft_theme() -> ThemeColors {
    // Design principles applied:
    // - Refactoring UI: 8-10 grey shades, warm-tinted; accent at 10% for selections
    // - Apple HIG / WCAG AA: 4.5:1 body text, 3:1 UI components
    // - 60-30-10: 60% FT pink main, 30% warm sidebar, 10% teal accent
    // - Tufte: every visual element earns its place
    ThemeColors {
        // 30% secondary surface: warm tan, ~8% darker than main
        bg: Color::from_rgb8(0xEB, 0xD8, 0xC2),
        // 60% dominant: the signature FT pink
        main_bg: Color::from_rgb8(0xFF, 0xF1, 0xE5),
        header_bg: Color::from_rgb8(0xEB, 0xD8, 0xC2),
        bar_bg: Color::from_rgb8(0xEB, 0xD8, 0xC2),
        // Cards: white for maximum content readability
        card_bg: Color::from_rgb8(0xFF, 0xFF, 0xFF),
        // Hover: warm darkening, clearly perceptible
        bg_hover: Color::from_rgb8(0xDF, 0xC8, 0xAE),
        // Active: gentle highlight — text weight does the heavy lifting
        bg_active: Color::from_rgb8(0xE2, 0xCE, 0xB6),
        user_bg: Color::from_rgb8(0xF5, 0xE8, 0xDA),
        input_bg: Color::from_rgb8(0xFF, 0xFF, 0xFF),
        code_bg: Color::from_rgb8(0xF6, 0xE9, 0xD8),
        // Text hierarchy: 10-15% luminance gaps between levels
        // head ~0.024 lum → 12:1 on main (AAA)
        text_head: Color::from_rgb8(0x1A, 0x17, 0x14),
        // body ~0.050 lum → 9:1 on main (AAA)
        text_body: Color::from_rgb8(0x3D, 0x38, 0x32),
        // sec ~0.100 lum → 6:1 on main (AA)
        text_sec: Color::from_rgb8(0x5C, 0x56, 0x4E),
        // muted ~0.155 lum → 4.5:1 on main (AA), 3.5:1 on sidebar (AA large)
        text_muted: Color::from_rgb8(0x6E, 0x66, 0x5C),
        // 10% accent: FT teal — used sparingly for interactive elements
        accent: Color::from_rgb8(0x0D, 0x76, 0x80),
        // Accent at ~12% opacity: teal wash for selections, chips
        accent_dim: Color::from_rgb8(0xCE, 0xE4, 0xE6),
        // Borders: warm-tinted, subtle but present (3:1 vs background)
        border_default: Color::from_rgb8(0xD2, 0xC0, 0xAA),
        border_subtle: Color::from_rgb8(0xDC, 0xCC, 0xB8),
        // Divider: slightly bolder than border for clear section breaks
        divider: Color::from_rgb8(0xCA, 0xB6, 0x9E),
        danger: Color::from_rgb8(0x99, 0x0F, 0x3D),
        success: Color::from_rgb8(0x09, 0x82, 0x5E),
        // Selection: teal wash (accent at 10%)
        selection: Color::from_rgb8(0xCE, 0xE4, 0xE6),
        error_bg: Color::from_rgb8(0xFB, 0xE7, 0xE8),
        error_border: Color::from_rgb8(0xE0, 0x9E, 0xA3),
        error_muted: Color::from_rgb8(0xC0, 0x7A, 0x80),
        diff_a_bg: Color::from_rgb8(0xD5, 0xEA, 0xDA),
        diff_b_bg: Color::from_rgb8(0xF5, 0xE6, 0xD0),
        diff_a_text: Color::from_rgb8(0x1E, 0x8A, 0x46),
        diff_b_text: Color::from_rgb8(0xAA, 0x7E, 0x20),
        overlay_bg: Color { r: 0.88, g: 0.84, b: 0.78, a: 0.85 },
        tag_bg: Color::from_rgb8(0xDC, 0xCC, 0xB8),
        separator: Color::from_rgb8(0xCA, 0xB6, 0x9E),
        // Tabs/chips: teal wash for active (accent coherence)
        tab_active_bg: Color::from_rgb8(0xCE, 0xE4, 0xE6),
        tab_hover_bg: Color::from_rgb8(0xDF, 0xC8, 0xAE),
        chip_active_bg: Color::from_rgb8(0xCE, 0xE4, 0xE6),
        saved_bg: Color::from_rgb8(0xD5, 0xEA, 0xDA),
        saved_border: Color::from_rgb8(0x09, 0x82, 0x5E),
        stop_btn_bg: Color::from_rgb8(0xC0, 0x5A, 0x6A),
        send_disabled_bg: Color::from_rgb8(0xDC, 0xCC, 0xB8),
        run_n_bg: Color::from_rgb8(0x09, 0x82, 0x5E),
        run_n_hover: Color::from_rgb8(0x0B, 0x9E, 0x72),
        conflict_bg: Color::from_rgb8(0xFB, 0xE7, 0xE8),
        debug_active_bg: Color::from_rgb8(0xD5, 0xEA, 0xDA),
        debug_active_border: Color::from_rgb8(0x09, 0x82, 0x5E),
    }
}

fn stoa_dark_theme() -> ThemeColors {
    ThemeColors {
        bg: Color::from_rgb8(0x10, 0x12, 0x18),
        main_bg: Color::from_rgb8(0x14, 0x16, 0x1e),
        header_bg: Color::from_rgb8(0x14, 0x16, 0x1e),
        bar_bg: Color::from_rgb8(0x10, 0x12, 0x18),
        card_bg: Color::from_rgb8(0x1a, 0x1d, 0x26),
        bg_hover: Color::from_rgb8(0x1e, 0x22, 0x2e),
        bg_active: Color::from_rgb8(0x1c, 0x20, 0x2a),
        user_bg: Color::from_rgb8(0x1e, 0x22, 0x2e),
        input_bg: Color::from_rgb8(0x18, 0x1b, 0x24),
        code_bg: Color::from_rgb8(0x0e, 0x10, 0x16),
        text_head: Color::from_rgb8(0xec, 0xed, 0xf0),
        text_body: Color::from_rgb8(0xc8, 0xca, 0xd0),
        text_sec: Color::from_rgb8(0x80, 0x85, 0x95),
        text_muted: Color::from_rgb8(0x48, 0x4e, 0x5e),
        accent: Color::from_rgb8(0x6e, 0xa0, 0xd4),
        accent_dim: Color::from_rgb8(0x2a, 0x3a, 0x50),
        border_default: Color::from_rgb8(0x24, 0x28, 0x34),
        border_subtle: Color::from_rgb8(0x1c, 0x1f, 0x28),
        divider: Color::from_rgb8(0x1c, 0x1f, 0x28),
        danger: Color::from_rgb8(0xe0, 0x60, 0x60),
        success: Color::from_rgb8(0x50, 0xc0, 0x8a),
        selection: Color::from_rgb8(0x2a, 0x3e, 0x55),
        error_bg: Color::from_rgb8(0x28, 0x14, 0x14),
        error_border: Color::from_rgb8(0x40, 0x20, 0x20),
        error_muted: Color::from_rgb8(0x80, 0x50, 0x50),
        diff_a_bg: Color::from_rgb8(0x18, 0x30, 0x22),
        diff_b_bg: Color::from_rgb8(0x30, 0x2c, 0x18),
        diff_a_text: Color::from_rgb8(0x60, 0xd0, 0x80),
        diff_b_text: Color::from_rgb8(0xd0, 0xb8, 0x60),
        overlay_bg: Color { r: 0.05, g: 0.06, b: 0.08, a: 0.90 },
        tag_bg: Color::from_rgb8(0x20, 0x24, 0x30),
        separator: Color::from_rgb8(0x1e, 0x28, 0x34),
        tab_active_bg: Color::from_rgb8(0x1e, 0x2a, 0x38),
        tab_hover_bg: Color::from_rgb8(0x16, 0x20, 0x2c),
        chip_active_bg: Color::from_rgb8(0x2a, 0x24, 0x14),
        saved_bg: Color::from_rgb8(0x14, 0x2a, 0x1e),
        saved_border: Color::from_rgb8(0x24, 0x50, 0x3a),
        stop_btn_bg: Color::from_rgb8(0x8a, 0x3a, 0x3a),
        send_disabled_bg: Color::from_rgb8(0x1a, 0x22, 0x2e),
        run_n_bg: Color::from_rgb8(0x1e, 0x5a, 0x42),
        run_n_hover: Color::from_rgb8(0x2a, 0x7a, 0x5a),
        conflict_bg: Color::from_rgb8(0x2a, 0x18, 0x18),
        debug_active_bg: Color::from_rgb8(0x1e, 0x3a, 0x2a),
        debug_active_border: Color::from_rgb8(0x2a, 0x64, 0x48),
    }
}

fn solarized_light_theme() -> ThemeColors {
    ThemeColors {
        bg: Color::from_rgb8(0xEE, 0xE8, 0xD5),
        main_bg: Color::from_rgb8(0xFD, 0xF6, 0xE3),
        header_bg: Color::from_rgb8(0xEE, 0xE8, 0xD5),
        bar_bg: Color::from_rgb8(0xEE, 0xE8, 0xD5),
        card_bg: Color::from_rgb8(0xFD, 0xF6, 0xE3),
        bg_hover: Color::from_rgb8(0xE4, 0xDE, 0xCA),
        bg_active: Color::from_rgb8(0xE8, 0xE2, 0xCF),
        user_bg: Color::from_rgb8(0xE8, 0xE2, 0xCF),
        input_bg: Color::from_rgb8(0xFD, 0xF6, 0xE3),
        code_bg: Color::from_rgb8(0xEE, 0xE8, 0xD5),
        text_head: Color::from_rgb8(0x07, 0x36, 0x42),
        text_body: Color::from_rgb8(0x65, 0x7B, 0x83),
        text_sec: Color::from_rgb8(0x83, 0x94, 0x96),
        text_muted: Color::from_rgb8(0x93, 0xA1, 0xA1),
        accent: Color::from_rgb8(0x26, 0x8B, 0xD2),
        accent_dim: Color::from_rgb8(0xD5, 0xE8, 0xF5),
        border_default: Color::from_rgb8(0xD5, 0xCF, 0xBD),
        border_subtle: Color::from_rgb8(0xE0, 0xDA, 0xC8),
        divider: Color::from_rgb8(0xD5, 0xCF, 0xBD),
        danger: Color::from_rgb8(0xDC, 0x32, 0x2F),
        success: Color::from_rgb8(0x85, 0x99, 0x00),
        selection: Color::from_rgb8(0xD5, 0xE8, 0xF5),
        error_bg: Color::from_rgb8(0xFD, 0xE8, 0xE8),
        error_border: Color::from_rgb8(0xDC, 0x96, 0x94),
        error_muted: Color::from_rgb8(0xC0, 0x7A, 0x78),
        diff_a_bg: Color::from_rgb8(0xE8, 0xF0, 0xD0),
        diff_b_bg: Color::from_rgb8(0xF5, 0xE8, 0xD0),
        diff_a_text: Color::from_rgb8(0x85, 0x99, 0x00),
        diff_b_text: Color::from_rgb8(0xB5, 0x89, 0x00),
        overlay_bg: Color { r: 0.93, g: 0.91, b: 0.85, a: 0.90 },
        tag_bg: Color::from_rgb8(0xE4, 0xDE, 0xCA),
        separator: Color::from_rgb8(0xD0, 0xCA, 0xB8),
        tab_active_bg: Color::from_rgb8(0xE4, 0xDE, 0xCA),
        tab_hover_bg: Color::from_rgb8(0xE0, 0xDA, 0xC6),
        chip_active_bg: Color::from_rgb8(0xD5, 0xE8, 0xF5),
        saved_bg: Color::from_rgb8(0xE8, 0xF0, 0xD0),
        saved_border: Color::from_rgb8(0x85, 0x99, 0x00),
        stop_btn_bg: Color::from_rgb8(0xD0, 0x5A, 0x58),
        send_disabled_bg: Color::from_rgb8(0xE0, 0xDA, 0xC8),
        run_n_bg: Color::from_rgb8(0x6A, 0x80, 0x00),
        run_n_hover: Color::from_rgb8(0x85, 0x99, 0x00),
        conflict_bg: Color::from_rgb8(0xFD, 0xE8, 0xE8),
        debug_active_bg: Color::from_rgb8(0xE8, 0xF0, 0xD0),
        debug_active_border: Color::from_rgb8(0x85, 0x99, 0x00),
    }
}

fn solarized_dark_theme() -> ThemeColors {
    ThemeColors {
        bg: Color::from_rgb8(0x00, 0x2B, 0x36),
        main_bg: Color::from_rgb8(0x07, 0x36, 0x42),
        header_bg: Color::from_rgb8(0x07, 0x36, 0x42),
        bar_bg: Color::from_rgb8(0x00, 0x2B, 0x36),
        card_bg: Color::from_rgb8(0x0A, 0x3B, 0x47),
        bg_hover: Color::from_rgb8(0x0D, 0x44, 0x50),
        bg_active: Color::from_rgb8(0x0B, 0x40, 0x4C),
        user_bg: Color::from_rgb8(0x0D, 0x44, 0x50),
        input_bg: Color::from_rgb8(0x05, 0x32, 0x3E),
        code_bg: Color::from_rgb8(0x00, 0x25, 0x30),
        text_head: Color::from_rgb8(0xEE, 0xE8, 0xD5),
        text_body: Color::from_rgb8(0x83, 0x94, 0x96),
        text_sec: Color::from_rgb8(0x65, 0x7B, 0x83),
        text_muted: Color::from_rgb8(0x58, 0x6E, 0x75),
        accent: Color::from_rgb8(0x26, 0x8B, 0xD2),
        accent_dim: Color::from_rgb8(0x0E, 0x3D, 0x5C),
        border_default: Color::from_rgb8(0x0C, 0x40, 0x4D),
        border_subtle: Color::from_rgb8(0x0A, 0x3A, 0x46),
        divider: Color::from_rgb8(0x0A, 0x3A, 0x46),
        danger: Color::from_rgb8(0xDC, 0x32, 0x2F),
        success: Color::from_rgb8(0x85, 0x99, 0x00),
        selection: Color::from_rgb8(0x0E, 0x3D, 0x5C),
        error_bg: Color::from_rgb8(0x2A, 0x15, 0x15),
        error_border: Color::from_rgb8(0x50, 0x22, 0x22),
        error_muted: Color::from_rgb8(0x8A, 0x44, 0x44),
        diff_a_bg: Color::from_rgb8(0x0A, 0x38, 0x20),
        diff_b_bg: Color::from_rgb8(0x38, 0x30, 0x0A),
        diff_a_text: Color::from_rgb8(0x85, 0x99, 0x00),
        diff_b_text: Color::from_rgb8(0xB5, 0x89, 0x00),
        overlay_bg: Color { r: 0.0, g: 0.10, b: 0.13, a: 0.90 },
        tag_bg: Color::from_rgb8(0x0A, 0x3B, 0x47),
        separator: Color::from_rgb8(0x0C, 0x42, 0x4F),
        tab_active_bg: Color::from_rgb8(0x0D, 0x44, 0x50),
        tab_hover_bg: Color::from_rgb8(0x0A, 0x3C, 0x48),
        chip_active_bg: Color::from_rgb8(0x0E, 0x3D, 0x5C),
        saved_bg: Color::from_rgb8(0x0A, 0x38, 0x20),
        saved_border: Color::from_rgb8(0x58, 0x6E, 0x00),
        stop_btn_bg: Color::from_rgb8(0x8A, 0x2A, 0x28),
        send_disabled_bg: Color::from_rgb8(0x08, 0x34, 0x40),
        run_n_bg: Color::from_rgb8(0x4A, 0x5C, 0x00),
        run_n_hover: Color::from_rgb8(0x60, 0x74, 0x00),
        conflict_bg: Color::from_rgb8(0x2A, 0x15, 0x15),
        debug_active_bg: Color::from_rgb8(0x0A, 0x38, 0x20),
        debug_active_border: Color::from_rgb8(0x58, 0x6E, 0x00),
    }
}

fn nord_theme() -> ThemeColors {
    ThemeColors {
        bg: Color::from_rgb8(0x2E, 0x34, 0x40),
        main_bg: Color::from_rgb8(0x3B, 0x42, 0x52),
        header_bg: Color::from_rgb8(0x3B, 0x42, 0x52),
        bar_bg: Color::from_rgb8(0x2E, 0x34, 0x40),
        card_bg: Color::from_rgb8(0x43, 0x4C, 0x5E),
        bg_hover: Color::from_rgb8(0x4C, 0x56, 0x6A),
        bg_active: Color::from_rgb8(0x48, 0x52, 0x64),
        user_bg: Color::from_rgb8(0x43, 0x4C, 0x5E),
        input_bg: Color::from_rgb8(0x38, 0x3E, 0x4E),
        code_bg: Color::from_rgb8(0x2E, 0x34, 0x40),
        text_head: Color::from_rgb8(0xEC, 0xEF, 0xF4),
        text_body: Color::from_rgb8(0xD8, 0xDE, 0xE9),
        text_sec: Color::from_rgb8(0x81, 0xA1, 0xC1),
        text_muted: Color::from_rgb8(0x4C, 0x56, 0x6A),
        accent: Color::from_rgb8(0x88, 0xC0, 0xD0),
        accent_dim: Color::from_rgb8(0x3B, 0x52, 0x5C),
        border_default: Color::from_rgb8(0x4C, 0x56, 0x6A),
        border_subtle: Color::from_rgb8(0x43, 0x4C, 0x5E),
        divider: Color::from_rgb8(0x43, 0x4C, 0x5E),
        danger: Color::from_rgb8(0xBF, 0x61, 0x6A),
        success: Color::from_rgb8(0xA3, 0xBE, 0x8C),
        selection: Color::from_rgb8(0x3B, 0x52, 0x5C),
        error_bg: Color::from_rgb8(0x3B, 0x2A, 0x2D),
        error_border: Color::from_rgb8(0x6A, 0x3A, 0x3E),
        error_muted: Color::from_rgb8(0x8A, 0x50, 0x55),
        diff_a_bg: Color::from_rgb8(0x34, 0x46, 0x38),
        diff_b_bg: Color::from_rgb8(0x46, 0x40, 0x34),
        diff_a_text: Color::from_rgb8(0xA3, 0xBE, 0x8C),
        diff_b_text: Color::from_rgb8(0xEB, 0xCB, 0x8B),
        overlay_bg: Color { r: 0.12, g: 0.14, b: 0.17, a: 0.90 },
        tag_bg: Color::from_rgb8(0x43, 0x4C, 0x5E),
        separator: Color::from_rgb8(0x4C, 0x56, 0x6A),
        tab_active_bg: Color::from_rgb8(0x4C, 0x56, 0x6A),
        tab_hover_bg: Color::from_rgb8(0x43, 0x4C, 0x5E),
        chip_active_bg: Color::from_rgb8(0x3B, 0x52, 0x5C),
        saved_bg: Color::from_rgb8(0x34, 0x46, 0x38),
        saved_border: Color::from_rgb8(0x6A, 0x8A, 0x5C),
        stop_btn_bg: Color::from_rgb8(0x8A, 0x44, 0x4A),
        send_disabled_bg: Color::from_rgb8(0x3B, 0x42, 0x52),
        run_n_bg: Color::from_rgb8(0x5A, 0x72, 0x50),
        run_n_hover: Color::from_rgb8(0x70, 0x8A, 0x64),
        conflict_bg: Color::from_rgb8(0x3B, 0x2A, 0x2D),
        debug_active_bg: Color::from_rgb8(0x34, 0x46, 0x38),
        debug_active_border: Color::from_rgb8(0x6A, 0x8A, 0x5C),
    }
}

// ── Thread-local storage ─────────────────────────────────────────────

thread_local! {
    static ACTIVE_THEME: RefCell<ThemeColors> = RefCell::new(ft_theme());
}

pub fn set_theme(name: ThemeName) {
    ACTIVE_THEME.with(|t| *t.borrow_mut() = name.colors());
}

fn with_theme<F, R>(f: F) -> R
where
    F: FnOnce(&ThemeColors) -> R,
{
    ACTIVE_THEME.with(|t| f(&t.borrow()))
}

// ── Accessor functions (match old constant names) ────────────────────

// Backgrounds
pub fn BG() -> Color { with_theme(|t| t.bg) }
pub fn MAIN_BG() -> Color { with_theme(|t| t.main_bg) }
pub fn HEADER_BG() -> Color { with_theme(|t| t.header_bg) }
pub fn BAR_BG() -> Color { with_theme(|t| t.bar_bg) }
pub fn CARD_BG() -> Color { with_theme(|t| t.card_bg) }
pub fn BG_HOVER() -> Color { with_theme(|t| t.bg_hover) }
pub fn BG_ACTIVE() -> Color { with_theme(|t| t.bg_active) }
pub fn USER_BG() -> Color { with_theme(|t| t.user_bg) }
pub fn INPUT_BG() -> Color { with_theme(|t| t.input_bg) }
pub fn CODE_BG() -> Color { with_theme(|t| t.code_bg) }

// Text
pub fn TEXT_HEAD() -> Color { with_theme(|t| t.text_head) }
pub fn TEXT_BODY() -> Color { with_theme(|t| t.text_body) }
pub fn TEXT_SEC() -> Color { with_theme(|t| t.text_sec) }
pub fn TEXT_MUTED() -> Color { with_theme(|t| t.text_muted) }

// Accent
pub fn ACCENT() -> Color { with_theme(|t| t.accent) }
pub fn ACCENT_DIM() -> Color { with_theme(|t| t.accent_dim) }

// Borders
pub fn BORDER_DEFAULT() -> Color { with_theme(|t| t.border_default) }
pub fn BORDER_SUBTLE() -> Color { with_theme(|t| t.border_subtle) }
pub fn DIVIDER() -> Color { with_theme(|t| t.divider) }

// Semantic
pub fn DANGER() -> Color { with_theme(|t| t.danger) }
pub fn SUCCESS() -> Color { with_theme(|t| t.success) }

// Selection
pub fn SELECTION() -> Color { with_theme(|t| t.selection) }

// Error
pub fn ERROR_BG() -> Color { with_theme(|t| t.error_bg) }
pub fn ERROR_BORDER() -> Color { with_theme(|t| t.error_border) }
pub fn ERROR_MUTED() -> Color { with_theme(|t| t.error_muted) }

// Diff
pub fn DIFF_A_BG() -> Color { with_theme(|t| t.diff_a_bg) }
pub fn DIFF_B_BG() -> Color { with_theme(|t| t.diff_b_bg) }
pub fn DIFF_A_TEXT() -> Color { with_theme(|t| t.diff_a_text) }
pub fn DIFF_B_TEXT() -> Color { with_theme(|t| t.diff_b_text) }

// Overlay
pub fn OVERLAY_BG() -> Color { with_theme(|t| t.overlay_bg) }

// Tag
pub fn TAG_BG() -> Color { with_theme(|t| t.tag_bg) }

// Derived
pub fn SEPARATOR() -> Color { with_theme(|t| t.separator) }
pub fn TAB_ACTIVE_BG() -> Color { with_theme(|t| t.tab_active_bg) }
pub fn TAB_HOVER_BG() -> Color { with_theme(|t| t.tab_hover_bg) }
pub fn CHIP_ACTIVE_BG() -> Color { with_theme(|t| t.chip_active_bg) }
pub fn SAVED_BG() -> Color { with_theme(|t| t.saved_bg) }
pub fn SAVED_BORDER() -> Color { with_theme(|t| t.saved_border) }
pub fn STOP_BTN_BG() -> Color { with_theme(|t| t.stop_btn_bg) }
pub fn SEND_DISABLED_BG() -> Color { with_theme(|t| t.send_disabled_bg) }
pub fn RUN_N_BG() -> Color { with_theme(|t| t.run_n_bg) }
pub fn RUN_N_HOVER() -> Color { with_theme(|t| t.run_n_hover) }
pub fn CONFLICT_BG() -> Color { with_theme(|t| t.conflict_bg) }
pub fn DEBUG_ACTIVE_BG() -> Color { with_theme(|t| t.debug_active_bg) }
pub fn DEBUG_ACTIVE_BORDER() -> Color { with_theme(|t| t.debug_active_border) }
