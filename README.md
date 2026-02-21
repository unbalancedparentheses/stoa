# Stoa

Stoa is a native Rust desktop app for comparing responses across multiple LLMs in parallel.

It is designed for research and evaluation workflows where you want to ask one prompt once, inspect multiple model outputs, and quickly compare differences.

## Who It Is For

- People comparing model quality, style, and factuality across providers
- Researchers running repeatable multi-model prompt experiments
- Engineers who want local-first chat history and fast keyboard-driven navigation

## Who It Is Not For (Yet)

- Agent-heavy workflows (MCP/tool orchestration)
- Built-in RAG / knowledge-base systems
- Team collaboration / cloud sync products

## Available Now

### Core Workflow
- Multi-model parallel streaming in a single conversation
- Side-by-side comparison mode
- Word-level response diffing
- Conversation forking
- Per-conversation system prompts

### Providers
- OpenAI-compatible streaming
- Anthropic streaming
- Ollama model discovery + local model execution
- OpenRouter model routing

### App Features
- Command Palette and Quick Switcher
- Conversation tags and pins
- Sidebar full-text search
- Markdown / HTML / JSON export to clipboard
- Basic web search context injection (DuckDuckGo Instant Answer API)
- Text file attachment and image attachment (sent as multimodal input when provider supports it)
- Ratings + analytics dashboard

## Experimental / Partial

- ChatGPT import parsing exists, but the current UI flow does not persist imported conversations into the database yet.

## Planned

- Better import/export workflows
- Richer multimodal UX polish
- Additional provider/tooling integrations

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Send to primary model |
| `Cmd/Ctrl+Shift+Enter` | Send to all models |
| `Cmd/Ctrl+K` | Quick Switcher |
| `Cmd/Ctrl+P` | Command Palette |
| `Cmd/Ctrl+E` | Export as Markdown |
| `Cmd/Ctrl+N` | New conversation |
| `Cmd/Ctrl+,` | Settings |
| `Esc` | Dismiss overlays |

In-app keybindings are configurable in **Settings → Keybindings**.

## Installation

### Prerequisites

- Rust (stable)
- GPU-capable system (iced + wgpu)
- Ollama (optional, for local models)

### Build and Run

```sh
git clone https://github.com/unbalancedparentheses/stoa
cd stoa
cargo run --release
```

On macOS, if keyboard focus does not move to the app after `cargo run`, use:

```sh
open target/debug/stoa
```

## Configuration

Config path: `~/.config/stoa/config.json`  
Data path: `~/.config/stoa/chat.db`

Includes:
- Provider credentials and endpoints
- Model defaults
- Generation settings
- Keybindings
- Optional local key-event debug logging

## Known Limitations

- No cloud sync / multi-device state
- No built-in RAG pipeline
- No agent runtime/tool graph
- Some provider capabilities vary by API/model (especially multimodal behavior)
- ChatGPT import is not fully wired end-to-end yet

## Privacy

- No telemetry pipeline in this app
- Optional key-event debug logging is local-only (`~/.config/stoa/key-events.log`)

## Architecture

```text
src/
  app.rs               App state, update loop, subscriptions
  shortcuts.rs         Shared shortcut specs + matching logic
  commands.rs          Shared command-palette data
  model.rs             Conversations, messages, providers
  db.rs                SQLite persistence
  api/                 Provider streaming implementations
  ui/                  Views, overlays, settings, panels
```

## License

MIT
