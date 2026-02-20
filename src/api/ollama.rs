/// Discover available Ollama models by calling GET /api/tags.
pub async fn discover_models(base_url: &str) -> Result<Vec<String>, String> {
    let tags_url = base_url
        .replace("/v1/chat/completions", "/api/tags")
        .replace("/v1/", "/api/");
    // Fallback: if URL doesn't contain /v1/, try appending /api/tags to base
    let tags_url = if tags_url.contains("/api/tags") {
        tags_url
    } else {
        format!("{}/api/tags", base_url.trim_end_matches('/'))
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(&tags_url).send().await.map_err(|e| format!("Ollama not reachable: {e}"))?;
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let models = json["models"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["name"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(models)
}
