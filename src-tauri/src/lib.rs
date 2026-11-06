pub mod history;
pub mod llm;
pub mod logger;
pub mod pipeline;
pub mod search;
pub mod ssrf;
pub mod tester;
pub mod vault;

use history::{get_recent_sessions, get_session_details, FullSession, SessionSummary};
use pipeline::{run_research_pipeline, PipelineConfig, ResearchOutput};
use tester::{test_llm_endpoint, test_search_endpoint, TestResult};

/// Validates raw user query for security: length constraints and disallowed control characters
pub fn validate_query(raw_query: &str) -> Result<String, String> {
    let trimmed = raw_query.trim();
    if trimmed.len() < 2 {
        return Err("Query too short: minimum 2 characters required.".into());
    }
    if trimmed.len() > 500 {
        return Err("Query too long: maximum 500 characters allowed.".into());
    }
    if trimmed.chars().any(|c| (c as u32) < 32 && c != '\t' && c != '\n' && c != '\r') {
        return Err("Query contains disallowed control characters.".into());
    }
    Ok(trimmed.to_string())
}

/// Sanitizes export filenames against directory traversal and whitelists file extensions
pub fn sanitize_export_filename(raw_filename: &str) -> Result<String, String> {
    let trimmed = raw_filename.trim();
    if trimmed.is_empty() {
        return Err("Filename cannot be empty.".into());
    }
    if trimmed.len() > 120 {
        return Err("Filename exceeds maximum length of 120 characters.".into());
    }
    // Path traversal guard: strictly reject path separators or parent directory traversal
    if trimmed.contains("..") || trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains(':') || trimmed.contains('\0') {
        return Err("Filename contains illegal path traversal characters ('/', '\\', '..', ':').".into());
    }
    // Extension whitelist: must be .md, .html, or .txt
    let lower = trimmed.to_lowercase();
    if !lower.ends_with(".md") && !lower.ends_with(".html") && !lower.ends_with(".txt") {
        return Err("Disallowed file format: only .md, .html, or .txt extensions are permitted.".into());
    }
    // Character whitelist: alphanumeric, dash, underscore, space, dot
    if !trimmed.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ' ') {
        return Err("Filename contains illegal characters. Only alphanumeric, dashes, underscores, and dots are permitted.".into());
    }
    Ok(trimmed.to_string())
}

#[tauri::command]
async fn test_llm(protocol: String, base_url: String, api_key: Option<String>) -> TestResult {
    let proto_clean = protocol.trim().to_lowercase();
    if proto_clean != "openai" && proto_clean != "anthropic" && proto_clean != "ollama" && proto_clean != "custom" {
        return TestResult {
            live: false,
            latency_ms: 0,
            message: format!("Unsupported LLM protocol: {}", protocol),
            detected_models: vec![],
        };
    }
    test_llm_endpoint(&proto_clean, &base_url, api_key.as_deref()).await
}

#[tauri::command]
async fn test_search(engine: String, url_or_key: String) -> TestResult {
    let eng_clean = engine.trim().to_lowercase();
    let allowed = ["searxng", "google news", "brave", "brave search", "duckduckgo", "arxiv", "hacker news"];
    if !allowed.iter().any(|a| *a == eng_clean) {
        return TestResult {
            live: false,
            latency_ms: 0,
            message: format!("Unsupported search engine: {}", engine),
            detected_models: vec![],
        };
    }
    test_search_endpoint(&eng_clean, &url_or_key).await
}

#[tauri::command]
async fn execute_research(
    app: tauri::AppHandle,
    query: String,
    mut config: PipelineConfig,
) -> Result<ResearchOutput, String> {
    let clean_query = validate_query(&query)?;

    // Validate mode
    let mode_lower = config.mode.to_lowercase();
    if !mode_lower.contains("deep")
        && !mode_lower.contains("instant")
        && !mode_lower.contains("compare")
        && !mode_lower.contains("offline")
    {
        config.mode = "Deep Search".to_string();
    }

    // Validate protocol
    let proto_lower = config.llm.protocol.to_lowercase();
    if proto_lower != "openai" && proto_lower != "anthropic" && proto_lower != "ollama" && proto_lower != "custom" {
        config.llm.protocol = "openai".to_string();
    }

    run_research_pipeline(Some(&app), &clean_query, &config).await
}

#[tauri::command]
fn get_history() -> Result<Vec<SessionSummary>, String> {
    get_recent_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_history_detail(session_id: i64) -> Result<Option<FullSession>, String> {
    get_session_details(session_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_history_item(session_id: i64) -> Result<(), String> {
    history::delete_session(session_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_history() -> Result<(), String> {
    history::clear_all_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
fn count_vault_notes(vault_path: String) -> usize {
    let p = std::path::Path::new(vault_path.trim());
    if !p.exists() || !p.is_dir() {
        return 0;
    }
    walkdir::WalkDir::new(p)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "md" || ext == "markdown")
                .unwrap_or(false)
            })
        .count()
}

#[tauri::command]
fn save_markdown_export(filename: String, content: String) -> Result<String, String> {
    let clean_name = sanitize_export_filename(&filename)?;

    // Limit content size to max 15MB
    if content.len() > 15 * 1024 * 1024 {
        return Err("Content size exceeds 15MB limit.".into());
    }

    let base_dir = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(|p| std::path::PathBuf::from(p).join("Downloads"))
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    if !base_dir.exists() {
        std::fs::create_dir_all(&base_dir).map_err(|e| e.to_string())?;
    }

    let target_path = base_dir.join(&clean_name);
    std::fs::write(&target_path, content.as_bytes()).map_err(|e| format!("Failed to write export file: {}", e))?;

    Ok(target_path.to_string_lossy().to_string())
}

#[tauri::command]
fn get_pipeline_logs() -> Vec<logger::LogEntry> {
    logger::get_all_logs()
}

#[tauri::command]
fn clear_pipeline_logs() {
    logger::clear_logs();
}

#[tauri::command]
fn save_logs_export() -> Result<String, String> {
    let logs = logger::get_all_logs();
    let mut text = String::from("# Research Agent Pipeline Execution Logs\n\n");
    for l in logs {
        text.push_str(&format!(
            "[{}] [{}] (+{}ms) {}\n",
            l.timestamp, l.level, l.elapsed_ms, l.message
        ));
    }
    let filename = format!(
        "research-agent-logs-{}.txt",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );
    save_markdown_export(filename, text)
}

#[tauri::command]
fn save_vault_export(
    vault_path: String,
    query: String,
    mode: String,
    content: String,
    citations: Vec<String>,
) -> Result<String, String> {
    vault::export_to_obsidian_vault(&vault_path, &query, &mode, &content, &citations)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            test_llm,
            test_search,
            execute_research,
            get_history,
            get_history_detail,
            delete_history_item,
            clear_history,
            count_vault_notes,
            save_markdown_export,
            get_pipeline_logs,
            clear_pipeline_logs,
            save_logs_export,
            save_vault_export,
        ])
        .run(tauri::generate_context!())
        .expect("error while running research agent application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_query_constraints() {
        assert!(validate_query("a").is_err(), "Too short must fail");
        assert!(validate_query("  ").is_err(), "Empty whitespace must fail");
        assert!(validate_query(&"x".repeat(501)).is_err(), "Too long must fail");
        assert!(validate_query("Valid Query 2028").is_ok(), "Normal query must succeed");
        assert!(validate_query("Query with \0 null").is_err(), "Control chars must fail");
    }

    #[test]
    fn test_sanitize_export_filename_path_traversal() {
        // Path traversal must fail
        assert!(sanitize_export_filename("../evil.md").is_err());
        assert!(sanitize_export_filename("..\\evil.md").is_err());
        assert!(sanitize_export_filename("/etc/passwd.txt").is_err());
        assert!(sanitize_export_filename("C:\\Windows\\System32\\test.md").is_err());
        assert!(sanitize_export_filename("evil.exe").is_err());
        assert!(sanitize_export_filename("report.sh").is_err());

        // Valid extensions must succeed
        assert!(sanitize_export_filename("research-report-2028.md").is_ok());
        assert!(sanitize_export_filename("research-report-2028.html").is_ok());
        assert!(sanitize_export_filename("logs-20261104.txt").is_ok());
    }
}
