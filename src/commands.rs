use crate::app::Message;
use crate::config::Keybindings;
use crate::shortcuts::{self, ShortcutAction};

#[derive(Debug, Clone)]
pub struct CommandEntry {
    pub label: &'static str,
    pub description: &'static str,
    pub shortcut: String,
    pub message: Message,
}

pub fn all_commands(bindings: &Keybindings) -> Vec<CommandEntry> {
    let shortcut = |action: ShortcutAction| bindings.get(action).to_string();

    vec![
        CommandEntry {
            label: "New Chat",
            description: "Create a new conversation",
            shortcut: shortcut(ShortcutAction::NewConversation),
            message: Message::NewConversation,
        },
        CommandEntry {
            label: "Settings",
            description: "Open settings",
            shortcut: shortcut(ShortcutAction::ShowSettings),
            message: Message::ShowSettings,
        },
        CommandEntry {
            label: "Home",
            description: "Go to chat view",
            shortcut: String::new(),
            message: Message::ShowChat,
        },
        CommandEntry {
            label: "Send to All",
            description: "Send to all available models",
            shortcut: shortcut(ShortcutAction::SendToAll),
            message: Message::SendToAll,
        },
        CommandEntry {
            label: "Toggle Comparison",
            description: "Switch comparison mode on/off",
            shortcut: String::new(),
            message: Message::ToggleComparisonMode,
        },
        CommandEntry {
            label: "Export Markdown",
            description: "Copy conversation as Markdown",
            shortcut: shortcut(ShortcutAction::ExportMarkdown),
            message: Message::ExportMarkdown,
        },
        CommandEntry {
            label: "Export HTML",
            description: "Copy conversation as styled HTML",
            shortcut: String::new(),
            message: Message::ExportHtml,
        },
        CommandEntry {
            label: "Export JSON",
            description: "Copy conversation as JSON",
            shortcut: String::new(),
            message: Message::ExportJson,
        },
        CommandEntry {
            label: "Import ChatGPT",
            description: "Import from ChatGPT export file",
            shortcut: String::new(),
            message: Message::ImportChatGpt,
        },
        CommandEntry {
            label: "Web Search",
            description: "Search web for current input",
            shortcut: String::new(),
            message: Message::WebSearch,
        },
        CommandEntry {
            label: "Refresh Ollama",
            description: "Re-scan local Ollama models",
            shortcut: String::new(),
            message: Message::RefreshOllamaModels,
        },
        CommandEntry {
            label: "Analytics",
            description: "View model stats and ratings",
            shortcut: String::new(),
            message: Message::ShowAnalytics,
        },
        CommandEntry {
            label: "Diagnostics",
            description: "Inspect keyboard/focus diagnostics",
            shortcut: String::new(),
            message: Message::ShowDiagnostics,
        },
        CommandEntry {
            label: "Quick Switcher",
            description: "Search conversations",
            shortcut: shortcut(ShortcutAction::QuickSwitcher),
            message: Message::ToggleQuickSwitcher,
        },
        CommandEntry {
            label: "Keyboard Shortcuts",
            description: "Show shortcut cheat sheet",
            shortcut: shortcut(ShortcutAction::ToggleShortcutHelp),
            message: Message::ToggleShortcutHelp,
        },
    ]
}

pub fn filtered_commands(query: &str, bindings: &Keybindings) -> Vec<CommandEntry> {
    let lowered = query.to_lowercase();
    let mut out = Vec::new();
    for cmd in all_commands(bindings) {
        if lowered.is_empty()
            || cmd.label.to_lowercase().contains(&lowered)
            || cmd.description.to_lowercase().contains(&lowered)
            || cmd.shortcut.to_lowercase().contains(&lowered)
        {
            out.push(cmd);
        }
    }
    out
}

pub fn shortcut_rows(bindings: &Keybindings) -> Vec<(String, &'static str)> {
    let mut rows = vec![("Enter".to_string(), "Send")];
    for spec in shortcuts::specs() {
        rows.push((bindings.get(spec.action).to_string(), spec.label));
    }
    rows
}
