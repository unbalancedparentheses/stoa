# rust-chat

A native, GPU-accelerated chat client built with Rust and [iced](https://github.com/iced-rs/iced). Zero Electron, zero JavaScript.

## Goals

- **Native performance** - Pure Rust/GPU rendering via iced, minimal CPU/memory usage
- **Multi-provider** - Connect to OpenAI (GPT-5, GPT-4.1, o3, o4-mini) and Anthropic (Opus, Sonnet, Haiku) from a single app
- **Streaming responses** - Real-time token streaming from all providers
- **Conversation management** - Chat history, multiple conversations, search
- **Configurable** - Per-provider API keys, model selection, custom endpoints (local Llama support)

## Architecture

```
src/
  main.rs          - Entry point, iced application setup
  app.rs           - Top-level app state and message routing
  ui/              - UI views (chat, sidebar, settings, input)
  api/             - LLM provider clients (OpenAI, Anthropic)
  model.rs         - Core data types (Conversation, Message, Provider)
  config.rs        - Settings persistence (API keys, preferences)
```

## Building

```sh
cargo run
```

## Requirements

- Rust 2024 edition
- GPU-capable system (iced uses wgpu for rendering)

## License

TBD
