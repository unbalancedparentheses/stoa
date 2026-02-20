use iced::Color;

// Backgrounds
pub const BG: Color = Color::from_rgb8(0x12, 0x1a, 0x24);
pub const MAIN_BG: Color = Color::from_rgb8(0x16, 0x1e, 0x2a);
pub const HEADER_BG: Color = Color::from_rgb8(0x14, 0x1c, 0x26);
pub const BAR_BG: Color = Color::from_rgb8(0x10, 0x17, 0x20);
pub const CARD_BG: Color = Color::from_rgb8(0x1a, 0x24, 0x30);
pub const BG_HOVER: Color = Color::from_rgb8(0x1a, 0x24, 0x30);
pub const BG_ACTIVE: Color = Color::from_rgb8(0x1e, 0x28, 0x36);
pub const USER_BG: Color = Color::from_rgb8(0x1e, 0x28, 0x36);
pub const INPUT_BG: Color = Color::from_rgb8(0x12, 0x1a, 0x24);
pub const CODE_BG: Color = Color::from_rgb8(0x0e, 0x15, 0x1e);

// Text
pub const TEXT_HEAD: Color = Color::from_rgb8(0xe8, 0xe0, 0xd0);
pub const TEXT_BODY: Color = Color::from_rgb8(0xd0, 0xc8, 0xb8);
pub const TEXT_SEC: Color = Color::from_rgb8(0x8a, 0x90, 0x9a);
pub const TEXT_MUTED: Color = Color::from_rgb8(0x50, 0x5a, 0x66);

// Accent
pub const ACCENT: Color = Color::from_rgb8(0xc9, 0xa8, 0x4c);
pub const ACCENT_DIM: Color = Color::from_rgb8(0x6a, 0x5e, 0x3a);

// Borders
pub const BORDER_DEFAULT: Color = Color::from_rgb8(0x3a, 0x4a, 0x5a);
pub const BORDER_SUBTLE: Color = Color::from_rgb8(0x1e, 0x28, 0x34);
pub const DIVIDER: Color = Color::from_rgb8(0x1e, 0x28, 0x34);

// Semantic
pub const DANGER: Color = Color::from_rgb8(0xda, 0x6b, 0x6b);
pub const SUCCESS: Color = Color::from_rgb8(0x3f, 0xb8, 0x8c);

// Selection
pub const SELECTION: Color = Color::from_rgb8(0x2a, 0x3e, 0x55);

// Error
pub const ERROR_BG: Color = Color::from_rgb8(0x2a, 0x18, 0x18);
pub const ERROR_BORDER: Color = Color::from_rgb8(0x44, 0x22, 0x22);
pub const ERROR_MUTED: Color = Color::from_rgb8(0x88, 0x55, 0x55);

// Diff
#[allow(dead_code)]
pub const DIFF_A_BG: Color = Color::from_rgb8(0x1a, 0x3a, 0x2a);
#[allow(dead_code)]
pub const DIFF_B_BG: Color = Color::from_rgb8(0x3a, 0x35, 0x1a);
pub const DIFF_A_TEXT: Color = Color::from_rgb8(0x6a, 0xd0, 0x8a);
pub const DIFF_B_TEXT: Color = Color::from_rgb8(0xd0, 0xc0, 0x6a);

// Overlay
pub const OVERLAY_BG: Color = Color { r: 0.07, g: 0.10, b: 0.14, a: 0.88 };

// Tags
#[allow(dead_code)]
pub const TAG_BG: Color = Color::from_rgb8(0x24, 0x2e, 0x3a);
