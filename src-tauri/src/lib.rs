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

#[tauri::command]
async fn test_llm(protocol: String, base_url: String, api_key: Option<String>) -> TestResult {
    test_llm_endpoint(&protocol, &base_url, api_key.as_deref()).await
}

#[tauri::command]
async fn test_search(engine: String, url_or_key: String) -> TestResult {
    test_search_endpoint(&engine, &url_or_key).await
}

#[tauri::command]
async fn execute_research(
    app: tauri::AppHandle,
    query: String,
    config: PipelineConfig,
) -> Result<ResearchOutput, String> {
    run_research_pipeline(Some(&app), &query, &config).await
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
    let p = std::path::Path::new(&vault_path);
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
    let base_dir = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(|p| std::path::PathBuf::from(p).join("Downloads"))
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    if !base_dir.exists() {
        std::fs::create_dir_all(&base_dir).map_err(|e| e.to_string())?;
    }

    let clean_name = if filename.ends_with(".md") || filename.ends_with(".html") || filename.ends_with(".txt") {
        filename
    } else {
        format!("{}.md", filename)
    };

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running research agent application");
}
