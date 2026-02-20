# rust-chat

A native, GPU-accelerated AI research platform built with Rust and [iced](https://github.com/iced-rs/iced). Send the same prompt to multiple models simultaneously, compare responses side-by-side, and diff where they agree or disagree. Zero Electron, zero JavaScript.

## Features

- **Parallel multi-model streaming** - Send a prompt to 2-7+ models at once and watch all responses stream in simultaneously
- **Side-by-side comparison** - Toggle comparison mode to see model responses in columns instead of stacked
- **Response diffing** - Word-level diff highlights where models agree vs. disagree, with agreement percentage
- **Ollama local LLMs** - Auto-discovers locally installed Ollama models, no API key needed
- **Cloud providers** - OpenAI (GPT-5, GPT-4.1, o3, o4-mini) and Anthropic (Opus, Sonnet, Haiku)
- **Cost tracking** - Estimated token counts and USD cost per message, per conversation, and per session
- **Quick Switcher (Ctrl+K)** - Fuzzy search across all conversations instantly
- **Command Palette (Ctrl+P)** - Access any action from a searchable command list
- **Full-text search** - Search sidebar filters conversations by title and message content
- **Tags & Pins** - Tag conversations for organization, pin important ones to the top
- **Markdown export (Ctrl+E)** - Copy any conversation as formatted Markdown to clipboard
- **Per-stream controls** - Stop individual model streams without affecting others
- **Conversation switching during streaming** - Browse other conversations while models are still responding
- **SQLite persistence** - All conversations stored locally in SQLite with WAL mode
- **Native performance** - Pure Rust/GPU rendering via iced, minimal CPU/memory footprint

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Send message to primary model |
| `Ctrl+Shift+Enter` | Send to all available models |
| `Ctrl+N` | New conversation |
| `Ctrl+K` | Quick Switcher |
| `Ctrl+P` | Command Palette |
| `Ctrl+E` | Export conversation as Markdown |
| `Esc` | Dismiss overlays / cancel |

## Installation

### Prerequisites

- Rust (2024 edition) - [Install Rust](https://rustup.rs/)
- GPU-capable system (iced uses wgpu for rendering)
- Ollama (optional) - [Install Ollama](https://ollama.com/) for local model support

### Build & Run

```sh
git clone https://github.com/unbalancedparen/rust-chat
cd rust-chat
cargo run
```

### Ollama Setup (Optional)

1. Install Ollama from [ollama.com](https://ollama.com/)
2. Pull a model: `ollama pull llama3.2`
3. Start Ollama: `ollama serve`
4. rust-chat will auto-discover local models on startup

## Usage

### Single Model Chat
Type a message and press Enter. The message goes to whatever model is selected in the model picker chip (bottom-left of the input bar).

### Parallel Multi-Model
1. Click the model chip to open the picker
2. Check multiple models using the checkbox icons
3. Type your prompt and click "Run N" (or use `Ctrl+Shift+Enter` for all models)
4. Watch all responses stream in simultaneously

### Comparison Mode
1. Send a prompt to multiple models
2. Click "Compare" in the chat header
3. Responses from the same prompt appear side-by-side in columns
4. Click "Show Diff" to see a word-level diff with agreement percentage

### Organization
- **Pin** important conversations using the pin icon on the active conversation
- **Tag** conversations by clicking "+ tag" under the active conversation in the sidebar
- **Search** conversations using the search box at the top of the sidebar
- **Quick Switch** between conversations with `Ctrl+K`

## Configuration

On first run, go to Settings to configure:

1. **API Keys** - Enter your OpenAI and/or Anthropic API keys
2. **Ollama** - Verify the Ollama URL (defaults to `http://localhost:11434/v1/chat/completions`)
3. **Model** - Select your preferred default model
4. **System Prompt** - Set a custom system prompt (applies to all models)
5. **Temperature / Max Tokens** - Tune generation parameters

Settings are stored in `~/.config/rust-chat/config.json`. Conversations are stored in `~/.config/rust-chat/chat.db`.

## Architecture

```
src/
  main.rs              Entry point, iced application setup
  app.rs               App state, Message enum, update loop, view composition
  model.rs             Core types: Conversation, ChatMessage, Provider, ProviderConfig
  config.rs            AppConfig with persistence, model routing, Ollama discovery
  db.rs                SQLite: schema, migrations, CRUD, full-text search
  cost.rs              Token estimation, pricing table, cost calculations
  diff.rs              Word-level LCS diff algorithm
  export.rs            Markdown export
  theme.rs             Color constants for the UI theme
  api/
    mod.rs             LlmEvent enum, stream dispatch
    openai.rs          OpenAI + Ollama SSE streaming (shared protocol)
    anthropic.rs       Anthropic SSE streaming
    ollama.rs          Ollama model discovery via /api/tags
  ui/
    chat_view.rs       Message rendering, comparison mode, diff panel
    input_bar.rs       Text input, model picker, multi-select, Run N/Stop All
    sidebar.rs         Navigation, search, conversation list, tags, pins, analyze
    right_panel.rs     Active streams, system info, cost, shortcuts
    settings.rs        Provider config, Ollama tab, presets
    quick_switcher.rs  Ctrl+K overlay for fuzzy conversation search
    command_palette.rs Ctrl+P overlay for command search
    markdown.rs        Markdown-to-iced renderer (pulldown-cmark)
    bottom_bar.rs      Status bar with session cost and shortcut hints
```

## Comparison with Alternatives

| Feature | rust-chat | BoltAI | msty | Cherry Studio | Jan |
|---------|-----------|--------|------|---------------|-----|
| Native (no Electron) | Rust/GPU | macOS native | Electron | Electron | Electron |
| Parallel multi-model | Yes | No | Split Chat | Yes | No |
| Side-by-side comparison | Yes | No | Yes | No | No |
| Response diffing | Yes | No | No | No | No |
| Ollama local LLMs | Yes | Yes | Yes | Yes | Yes |
| Cost tracking | Yes | No | No | No | No |
| Quick Switcher / Cmd Palette | Yes | No | No | No | No |
| Open source | Yes | No | No | Yes | Yes |
| Cross-platform | Yes | macOS only | Yes | Yes | Yes |
| Price | Free | $29.99 | Freemium | Free | Free |

## License

MIT
