/// Simple web search using DuckDuckGo instant answer API.
pub async fn search(query: &str, max_results: usize) -> Result<Vec<SearchResult>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_redirect=1&no_html=1",
        urlencoding::encode(query)
    );

    let resp = client.get(&url).send().await.map_err(|e| format!("Search failed: {e}"))?;
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let mut results = Vec::new();

    // Abstract (main answer)
    if let Some(abstract_text) = json["AbstractText"].as_str() {
        if !abstract_text.is_empty() {
            results.push(SearchResult {
                title: json["Heading"].as_str().unwrap_or("").to_string(),
                snippet: abstract_text.to_string(),
                url: json["AbstractURL"].as_str().unwrap_or("").to_string(),
            });
        }
    }

    // Related topics
    if let Some(topics) = json["RelatedTopics"].as_array() {
        for topic in topics {
            if results.len() >= max_results { break; }
            if let (Some(text), Some(url)) = (topic["Text"].as_str(), topic["FirstURL"].as_str()) {
                if !text.is_empty() {
                    results.push(SearchResult {
                        title: text.chars().take(80).collect(),
                        snippet: text.to_string(),
                        url: url.to_string(),
                    });
                }
            }
        }
    }

    Ok(results)
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub snippet: String,
    pub url: String,
}

/// Format search results as context to prepend to a message.
pub fn format_results(results: &[SearchResult]) -> String {
    if results.is_empty() {
        return String::new();
    }
    let mut out = String::from("[Web search results]\n\n");
    for (i, r) in results.iter().enumerate() {
        out.push_str(&format!("{}. {}\n{}\nSource: {}\n\n", i + 1, r.title, r.snippet, r.url));
    }
    out.push_str("---\n\n");
    out
}
