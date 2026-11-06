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
    #[serde(default)]
    pub wikilinks: Vec<String>,
    #[serde(default)]
    pub backlinks: Vec<String>,
}

/// Extracts Obsidian [[wikilinks]] from markdown content
pub fn extract_wikilinks(content: &str) -> Vec<String> {
    let re = match regex::Regex::new(r"\[\[([^\]\|#]+)(?:[\|#][^\]]*)?\]\]") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut links = Vec::new();
    for cap in re.captures_iter(content) {
        let link_target = cap[1].trim().to_string();
        if !link_target.is_empty() && !links.contains(&link_target) {
            links.push(link_target);
        }
    }
    links
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

    struct ScannedNote {
        stem: String,
        file_name: String,
        relative: String,
        full_path: String,
        excerpt: String,
        score: usize,
        wikilinks: Vec<String>,
    }

    let mut scanned_notes: Vec<ScannedNote> = Vec::new();

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

            let file_name = entry_path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("Note")
                .to_string();

            let stem = entry_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&file_name)
                .to_string();

            let relative = entry_path
                .strip_prefix(path)
                .unwrap_or(entry_path)
                .to_string_lossy()
                .to_string();

            let excerpt = if score > 0 {
                extract_best_paragraph(&content, &query_terms)
            } else {
                String::new()
            };

            let wikilinks = extract_wikilinks(&content);

            scanned_notes.push(ScannedNote {
                stem,
                file_name,
                relative,
                full_path: entry_path.to_string_lossy().to_string(),
                excerpt,
                score,
                wikilinks,
            });
        }
    }

    // Filter to notes matching query terms and compute backlinks from all scanned notes
    let mut matches = Vec::new();

    for note in &scanned_notes {
        if note.score == 0 {
            continue;
        }

        let note_stem_lower = note.stem.to_lowercase();
        let mut backlinks = Vec::new();

        for other in &scanned_notes {
            if other.full_path == note.full_path {
                continue;
            }
            let links_to_note = other.wikilinks.iter().any(|link| {
                link.to_lowercase() == note_stem_lower
            });
            if links_to_note && !backlinks.contains(&other.stem) {
                backlinks.push(other.stem.clone());
            }
        }

        matches.push(VaultNoteMatch {
            file_name: note.file_name.clone(),
            relative_path: note.relative.clone(),
            full_path: note.full_path.clone(),
            matched_excerpt: note.excerpt.clone(),
            score: note.score,
            wikilinks: note.wikilinks.clone(),
            backlinks,
        });
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

/// Exports research output directly into an Obsidian vault under a "Research Agent" folder
pub fn export_to_obsidian_vault(
    vault_dir: &str,
    query: &str,
    mode: &str,
    markdown_content: &str,
    citations: &[String],
) -> Result<String, String> {
    let vault_path = Path::new(vault_dir.trim());
    if !vault_path.exists() || !vault_path.is_dir() {
        return Err(format!("Vault directory does not exist or is not a folder: {}", vault_dir));
    }

    let canonical_vault = vault_path.canonicalize().map_err(|e| format!("Invalid vault path: {}", e))?;
    let canonical_str = canonical_vault.to_string_lossy();
    if canonical_str == "/" || canonical_str == "C:\\" || canonical_str.to_lowercase().starts_with("c:\\windows") {
        return Err("Export to system root or OS directories is forbidden for security.".into());
    }

    let target_folder = vault_path.join("Research Agent");
    fs::create_dir_all(&target_folder).map_err(|e| format!("Failed to create folder: {}", e))?;

    let sanitized_title: String = query
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == ' ' { c } else { '_' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .to_lowercase();
    let safe_name = if sanitized_title.is_empty() { "research-note".to_string() } else { sanitized_title };
    let file_stem = safe_name.chars().take(40).collect::<String>();
    let file_name = format!("{}.md", file_stem);
    let file_path = target_folder.join(&file_name);

    let date_str = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();

    let mut doc = String::new();
    doc.push_str("---\n");
    doc.push_str(&format!("title: \"{}\"\n", query.replace('"', "\"")));
    doc.push_str(&format!("date: {}\n", date_str));
    doc.push_str(&format!("mode: {}\n", mode));
    doc.push_str("tags: [research, deep-search, proof-over-hallucination]\n");
    doc.push_str(&format!("sources_count: {}\n", citations.len()));
    doc.push_str("---\n\n");

    doc.push_str(markdown_content);

    if !citations.is_empty() {
        doc.push_str("\n\n## Cited Sources\n\n");
        for (i, c) in citations.iter().enumerate() {
            doc.push_str(&format!("{}. [{}]({})\n", i + 1, c, c));
        }
    }

    fs::write(&file_path, doc).map_err(|e| format!("Failed to write note: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
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
        writeln!(f1, "# Rust Concurrency\n\nRust provides memory safety without a garbage collector through ownership and borrowing rules.\n\nSee also [[Memory Management]] and [[Type Safety|Types]].\n\nConcurrency in Rust uses thread pools and channels.").unwrap();

        let note2 = temp_dir.join("python_notes.md");
        let mut f2 = File::create(&note2).unwrap();
        writeln!(f2, "# Python GIL\n\nPython uses a Global Interpreter Lock for memory management. Unlike [[rust_notes]], Python has a runtime.").unwrap();

        let matches = search_markdown_vault(temp_dir.to_str().unwrap(), "rust ownership memory");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].file_name, "rust_notes.md");
        assert!(matches[0].matched_excerpt.contains("ownership"));
        assert!(matches[0].wikilinks.contains(&"Memory Management".to_string()));
        assert!(matches[0].backlinks.contains(&"python_notes".to_string()));

        // Clean up
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_export_to_obsidian_vault() {
        let temp_dir = std::env::temp_dir().join("research_agent_export_test");
        fs::create_dir_all(&temp_dir).unwrap();

        let query = "Frontier AI Models 2026";
        let mode = "Deep Search";
        let content = "## Executive Summary\nVerified frontier models.";
        let citations = vec!["https://example.com/ai".to_string()];

        let result = export_to_obsidian_vault(
            temp_dir.to_str().unwrap(),
            query,
            mode,
            content,
            &citations,
        );

        assert!(result.is_ok());
        let saved_path = result.unwrap();
        assert!(saved_path.contains("Research Agent"));
        assert!(saved_path.ends_with(".md"));

        let saved_content = fs::read_to_string(&saved_path).unwrap();
        assert!(saved_content.contains("title: \"Frontier AI Models 2026\""));
        assert!(saved_content.contains("tags: [research, deep-search, proof-over-hallucination]"));
        assert!(saved_content.contains("https://example.com/ai"));

        // Clean up
        fs::remove_dir_all(&temp_dir).ok();
    }
}
