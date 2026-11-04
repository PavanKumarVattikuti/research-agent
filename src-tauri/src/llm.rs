use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub protocol: String, // "openai" | "anthropic" | "ollama" | "custom"
    pub base_url: String, // e.g. "http://localhost:1234/v1"
    pub model_name: String,
    pub api_key: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            protocol: "openai".to_string(),
            base_url: "http://localhost:1234/v1".to_string(),
            model_name: "local-model".to_string(),
            api_key: None,
        }
    }
}

/// Normalizes OpenAI-compatible chat URLs to avoid double suffixes or missing paths
pub fn normalize_openai_chat_url(base: &str) -> String {
    let trimmed = base.trim().trim_end_matches('/');
    if trimmed.ends_with("/chat/completions") {
        trimmed.to_string()
    } else if trimmed.ends_with("/v1") {
        format!("{}/chat/completions", trimmed)
    } else {
        format!("{}/v1/chat/completions", trimmed)
    }
}

/// Normalizes Ollama API chat URLs
pub fn normalize_ollama_chat_url(base: &str) -> String {
    let trimmed = base.trim().trim_end_matches('/');
    if trimmed.ends_with("/api/chat") {
        trimmed.to_string()
    } else {
        format!("{}/api/chat", trimmed)
    }
}

/// Executes a single atomic micro-task with an LLM.
/// One task at a time ensures laser-sharp focus and zero context-spill hallucination.
pub async fn invoke_micro_task(
    system_prompt: &str,
    user_prompt: &str,
    config: &LlmConfig,
) -> Result<String, String> {
    invoke_micro_task_with_tokens(None, system_prompt, user_prompt, 4096, config).await
}

/// Executes a single atomic micro-task with explicit token ceiling and strict 60s fail-fast timeout.
pub async fn invoke_micro_task_with_tokens(
    app: Option<&tauri::AppHandle>,
    system_prompt: &str,
    user_prompt: &str,
    max_tokens: u32,
    config: &LlmConfig,
) -> Result<String, String> {
    let start = std::time::Instant::now();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let fut = async {
        match config.protocol.to_lowercase().as_str() {
            "anthropic" => invoke_anthropic(&client, system_prompt, user_prompt, max_tokens, config).await,
            "ollama" => invoke_ollama(&client, system_prompt, user_prompt, max_tokens, config).await,
            _ => invoke_openai_compatible(&client, system_prompt, user_prompt, max_tokens, config).await,
        }
    };

    match tokio::time::timeout(Duration::from_secs(60), fut).await {
        Ok(res) => {
            let elapsed_ms = start.elapsed().as_millis() as u64;
            match &res {
                Ok(content) => {
                    crate::logger::log_event(
                        app,
                        "LLM",
                        elapsed_ms,
                        &format!(
                            "LLM ({}) responded in {}ms ({} chars, max_tokens: {})",
                            config.protocol,
                            elapsed_ms,
                            content.len(),
                            max_tokens
                        ),
                    );
                }
                Err(err) => {
                    crate::logger::log_event(
                        app,
                        "WARN",
                        elapsed_ms,
                        &format!("LLM error after {}ms: {}", elapsed_ms, err),
                    );
                }
            }
            res
        }
        Err(_) => {
            let elapsed_ms = start.elapsed().as_millis() as u64;
            let msg = format!(
                "LLM request timed out after 60s at {}. Please verify that your local LLM server is responding.",
                config.base_url
            );
            crate::logger::log_event(app, "ERROR", elapsed_ms, &msg);
            Err(msg)
        }
    }
}

/// Helper to extract text from OpenAI response, supporting reasoning_content / reasoning fallbacks for thinking models
fn extract_openai_message_content(json: &serde_json::Value) -> Option<String> {
    // 1. Try standard choices[0].message.content
    if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    // 2. Fallback to reasoning_content (e.g. DeepSeek-R1 / QwQ / LM Studio thinking models)
    if let Some(reasoning) = json["choices"][0]["message"]["reasoning_content"].as_str() {
        let trimmed = reasoning.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    // 3. Fallback to reasoning
    if let Some(reasoning) = json["choices"][0]["message"]["reasoning"].as_str() {
        let trimmed = reasoning.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    // 4. Fallback to choices[0].text
    if let Some(text) = json["choices"][0]["text"].as_str() {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    None
}

/// Invokes standard OpenAI / ChatGPT compatible endpoints (LM Studio, OpenAI, vLLM, etc.)
async fn invoke_openai_compatible(
    client: &reqwest::Client,
    system_prompt: &str,
    user_prompt: &str,
    max_tokens: u32,
    config: &LlmConfig,
) -> Result<String, String> {
    let url = normalize_openai_chat_url(&config.base_url);

    // Primary payload with system role and factual 0.2 temperature
    let payload = serde_json::json!({
        "model": config.model_name,
        "temperature": 0.2,
        "max_tokens": max_tokens,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ]
    });

    let mut req = client.post(&url).json(&payload);
    if let Some(key) = &config.api_key {
        if !key.trim().is_empty() {
            req = req.bearer_auth(key.trim());
        }
    }

    let resp = req.send().await.map_err(|e| {
        format!(
            "Failed to connect to LLM at {}: {}. Please check that your local server (LM Studio / Ollama) is active.",
            url, e
        )
    })?;

    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();

        // Automatic fallback: some local models reject { "role": "system" }
        if err_text.to_lowercase().contains("system") || err_text.contains("400") {
            let fallback_payload = serde_json::json!({
                "model": config.model_name,
                "temperature": 0.2,
                "max_tokens": max_tokens,
                "messages": [
                    {
                        "role": "user",
                        "content": format!("[SYSTEM INSTRUCTIONS]\n{}\n\n[TASK]\n{}", system_prompt, user_prompt)
                    }
                ]
            });

            let mut fallback_req = client.post(&url).json(&fallback_payload);
            if let Some(key) = &config.api_key {
                if !key.trim().is_empty() {
                    fallback_req = fallback_req.bearer_auth(key.trim());
                }
            }

            if let Ok(fallback_resp) = fallback_req.send().await {
                if fallback_resp.status().is_success() {
                    if let Ok(json) = fallback_resp.json::<serde_json::Value>().await {
                        if let Some(content) = extract_openai_message_content(&json) {
                            return Ok(content);
                        }
                    }
                }
            }
        }

        return Err(format!("LLM endpoint ({}) returned status error: {}", url, err_text));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Invalid LLM JSON from {}: {}", url, e))?;
    let content = extract_openai_message_content(&json)
        .ok_or_else(|| format!("Empty content in LLM response from {}. Check if model thinking tokens exhausted max_tokens.", url))?;

    Ok(content)
}

/// Invokes Anthropic Claude format endpoints
async fn invoke_anthropic(
    client: &reqwest::Client,
    system_prompt: &str,
    user_prompt: &str,
    max_tokens: u32,
    config: &LlmConfig,
) -> Result<String, String> {
    let base = if config.base_url.is_empty() {
        "https://api.anthropic.com/v1"
    } else {
        config.base_url.trim_end_matches('/')
    };
    let url = format!("{}/messages", base);

    let payload = serde_json::json!({
        "model": config.model_name,
        "max_tokens": max_tokens,
        "temperature": 0.2,
        "system": system_prompt,
        "messages": [
            { "role": "user", "content": user_prompt }
        ]
    });

    let key = config.api_key.as_deref().unwrap_or("").trim();
    let resp = client
        .post(&url)
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Anthropic request error: {}", e))?;

    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(format!("Anthropic status error: {}", err_text));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Invalid JSON: {}", e))?;
    let content = json["content"][0]["text"]
        .as_str()
        .ok_or_else(|| "Missing text in Anthropic response".to_string())?;

    Ok(content.trim().to_string())
}

/// Invokes Ollama native API format
async fn invoke_ollama(
    client: &reqwest::Client,
    system_prompt: &str,
    user_prompt: &str,
    max_tokens: u32,
    config: &LlmConfig,
) -> Result<String, String> {
    let url = normalize_ollama_chat_url(&config.base_url);

    let payload = serde_json::json!({
        "model": config.model_name,
        "stream": false,
        "options": {
            "temperature": 0.2,
            "num_predict": max_tokens
        },
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ]
    });

    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Ollama at {}: {}", url, e))?;

    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(format!("Ollama status error from {}: {}", url, err_text));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Invalid JSON from {}: {}", url, e))?;
    let mut content = json["message"]["content"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string();

    if content.is_empty() {
        if let Some(thinking) = json["message"]["thinking"].as_str() {
            content = thinking.trim().to_string();
        }
    }

    if content.is_empty() {
        return Err(format!("Missing or empty content in Ollama response from {}", url));
    }

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_openai_chat_url() {
        assert_eq!(
            normalize_openai_chat_url("http://localhost:1234"),
            "http://localhost:1234/v1/chat/completions"
        );
        assert_eq!(
            normalize_openai_chat_url("http://localhost:1234/v1"),
            "http://localhost:1234/v1/chat/completions"
        );
        assert_eq!(
            normalize_openai_chat_url("http://localhost:1234/v1/chat/completions"),
            "http://localhost:1234/v1/chat/completions"
        );
        assert_eq!(
            normalize_openai_chat_url("https://api.openai.com/v1/"),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn test_normalize_ollama_chat_url() {
        assert_eq!(
            normalize_ollama_chat_url("http://localhost:11434"),
            "http://localhost:11434/api/chat"
        );
        assert_eq!(
            normalize_ollama_chat_url("http://localhost:11434/api/chat"),
            "http://localhost:11434/api/chat"
        );
    }
}
