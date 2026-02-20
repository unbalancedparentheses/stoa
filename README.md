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

### Analytics & Cost
- **Cost tracking** — Estimated token counts and USD cost per message, conversation, and session
- **Actual token counts** — Parsed from OpenAI and Anthropic API responses when available
- **Response ratings** — Thumbs up/down on any response, tracked per model
- **Analytics dashboard** — Per-model stats: response count, tokens, cost, latency, approval rate

### Organization
- **Tags & Pins** — Tag conversations, pin important ones to the top
- **Quick Switcher (Cmd+K)** — Fuzzy search across all conversations
- **Command Palette (Cmd+P)** — Every action in one searchable list
- **Full-text search** — Sidebar search filters by title and message content
- **Markdown export (Cmd+E)** — Copy any conversation as formatted Markdown
- **Auto-titling** — AI generates meaningful titles after your first exchange

### File Attachment
- **Attach text files** — Native file dialog supports .txt, .md, .rs, .py, .js, .ts, .go, .c, .json, .toml, .yaml, and more
- **Context injection** — File contents are prepended to your message in a code block

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Send to primary model |
| `Cmd+Shift+Enter` | Send to all models |
| `Cmd+K` | Quick Switcher |
| `Cmd+P` | Command Palette |
| `Cmd+E` | Export as Markdown |
| `Cmd+N` | New conversation |
| `Cmd+,` | Settings |
| `Esc` | Dismiss overlays |

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
2. Check multiple models with the checkboxes
3. Type your prompt, click **Run N** (or `Cmd+Shift+Enter` for all)
4. All responses stream in simultaneously

### Comparison Mode
1. Send to multiple models
2. Click **Compare** in the chat header
3. Responses appear side-by-side in columns
4. Click **Show Diff** to see word-level differences

### Research Workflow
- Set a per-conversation system prompt via the **Sys** button in the header
- **Fork** at any message to branch the conversation
- **Rate** responses with thumbs up/down to track model quality
- View aggregated stats in the **Analytics** tab
- **Tag** conversations for organization, **pin** important ones

## Configuration

Settings are at `~/.config/stoa/config.json`. Conversations at `~/.config/stoa/chat.db`.

| Setting | Description |
|---------|-------------|
| API Keys | OpenAI and Anthropic keys (stored locally) |
| Ollama URL | Defaults to `http://localhost:11434/v1/chat/completions` |
| System Prompt | Global default (overridden by per-conversation prompts) |
| Temperature | Generation temperature (default 0.7) |
| Max Tokens | Max output tokens (default 4096) |

## Architecture

```
src/
  main.rs              Entry point
  lib.rs               Library root (for tests)
  app.rs               State, Message enum, update loop, view
  model.rs             Conversation, ChatMessage, Provider
  config.rs            AppConfig, model routing, Ollama config
  db.rs                SQLite schema, migrations, CRUD, search
  cost.rs              Pricing table, token estimation
  diff.rs              Word-level LCS diff
  export.rs            Markdown export
  api/
    mod.rs             LlmEvent, stream dispatch
    openai.rs          OpenAI + Ollama streaming (shared SSE)
    anthropic.rs       Anthropic streaming
    ollama.rs          Model discovery via /api/tags
  ui/
    chat_view.rs       Messages, comparison mode, diff panel
    input_bar.rs       Input, model picker, file attach
    sidebar.rs         Nav, search, history, tags, pins
    right_panel.rs     Streams, system info, cost, shortcuts
    analytics.rs       Per-model stats dashboard
    settings.rs        Provider config, Ollama tab
    quick_switcher.rs  Cmd+K overlay
    command_palette.rs Cmd+P overlay
    markdown.rs        Markdown renderer (pulldown-cmark)
    bottom_bar.rs      Status bar
```

## Comparison

| Feature | **Stoa** | **BoltAI** | **msty** | **Cherry Studio** | **Jan** | **AnythingLLM** | **MindMac** | **TypingMind** | **Open WebUI** | **LibreChat** | **Chatbox AI** |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Platform | Rust/GPU native | macOS | Mac/Win/Linux | Mac/Win/Linux | Mac/Win/Linux | Mac/Win/Linux | macOS | Web (PWA) | Web (self-host) | Web (self-host) | All platforms |
| Open source | Yes | No | No | Yes (AGPL) | Yes (AGPL) | Yes (MIT) | No | No | Custom | Yes (MIT) | GPLv3/closed |
| Price | Free | $37-57 | Free/$149yr | Free | Free | Free | $29-69 | $39-79 | Free | Free | Free/paid |
| Multi-model parallel | **Yes** | No | Yes | Yes | No | No | No | Yes | Yes | No | No |
| Side-by-side comparison | **Yes** | No | Yes | Yes | No | No | No | Yes | Yes | No | No |
| Response diffing | **Yes** | No | No | No | No | No | No | No | No | No | No |
| Local LLMs (Ollama) | **Yes** | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Cloud providers | OpenAI, Anthropic, OpenRouter | 6+ | 10+ | 50+ | 5+ | 3+ | 7+ | Many | OpenAI-compat | 8+ | Many |
| RAG / Knowledge base | No | No | Yes | Yes | No | **Yes** | No | Yes | **Yes** | No | Yes |
| AI Agents / MCP | No | Yes | Yes | Yes | Yes | **Yes** | No | Yes | **Yes** | **Yes** | Yes |
| Cost tracking | **Yes** | No | Yes | No | No | Partial | **Yes** | Partial | **Yes** | Yes | No |
| Command palette | **Yes** | No | No | No | No | No | No | No | No | No | No |
| Quick switcher | **Yes** | No | No | No | No | No | No | No | No | No | No |
| Conversation forking | **Yes** | Yes | Yes | No | No | No | No | Yes | Partial | **Yes** | No |
| Tags / Organization | **Yes** | Yes | ? | Yes | ? | Yes | Yes | Yes | **Yes** | Yes | ? |
| File attachment | **Yes** (text) | Yes (all) | Yes | Yes | Yes | **Yes** | Yes | Yes | **Yes** | **Yes** | Yes |
| Image/Vision | No | Yes | ? | Yes | Yes | Yes | Yes | ? | **Yes** | **Yes** | Yes |
| Export | MD clipboard | PDF/MD/HTML | Yes | MD/Word | Partial | Yes | PDF/MD | ? | JSON | MD/JSON/PNG | HTML/MD |
| Search | **Yes** | Yes | ? | Yes | ? | Yes | Yes | Yes | **Yes** | **Yes** | Yes |
| Plugins/Extensions | No | Yes | Yes | Yes | Yes | **Yes** | No | Yes | **Yes** | **Yes** | Yes |
| Analytics/Stats | **Yes** | No | Yes | No | No | Partial | Yes | No | **Yes** | Partial | No |
| Response ratings | **Yes** | No | No | No | No | No | No | No | **Yes** | No | No |
| Streaming markdown | **Yes** | Yes | Yes | Yes | Yes | Yes | Yes | Yes | **Yes** | Yes | Yes |
| Web search | No | Yes | Yes | ? | ? | ? | Yes | Yes | **Yes** | Yes | ? |
| Voice I/O | No | Yes | ? | ? | ? | ? | ? | ? | **Yes** | Yes | ? |

### Stoa's Unique Advantages

- **Response diffing** — Word-level LCS diff with agreement percentage. No competitor has this.
- **Command palette + Quick switcher** — Obsidian-style keyboard-driven navigation.
- **Native Rust/GPU** — Not Electron, not Tauri, not web. Pure Rust + wgpu rendering.
- **Per-conversation system prompts** — Different personas per research thread.
- **Model analytics with approval rates** — Track which models perform best over time.

### Roadmap

- [ ] OpenRouter provider (access to 200+ models via single API)
- [ ] Image/Vision support for multimodal models
- [ ] Web search integration
- [ ] MCP tool support
- [ ] RAG / Knowledge base
- [ ] Richer export (PDF, HTML, JSON)
- [ ] PDF/DOCX file attachment
- [ ] Import from ChatGPT/Claude exports
- [ ] Conversation folders (nested)
- [ ] Plugin system
- [ ] Voice input/output
- [ ] Cross-platform testing (Windows, Linux)

## License

MIT
