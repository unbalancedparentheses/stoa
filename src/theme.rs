use iced::Color;

// Backgrounds — softer, warmer tones
pub const BG: Color = Color::from_rgb8(0x10, 0x12, 0x18);
pub const MAIN_BG: Color = Color::from_rgb8(0x14, 0x16, 0x1e);
pub const HEADER_BG: Color = Color::from_rgb8(0x14, 0x16, 0x1e);
pub const BAR_BG: Color = Color::from_rgb8(0x10, 0x12, 0x18);
pub const CARD_BG: Color = Color::from_rgb8(0x1a, 0x1d, 0x26);
pub const BG_HOVER: Color = Color::from_rgb8(0x1e, 0x22, 0x2e);
pub const BG_ACTIVE: Color = Color::from_rgb8(0x1c, 0x20, 0x2a);
pub const USER_BG: Color = Color::from_rgb8(0x1e, 0x22, 0x2e);
pub const INPUT_BG: Color = Color::from_rgb8(0x18, 0x1b, 0x24);
pub const CODE_BG: Color = Color::from_rgb8(0x0e, 0x10, 0x16);

// Text — cleaner hierarchy
pub const TEXT_HEAD: Color = Color::from_rgb8(0xec, 0xed, 0xf0);
pub const TEXT_BODY: Color = Color::from_rgb8(0xc8, 0xca, 0xd0);
pub const TEXT_SEC: Color = Color::from_rgb8(0x80, 0x85, 0x95);
pub const TEXT_MUTED: Color = Color::from_rgb8(0x48, 0x4e, 0x5e);

// Accent — subtle blue instead of gold
pub const ACCENT: Color = Color::from_rgb8(0x6e, 0xa0, 0xd4);
pub const ACCENT_DIM: Color = Color::from_rgb8(0x2a, 0x3a, 0x50);

// Borders — barely visible
pub const BORDER_DEFAULT: Color = Color::from_rgb8(0x24, 0x28, 0x34);
pub const BORDER_SUBTLE: Color = Color::from_rgb8(0x1c, 0x1f, 0x28);
pub const DIVIDER: Color = Color::from_rgb8(0x1c, 0x1f, 0x28);

// Semantic
pub const DANGER: Color = Color::from_rgb8(0xe0, 0x60, 0x60);
pub const SUCCESS: Color = Color::from_rgb8(0x50, 0xc0, 0x8a);

// Selection
pub const SELECTION: Color = Color::from_rgb8(0x2a, 0x3e, 0x55);

// Error
pub const ERROR_BG: Color = Color::from_rgb8(0x28, 0x14, 0x14);
pub const ERROR_BORDER: Color = Color::from_rgb8(0x40, 0x20, 0x20);
pub const ERROR_MUTED: Color = Color::from_rgb8(0x80, 0x50, 0x50);

// Diff
#[allow(dead_code)]
pub const DIFF_A_BG: Color = Color::from_rgb8(0x18, 0x30, 0x22);
#[allow(dead_code)]
pub const DIFF_B_BG: Color = Color::from_rgb8(0x30, 0x2c, 0x18);
pub const DIFF_A_TEXT: Color = Color::from_rgb8(0x60, 0xd0, 0x80);
pub const DIFF_B_TEXT: Color = Color::from_rgb8(0xd0, 0xb8, 0x60);

// Overlay
pub const OVERLAY_BG: Color = Color { r: 0.05, g: 0.06, b: 0.08, a: 0.90 };

#[allow(dead_code)]
pub const TAG_BG: Color = Color::from_rgb8(0x20, 0x24, 0x30);
