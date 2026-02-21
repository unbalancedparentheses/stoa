use iced::keyboard;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    SendToAll,
    NewConversation,
    ShowSettings,
    QuickSwitcher,
    CommandPalette,
    ExportMarkdown,
    ToggleShortcutHelp,
}

#[derive(Debug, Clone, Copy)]
pub struct ShortcutSpec {
    pub action: ShortcutAction,
    pub label: &'static str,
    pub description: &'static str,
    pub default_macos: &'static str,
    pub default_other: &'static str,
}

const SPECS: [ShortcutSpec; 7] = [
    ShortcutSpec {
        action: ShortcutAction::SendToAll,
        label: "Send to all models",
        description: "Send to all available models",
        default_macos: "Cmd+Shift+Enter",
        default_other: "Ctrl+Shift+Enter",
    },
    ShortcutSpec {
        action: ShortcutAction::NewConversation,
        label: "New conversation",
        description: "Create a new conversation",
        default_macos: "Cmd+N",
        default_other: "Ctrl+N",
    },
    ShortcutSpec {
        action: ShortcutAction::ShowSettings,
        label: "Settings",
        description: "Open settings",
        default_macos: "Cmd+,",
        default_other: "Ctrl+,",
    },
    ShortcutSpec {
        action: ShortcutAction::QuickSwitcher,
        label: "Quick switcher",
        description: "Search conversations",
        default_macos: "Cmd+K",
        default_other: "Ctrl+K",
    },
    ShortcutSpec {
        action: ShortcutAction::CommandPalette,
        label: "Command palette",
        description: "Open command palette",
        default_macos: "Cmd+P",
        default_other: "Ctrl+P",
    },
    ShortcutSpec {
        action: ShortcutAction::ExportMarkdown,
        label: "Export Markdown",
        description: "Copy conversation as Markdown",
        default_macos: "Cmd+E",
        default_other: "Ctrl+E",
    },
    ShortcutSpec {
        action: ShortcutAction::ToggleShortcutHelp,
        label: "Shortcut help",
        description: "Show keyboard shortcuts",
        default_macos: "Cmd+/",
        default_other: "Ctrl+/",
    },
];

pub fn specs() -> &'static [ShortcutSpec] {
    &SPECS
}

pub fn action_label(action: ShortcutAction) -> &'static str {
    SPECS
        .iter()
        .find(|s| s.action == action)
        .map(|s| s.label)
        .unwrap_or("Unknown")
}

pub fn default_binding(action: ShortcutAction) -> &'static str {
    let spec = SPECS.iter().find(|s| s.action == action).expect("missing shortcut spec");
    if cfg!(target_os = "macos") {
        spec.default_macos
    } else {
        spec.default_other
    }
}

#[allow(dead_code)]
pub fn docs_binding(action: ShortcutAction) -> String {
    let spec = SPECS.iter().find(|s| s.action == action).expect("missing shortcut spec");
    if spec.default_macos == spec.default_other {
        return spec.default_macos.to_string();
    }
    if let Some(rest) = spec.default_macos.strip_prefix("Cmd+") {
        if spec.default_other == format!("Ctrl+{rest}") {
            return format!("Cmd/Ctrl+{rest}");
        }
    }
    format!("{}/{}", spec.default_macos, spec.default_other)
}

#[derive(Debug, Clone)]
struct ParsedChord {
    need_cmd: bool,
    need_ctrl: bool,
    need_cmd_or_ctrl: bool,
    need_shift: bool,
    need_alt: bool,
    key: ParsedKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParsedKey {
    Enter,
    Escape,
    Up,
    Down,
    Comma,
    Slash,
    Char(char),
}

fn parse_chord(raw: &str) -> Option<ParsedChord> {
    let mut need_cmd = false;
    let mut need_ctrl = false;
    let mut need_cmd_or_ctrl = false;
    let mut need_shift = false;
    let mut need_alt = false;
    let mut key: Option<ParsedKey> = None;

    for token in raw.split('+').map(|t| t.trim().to_lowercase()) {
        if token.is_empty() {
            continue;
        }
        match token.as_str() {
            "cmd" | "command" => need_cmd = true,
            "ctrl" | "control" => need_ctrl = true,
            "cmd/ctrl" | "ctrl/cmd" => need_cmd_or_ctrl = true,
            "shift" => need_shift = true,
            "alt" | "option" => need_alt = true,
            "enter" | "return" => key = Some(ParsedKey::Enter),
            "esc" | "escape" => key = Some(ParsedKey::Escape),
            "up" | "arrowup" => key = Some(ParsedKey::Up),
            "down" | "arrowdown" => key = Some(ParsedKey::Down),
            "," | "comma" => key = Some(ParsedKey::Comma),
            "/" | "slash" => key = Some(ParsedKey::Slash),
            other if other.len() == 1 => {
                key = other.chars().next().map(ParsedKey::Char);
            }
            _ => return None,
        }
    }

    key.map(|key| ParsedChord {
        need_cmd,
        need_ctrl,
        need_cmd_or_ctrl,
        need_shift,
        need_alt,
        key,
    })
}

fn parsed_key_matches(
    parsed: ParsedKey,
    key: &keyboard::Key,
    physical_key: &keyboard::key::Physical,
) -> bool {
    match parsed {
        ParsedKey::Enter => matches!(key, keyboard::Key::Named(keyboard::key::Named::Enter)),
        ParsedKey::Escape => matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)),
        ParsedKey::Up => matches!(key, keyboard::Key::Named(keyboard::key::Named::ArrowUp)),
        ParsedKey::Down => matches!(key, keyboard::Key::Named(keyboard::key::Named::ArrowDown)),
        ParsedKey::Comma => {
            matches!(key, keyboard::Key::Character(c) if c == ",")
                || matches!(physical_key, keyboard::key::Physical::Code(keyboard::key::Code::Comma))
        }
        ParsedKey::Slash => {
            matches!(key, keyboard::Key::Character(c) if c == "/" || c == "?")
                || matches!(physical_key, keyboard::key::Physical::Code(keyboard::key::Code::Slash))
        }
        ParsedKey::Char(ch) => {
            let lower = ch.to_ascii_lowercase();
            let from_char = matches!(key, keyboard::Key::Character(c) if c.to_lowercase() == lower.to_string());
            let from_code = match lower {
                'a'..='z' => {
                    let code = match lower {
                        'a' => keyboard::key::Code::KeyA,
                        'b' => keyboard::key::Code::KeyB,
                        'c' => keyboard::key::Code::KeyC,
                        'd' => keyboard::key::Code::KeyD,
                        'e' => keyboard::key::Code::KeyE,
                        'f' => keyboard::key::Code::KeyF,
                        'g' => keyboard::key::Code::KeyG,
                        'h' => keyboard::key::Code::KeyH,
                        'i' => keyboard::key::Code::KeyI,
                        'j' => keyboard::key::Code::KeyJ,
                        'k' => keyboard::key::Code::KeyK,
                        'l' => keyboard::key::Code::KeyL,
                        'm' => keyboard::key::Code::KeyM,
                        'n' => keyboard::key::Code::KeyN,
                        'o' => keyboard::key::Code::KeyO,
                        'p' => keyboard::key::Code::KeyP,
                        'q' => keyboard::key::Code::KeyQ,
                        'r' => keyboard::key::Code::KeyR,
                        's' => keyboard::key::Code::KeyS,
                        't' => keyboard::key::Code::KeyT,
                        'u' => keyboard::key::Code::KeyU,
                        'v' => keyboard::key::Code::KeyV,
                        'w' => keyboard::key::Code::KeyW,
                        'x' => keyboard::key::Code::KeyX,
                        'y' => keyboard::key::Code::KeyY,
                        _ => keyboard::key::Code::KeyZ,
                    };
                    matches!(physical_key, keyboard::key::Physical::Code(c) if *c == code)
                }
                _ => false,
            };
            from_char || from_code
        }
    }
}

fn chord_matches(
    chord: &ParsedChord,
    key: &keyboard::Key,
    physical_key: &keyboard::key::Physical,
    modifiers: keyboard::Modifiers,
) -> bool {
    if chord.need_cmd && !modifiers.command() {
        return false;
    }
    if chord.need_ctrl && !modifiers.control() {
        return false;
    }
    if chord.need_cmd_or_ctrl && !(modifiers.command() || modifiers.control()) {
        return false;
    }
    if chord.need_shift != modifiers.shift() && chord.need_shift {
        return false;
    }
    if chord.need_alt && !modifiers.alt() {
        return false;
    }
    parsed_key_matches(chord.key, key, physical_key)
}

pub fn action_for_event<F: Fn(ShortcutAction) -> String>(
    binding_for: F,
    key: &keyboard::Key,
    physical_key: &keyboard::key::Physical,
    modifiers: keyboard::Modifiers,
) -> Option<ShortcutAction> {
    for spec in specs() {
        let binding = binding_for(spec.action);
        if let Some(chord) = parse_chord(&binding) {
            if chord_matches(&chord, key, physical_key, modifiers) {
                return Some(spec.action);
            }
        }
    }
    None
}

pub fn append_debug_key_log(line: &str) {
    let Some(path) = key_log_path() else { return };
    if let Some(parent) = path.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return;
        }
    }

    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(file, "{line}");
}

pub fn key_log_path() -> Option<std::path::PathBuf> {
    let mut path = dirs::config_dir()?;
    path.push("stoa");
    path.push("key-events.log");
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_matches_cmd_k() {
        let key = keyboard::Key::Character("k".into());
        let physical = keyboard::key::Physical::Code(keyboard::key::Code::KeyK);
        let modifiers = keyboard::Modifiers::from_bits_truncate(keyboard::Modifiers::COMMAND.bits());
        let parsed = parse_chord("Cmd+K").expect("parse cmd+k");
        assert!(chord_matches(&parsed, &key, &physical, modifiers));
    }
}
