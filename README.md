# Stoa

A native AI research platform built with Rust. Send the same prompt to multiple models simultaneously, compare responses side-by-side, diff where they agree or disagree, and track costs — all from a single keyboard-driven interface. Zero Electron, zero JavaScript.

## Why Stoa?

Most AI chat apps let you talk to one model at a time. Stoa lets you **interrogate multiple models in parallel** and see where they converge or diverge. Built for researchers, engineers, and anyone who wants to think harder with AI.

- **Native performance** — Pure Rust + GPU rendering via [iced](https://github.com/iced-rs/iced). Starts instantly, uses minimal resources.
- **Local-first** — All data stays on your machine. SQLite database, no cloud sync, no telemetry.
- **Keyboard-driven** — Quick Switcher, Command Palette, and shortcuts for everything.

## Features

### Multi-Model Research
- **Parallel streaming** — Send to 2-7+ models at once, watch all responses stream simultaneously
- **Side-by-side comparison** — Toggle comparison mode to view responses in columns
- **Response diffing** — Word-level diff highlights agreements vs. differences with percentage
- **Per-conversation system prompts** — Different personas for different research threads
- **Conversation forking** — Branch at any message to explore alternative directions

### Model Support
- **OpenAI** — GPT-5, GPT-4.1, o3, o4-mini
- **Anthropic** — Claude Opus, Sonnet, Haiku
- **Ollama** — Auto-discovers locally installed models. No API key needed.
- **OpenRouter** — 200+ models via a single API key (Gemini, Llama, Mistral, DeepSeek, Qwen, and more)

### Analytics & Cost
- **Cost tracking** — Estimated token counts and USD cost per message, conversation, and session
- **Response ratings** — Thumbs up/down on any response, tracked per model
- **Analytics dashboard** — Per-model stats: response count, tokens, cost, latency, approval rate

### Organization
- **Tags & Pins** — Tag conversations, pin important ones to the top
- **Quick Switcher (Cmd+K)** — Fuzzy search across all conversations
- **Command Palette (Cmd+P)** — Every action in one searchable list
- **Full-text search** — FTS5-powered search across titles and message content
- **Markdown / HTML / JSON export** — Copy any conversation to clipboard in multiple formats
- **Auto-titling** — AI generates meaningful titles after your first exchange
- **ChatGPT import** — Import your full ChatGPT history from `conversations.json`

### Multimodal & Web
- **Image/Vision** — Attach images and send them to vision-capable models
- **Text file attachment** — Native file dialog for code, text, and config files
- **Web search** — Inject DuckDuckGo search context into your prompt

### Security
- **OS keychain** — API keys stored in macOS Keychain / Linux Secret Service / Windows Credential Manager
- **No telemetry** — Zero data collection, all conversations stored locally in SQLite

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

All keybindings are configurable in **Settings**.

## Installation

### Prerequisites

- **Rust** (2024 edition) — [rustup.rs](https://rustup.rs/)
- **GPU-capable system** — iced uses wgpu for rendering
- **Ollama** (optional) — [ollama.com](https://ollama.com/) for local models

### Build & Run

```sh
git clone https://github.com/unbalancedparentheses/stoa
cd stoa
cargo run --release
```

### Ollama Setup (Optional)

```sh
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Pull a model
ollama pull llama3.2

# Stoa auto-discovers local models on startup
```

## Usage

### Single Model
Type a message, press Enter. Goes to the model shown in the picker chip.

### Parallel Multi-Model
1. Click the model chip to open the picker
2. Check multiple models
3. Type your prompt, click **Run N** (or `Cmd+Shift+Enter` for all)
4. All responses stream in simultaneously

### Comparison Mode
1. Send to multiple models
2. Click **Compare** in the chat header
3. Responses appear side-by-side in columns
4. Click **Show Diff** to see word-level differences

### Research Workflow
- Set a per-conversation system prompt via the **Sys** button
- **Fork** at any message to branch the conversation
- **Rate** responses with thumbs up/down to track model quality
- View aggregated stats in the **Analytics** tab
- **Tag** conversations for organization, **pin** important ones

## Configuration

Config path: `~/.config/stoa/config.json`
Data path: `~/.config/stoa/chat.db`

| Setting | Description |
|---------|-------------|
| API Keys | Stored in OS keychain (falls back to config file) |
| Ollama URL | Defaults to `http://localhost:11434/v1/chat/completions` |
| System Prompt | Global default (overridden by per-conversation prompts) |
| Temperature | Generation temperature (default 0.7) |
| Max Tokens | Max output tokens (default 4096) |
| Keybindings | All shortcuts are remappable |

Enable `RUST_LOG=info` (or `debug`) to see structured log output.

## Comparison

| Feature | **Stoa** | BoltAI | msty | Cherry Studio | Jan | AnythingLLM | MindMac | TypingMind | Open WebUI | LibreChat | Chatbox AI |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Platform | Rust/GPU native | macOS | All | All | All | All | macOS | Web | Web | Web | All |
| Open source | Yes (MIT) | No | No | AGPL | AGPL | MIT | No | No | Custom | MIT | GPLv3 |
| Multi-model parallel | **Yes** | No | Yes | Yes | No | No | No | Yes | Yes | No | No |
| Side-by-side comparison | **Yes** | No | Yes | Yes | No | No | No | Yes | Yes | No | No |
| Response diffing | **Yes** | No | No | No | No | No | No | No | No | No | No |
| Local LLMs (Ollama) | **Yes** | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Cloud providers | 4+ (200+ via OR) | 6+ | 10+ | 50+ | 5+ | 3+ | 7+ | Many | OAI-compat | 8+ | Many |
| Cost tracking | **Yes** | No | Yes | No | No | Partial | Yes | Partial | Yes | Yes | No |
| Command palette | **Yes** | No | No | No | No | No | No | No | No | No | No |
| Quick switcher | **Yes** | No | No | No | No | No | No | No | No | No | No |
| Conversation forking | **Yes** | Yes | Yes | No | No | No | No | Yes | Partial | Yes | No |
| Image/Vision | **Yes** | Yes | ? | Yes | Yes | Yes | Yes | ? | Yes | Yes | Yes |
| Web search | **Yes** | Yes | Yes | ? | ? | ? | Yes | Yes | Yes | Yes | ? |
| ChatGPT import | **Yes** | ? | ? | ? | ? | ? | ? | ? | ? | ? | ? |
| OS keychain | **Yes** | ? | ? | No | No | No | ? | N/A | N/A | N/A | ? |
| FTS5 search | **Yes** | ? | ? | ? | ? | Yes | ? | ? | Yes | Yes | ? |
| Analytics/Stats | **Yes** | No | Yes | No | No | Partial | Yes | No | Yes | Partial | No |
| Response ratings | **Yes** | No | No | No | No | No | No | No | Yes | No | No |
| RAG / Knowledge base | No | No | Yes | Yes | No | **Yes** | No | Yes | **Yes** | No | Yes |
| AI Agents / MCP | No | Yes | Yes | Yes | Yes | **Yes** | No | Yes | **Yes** | **Yes** | Yes |
| Plugins | No | Yes | Yes | Yes | Yes | **Yes** | No | Yes | **Yes** | **Yes** | Yes |

### Stoa's Unique Advantages

- **Response diffing** — Word-level LCS diff with agreement percentage. No competitor has this.
- **Command palette + Quick switcher** — Obsidian-style keyboard-driven navigation.
- **Native Rust/GPU** — Not Electron, not Tauri, not web. Pure Rust + wgpu rendering.
- **OS keychain** — API keys in your system's secure credential store, not plaintext config files.

## Architecture

```
src/
  main.rs              Entry point
  lib.rs               Library root
  app.rs               State, Message enum, update loop, view
  model.rs             Conversation, ChatMessage, Provider
  config.rs            AppConfig, model routing, keychain integration
  db.rs                SQLite + FTS5 persistence, migrations, search
  cost.rs              Pricing table, token estimation
  diff.rs              Word-level LCS diff
  export.rs            Markdown / HTML / JSON export
  import.rs            ChatGPT import parser
  shortcuts.rs         Shortcut specs + key matching
  commands.rs          Command palette entries
  web_search.rs        DuckDuckGo search integration
  theme.rs             Color palette
  handlers/
    streaming.rs       Stream lifecycle, auto-titling
    send.rs            Message sending, retry, review, analyze
  api/
    mod.rs             LlmEvent, stream dispatch, shared HTTP client
    openai.rs          OpenAI + Ollama + OpenRouter streaming
    anthropic.rs       Anthropic streaming
    ollama.rs          Model discovery via /api/tags
  ui/
    chat_view.rs       Messages, comparison mode, diff panel
    input_bar.rs       Input, model picker, file/image attach
    sidebar.rs         Nav, search, history, tags, pins
    right_panel.rs     Streams, system info, cost, shortcuts
    analytics.rs       Per-model stats dashboard
    settings.rs        Provider config, keybindings
    quick_switcher.rs  Cmd+K overlay
    command_palette.rs Cmd+P overlay
    shortcut_help.rs   Shortcut cheat sheet
    markdown.rs        Markdown renderer (pulldown-cmark)
    diagnostics.rs     Debug diagnostics view
    bottom_bar.rs      Status bar
```

## Roadmap

- [ ] MCP tool support
- [ ] RAG / Knowledge base
- [ ] PDF/DOCX file attachment
- [ ] Conversation folders
- [ ] Plugin system
- [ ] Voice input/output
- [ ] Cross-platform testing (Windows, Linux)

## License

MIT
