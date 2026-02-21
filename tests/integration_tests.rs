use stoa::model::*;
use stoa::config::AppConfig;
use stoa::cost;
use stoa::diff;
use stoa::export;

// ── Model Tests ──────────────────────────────────────────────

#[test]
fn conversation_new_has_defaults() {
    let conv = Conversation::new();
    assert_eq!(conv.title, "New Chat");
    assert!(conv.messages.is_empty());
    assert!(conv.tags.is_empty());
    assert!(!conv.pinned);
    assert!(conv.system_prompt.is_empty());
    assert!(conv.forked_from.is_none());
    assert!(!conv.id.is_empty());
}

#[test]
fn conversation_ids_are_unique() {
    let a = Conversation::new();
    let b = Conversation::new();
    assert_ne!(a.id, b.id);
}

#[test]
fn add_user_message_sets_title() {
    let mut conv = Conversation::new();
    conv.add_user_message("Hello world, this is a test message", None);
    assert_eq!(conv.messages.len(), 1);
    assert_eq!(conv.messages[0].role, Role::User);
    assert_eq!(conv.messages[0].content, "Hello world, this is a test message");
    // Title auto-set from first message (truncated to 30 chars)
    assert_eq!(conv.title, "Hello world, this is a test me");
}

#[test]
fn add_user_message_does_not_overwrite_custom_title() {
    let mut conv = Conversation::new();
    conv.title = "My Research".to_string();
    conv.add_user_message("test", None);
    assert_eq!(conv.title, "My Research");
}

#[test]
fn push_streaming_assistant_returns_correct_index() {
    let mut conv = Conversation::new();
    conv.add_user_message("hi", None);
    let idx = conv.push_streaming_assistant(Some("gpt-4.1".to_string()));
    assert_eq!(idx, 1);
    assert!(conv.messages[1].streaming);
    assert_eq!(conv.messages[1].role, Role::Assistant);
    assert!(conv.messages[1].content.is_empty());
}

#[test]
fn update_streaming_at_updates_correct_message() {
    let mut conv = Conversation::new();
    conv.add_user_message("hi", None);
    let idx = conv.push_streaming_assistant(Some("gpt-4.1".to_string()));
    conv.update_streaming_at(idx, "Hello!");
    assert_eq!(conv.messages[idx].content, "Hello!");
    assert!(conv.messages[idx].streaming);
}

#[test]
fn update_streaming_at_ignores_non_streaming() {
    let mut conv = Conversation::new();
    conv.add_user_message("hi", None);
    let idx = conv.push_streaming_assistant(None);
    conv.finalize_at(idx, "done");
    conv.update_streaming_at(idx, "should not update");
    assert_eq!(conv.messages[idx].content, "done");
}

#[test]
fn finalize_at_sets_streaming_false() {
    let mut conv = Conversation::new();
    conv.add_user_message("hi", None);
    let idx = conv.push_streaming_assistant(Some("claude-sonnet-4-20250514".to_string()));
    conv.finalize_at(idx, "Final response");
    assert_eq!(conv.messages[idx].content, "Final response");
    assert!(!conv.messages[idx].streaming);
}

#[test]
fn finalize_at_out_of_bounds_is_safe() {
    let mut conv = Conversation::new();
    conv.finalize_at(999, "nope"); // should not panic
    assert!(conv.messages.is_empty());
}

#[test]
fn fork_creates_correct_copy() {
    let mut conv = Conversation::new();
    conv.system_prompt = "Be helpful".to_string();
    conv.tags = vec!["research".to_string()];
    conv.add_user_message("question 1", Some("gpt-4.1".to_string()));
    conv.messages.push(ChatMessage {
        role: Role::Assistant,
        content: "answer 1".to_string(),
        streaming: false,
        model: Some("gpt-4.1".to_string()),
        token_count: None,
        rating: 1,
        latency_ms: Some(500),
        images: Vec::new(),
    });
    conv.add_user_message("question 2", Some("gpt-4.1".to_string()));
    conv.messages.push(ChatMessage {
        role: Role::Assistant,
        content: "answer 2".to_string(),
        streaming: false,
        model: Some("gpt-4.1".to_string()),
        token_count: None,
        rating: 0,
        latency_ms: None,
        images: Vec::new(),
    });

    // Fork at message index 1 (after first Q&A)
    let forked = conv.fork(1);
    assert_ne!(forked.id, conv.id);
    assert!(forked.title.starts_with("Fork of"));
    assert_eq!(forked.messages.len(), 2); // user + assistant
    assert_eq!(forked.messages[0].content, "question 1");
    assert_eq!(forked.messages[1].content, "answer 1");
    assert_eq!(forked.system_prompt, "Be helpful");
    assert_eq!(forked.tags, vec!["research"]);
    assert_eq!(forked.forked_from, Some(conv.id.clone()));
    assert!(!forked.pinned);
}

#[test]
fn fork_excludes_streaming_messages() {
    let mut conv = Conversation::new();
    conv.add_user_message("test", None);
    conv.push_streaming_assistant(Some("gpt-4.1".to_string()));
    // streaming message at index 1
    let forked = conv.fork(1);
    // streaming message should be excluded
    assert_eq!(forked.messages.len(), 1); // only the user message
}

#[test]
fn message_rating_defaults_to_zero() {
    let mut conv = Conversation::new();
    conv.add_user_message("test", None);
    assert_eq!(conv.messages[0].rating, 0);
}

// ── Provider Config Tests ────────────────────────────────────

#[test]
fn default_provider_configs() {
    let openai = ProviderConfig::default_openai();
    assert_eq!(openai.provider, Provider::OpenAI);
    assert!(openai.api_url.contains("openai.com"));
    assert_eq!(openai.model, "gpt-4.1");

    let anthropic = ProviderConfig::default_anthropic();
    assert_eq!(anthropic.provider, Provider::Anthropic);
    assert!(anthropic.api_url.contains("anthropic.com"));

    let ollama = ProviderConfig::default_ollama();
    assert_eq!(ollama.provider, Provider::Ollama);
    assert!(ollama.api_url.contains("localhost:11434"));
    assert!(ollama.api_key.is_empty());
}

// ── Config Tests ─────────────────────────────────────────────

#[test]
fn config_default_has_all_providers() {
    let config = AppConfig::default();
    assert_eq!(config.active_provider, Provider::OpenAI);
    assert_eq!(config.openai.provider, Provider::OpenAI);
    assert_eq!(config.anthropic.provider, Provider::Anthropic);
    assert_eq!(config.ollama.provider, Provider::Ollama);
    assert!(config.ollama_models.is_empty());
}

#[test]
fn config_available_models_returns_all_cloud() {
    let models = AppConfig::available_models();
    assert!(models.len() >= 7);
    let ids: Vec<&str> = models.iter().map(|(_, id)| *id).collect();
    assert!(ids.contains(&"gpt-4.1"));
    assert!(ids.contains(&"gpt-5"));
    assert!(ids.contains(&"claude-opus-4-20250514"));
    assert!(ids.contains(&"claude-sonnet-4-20250514"));
}

#[test]
fn config_all_models_includes_ollama() {
    let mut config = AppConfig::default();
    config.ollama_models = vec!["llama3.2".to_string(), "mistral".to_string()];
    let all = config.all_models();
    assert!(all.len() >= 9); // 7 cloud + 2 ollama
    let ids: Vec<String> = all.iter().map(|(_, id)| id.clone()).collect();
    assert!(ids.contains(&"llama3.2".to_string()));
    assert!(ids.contains(&"mistral".to_string()));
}

#[test]
fn config_provider_config_for_model_routes_correctly() {
    let mut config = AppConfig::default();
    config.ollama_models = vec!["llama3.2".to_string()];

    let openai = config.provider_config_for_model("gpt-4.1");
    assert_eq!(openai.provider, Provider::OpenAI);

    let anthropic = config.provider_config_for_model("claude-sonnet-4-20250514");
    assert_eq!(anthropic.provider, Provider::Anthropic);

    let ollama = config.provider_config_for_model("llama3.2");
    assert_eq!(ollama.provider, Provider::Ollama);
    assert!(ollama.api_key.is_empty());
}

#[test]
fn config_apply_preset_updates_provider() {
    let mut config = AppConfig::default();
    config.apply_preset("Sonnet");
    assert_eq!(config.active_provider, Provider::Anthropic);
    assert!(config.anthropic.model.contains("sonnet"));

    config.apply_preset("GPT-5");
    assert_eq!(config.active_provider, Provider::OpenAI);
    assert_eq!(config.openai.model, "gpt-5");
}

// ── Cost Tests ───────────────────────────────────────────────

#[test]
fn estimate_tokens_heuristic() {
    assert_eq!(cost::estimate_tokens(""), 0);
    assert_eq!(cost::estimate_tokens("test"), 1); // 4 chars / 4 = 1
    assert_eq!(cost::estimate_tokens("hello world"), 3); // 11 chars / 4 = 2.75 -> 3
    let long = "a".repeat(1000);
    assert_eq!(cost::estimate_tokens(&long), 250);
}

#[test]
fn message_cost_known_models() {
    let tokens = 1000;
    let cost_openai = cost::message_cost("gpt-4.1", &Role::Assistant, tokens);
    assert!(cost_openai > 0.0);
    // gpt-4.1 output: $8/M tokens, so 1000 tokens = $0.008
    assert!((cost_openai - 0.008).abs() < 0.0001);

    let cost_anthropic = cost::message_cost("claude-sonnet-4-20250514", &Role::Assistant, tokens);
    assert!(cost_anthropic > 0.0);
    // sonnet output: $15/M tokens, so 1000 tokens = $0.015
    assert!((cost_anthropic - 0.015).abs() < 0.0001);
}

#[test]
fn message_cost_unknown_model_is_free() {
    let cost = cost::message_cost("llama3.2", &Role::Assistant, 1000);
    assert_eq!(cost, 0.0);
}

#[test]
fn message_cost_input_vs_output_pricing() {
    let tokens = 1_000_000;
    let input_cost = cost::message_cost("gpt-4.1", &Role::User, tokens);
    let output_cost = cost::message_cost("gpt-4.1", &Role::Assistant, tokens);
    // Input: $2/M, Output: $8/M
    assert!((input_cost - 2.0).abs() < 0.01);
    assert!((output_cost - 8.0).abs() < 0.01);
    assert!(output_cost > input_cost);
}

#[test]
fn conversation_cost_sums_all_messages() {
    let messages = vec![
        ChatMessage {
            role: Role::User,
            content: "a".repeat(400), // ~100 tokens
            streaming: false,
            model: Some("gpt-4.1".to_string()),
            token_count: Some(100),
            rating: 0,
            latency_ms: None,
            images: Vec::new(),
        },
        ChatMessage {
            role: Role::Assistant,
            content: "b".repeat(800), // ~200 tokens
            streaming: false,
            model: Some("gpt-4.1".to_string()),
            token_count: Some(200),
            rating: 0,
            latency_ms: None,
            images: Vec::new(),
        },
    ];
    let total = cost::conversation_cost(&messages);
    // 100 tokens input at $2/M + 200 tokens output at $8/M
    // = 0.0002 + 0.0016 = 0.0018
    assert!(total > 0.0);
    assert!((total - 0.0018).abs() < 0.0001);
}

#[test]
fn conversation_cost_skips_streaming() {
    let messages = vec![
        ChatMessage {
            role: Role::Assistant,
            content: "streaming...".to_string(),
            streaming: true,
            model: Some("gpt-4.1".to_string()),
            token_count: None,
            rating: 0,
            latency_ms: None,
            images: Vec::new(),
        },
    ];
    assert_eq!(cost::conversation_cost(&messages), 0.0);
}

// ── Diff Tests ───────────────────────────────────────────────

#[test]
fn diff_identical_texts() {
    let segments = diff::word_diff("hello world", "hello world");
    assert_eq!(segments.len(), 1);
    match &segments[0] {
        diff::DiffSegment::Common(t) => assert_eq!(t, "hello world"),
        _ => panic!("expected Common"),
    }
}

#[test]
fn diff_completely_different() {
    let segments = diff::word_diff("alpha beta", "gamma delta");
    let has_only_a = segments.iter().any(|s| matches!(s, diff::DiffSegment::OnlyA(_)));
    let has_only_b = segments.iter().any(|s| matches!(s, diff::DiffSegment::OnlyB(_)));
    assert!(has_only_a);
    assert!(has_only_b);
}

#[test]
fn diff_partial_overlap() {
    let segments = diff::word_diff("the cat sat on the mat", "the dog sat on the rug");
    let common_words: Vec<&str> = segments.iter().filter_map(|s| match s {
        diff::DiffSegment::Common(t) => Some(t.as_str()),
        _ => None,
    }).collect();
    // "the", "sat on the" should be common
    assert!(common_words.iter().any(|w| w.contains("the")));
    assert!(common_words.iter().any(|w| w.contains("sat")));
}

#[test]
fn diff_empty_inputs() {
    let segments = diff::word_diff("", "");
    assert!(segments.is_empty());
}

#[test]
fn diff_one_empty() {
    let segments = diff::word_diff("hello world", "");
    assert!(!segments.is_empty());
    assert!(segments.iter().all(|s| matches!(s, diff::DiffSegment::OnlyA(_))));
}

#[test]
fn agreement_percentage_identical() {
    let pct = diff::agreement_percentage("hello world", "hello world");
    assert!((pct - 100.0).abs() < 0.1);
}

#[test]
fn agreement_percentage_completely_different() {
    let pct = diff::agreement_percentage("alpha beta gamma", "delta epsilon zeta");
    assert_eq!(pct, 0.0);
}

#[test]
fn agreement_percentage_partial() {
    let pct = diff::agreement_percentage("the cat sat", "the dog sat");
    // 2 out of 3 words match
    assert!(pct > 50.0);
    assert!(pct < 100.0);
}

#[test]
fn agreement_percentage_empty() {
    assert_eq!(diff::agreement_percentage("", ""), 100.0);
}

// ── Export Tests ──────────────────────────────────────────────

#[test]
fn export_empty_conversation() {
    let conv = Conversation::new();
    let md = export::conversation_to_markdown(&conv);
    assert!(md.starts_with("# New Chat"));
}

#[test]
fn export_with_messages() {
    let mut conv = Conversation::new();
    conv.title = "Test Export".to_string();
    conv.add_user_message("Hello", Some("gpt-4.1".to_string()));
    conv.messages.push(ChatMessage {
        role: Role::Assistant,
        content: "Hi there!".to_string(),
        streaming: false,
        model: Some("gpt-4.1".to_string()),
        token_count: None,
        rating: 0,
        latency_ms: None,
        images: Vec::new(),
    });
    let md = export::conversation_to_markdown(&conv);
    assert!(md.contains("# Test Export"));
    assert!(md.contains("**You**"));
    assert!(md.contains("Hello"));
    assert!(md.contains("**Assistant (gpt-4.1)**"));
    assert!(md.contains("Hi there!"));
    assert!(md.contains("---"));
}

#[test]
fn export_with_tags() {
    let mut conv = Conversation::new();
    conv.tags = vec!["research".to_string(), "physics".to_string()];
    let md = export::conversation_to_markdown(&conv);
    assert!(md.contains("**Tags:** research, physics"));
}

#[test]
fn export_skips_streaming() {
    let mut conv = Conversation::new();
    conv.messages.push(ChatMessage {
        role: Role::Assistant,
        content: "streaming...".to_string(),
        streaming: true,
        model: None,
        token_count: None,
        rating: 0,
        latency_ms: None,
        images: Vec::new(),
    });
    let md = export::conversation_to_markdown(&conv);
    assert!(!md.contains("streaming..."));
}

// ── DB Tests ─────────────────────────────────────────────────

#[test]
fn db_roundtrip_conversation() {
    let conn = stoa::db::open_in_memory();
    let mut conv = Conversation::new();
    conv.title = "DB Test".to_string();
    conv.tags = vec!["test".to_string()];
    conv.pinned = true;
    conv.system_prompt = "Be concise".to_string();
    conv.add_user_message("Hello", Some("gpt-4.1".to_string()));
    conv.messages.push(ChatMessage {
        role: Role::Assistant,
        content: "Hi!".to_string(),
        streaming: false,
        model: Some("gpt-4.1".to_string()),
        token_count: Some(42),
        rating: 1,
        latency_ms: Some(350),
        images: Vec::new(),
    });

    stoa::db::save_conversation(&conn, &conv).unwrap();
    let loaded = stoa::db::load_all(&conn);

    assert_eq!(loaded.len(), 1);
    let c = &loaded[0];
    assert_eq!(c.id, conv.id);
    assert_eq!(c.title, "DB Test");
    assert_eq!(c.tags, vec!["test"]);
    assert!(c.pinned);
    assert_eq!(c.system_prompt, "Be concise");
    assert_eq!(c.messages.len(), 2);
    assert_eq!(c.messages[1].content, "Hi!");
    assert_eq!(c.messages[1].token_count, Some(42));
    assert_eq!(c.messages[1].rating, 1);
    assert_eq!(c.messages[1].latency_ms, Some(350));
}

#[test]
fn db_delete_conversation() {
    let conn = stoa::db::open_in_memory();
    let conv = Conversation::new();
    stoa::db::save_conversation(&conn, &conv).unwrap();
    assert_eq!(stoa::db::load_all(&conn).len(), 1);

    stoa::db::delete_conversation(&conn, &conv.id).unwrap();
    assert_eq!(stoa::db::load_all(&conn).len(), 0);
}

#[test]
fn db_rename_conversation() {
    let conn = stoa::db::open_in_memory();
    let conv = Conversation::new();
    stoa::db::save_conversation(&conn, &conv).unwrap();

    stoa::db::rename_conversation(&conn, &conv.id, "Renamed").unwrap();
    let loaded = stoa::db::load_all(&conn);
    assert_eq!(loaded[0].title, "Renamed");
}

#[test]
fn db_toggle_pin() {
    let conn = stoa::db::open_in_memory();
    let conv = Conversation::new();
    stoa::db::save_conversation(&conn, &conv).unwrap();

    stoa::db::toggle_pin(&conn, &conv.id, true).unwrap();
    let loaded = stoa::db::load_all(&conn);
    assert!(loaded[0].pinned);

    stoa::db::toggle_pin(&conn, &conv.id, false).unwrap();
    let loaded = stoa::db::load_all(&conn);
    assert!(!loaded[0].pinned);
}

#[test]
fn db_set_tags() {
    let conn = stoa::db::open_in_memory();
    let conv = Conversation::new();
    stoa::db::save_conversation(&conn, &conv).unwrap();

    stoa::db::set_tags(&conn, &conv.id, &["alpha".to_string(), "beta".to_string()]).unwrap();
    let loaded = stoa::db::load_all(&conn);
    assert_eq!(loaded[0].tags, vec!["alpha", "beta"]);
}

#[test]
fn db_search_conversations() {
    let conn = stoa::db::open_in_memory();

    let mut conv1 = Conversation::new();
    conv1.title = "Physics Research".to_string();
    conv1.add_user_message("What is quantum entanglement?", None);
    stoa::db::save_conversation(&conn, &conv1).unwrap();

    let mut conv2 = Conversation::new();
    conv2.title = "Cooking Tips".to_string();
    conv2.add_user_message("How to make pasta?", None);
    stoa::db::save_conversation(&conn, &conv2).unwrap();

    // Search by title
    let results = stoa::db::search_conversations(&conn, "Physics");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], conv1.id);

    // Search by message content
    let results = stoa::db::search_conversations(&conn, "quantum");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], conv1.id);

    // Search that matches nothing
    let results = stoa::db::search_conversations(&conn, "javascript");
    assert!(results.is_empty());

    // Search that matches both (via content)
    let results = stoa::db::search_conversations(&conn, "What");
    assert_eq!(results.len(), 1); // only conv1 has "What"
}

#[test]
fn db_pinned_sort_first() {
    let conn = stoa::db::open_in_memory();

    let mut conv1 = Conversation::new();
    conv1.title = "Unpinned".to_string();
    stoa::db::save_conversation(&conn, &conv1).unwrap();

    let mut conv2 = Conversation::new();
    conv2.title = "Pinned".to_string();
    conv2.pinned = true;
    stoa::db::save_conversation(&conn, &conv2).unwrap();

    let loaded = stoa::db::load_all(&conn);
    assert_eq!(loaded[0].title, "Pinned");
    assert_eq!(loaded[1].title, "Unpinned");
}

#[test]
fn db_skips_streaming_messages() {
    let conn = stoa::db::open_in_memory();
    let mut conv = Conversation::new();
    conv.add_user_message("test", None);
    conv.push_streaming_assistant(Some("gpt-4.1".to_string()));
    conv.update_streaming_at(1, "partial response...");
    // Message at index 1 is still streaming

    stoa::db::save_conversation(&conn, &conv).unwrap();
    let loaded = stoa::db::load_all(&conn);
    // Streaming message should not be persisted
    assert_eq!(loaded[0].messages.len(), 1);
    assert_eq!(loaded[0].messages[0].role, Role::User);
}

#[test]
fn db_forked_from_persists() {
    let conn = stoa::db::open_in_memory();
    let mut conv = Conversation::new();
    conv.forked_from = Some("parent-id-123".to_string());
    stoa::db::save_conversation(&conn, &conv).unwrap();

    let loaded = stoa::db::load_all(&conn);
    assert_eq!(loaded[0].forked_from, Some("parent-id-123".to_string()));
}

#[test]
fn db_system_prompt_persists() {
    let conn = stoa::db::open_in_memory();
    let mut conv = Conversation::new();
    conv.system_prompt = "You are a helpful assistant".to_string();
    stoa::db::save_conversation(&conn, &conv).unwrap();

    let loaded = stoa::db::load_all(&conn);
    assert_eq!(loaded[0].system_prompt, "You are a helpful assistant");
}

// ── OpenRouter Config Tests ──────────────────────────────────

#[test]
fn config_openrouter_models_list() {
    let models = AppConfig::openrouter_models();
    assert!(models.len() >= 6);
    let ids: Vec<&str> = models.iter().map(|(_, id)| *id).collect();
    assert!(ids.iter().any(|id| id.contains("google/")));
    assert!(ids.iter().any(|id| id.contains("meta-llama/")));
    assert!(ids.iter().any(|id| id.contains("deepseek/")));
}

#[test]
fn config_all_models_includes_openrouter_when_key_set() {
    let mut config = AppConfig::default();
    config.openrouter.api_key = "test-key".to_string();
    let all = config.all_models();
    let ids: Vec<String> = all.iter().map(|(_, id)| id.clone()).collect();
    assert!(ids.iter().any(|id| id.contains("google/")));
}

#[test]
fn config_all_models_excludes_openrouter_without_key() {
    let config = AppConfig::default();
    let all = config.all_models();
    let ids: Vec<String> = all.iter().map(|(_, id)| id.clone()).collect();
    assert!(!ids.iter().any(|id| id.contains("google/")));
}

#[test]
fn config_provider_routes_openrouter_by_slash() {
    let config = AppConfig::default();
    let pc = config.provider_config_for_model("google/gemini-2.5-flash-preview");
    assert_eq!(pc.provider, Provider::OpenRouter);
}

#[test]
fn default_openrouter_config() {
    let pc = ProviderConfig::default_openrouter();
    assert_eq!(pc.provider, Provider::OpenRouter);
    assert!(pc.api_url.contains("openrouter.ai"));
    assert!(pc.api_key.is_empty());
}

// ── Image Support Tests ──────────────────────────────────────

#[test]
fn add_user_message_with_images() {
    let mut conv = Conversation::new();
    conv.add_user_message_with_images("describe this", None, vec!["base64data".to_string()]);
    assert_eq!(conv.messages.len(), 1);
    assert_eq!(conv.messages[0].images.len(), 1);
    assert_eq!(conv.messages[0].images[0], "base64data");
    assert_eq!(conv.messages[0].content, "describe this");
}

#[test]
fn add_user_message_no_images_default() {
    let mut conv = Conversation::new();
    conv.add_user_message("test", None);
    assert!(conv.messages[0].images.is_empty());
}

// ── Folder Tests ─────────────────────────────────────────────

#[test]
fn conversation_folder_default_none() {
    let conv = Conversation::new();
    assert_eq!(conv.folder, None);
}

#[test]
fn db_folder_persists() {
    let conn = stoa::db::open_in_memory();
    let mut conv = Conversation::new();
    conv.folder = Some("research".to_string());
    stoa::db::save_conversation(&conn, &conv).unwrap();

    let loaded = stoa::db::load_all(&conn);
    assert_eq!(loaded[0].folder, Some("research".to_string()));
}

#[test]
fn fork_inherits_folder() {
    let mut conv = Conversation::new();
    conv.folder = Some("physics".to_string());
    conv.add_user_message("test", None);
    let forked = conv.fork(0);
    assert_eq!(forked.folder, Some("physics".to_string()));
}

// ── Export Format Tests ──────────────────────────────────────

#[test]
fn export_json_roundtrip() {
    let mut conv = Conversation::new();
    conv.title = "JSON Test".to_string();
    conv.add_user_message("hello", None);
    let json = stoa::export::conversation_to_json(&conv);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["title"].as_str().unwrap(), "JSON Test");
}

#[test]
fn export_html_contains_structure() {
    let mut conv = Conversation::new();
    conv.title = "HTML Test".to_string();
    conv.add_user_message("hello", None);
    conv.messages.push(ChatMessage {
        role: Role::Assistant,
        content: "world".to_string(),
        streaming: false,
        model: Some("gpt-4.1".to_string()),
        token_count: Some(10),
        rating: 0,
        latency_ms: Some(100),
        images: Vec::new(),
    });
    let html = stoa::export::conversation_to_html(&conv);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("HTML Test"));
    assert!(html.contains("hello"));
    assert!(html.contains("world"));
    assert!(html.contains("gpt-4.1"));
    assert!(html.contains("10 tokens"));
    assert!(html.contains("100 ms"));
}

// ── Import Tests ─────────────────────────────────────────────

#[test]
fn import_chatgpt_empty() {
    let convs = stoa::import::import_chatgpt("[]");
    assert!(convs.is_empty());
}

#[test]
fn import_chatgpt_invalid_json() {
    let convs = stoa::import::import_chatgpt("not json");
    assert!(convs.is_empty());
}

// ── Web Search Tests ─────────────────────────────────────────

#[test]
fn web_search_format_results_empty() {
    let formatted = stoa::web_search::format_results(&[]);
    assert!(formatted.is_empty());
}

#[test]
fn web_search_format_results_with_data() {
    let results = vec![
        stoa::web_search::SearchResult {
            title: "Test".to_string(),
            snippet: "A test result".to_string(),
            url: "https://example.com".to_string(),
        },
    ];
    let formatted = stoa::web_search::format_results(&results);
    assert!(formatted.contains("[Web search results]"));
    assert!(formatted.contains("Test"));
    assert!(formatted.contains("A test result"));
    assert!(formatted.contains("https://example.com"));
}

// ── Cost Tests for OpenRouter Models ─────────────────────────

#[test]
fn openrouter_models_are_free_in_cost() {
    // OpenRouter models not in pricing table = free (cost handled by OpenRouter billing)
    let cost = stoa::cost::message_cost("google/gemini-2.5-flash-preview", &Role::Assistant, 1000);
    assert_eq!(cost, 0.0);
}

// ── Shortcut / Overlay Regression Tests ─────────────────────

#[test]
fn esc_dismisses_overlays_one_layer_at_a_time() {
    let mut app = stoa::app::ChatApp::new_for_tests();
    app.shortcut_help_open = true;
    app.quick_switcher_open = true;
    app.command_palette_open = true;
    app.model_picker_open = true;

    let _ = app.update(stoa::app::Message::DismissOverlay);
    assert!(!app.shortcut_help_open);
    assert!(app.quick_switcher_open);
    assert!(app.command_palette_open);
    assert!(app.model_picker_open);

    let _ = app.update(stoa::app::Message::DismissOverlay);
    assert!(!app.quick_switcher_open);
    assert!(app.command_palette_open);
    assert!(app.model_picker_open);

    let _ = app.update(stoa::app::Message::DismissOverlay);
    assert!(!app.command_palette_open);
    assert!(app.model_picker_open);

    let _ = app.update(stoa::app::Message::DismissOverlay);
    assert!(!app.model_picker_open);
}

#[test]
fn readme_shortcut_rows_match_shared_shortcut_table() {
    let readme = std::fs::read_to_string("README.md").expect("read README");

    let expected = [
        ("Enter", "Send to primary model"),
        (&stoa::shortcuts::docs_binding(stoa::shortcuts::ShortcutAction::SendToAll), "Send to all models"),
        (&stoa::shortcuts::docs_binding(stoa::shortcuts::ShortcutAction::QuickSwitcher), "Quick Switcher"),
        (&stoa::shortcuts::docs_binding(stoa::shortcuts::ShortcutAction::CommandPalette), "Command Palette"),
        (&stoa::shortcuts::docs_binding(stoa::shortcuts::ShortcutAction::ExportMarkdown), "Export as Markdown"),
        (&stoa::shortcuts::docs_binding(stoa::shortcuts::ShortcutAction::NewConversation), "New conversation"),
        (&stoa::shortcuts::docs_binding(stoa::shortcuts::ShortcutAction::ShowSettings), "Settings"),
    ];

    for (binding, action) in expected {
        let row = format!("| `{binding}` | {action} |");
        assert!(readme.contains(&row), "README missing shortcut row: {row}");
    }
}

#[test]
fn keybinding_conflicts_are_detected() {
    let mut keys = stoa::config::Keybindings::default();
    keys.new_conversation = "Cmd+K".to_string();
    keys.quick_switcher = "Cmd+K".to_string();
    let conflicts = keys.conflicts();
    assert!(conflicts.iter().any(|(binding, actions)|
        binding == "Cmd+K"
            && actions.contains(&stoa::shortcuts::ShortcutAction::NewConversation)
            && actions.contains(&stoa::shortcuts::ShortcutAction::QuickSwitcher)
    ));
}

#[test]
fn config_migration_sets_current_schema_version() {
    let mut cfg = stoa::config::AppConfig::default();
    cfg.schema_version = 0;
    cfg.migrate();
    assert_eq!(cfg.schema_version, stoa::config::CONFIG_SCHEMA_VERSION);
}
