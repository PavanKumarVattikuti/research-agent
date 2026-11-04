use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultNoteMatch {
    pub file_name: String,
    pub relative_path: String,
    pub full_path: String,
    pub matched_excerpt: String,
    pub score: usize,
}

/// Recursively scans an Obsidian / Markdown folder and searches for matches against the query.
pub fn search_markdown_vault(vault_dir: &str, query: &str) -> Vec<VaultNoteMatch> {
    let path = Path::new(vault_dir);
    if !path.exists() || !path.is_dir() {
        return vec![];
    }

    let query_terms: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .filter(|t| t.len() > 2)
        .map(|s| s.to_string())
        .collect();

    if query_terms.is_empty() {
        return vec![];
    }

    let mut matches = Vec::new();

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }

        let ext = entry_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext != "md" && ext != "markdown" {
            continue;
        }

        if let Ok(content) = fs::read_to_string(entry_path) {
            let lower_content = content.to_lowercase();
            let mut score = 0;

            for term in &query_terms {
                score += lower_content.matches(term).count();
            }

            if score > 0 {
                let file_name = entry_path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("Note")
                    .to_string();

                let relative = entry_path
                    .strip_prefix(path)
                    .unwrap_or(entry_path)
                    .to_string_lossy()
                    .to_string();

                // Find the best snippet containing matching terms
                let excerpt = extract_best_paragraph(&content, &query_terms);

                matches.push(VaultNoteMatch {
                    file_name,
                    relative_path: relative,
                    full_path: entry_path.to_string_lossy().to_string(),
                    matched_excerpt: excerpt,
                    score,
                });
            }
        }
    }

    // Sort by relevance score descending
    matches.sort_by(|a, b| b.score.cmp(&a.score));
    matches.into_iter().take(5).collect()
}

fn extract_best_paragraph(content: &str, terms: &[String]) -> String {
    let paragraphs: Vec<&str> = content
        .split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();

    let mut best_p = "";
    let mut best_matches = 0;

    for p in &paragraphs {
        let lower = p.to_lowercase();
        let count = terms.iter().filter(|t| lower.contains(t.as_str())).count();
        if count > best_matches {
            best_matches = count;
            best_p = p;
        }
    }

    if best_p.is_empty() && !paragraphs.is_empty() {
        best_p = paragraphs[0];
    }

    let max_len = 350;
    if best_p.len() > max_len {
        format!("{}...", &best_p[..max_len])
    } else {
        best_p.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_vault_search() {
        let temp_dir = std::env::temp_dir().join("research_agent_vault_test");
        fs::create_dir_all(&temp_dir).unwrap();

        let note1 = temp_dir.join("rust_notes.md");
        let mut f1 = File::create(&note1).unwrap();
        writeln!(f1, "# Rust Concurrency\n\nRust provides memory safety without a garbage collector through ownership and borrowing rules.\n\nConcurrency in Rust uses thread pools and channels.").unwrap();

        let note2 = temp_dir.join("python_notes.md");
        let mut f2 = File::create(&note2).unwrap();
        writeln!(f2, "# Python GIL\n\nPython uses a Global Interpreter Lock for memory management.").unwrap();

        let matches = search_markdown_vault(temp_dir.to_str().unwrap(), "rust ownership memory");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].file_name, "rust_notes.md");
        assert!(matches[0].matched_excerpt.contains("ownership"));

        // Clean up
        fs::remove_dir_all(&temp_dir).ok();
    }
}
