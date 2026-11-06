use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: i64,
    pub query: String,
    pub mode: String,
    pub created_at: String,
    pub layer_count: usize,
    pub candidate_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullSession {
    pub summary: SessionSummary,
    pub report_markdown: String,
    pub citations: Vec<String>,
    #[serde(default)]
    pub output_json: Option<String>,
    #[serde(default)]
    pub output: Option<crate::pipeline::ResearchOutput>,
}

fn get_db_path() -> PathBuf {
    let mut dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("research-agent");
    fs::create_dir_all(&dir).ok();
    dir.push("history.db");
    dir
}

pub fn init_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            query TEXT NOT NULL,
            mode TEXT NOT NULL,
            report_markdown TEXT NOT NULL,
            citations_json TEXT NOT NULL,
            layer_count INTEGER NOT NULL,
            candidate_count INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            output_json TEXT
        )",
        [],
    )?;

    // Safe migration: add output_json column if older table exists
    let _ = conn.execute("ALTER TABLE sessions ADD COLUMN output_json TEXT", []);

    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    // Check if initial welcome chat has been seeded
    let mut stmt = conn.prepare("SELECT value FROM app_meta WHERE key = 'seeded_welcome'")?;
    let seeded: Option<String> = stmt.query_row([], |row| row.get(0)).ok();

    if seeded.is_none() {
        let welcome_query = "What is Research Agent?";
        let welcome_mode = "Instant Search";
        let welcome_report = r#"# Welcome to Research Agent v2.0
> **"Proof Over Hallucination."** — The Open-Source, Offline-First Perplexity Alternative.

Research Agent is designed to eliminate LLM overthinking and produce verified, cited answers with zero cloud leaks.

### Key Capabilities

* **🧠 Single Micro-Task Principle**: Eliminates hallucinations by isolating keyword constraints (e.g., *"this week"*, *"open-source"*, *"best"*) into atomic micro-prompts.
* **🔍 4 Investigation Modes**:
  * **Deep Search**: Recursive 5-Whys root-cause subtask tree with early stopping and 5-candidate ranking.
  * **Instant Search**: Fast 1-pass verification (2–3 seconds) with public citations.
  * **Compare Search**: Comparative matrix across competing tools or solutions.
  * **Offline Search**: 100% offline local Obsidian / Markdown vault exploration with `[[Note]]` citations.
* **⚡ Local & Cloud LLMs**: Seamless support for LM Studio, Ollama, OpenAI, Anthropic, or custom REST endpoints with live latency testing.
* **🔒 Strict SSRF Protection**: All outbound requests strictly block local networks, metadata endpoints, and private IP ranges.

### Quick Shortcuts & Controls
* Press **Shift + Enter** inside the query bar to immediately trigger an investigation.
* Click **[📥 Export .md]** at the bottom of any report to save it as Markdown.
* Click **[📋 Copy]** to copy the full formatted text to your clipboard.
* Hover over any search in the left sidebar and click **✕** to delete that chat.
"#;
        let welcome_citations = vec![
            "https://github.com/PavanKumarVattikuti/research-agent".to_string(),
        ];
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        let citations_json = serde_json::to_string(&welcome_citations).unwrap_or_else(|_| "[]".into());

        conn.execute(
            "INSERT INTO sessions (query, mode, report_markdown, citations_json, layer_count, candidate_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                welcome_query,
                welcome_mode,
                welcome_report,
                citations_json,
                1i64,
                1i64,
                now
            ],
        )?;

        conn.execute(
            "INSERT OR REPLACE INTO app_meta (key, value) VALUES ('seeded_welcome', '1')",
            [],
        )?;
    }

    Ok(())
}

pub fn init_db() -> Result<Connection> {
    let path = get_db_path();
    let conn = Connection::open(path)?;
    init_tables(&conn)?;
    Ok(conn)
}

pub fn save_session(
    query: &str,
    mode: &str,
    report_markdown: &str,
    citations: &[String],
    layer_count: usize,
    candidate_count: usize,
    output_json: Option<&str>,
) -> Result<i64> {
    let conn = init_db()?;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let citations_json = serde_json::to_string(citations).unwrap_or_else(|_| "[]".into());

    conn.execute(
        "INSERT INTO sessions (query, mode, report_markdown, citations_json, layer_count, candidate_count, created_at, output_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            query,
            mode,
            report_markdown,
            citations_json,
            layer_count as i64,
            candidate_count as i64,
            now,
            output_json
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn delete_session(session_id: i64) -> Result<()> {
    let conn = init_db()?;
    conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])?;
    Ok(())
}

pub fn clear_all_sessions() -> Result<()> {
    let conn = init_db()?;
    conn.execute("DELETE FROM sessions", [])?;
    Ok(())
}

pub fn get_recent_sessions() -> Result<Vec<SessionSummary>> {
    let conn = init_db()?;
    let mut stmt = conn.prepare(
        "SELECT id, query, mode, layer_count, candidate_count, created_at
         FROM sessions ORDER BY id DESC LIMIT 25",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(SessionSummary {
            id: row.get(0)?,
            query: row.get(1)?,
            mode: row.get(2)?,
            layer_count: row.get::<_, i64>(3)? as usize,
            candidate_count: row.get::<_, i64>(4)? as usize,
            created_at: row.get(5)?,
        })
    })?;

    let mut sessions = Vec::new();
    for r in rows.flatten() {
        sessions.push(r);
    }
    Ok(sessions)
}

pub fn get_session_details(session_id: i64) -> Result<Option<FullSession>> {
    let conn = init_db()?;
    let mut stmt = conn.prepare(
        "SELECT id, query, mode, report_markdown, citations_json, layer_count, candidate_count, created_at, output_json
         FROM sessions WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![session_id])?;
    if let Some(row) = rows.next()? {
        let citations_str: String = row.get(4)?;
        let citations: Vec<String> = serde_json::from_str(&citations_str).unwrap_or_default();
        let output_json_str: Option<String> = row.get(8).ok();
        let output: Option<crate::pipeline::ResearchOutput> = output_json_str
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());

        let summary = SessionSummary {
            id: row.get(0)?,
            query: row.get(1)?,
            mode: row.get(2)?,
            layer_count: row.get::<_, i64>(5)? as usize,
            candidate_count: row.get::<_, i64>(6)? as usize,
            created_at: row.get(7)?,
        };

        let full = FullSession {
            summary,
            report_markdown: row.get(3)?,
            citations,
            output_json: output_json_str,
            output,
        };
        Ok(Some(full))
    } else {
        Ok(None)
    }
}

mod dirs {
    use std::path::PathBuf;
    pub fn data_local_dir() -> Option<PathBuf> {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_history_crud() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();

        // Verify seeded welcome chat is present on clean DB
        let mut stmt = conn.prepare("SELECT query FROM sessions").unwrap();
        let query: String = stmt.query_row([], |r| r.get(0)).unwrap();
        assert_eq!(query, "What is Research Agent?");

        // Insert new session
        conn.execute(
            "INSERT INTO sessions (query, mode, report_markdown, citations_json, layer_count, candidate_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                "Test query",
                "Deep Search",
                "Test report",
                "[\"https://example.com\"]",
                1i64,
                1i64,
                "2026-11-03 12:00"
            ],
        ).unwrap();

        let new_id = conn.last_insert_rowid();
        assert!(new_id > 1);

        // Delete session
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![new_id]).unwrap();

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM sessions WHERE id = ?1", params![new_id], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_history_persists_and_restores_complete_output_json() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();

        let sample_output = crate::pipeline::ResearchOutput {
            query: "Top 10 upcoming games in 2028".into(),
            mode: "Deep Search".into(),
            constraints: vec![],
            layers: vec![],
            candidates: vec![],
            final_report: "## Executive Summary\nComplete report text".into(),
            citations: vec!["https://example.com/2028".into()],
            stopped_early: true,
            template_hint: "ranked_list".into(),
            structured_data: Some(crate::pipeline::StructuredResearchData {
                executive_summary: "Confirmed roadmaps include Witcher 4".into(),
                requested_count: 10,
                verified_count: 2,
                uncertain_count: 1,
                entities: vec![],
                comparison_matrix: vec![],
                limitations: vec![],
                source_digests: vec![
                    crate::pipeline::SourceDigest {
                        source_id: 1,
                        title: "Game Tracker".into(),
                        url: "https://example.com/2028".into(),
                        summary: "Detailed 2028 list".into(),
                        entities: vec!["The Witcher 4".into()],
                        relevance: "high".into(),
                    }
                ],
            }),
        };

        let output_json = serde_json::to_string(&sample_output).unwrap();
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();

        conn.execute(
            "INSERT INTO sessions (query, mode, report_markdown, citations_json, layer_count, candidate_count, created_at, output_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                sample_output.query,
                sample_output.mode,
                sample_output.final_report,
                "[\"https://example.com/2028\"]",
                1i64,
                1i64,
                now,
                output_json
            ],
        ).unwrap();

        let inserted_id = conn.last_insert_rowid();

        // Query session back with output_json deserialization
        let mut stmt = conn.prepare(
            "SELECT id, query, mode, report_markdown, citations_json, layer_count, candidate_count, created_at, output_json
             FROM sessions WHERE id = ?1",
        ).unwrap();

        let mut rows = stmt.query(params![inserted_id]).unwrap();
        let row = rows.next().unwrap().unwrap();
        let raw_json: Option<String> = row.get(8).ok();
        assert!(raw_json.is_some());

        let restored_output: crate::pipeline::ResearchOutput = serde_json::from_str(&raw_json.unwrap()).unwrap();
        assert_eq!(restored_output.query, "Top 10 upcoming games in 2028");
        assert!(restored_output.structured_data.is_some());
        let sd = restored_output.structured_data.unwrap();
        assert_eq!(sd.requested_count, 10);
        assert_eq!(sd.source_digests.len(), 1);
        assert_eq!(sd.source_digests[0].title, "Game Tracker");
    }
}
