use serde::Serialize;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    pub live: bool,
    pub latency_ms: u64,
    pub message: String,
    pub detected_models: Vec<String>,
}

/// Tests connectivity to an LLM endpoint, measuring ping latency and discovering loaded models.
pub async fn test_llm_endpoint(
    protocol: &str,
    base_url: &str,
    api_key: Option<&str>,
) -> TestResult {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(4500))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return TestResult {
                live: false,
                latency_ms: 0,
                message: format!("Client builder error: {}", e),
                detected_models: vec![],
            }
        }
    };

    let start = Instant::now();
    let clean_url = base_url.trim_end_matches('/');

    match protocol.to_lowercase().as_str() {
        "ollama" => {
            // Ollama model discovery endpoint is /api/tags
            let tags_url = format!("{}/api/tags", clean_url);
            match client.get(&tags_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let latency = start.elapsed().as_millis() as u64;
                    let mut models = Vec::new();
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        if let Some(list) = json["models"].as_array() {
                            for m in list {
                                if let Some(name) = m["name"].as_str() {
                                    models.push(name.to_string());
                                }
                            }
                        }
                    }
                    TestResult {
                        live: true,
                        latency_ms: latency,
                        message: format!("Ollama live with {} model(s)", models.len()),
                        detected_models: models,
                    }
                }
                Ok(resp) => TestResult {
                    live: false,
                    latency_ms: 0,
                    message: format!("Ollama returned HTTP {}", resp.status()),
                    detected_models: vec![],
                },
                Err(e) => TestResult {
                    live: false,
                    latency_ms: 0,
                    message: format!("Ollama unreachable: {}", e),
                    detected_models: vec![],
                },
            }
        }
        "anthropic" => {
            // Anthropic doesn't have a public free ping, but we check reaching api.anthropic.com
            let url = if clean_url.is_empty() {
                "https://api.anthropic.com".to_string()
            } else {
                clean_url.to_string()
            };
            match client.get(&url).send().await {
                Ok(_) => TestResult {
                    live: true,
                    latency_ms: start.elapsed().as_millis() as u64,
                    message: "Anthropic reachable".into(),
                    detected_models: vec![
                        "claude-3-5-sonnet-latest".into(),
                        "claude-3-haiku-latest".into(),
                    ],
                },
                Err(e) => TestResult {
                    live: false,
                    latency_ms: 0,
                    message: format!("Connection error: {}", e),
                    detected_models: vec![],
                },
            }
        }
        _ => {
            // OpenAI / LM Studio compatible: query /models
            let models_url = format!("{}/models", clean_url);
            let mut req = client.get(&models_url);
            if let Some(key) = api_key {
                if !key.trim().is_empty() {
                    req = req.bearer_auth(key.trim());
                }
            }

            match req.send().await {
                Ok(resp) if resp.status().is_success() => {
                    let latency = start.elapsed().as_millis() as u64;
                    let mut models = Vec::new();
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        if let Some(data) = json["data"].as_array() {
                            for m in data {
                                if let Some(id) = m["id"].as_str() {
                                    models.push(id.to_string());
                                }
                            }
                        }
                    }
                    TestResult {
                        live: true,
                        latency_ms: latency,
                        message: format!("Endpoint online ({} models loaded)", models.len()),
                        detected_models: models,
                    }
                }
                Ok(resp) => TestResult {
                    live: false,
                    latency_ms: 0,
                    message: format!("Endpoint returned HTTP {}", resp.status()),
                    detected_models: vec![],
                },
                Err(e) => TestResult {
                    live: false,
                    latency_ms: 0,
                    message: format!("Unreachable: {}", e),
                    detected_models: vec![],
                },
            }
        }
    }
}

/// Tests connectivity to a search backend.
pub async fn test_search_endpoint(engine: &str, url_or_key: &str) -> TestResult {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(5000))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return TestResult {
                live: false,
                latency_ms: 0,
                message: format!("Client error: {}", e),
                detected_models: vec![],
            }
        }
    };

    let start = Instant::now();

    match engine.to_lowercase().as_str() {
        "searxng" => {
            let base = url_or_key.trim_end_matches('/');
            if base.is_empty() {
                return TestResult {
                    live: false,
                    latency_ms: 0,
                    message: "SearXNG URL is required".into(),
                    detected_models: vec![],
                };
            }
            let search_url = format!("{}/search?q=test&format=json", base);
            match client.get(&search_url).send().await {
                Ok(resp) if resp.status().is_success() => TestResult {
                    live: true,
                    latency_ms: start.elapsed().as_millis() as u64,
                    message: "SearXNG online".into(),
                    detected_models: vec![],
                },
                Ok(resp) => TestResult {
                    live: false,
                    latency_ms: 0,
                    message: format!("HTTP {}", resp.status()),
                    detected_models: vec![],
                },
                Err(e) => TestResult {
                    live: false,
                    latency_ms: 0,
                    message: format!("SearXNG error: {}", e),
                    detected_models: vec![],
                },
            }
        }
        "brave search" | "brave" => {
            if url_or_key.trim().is_empty() {
                return TestResult {
                    live: false,
                    latency_ms: 0,
                    message: "Brave API key is required".into(),
                    detected_models: vec![],
                };
            }
            match client
                .get("https://api.search.brave.com/res/v1/web/search?q=test&count=1")
                .header("X-Subscription-Token", url_or_key.trim())
                .header("Accept", "application/json")
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => TestResult {
                    live: true,
                    latency_ms: start.elapsed().as_millis() as u64,
                    message: "Brave API authorized".into(),
                    detected_models: vec![],
                },
                Ok(resp) => TestResult {
                    live: false,
                    latency_ms: 0,
                    message: format!("Brave returned HTTP {}", resp.status()),
                    detected_models: vec![],
                },
                Err(e) => TestResult {
                    live: false,
                    latency_ms: 0,
                    message: format!("Brave error: {}", e),
                    detected_models: vec![],
                },
            }
        }
        _ => {
            // DuckDuckGo test
            match client
                .get("https://html.duckduckgo.com/html/?q=ping")
                .header(
                    "User-Agent",
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
                )
                .send()
                .await
            {
                Ok(_) => TestResult {
                    live: true,
                    latency_ms: start.elapsed().as_millis() as u64,
                    message: "DuckDuckGo active".into(),
                    detected_models: vec![],
                },
                Err(e) => TestResult {
                    live: false,
                    latency_ms: 0,
                    message: format!("DDG error: {}", e),
                    detected_models: vec![],
                },
            }
        }
    }
}
