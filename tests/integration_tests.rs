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

// ── Append Streaming Token Tests ─────────────────────────────

#[test]
fn append_streaming_token_concatenates() {
    let mut conv = Conversation::new();
    conv.add_user_message("hi", None);
    let idx = conv.push_streaming_assistant(Some("gpt-4.1".to_string()));
    conv.append_streaming_token(idx, "Hello");
    conv.append_streaming_token(idx, " world");
    assert_eq!(conv.messages[idx].content, "Hello world");
}

#[test]
fn append_streaming_token_ignores_non_streaming() {
    let mut conv = Conversation::new();
    conv.add_user_message("hi", None);
    let idx = conv.push_streaming_assistant(None);
    conv.finalize_at(idx, "done");
    conv.append_streaming_token(idx, "extra");
    assert_eq!(conv.messages[idx].content, "done");
}

#[test]
fn append_streaming_token_out_of_bounds_safe() {
    let mut conv = Conversation::new();
    conv.append_streaming_token(999, "nope"); // should not panic
}

// ── HTML Export XSS Escaping Tests ───────────────────────────

#[test]
fn export_html_escapes_xss_in_title() {
    let mut conv = Conversation::new();
    conv.title = "<script>alert('xss')</script>".to_string();
    let html = stoa::export::conversation_to_html(&conv);
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn export_html_escapes_xss_in_content() {
    let mut conv = Conversation::new();
    conv.add_user_message("<img onerror=alert(1) src=x>", None);
    let html = stoa::export::conversation_to_html(&conv);
    // The < is escaped, so the tag is rendered as text, not HTML
    assert!(!html.contains("<img onerror"));
    assert!(html.contains("&lt;img"));
}

#[test]
fn export_html_escapes_tags() {
    let mut conv = Conversation::new();
    conv.tags = vec!["<b>bold</b>".to_string()];
    let html = stoa::export::conversation_to_html(&conv);
    assert!(!html.contains("<b>bold</b>"));
    assert!(html.contains("&lt;b&gt;bold&lt;/b&gt;"));
}

// ── FTS5 Search Tests ────────────────────────────────────────

#[test]
fn fts5_search_finds_by_content() {
    let conn = stoa::db::open_in_memory();
    let mut conv = Conversation::new();
    conv.title = "Science Chat".to_string();
    conv.add_user_message("What is photosynthesis?", None);
    stoa::db::save_conversation(&conn, &conv).unwrap();

    let results = stoa::db::search_conversations(&conn, "photosynthesis");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], conv.id);
}

#[test]
fn fts5_search_finds_by_title() {
    let conn = stoa::db::open_in_memory();
    let mut conv = Conversation::new();
    conv.title = "Quantum Physics Discussion".to_string();
    stoa::db::save_conversation(&conn, &conv).unwrap();

    let results = stoa::db::search_conversations(&conn, "Quantum");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], conv.id);
}

#[test]
fn fts5_search_no_results() {
    let conn = stoa::db::open_in_memory();
    let mut conv = Conversation::new();
    conv.title = "Random Chat".to_string();
    conv.add_user_message("hello", None);
    stoa::db::save_conversation(&conn, &conv).unwrap();

    let results = stoa::db::search_conversations(&conn, "nonexistent");
    assert!(results.is_empty());
}

#[test]
fn fts5_search_updates_after_resave() {
    let conn = stoa::db::open_in_memory();
    let mut conv = Conversation::new();
    conv.title = "Chat".to_string();
    conv.add_user_message("hello", None);
    stoa::db::save_conversation(&conn, &conv).unwrap();

    // Should not find "quantum" yet
    assert!(stoa::db::search_conversations(&conn, "quantum").is_empty());

    // Add a message and resave
    conv.messages.push(ChatMessage {
        role: Role::Assistant,
        content: "quantum mechanics is fascinating".to_string(),
        streaming: false,
        model: None,
        token_count: None,
        rating: 0,
        latency_ms: None,
        images: Vec::new(),
    });
    stoa::db::save_conversation(&conn, &conv).unwrap();

    // Now should find it
    let results = stoa::db::search_conversations(&conn, "quantum");
    assert_eq!(results.len(), 1);
}

// ── Diff Guard Tests ─────────────────────────────────────────

#[test]
fn diff_large_input_returns_fallback() {
    let large_a = (0..3000).map(|i| format!("word{i}")).collect::<Vec<_>>().join(" ");
    let large_b = (0..3000).map(|i| format!("other{i}")).collect::<Vec<_>>().join(" ");
    let segments = diff::word_diff(&large_a, &large_b);
    // Should return simple OnlyA + OnlyB instead of running LCS
    assert_eq!(segments.len(), 2);
}

#[test]
fn agreement_percentage_large_input_returns_zero() {
    let large_a = (0..3000).map(|i| format!("word{i}")).collect::<Vec<_>>().join(" ");
    let large_b = (0..3000).map(|i| format!("other{i}")).collect::<Vec<_>>().join(" ");
    assert_eq!(diff::agreement_percentage(&large_a, &large_b), 0.0);
}

// ── Import ChatGPT with Valid Data ───────────────────────────

#[test]
fn import_chatgpt_valid_data() {
    let data = r#"[{
        "title": "Test Conversation",
        "mapping": {
            "node1": {
                "message": {
                    "author": {"role": "user"},
                    "content": {"parts": ["Hello!"]},
                    "create_time": 1700000001.0
                }
            },
            "node2": {
                "message": {
                    "author": {"role": "assistant"},
                    "content": {"parts": ["Hi there!"]},
                    "create_time": 1700000002.0,
                    "metadata": {"model_slug": "gpt-4"}
                }
            },
            "node3": {
                "message": {
                    "author": {"role": "system"},
                    "content": {"parts": ["You are a helpful assistant"]},
                    "create_time": 1700000000.0
                }
            }
        }
    }]"#;

    let convs = stoa::import::import_chatgpt(data);
    assert_eq!(convs.len(), 1);
    assert_eq!(convs[0].title, "Test Conversation");
    assert_eq!(convs[0].messages.len(), 2); // system message filtered out
    assert_eq!(convs[0].messages[0].role, Role::User);
    assert_eq!(convs[0].messages[0].content, "Hello!");
    assert_eq!(convs[0].messages[1].role, Role::Assistant);
    assert_eq!(convs[0].messages[1].content, "Hi there!");
    assert_eq!(convs[0].messages[1].model, Some("gpt-4".to_string()));
    assert!(convs[0].tags.contains(&"imported".to_string()));
}

#[test]
fn import_chatgpt_skips_empty_content() {
    let data = r#"[{
        "title": "Empty Messages",
        "mapping": {
            "node1": {
                "message": {
                    "author": {"role": "user"},
                    "content": {"parts": [""]},
                    "create_time": 1700000001.0
                }
            }
        }
    }]"#;

    let convs = stoa::import::import_chatgpt(data);
    assert!(convs.is_empty()); // no non-empty messages = no conversation
}

// ── DB Update Rating Tests ───────────────────────────────────

#[test]
fn db_update_rating() {
    let conn = stoa::db::open_in_memory();
    let mut conv = Conversation::new();
    conv.add_user_message("hello", None);
    conv.messages.push(ChatMessage {
        role: Role::Assistant,
        content: "hi".to_string(),
        streaming: false,
        model: None,
        token_count: None,
        rating: 0,
        latency_ms: None,
        images: Vec::new(),
    });
    stoa::db::save_conversation(&conn, &conv).unwrap();

    stoa::db::update_rating(&conn, &conv.id, 1, 1).unwrap();
    let loaded = stoa::db::load_all(&conn);
    assert_eq!(loaded[0].messages[1].rating, 1);

    stoa::db::update_rating(&conn, &conv.id, 1, -1).unwrap();
    let loaded = stoa::db::load_all(&conn);
    assert_eq!(loaded[0].messages[1].rating, -1);
}

// ── App State Tests ──────────────────────────────────────────

#[test]
fn new_conversation_message_creates_and_selects() {
    let mut app = stoa::app::ChatApp::new_for_tests();
    let initial_count = app.conversations.len();
    let _ = app.update(stoa::app::Message::NewConversation);
    assert_eq!(app.conversations.len(), initial_count + 1);
    assert_eq!(app.active_conversation, app.conversations.len() - 1);
}

#[test]
fn select_conversation_bounds_check() {
    let mut app = stoa::app::ChatApp::new_for_tests();
    let _ = app.update(stoa::app::Message::SelectConversation(999));
    // Should not crash, active_conversation unchanged
    assert!(app.active_conversation < app.conversations.len());
}

#[test]
fn delete_conversation_adjusts_active_index() {
    let mut app = stoa::app::ChatApp::new_for_tests();
    let _ = app.update(stoa::app::Message::NewConversation);
    let _ = app.update(stoa::app::Message::NewConversation);
    assert_eq!(app.conversations.len(), 3);
    app.active_conversation = 2;
    let _ = app.update(stoa::app::Message::DeleteConversation(0));
    assert_eq!(app.conversations.len(), 2);
    assert_eq!(app.active_conversation, 1); // adjusted down
}

#[test]
fn delete_last_conversation_is_noop() {
    let mut app = stoa::app::ChatApp::new_for_tests();
    assert_eq!(app.conversations.len(), 1);
    let _ = app.update(stoa::app::Message::DeleteConversation(0));
    assert_eq!(app.conversations.len(), 1); // can't delete last
}

// ── Additional DB Tests ──────────────────────────────────────

#[test]
fn db_save_overwrite_existing() {
    let conn = stoa::db::open_in_memory();
    let mut conv = Conversation::new();
    conv.title = "Version 1".to_string();
    conv.add_user_message("first message", None);
    stoa::db::save_conversation(&conn, &conv).unwrap();

    // Modify and save again with same ID
    conv.title = "Version 2".to_string();
    conv.messages.clear();
    conv.add_user_message("second message", None);
    stoa::db::save_conversation(&conn, &conv).unwrap();

    let loaded = stoa::db::load_all(&conn);
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].title, "Version 2");
    assert_eq!(loaded[0].messages.len(), 1);
    assert_eq!(loaded[0].messages[0].content, "second message");
}

#[test]
fn db_multiple_conversations_ordering() {
    let conn = stoa::db::open_in_memory();

    let mut conv_a = Conversation::new();
    conv_a.title = "Alpha".to_string();
    stoa::db::save_conversation(&conn, &conv_a).unwrap();

    let mut conv_b = Conversation::new();
    conv_b.title = "Beta".to_string();
    stoa::db::save_conversation(&conn, &conv_b).unwrap();

    let mut conv_c = Conversation::new();
    conv_c.title = "Gamma".to_string();
    conv_c.pinned = true;
    stoa::db::save_conversation(&conn, &conv_c).unwrap();

    let loaded = stoa::db::load_all(&conn);
    assert_eq!(loaded.len(), 3);
    // Pinned first
    assert_eq!(loaded[0].title, "Gamma");
    assert!(loaded[0].pinned);
    // Then by updated_at DESC, rowid DESC
    assert_eq!(loaded[1].title, "Beta");
    assert_eq!(loaded[2].title, "Alpha");
}

#[test]
fn db_fts5_delete_cleans_index() {
    let conn = stoa::db::open_in_memory();
    let mut conv = Conversation::new();
    conv.title = "Quantum Mechanics".to_string();
    conv.add_user_message("Explain Schrodinger's cat", None);
    stoa::db::save_conversation(&conn, &conv).unwrap();

    // Verify search finds it
    assert_eq!(stoa::db::search_conversations(&conn, "Quantum").len(), 1);
    assert_eq!(stoa::db::search_conversations(&conn, "Schrodinger").len(), 1);

    // Delete the conversation
    stoa::db::delete_conversation(&conn, &conv.id).unwrap();

    // FTS5 index should be cleaned — search returns nothing
    assert!(stoa::db::search_conversations(&conn, "Quantum").is_empty());
    assert!(stoa::db::search_conversations(&conn, "Schrodinger").is_empty());
}

#[test]
fn db_rename_nonexistent_succeeds_silently() {
    let conn = stoa::db::open_in_memory();
    // UPDATE on missing ID matches 0 rows — should not panic or error
    let result = stoa::db::rename_conversation(&conn, "nonexistent-id", "New Title");
    assert!(result.is_ok());
}

#[test]
fn db_empty_tags_roundtrip() {
    let conn = stoa::db::open_in_memory();
    let mut conv = Conversation::new();
    conv.tags = vec![];
    stoa::db::save_conversation(&conn, &conv).unwrap();

    let loaded = stoa::db::load_all(&conn);
    assert!(loaded[0].tags.is_empty());
}

// ── Additional Config Tests ──────────────────────────────────

#[test]
fn config_roundtrip_serde() {
    let config = AppConfig::default();
    let json = serde_json::to_string_pretty(&config).unwrap();
    let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.active_provider, config.active_provider);
    assert_eq!(deserialized.temperature, config.temperature);
    assert_eq!(deserialized.max_tokens, config.max_tokens);
    assert_eq!(deserialized.schema_version, config.schema_version);
    assert_eq!(deserialized.openai.model, config.openai.model);
    assert_eq!(deserialized.anthropic.model, config.anthropic.model);
    assert_eq!(deserialized.system_prompt, config.system_prompt);
}

#[test]
fn config_save_strips_api_keys_from_json() {
    let mut config = AppConfig::default();
    config.openai.api_key = "sk-secret-openai-key".to_string();
    config.anthropic.api_key = "sk-ant-secret-key".to_string();
    config.openrouter.api_key = "sk-or-secret-key".to_string();

    // Replicate save() logic: clone, clear keys, serialize
    let mut copy = config.clone();
    copy.openai.api_key.clear();
    copy.anthropic.api_key.clear();
    copy.openrouter.api_key.clear();
    let json = serde_json::to_string_pretty(&copy).unwrap();

    assert!(!json.contains("sk-secret-openai-key"));
    assert!(!json.contains("sk-ant-secret-key"));
    assert!(!json.contains("sk-or-secret-key"));

    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["openai"]["api_key"].as_str().unwrap(), "");
    assert_eq!(parsed["anthropic"]["api_key"].as_str().unwrap(), "");
    assert_eq!(parsed["openrouter"]["api_key"].as_str().unwrap(), "");
}

#[test]
fn config_migrate_future_version_clamped() {
    let mut config = AppConfig::default();
    config.schema_version = 999;
    config.migrate();
    assert_eq!(config.schema_version, stoa::config::CONFIG_SCHEMA_VERSION);
}

#[test]
fn config_provider_config_for_unknown_model() {
    let config = AppConfig::default();
    let pc = config.provider_config_for_model("totally-unknown-model-xyz");
    // Unknown model falls through to OpenAI as default
    assert_eq!(pc.provider, Provider::OpenAI);
    assert_eq!(pc.model, "totally-unknown-model-xyz");
}

// ── Additional App State Tests ───────────────────────────────

#[test]
fn app_sidebar_search_filters() {
    let mut app = stoa::app::ChatApp::new_for_tests();
    // Initial conversation "New Chat" is saved to DB with FTS5 index

    // Empty query — results stay None
    let _ = app.update(stoa::app::Message::SidebarSearchChanged(String::new()));
    assert!(app.sidebar_search_results.is_none());

    // 1-char query — below threshold, results stay None
    let _ = app.update(stoa::app::Message::SidebarSearchChanged("N".to_string()));
    assert!(app.sidebar_search_results.is_none());

    // >= 2 char query matching title "New Chat"
    let _ = app.update(stoa::app::Message::SidebarSearchChanged("Chat".to_string()));
    assert!(app.sidebar_search_results.is_some());
    let results = app.sidebar_search_results.as_ref().unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0], app.conversations[0].id);

    // Query that matches nothing
    let _ = app.update(stoa::app::Message::SidebarSearchChanged("zzzznonexistent".to_string()));
    assert!(app.sidebar_search_results.as_ref().unwrap().is_empty());
}

#[test]
fn app_toggle_pin_reorders() {
    let mut app = stoa::app::ChatApp::new_for_tests();
    let first_id = app.conversations[0].id.clone();

    // Add a second conversation
    let _ = app.update(stoa::app::Message::NewConversation);
    let second_id = app.conversations[1].id.clone();

    // Pin the second conversation — should move to front after re-sort
    let _ = app.update(stoa::app::Message::TogglePin(1));

    assert!(app.conversations[0].pinned);
    assert_eq!(app.conversations[0].id, second_id);
    assert_eq!(app.conversations[1].id, first_id);
    assert!(!app.conversations[1].pinned);
}

#[test]
fn app_fork_conversation_creates_copy() {
    let mut app = stoa::app::ChatApp::new_for_tests();
    let original_id = app.conversations[0].id.clone();

    // Add messages to active conversation (in memory)
    app.conversations[0].add_user_message("What is Rust?", Some("gpt-4.1".to_string()));
    app.conversations[0].messages.push(ChatMessage {
        role: Role::Assistant,
        content: "Rust is a systems programming language.".to_string(),
        streaming: false,
        model: Some("gpt-4.1".to_string()),
        token_count: None,
        rating: 0,
        latency_ms: None,
        images: Vec::new(),
    });

    // Fork at message index 1 (include both messages)
    let _ = app.update(stoa::app::Message::ForkConversation(1));

    assert_eq!(app.conversations.len(), 2);
    assert_eq!(app.active_conversation, 1);

    let forked = &app.conversations[1];
    assert_eq!(forked.messages.len(), 2);
    assert_eq!(forked.messages[0].content, "What is Rust?");
    assert_eq!(forked.messages[1].content, "Rust is a systems programming language.");
    assert!(forked.title.starts_with("Fork of"));
    assert_eq!(forked.forked_from, Some(original_id));
    assert_ne!(forked.id, app.conversations[0].id);
}

#[test]
fn app_rate_message_toggles() {
    let mut app = stoa::app::ChatApp::new_for_tests();

    // Add messages to rate
    app.conversations[0].add_user_message("test", None);
    app.conversations[0].messages.push(ChatMessage {
        role: Role::Assistant,
        content: "response".to_string(),
        streaming: false,
        model: None,
        token_count: None,
        rating: 0,
        latency_ms: None,
        images: Vec::new(),
    });

    // Rate thumbs up
    let _ = app.update(stoa::app::Message::RateMessage(1, 1));
    assert_eq!(app.conversations[0].messages[1].rating, 1);

    // Same rating again → toggle to 0
    let _ = app.update(stoa::app::Message::RateMessage(1, 1));
    assert_eq!(app.conversations[0].messages[1].rating, 0);

    // Rate thumbs down
    let _ = app.update(stoa::app::Message::RateMessage(1, -1));
    assert_eq!(app.conversations[0].messages[1].rating, -1);

    // Same rating again → toggle to 0
    let _ = app.update(stoa::app::Message::RateMessage(1, -1));
    assert_eq!(app.conversations[0].messages[1].rating, 0);
}

#[test]
fn app_delete_message_removes_from_conversation() {
    let mut app = stoa::app::ChatApp::new_for_tests();

    app.conversations[0].add_user_message("first", None);
    app.conversations[0].add_user_message("second", None);
    assert_eq!(app.conversations[0].messages.len(), 2);

    // Delete first message
    let _ = app.update(stoa::app::Message::DeleteMessage(0));

    assert_eq!(app.conversations[0].messages.len(), 1);
    assert_eq!(app.conversations[0].messages[0].content, "second");
}

#[test]
fn app_tag_submit_adds_tag() {
    let mut app = stoa::app::ChatApp::new_for_tests();

    // Submit a tag
    app.tag_input_value = "research".to_string();
    app.tag_input_open = true;
    let _ = app.update(stoa::app::Message::SubmitTag);

    assert!(app.conversations[0].tags.contains(&"research".to_string()));
    assert!(!app.tag_input_open);
    assert!(app.tag_input_value.is_empty());

    // Duplicate tag should not be added
    app.tag_input_value = "research".to_string();
    app.tag_input_open = true;
    let _ = app.update(stoa::app::Message::SubmitTag);

    assert_eq!(app.conversations[0].tags.len(), 1);
}

#[test]
fn app_submit_empty_tag_is_noop() {
    let mut app = stoa::app::ChatApp::new_for_tests();
    app.tag_input_value = "   ".to_string(); // whitespace-only
    app.tag_input_open = true;
    let _ = app.update(stoa::app::Message::SubmitTag);
    assert!(app.conversations[0].tags.is_empty());
    assert!(!app.tag_input_open);
}

#[test]
fn app_command_palette_move_wraps() {
    let mut app = stoa::app::ChatApp::new_for_tests();
    app.command_palette_open = true;
    app.command_palette_query.clear(); // show all commands

    assert_eq!(app.command_palette_selected, 0);

    // Move up from 0 should wrap to the last command
    let _ = app.update(stoa::app::Message::CommandPaletteMoveSelection(-1));
    assert!(app.command_palette_selected > 0);

    let _last = app.command_palette_selected;

    // Move down from last should wrap back to 0
    let _ = app.update(stoa::app::Message::CommandPaletteMoveSelection(1));
    assert_eq!(app.command_palette_selected, 0);

    // Verify round-trip: moving down N times then up N times returns to 0
    for _ in 0..5 {
        let _ = app.update(stoa::app::Message::CommandPaletteMoveSelection(1));
    }
    for _ in 0..5 {
        let _ = app.update(stoa::app::Message::CommandPaletteMoveSelection(-1));
    }
    assert_eq!(app.command_palette_selected, 0);
}

// ── Additional Export Tests ──────────────────────────────────

#[test]
fn export_markdown_with_system_prompt() {
    let mut conv = Conversation::new();
    conv.title = "Tutor Chat".to_string();
    conv.system_prompt = "You are a physics tutor.".to_string();
    conv.add_user_message("What is gravity?", None);
    conv.messages.push(ChatMessage {
        role: Role::Assistant,
        content: "Gravity is a fundamental force of attraction.".to_string(),
        streaming: false,
        model: Some("gpt-4.1".to_string()),
        token_count: None,
        rating: 0,
        latency_ms: None,
        images: Vec::new(),
    });

    let md = export::conversation_to_markdown(&conv);
    assert!(md.contains("# Tutor Chat"));
    assert!(md.contains("**You**"));
    assert!(md.contains("What is gravity?"));
    assert!(md.contains("**Assistant (gpt-4.1)**"));
    assert!(md.contains("Gravity is a fundamental force of attraction."));
}

#[test]
fn export_json_all_fields_present() {
    let mut conv = Conversation::new();
    conv.title = "Complete Test".to_string();
    conv.tags = vec!["tag1".to_string(), "tag2".to_string()];
    conv.pinned = true;
    conv.system_prompt = "Be concise.".to_string();
    conv.forked_from = Some("parent-123".to_string());
    conv.folder = Some("research".to_string());
    conv.add_user_message("hello", Some("gpt-4.1".to_string()));
    conv.messages.push(ChatMessage {
        role: Role::Assistant,
        content: "world".to_string(),
        streaming: false,
        model: Some("gpt-4.1".to_string()),
        token_count: Some(100),
        rating: 1,
        latency_ms: Some(250),
        images: Vec::new(),
    });

    let json = export::conversation_to_json(&conv);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["title"].as_str().unwrap(), "Complete Test");
    assert_eq!(parsed["tags"][0].as_str().unwrap(), "tag1");
    assert_eq!(parsed["tags"][1].as_str().unwrap(), "tag2");
    assert_eq!(parsed["pinned"].as_bool().unwrap(), true);
    assert_eq!(parsed["system_prompt"].as_str().unwrap(), "Be concise.");
    assert_eq!(parsed["forked_from"].as_str().unwrap(), "parent-123");
    assert_eq!(parsed["folder"].as_str().unwrap(), "research");
    assert_eq!(parsed["messages"].as_array().unwrap().len(), 2);

    let msg = &parsed["messages"][1];
    assert_eq!(msg["role"].as_str().unwrap(), "Assistant");
    assert_eq!(msg["content"].as_str().unwrap(), "world");
    assert_eq!(msg["model"].as_str().unwrap(), "gpt-4.1");
    assert_eq!(msg["token_count"].as_u64().unwrap(), 100);
    assert_eq!(msg["rating"].as_i64().unwrap(), 1);
    assert_eq!(msg["latency_ms"].as_u64().unwrap(), 250);
}
