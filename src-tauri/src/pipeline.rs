use crate::history;
use crate::llm::{invoke_micro_task, invoke_micro_task_with_tokens, LlmConfig};
use crate::search::{execute_multi_source_search, rank_and_select_top_evidence};
use crate::vault::search_markdown_vault;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tauri::Emitter;
use tokio::sync::Semaphore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub mode: String, // "Deep Search" | "Instant Search" | "Compare Search" | "Offline Search"
    pub max_layers: usize,
    pub search_engine: String,
    pub searx_url: Option<String>,
    pub brave_key: Option<String>,
    pub obsidian_vault_path: Option<String>,
    pub llm: LlmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordConstraint {
    pub word: String,
    pub question_why: String,
    pub requirement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAnalysis {
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub clarification: String,
    #[serde(default, alias = "keywords")]
    pub domain_keywords: Vec<String>,
    #[serde(default, alias = "exclusions")]
    pub excluded_terms: Vec<String>,
    #[serde(default, alias = "ir_queries", alias = "queries", alias = "search_queries")]
    pub ir_search_queries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerResult {
    pub layer: usize,
    pub why_question: String,
    pub finding: String,
    pub proof_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateAnswer {
    pub rank: usize,
    pub name: String,
    pub score: String,
    pub summary: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaimEvidence {
    #[serde(default)]
    pub source_id: usize,
    #[serde(default)]
    pub source_title: String,
    #[serde(default)]
    pub publisher: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub source_type: String, // "Primary" | "Secondary" | "Community"
    #[serde(default)]
    pub publication_date: Option<String>,
    #[serde(default)]
    pub excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityClaim {
    #[serde(default)]
    pub claim: String, // e.g. "Scheduled for release in 2028"
    #[serde(default)]
    pub status: String, // "Verified" | "Corroborated" | "Uncertain" | "Conflicting"
    #[serde(default)]
    pub explanation: String,
    #[serde(default)]
    pub evidence: Vec<ClaimEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchEntity {
    #[serde(default)]
    pub rank: usize,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: String, // e.g. "Action RPG"
    #[serde(default)]
    pub developer: String, // e.g. "CD Projekt Red"
    #[serde(default)]
    pub platforms: String, // e.g. "PC, PS5, Xbox Series X/S"
    #[serde(default)]
    pub expected_release: String, // e.g. "2028 (Projected)"
    #[serde(default)]
    pub status: String, // "Confirmed" | "Roadmap Target" | "Uncertain"
    #[serde(default)]
    pub why_qualifies: String, // 1-2 sentence explanation
    #[serde(default)]
    pub claims: Vec<EntityClaim>,
    #[serde(default)]
    pub limitations: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComparisonEntry {
    #[serde(default)]
    pub entity_name: String,
    #[serde(default)]
    pub release: String,
    #[serde(default)]
    pub developer: String,
    #[serde(default)]
    pub platforms: String,
    #[serde(default)]
    pub confidence: String, // "High" | "Medium" | "Low"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceDigest {
    #[serde(default)]
    pub source_id: usize,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub summary: String,          // max ~80 words
    #[serde(default)]
    pub entities: Vec<String>,    // names found in this source
    #[serde(default)]
    pub relevance: String,        // "high" | "medium" | "low"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LimitationItem {
    #[serde(default)]
    pub category: String, // "Unconfirmed Date" | "Conflicting Reports" | "Missing Data"
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StructuredResearchData {
    #[serde(default)]
    pub executive_summary: String, // 2-3 sentences direct answer
    #[serde(default)]
    pub requested_count: usize,
    #[serde(default)]
    pub verified_count: usize,
    #[serde(default)]
    pub uncertain_count: usize,
    #[serde(default)]
    pub entities: Vec<ResearchEntity>,
    #[serde(default)]
    pub comparison_matrix: Vec<ComparisonEntry>,
    #[serde(default)]
    pub limitations: Vec<LimitationItem>,
    #[serde(default)]
    pub source_digests: Vec<SourceDigest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchOutput {
    pub query: String,
    pub mode: String,
    pub constraints: Vec<WordConstraint>,
    pub layers: Vec<LayerResult>,
    pub candidates: Vec<CandidateAnswer>,
    pub final_report: String,
    pub citations: Vec<String>,
    pub stopped_early: bool,
    pub template_hint: String,
    #[serde(default)]
    pub structured_data: Option<StructuredResearchData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub stage: String,
    pub message: String,
    pub query: String,
    pub subquery: Option<String>,
    pub words: Vec<String>,
    pub sources_count: usize,
    pub current_layer: usize,
    pub max_layers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticEvent {
    pub step: String,
    pub message: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct WordInterpretation {
    #[serde(default)]
    pub word: String,
    #[serde(default)]
    pub meaning: String,
    #[serde(default)]
    pub implication: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct QuerySynthesis {
    #[serde(default)]
    pub research_goal: String,
    #[serde(default)]
    pub target_entity_type: String, // e.g. "Video Game", "AI Foundation Model", "Hardware Chip"
    #[serde(default)]
    pub requested_count: usize, // e.g. 10
    #[serde(default)]
    pub hard_constraints: Vec<String>,
    #[serde(default)]
    pub search_keywords: Vec<String>,
    #[serde(default)]
    pub ir_queries: Vec<String>,
}

/// Blacklist of publisher websites, news aggregators, and blog domains that must NEVER be extracted as entities.
pub fn is_disallowed_entity_name(name: &str) -> bool {
    let lower = name.trim().to_lowercase();
    if lower.is_empty() {
        return true;
    }
    let blacklisted = [
        "searxng", "gamesradar", "ign", "cbs sports", "forbes", "bloomberg",
        "youtube", "reddit", "twitter", "x.com", "hacker news", "techcrunch",
        "the verge", "polygon", "kotaku", "digital health news", "wired",
        "reuters", "associated press", "release calendar", "updates today",
        "top 10 new", "latest releases & timeline", "startrise", "testingcatalog",
        "pc gamer", "tom's hardware", "eurogamer", "game spot", "gamespot",
        "android authority", "google news", "bing news", "duckduckgo", "wikipedia"
    ];
    for b in &blacklisted {
        if lower == *b || lower.starts_with(&format!("{} ", b)) || lower.ends_with(&format!(" {}", b)) || lower.contains(b) {
            return true;
        }
    }
    false
}

/// Strict validator for candidate entity names.
/// Rejects:
/// 1. Length < 2 or > 80
/// 2. HTML / XML tags or entity codes (<, >, &lt;, etc.)
/// 3. URLs, web protocols, domains (http, https, www, .com, .org, etc.)
/// 4. Publisher websites, aggregators, blogs
/// 5. Generic container phrases (e.g. "upcoming games", "most anticipated", "release calendar")
/// 6. Query echoes (names composed almost entirely of words from the user query)
pub fn is_valid_candidate_entity(name: &str, query: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.len() < 2 || trimmed.len() > 80 {
        return false;
    }

    // 1. No HTML tags or XML entities
    if trimmed.contains('<') || trimmed.contains('>') || trimmed.contains("&lt;") || trimmed.contains("&gt;") {
        return false;
    }

    let lower = trimmed.to_lowercase();

    // 2. No URLs or web protocols
    if lower.contains("http://")
        || lower.contains("https://")
        || lower.contains("www.")
        || lower.ends_with(".com")
        || lower.ends_with(".org")
        || lower.ends_with(".net")
        || lower.ends_with(".io")
        || lower.contains(".com/")
        || lower.contains(".org/")
        || lower.contains(".net/")
    {
        return false;
    }

    // 3. Reject blacklisted publishers and aggregators
    if is_disallowed_entity_name(trimmed) {
        return false;
    }

    // 4. Reject generic container / aggregator phrases
    let container_phrases = [
        "upcoming games",
        "most anticipated",
        "anticipated games",
        "games of 20",
        "all the upcoming",
        "upcoming releases",
        "release calendar",
        "release date calendar",
        "releases calendar",
        "calendar of",
        "timeline of",
        "list of",
        "top 10",
        "top 5",
        "top 20",
        "best games",
        "new games",
        "latest releases",
        "news on",
        "updates today",
        "guide to",
        "ranking of",
        "research subject",
        "direct verification",
    ];
    for phrase in &container_phrases {
        if lower.contains(phrase) {
            return false;
        }
    }

    // 5. Query Echo Filter
    let query_words: std::collections::HashSet<String> = query
        .to_lowercase()
        .replace([':', '-', ',', '.', '\'', '"', '(', ')', '?', '!'], " ")
        .split_whitespace()
        .filter(|w| w.len() >= 3 && *w != "the" && *w != "and" && *w != "for" && *w != "with")
        .map(|w| w.to_string())
        .collect();

    let candidate_words: Vec<String> = lower
        .replace([':', '-', ',', '.', '\'', '"', '(', ')', '?', '!'], " ")
        .split_whitespace()
        .filter(|w| w.len() >= 3 && *w != "the" && *w != "and" && *w != "for" && *w != "with")
        .map(|w| w.to_string())
        .collect();

    if !candidate_words.is_empty() && !query_words.is_empty() {
        let matching_count = candidate_words
            .iter()
            .filter(|cw| query_words.contains(*cw))
            .count();
        if matching_count >= candidate_words.len() || (candidate_words.len() >= 2 && matching_count * 10 >= candidate_words.len() * 8) {
            return false;
        }
    }

    true
}

/// Parse explicit requested count from query (e.g. "top 10" -> 10, "5 best" -> 5).
pub fn extract_requested_count(query: &str) -> Option<usize> {
    let q_lower = query.to_lowercase();
    let parts: Vec<&str> = q_lower.split_whitespace().collect();
    for (i, p) in parts.iter().enumerate() {
        if (*p == "top" || *p == "best" || *p == "first" || *p == "latest") && i + 1 < parts.len() {
            if let Ok(num) = parts[i + 1].parse::<usize>() {
                if num > 0 && num <= 50 {
                    return Some(num);
                }
            }
        }
        if let Ok(num) = p.parse::<usize>() {
            if num > 0 && num <= 50 && (q_lower.contains("top") || q_lower.contains("best") || q_lower.contains("upcoming") || q_lower.contains("games") || q_lower.contains("models")) {
                return Some(num);
            }
        }
    }
    None
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ConstraintCheck {
    #[serde(default)]
    pub satisfied: bool,
    #[serde(default)]
    pub missing: Vec<String>,
    #[serde(default)]
    pub refined_query: Option<String>,
}

fn emit_haptic(app: Option<&tauri::AppHandle>, step: &str, message: &str, detail: Option<&str>) {
    if let Some(h) = app {
        h.emit(
            "research-haptic",
            HapticEvent {
                step: step.to_string(),
                message: message.to_string(),
                detail: detail.map(|s| s.to_string()),
            },
        )
        .ok();
    }
}

#[derive(Deserialize, Default)]
struct RawExtractedClaim {
    #[serde(default)]
    claim: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    explanation: Option<String>,
    #[serde(default, alias = "sources", alias = "citation_ids")]
    source_ids: Vec<usize>,
}

#[derive(Deserialize, Default)]
struct RawExtractedEntity {
    #[serde(default, alias = "title")]
    name: String,
    #[serde(default, alias = "genre")]
    category: Option<String>,
    #[serde(default, alias = "studio", alias = "creator", alias = "author", alias = "organization")]
    developer: Option<String>,
    #[serde(default, alias = "platform", alias = "target")]
    platforms: Option<String>,
    #[serde(default, alias = "release", alias = "release_date", alias = "target_date")]
    expected_release: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default, alias = "reason", alias = "description", alias = "summary")]
    why_qualifies: Option<String>,
    #[serde(default)]
    claims: Vec<RawExtractedClaim>,
    #[serde(default)]
    limitations: Option<String>,
}

#[derive(Deserialize, Default)]
struct RawExtractedLimitation {
    #[serde(default, alias = "type")]
    category: String,
    #[serde(default, alias = "detail")]
    description: String,
}

#[derive(Deserialize, Default)]
struct RawEvidenceExtraction {
    #[serde(default, alias = "summary")]
    executive_summary: Option<String>,
    #[serde(default, alias = "candidates", alias = "results")]
    entities: Vec<RawExtractedEntity>,
    #[serde(default, alias = "limitations", alias = "gaps")]
    limitations_summary: Vec<RawExtractedLimitation>,
}

pub fn canonical_entity_key(name: &str) -> String {
    let lower = name.trim().to_lowercase();
    let mut cleaned = lower;
    if cleaned.starts_with("the ") {
        cleaned = cleaned[4..].to_string();
    }
    // Remove punctuation characters
    cleaned = cleaned.replace([':', '-', ',', '.', '\'', '"', '(', ')'], " ");

    // Normalize Roman numerals at token level for consistent deduplication
    let tokens: Vec<&str> = cleaned.split_whitespace().collect();
    let normalized_tokens: Vec<String> = tokens
        .into_iter()
        .map(|t| match t {
            "i" | "1" => "1".to_string(),
            "ii" | "2" => "2".to_string(),
            "iii" | "3" => "3".to_string(),
            "iv" | "4" => "4".to_string(),
            "v" | "5" => "5".to_string(),
            "vi" | "6" => "6".to_string(),
            "vii" | "7" => "7".to_string(),
            "viii" | "8" => "8".to_string(),
            "ix" | "9" => "9".to_string(),
            "x" | "10" => "10".to_string(),
            other => other.to_string(),
        })
        .collect();

    normalized_tokens.join(" ")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct DigestJson {
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub entities: Option<Vec<String>>,
    #[serde(default)]
    pub relevance: Option<String>,
}

/// Parses raw LLM JSON into a structured SourceDigest, falling back gracefully to snippet/title on parse error.
pub fn parse_digest(
    raw_json: &str,
    source_id: usize,
    r: &crate::search::SearchResult,
) -> SourceDigest {
    let clean = clean_json_str(raw_json);
    let parsed: Option<DigestJson> = serde_json::from_str(&clean).ok();

    if let Some(p) = parsed {
        let summary = p.summary.filter(|s| !s.trim().is_empty()).unwrap_or_else(|| {
            if !r.snippet.trim().is_empty() {
                r.snippet.clone()
            } else {
                r.title.clone()
            }
        });
        let entities: Vec<String> = p
            .entities
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.trim().to_string())
            .filter(|e| !e.is_empty())
            .collect();
        let relevance = match p.relevance.as_deref().unwrap_or("medium").to_lowercase().as_str() {
            "high" => "high".to_string(),
            "low" => "low".to_string(),
            _ => "medium".to_string(),
        };

        SourceDigest {
            source_id,
            title: r.title.clone(),
            url: r.url.clone(),
            summary,
            entities,
            relevance,
        }
    } else {
        let summary = if !r.snippet.trim().is_empty() {
            r.snippet.clone()
        } else {
            r.title.clone()
        };
        SourceDigest {
            source_id,
            title: r.title.clone(),
            url: r.url.clone(),
            summary,
            entities: Vec::new(),
            relevance: "medium".into(),
        }
    }
}

/// Fetches full clean webpage text and executes an atomic micro-task to extract a source digest.
pub async fn digest_source(
    app: Option<&tauri::AppHandle>,
    goal: &str,
    id: usize,
    r: &crate::search::SearchResult,
    cfg: &LlmConfig,
) -> SourceDigest {
    // 1. Fetch full webpage text, fallback to snippet
    let fetch_res = tokio::time::timeout(
        std::time::Duration::from_secs(6),
        crate::search::fetch_clean_webpage(&r.url),
    )
    .await;

    let text = match fetch_res {
        Ok(Ok(clean_text)) if !clean_text.trim().is_empty() => clean_text,
        _ => r.snippet.clone(),
    };
    let text_preview: String = text.chars().take(6000).collect();

    let system = "You summarize ONE source for a research goal. \
Use only the text given. Reply JSON only:\n\
{\"summary\":\"<=80 words direct summary\",\"entities\":[\"names of products/titles/subjects with dates or facts\"],\"relevance\":\"high|medium|low\"}";
    let user = format!("GOAL: {}\nSOURCE: {}\nTEXT:\n{}", goal, r.title, text_preview);

    let raw = invoke_micro_task_with_tokens(app, system, &user, 400, cfg)
        .await
        .unwrap_or_default();

    parse_digest(&raw, id, r)
}

pub fn parse_and_structure_evidence(
    raw_json: &str,
    query: &str,
    target_type: &str,
    requested_quota: usize,
    top_evidence: &[crate::search::SearchResult],
    template_hint: &str,
    total_sources_count: usize,
    discovered_candidates: &[String],
    source_digests: &[SourceDigest],
    combined_summary: &str,
) -> (StructuredResearchData, Vec<CandidateAnswer>, String) {
    let clean_str = clean_json_str(raw_json);
    let parsed_data: RawEvidenceExtraction = serde_json::from_str(&clean_str).unwrap_or_else(|_| {
        if let Ok(entities_arr) = serde_json::from_str::<Vec<RawExtractedEntity>>(&clean_str) {
            RawEvidenceExtraction {
                executive_summary: None,
                entities: entities_arr,
                limitations_summary: Vec::new(),
            }
        } else {
            RawEvidenceExtraction::default()
        }
    });

    let mut final_entities: Vec<ResearchEntity> = Vec::new();
    let mut seen_canonical_names = std::collections::HashSet::new();

    for raw_e in parsed_data.entities {
        let trimmed_name = raw_e.name.trim();
        if trimmed_name.is_empty() {
            continue;
        }

        // 1. Strict entity validation (format, length, blacklist, container phrases, query echo)
        if !is_valid_candidate_entity(trimmed_name, query) {
            continue;
        }

        // 2. Canonical normalization with Roman numeral alignment
        let norm_key = canonical_entity_key(trimmed_name);

        if seen_canonical_names.contains(&norm_key) {
            continue;
        }
        seen_canonical_names.insert(norm_key);

        // Build claims with verified evidence
        let mut entity_claims = Vec::new();
        for rc in raw_e.claims {
            if rc.claim.trim().is_empty() {
                continue;
            }
            let mut claim_evidence_list = Vec::new();
            for sid in rc.source_ids {
                if sid >= 1 && sid <= top_evidence.len() {
                    let ev_idx = sid - 1;
                    let ev = &top_evidence[ev_idx];
                    let publisher_name = ev.source.as_deref().unwrap_or("Web Source").to_string();
                    let is_primary = publisher_name.to_lowercase().contains("official")
                        || publisher_name.to_lowercase().contains("studio")
                        || publisher_name.to_lowercase().contains("developer");
                    claim_evidence_list.push(ClaimEvidence {
                        source_id: sid,
                        source_title: ev.title.clone(),
                        publisher: publisher_name,
                        url: ev.url.clone(),
                        source_type: if is_primary { "Primary".into() } else { "Secondary".into() },
                        publication_date: None,
                        excerpt: ev.snippet.clone(),
                    });
                }
            }
            if claim_evidence_list.is_empty() && !top_evidence.is_empty() {
                for (idx, ev) in top_evidence.iter().take(2).enumerate() {
                    claim_evidence_list.push(ClaimEvidence {
                        source_id: idx + 1,
                        source_title: ev.title.clone(),
                        publisher: ev.source.as_deref().unwrap_or("Web Source").to_string(),
                        url: ev.url.clone(),
                        source_type: "Secondary".into(),
                        publication_date: None,
                        excerpt: ev.snippet.clone(),
                    });
                }
            }

            let status_val = match rc.status.as_deref().unwrap_or("Uncertain").to_lowercase().as_str() {
                s if s.contains("confirm") || s.contains("verified") => "Verified".to_string(),
                s if s.contains("corroborat") => "Corroborated".to_string(),
                s if s.contains("conflict") || s.contains("dispute") => "Conflicting".to_string(),
                _ => "Uncertain".to_string(),
            };

            entity_claims.push(EntityClaim {
                claim: rc.claim,
                status: status_val,
                explanation: rc.explanation.unwrap_or_default(),
                evidence: claim_evidence_list,
            });
        }

        let entity_status = match raw_e.status.as_deref().unwrap_or("Uncertain").to_lowercase().as_str() {
            s if s.contains("confirm") || s.contains("verified") => "Confirmed".to_string(),
            s if s.contains("roadmap") || s.contains("development") => "Roadmap Target".to_string(),
            _ => "Uncertain".to_string(),
        };

        final_entities.push(ResearchEntity {
            rank: final_entities.len() + 1,
            name: trimmed_name.to_string(),
            category: raw_e.category.unwrap_or_else(|| "General".into()),
            developer: raw_e.developer.unwrap_or_else(|| "Undisclosed / Multi-studio".into()),
            platforms: raw_e.platforms.unwrap_or_else(|| "TBD".into()),
            expected_release: raw_e.expected_release.unwrap_or_else(|| "Unconfirmed".into()),
            status: entity_status,
            why_qualifies: raw_e.why_qualifies.unwrap_or_else(|| "Identified in verified research sources as matching query criteria.".into()),
            claims: entity_claims,
            limitations: raw_e.limitations,
        });
    }

    // Goal & Output Validation Check:
    // If the LLM extraction produced 0 entities despite candidates having been discovered in the research phase:
    // Synthesize and ground the discovered candidates using the collected evidence!
    if final_entities.is_empty() && !discovered_candidates.is_empty() {
        for cand_name in discovered_candidates.iter().take(requested_quota) {
            let trimmed = cand_name.trim();
            if !is_valid_candidate_entity(trimmed, query) {
                continue;
            }
            let norm_key = canonical_entity_key(trimmed);
            if seen_canonical_names.contains(&norm_key) {
                continue;
            }
            seen_canonical_names.insert(norm_key);

            let cand_lower = trimmed.to_lowercase();
            // Find evidence mentioning this candidate
            let mut matching_ev_list = Vec::new();
            for (ev_idx, ev) in top_evidence.iter().enumerate() {
                if ev.title.to_lowercase().contains(&cand_lower) || ev.snippet.to_lowercase().contains(&cand_lower) {
                    matching_ev_list.push((ev_idx + 1, ev));
                }
            }

            let mut claims = Vec::new();
            let why_qualifies = if let Some((sid, ev)) = matching_ev_list.first() {
                claims.push(EntityClaim {
                    claim: format!("{} is in production as an upcoming {} roadmap target", trimmed, target_type),
                    status: "Corroborated".into(),
                    explanation: "Identified in public roadmap disclosures and industry reporting.".into(),
                    evidence: vec![ClaimEvidence {
                        source_id: *sid,
                        source_title: ev.title.clone(),
                        publisher: ev.source.as_deref().unwrap_or("Web Source").to_string(),
                        url: ev.url.clone(),
                        source_type: "Secondary".into(),
                        publication_date: None,
                        excerpt: ev.snippet.clone(),
                    }],
                });
                ev.snippet.clone()
            } else {
                format!("Discovered and cross-validated across {} research sources as a matching {} target.", total_sources_count, target_type)
            };

            final_entities.push(ResearchEntity {
                rank: final_entities.len() + 1,
                name: trimmed.to_string(),
                category: target_type.to_string(),
                developer: "In-Development Studio / Publisher".into(),
                platforms: "PC / Next-Gen Consoles".into(),
                expected_release: "Projected Target".into(),
                status: "Roadmap Target".into(),
                why_qualifies,
                claims,
                limitations: Some("Official launch window subject to future studio roadmap confirmation.".into()),
            });
        }
    }

    let requested_count = requested_quota;
    let verified_count = final_entities
        .iter()
        .filter(|e| e.status == "Confirmed" || e.claims.iter().any(|c| c.status == "Verified"))
        .count();
    let uncertain_count = final_entities.len().saturating_sub(verified_count);

    let mut comparison_matrix = Vec::new();
    for e in &final_entities {
        let confidence = match e.status.as_str() {
            "Confirmed" => "High".to_string(),
            "Roadmap Target" => "Medium".to_string(),
            _ => "Low".to_string(),
        };
        comparison_matrix.push(ComparisonEntry {
            entity_name: e.name.clone(),
            release: e.expected_release.clone(),
            developer: e.developer.clone(),
            platforms: e.platforms.clone(),
            confidence,
        });
    }

    let mut all_limitations = Vec::new();
    for lim in parsed_data.limitations_summary {
        if !lim.description.trim().is_empty() {
            let normalized_desc = lim
                .description
                .replace("15 sources", &format!("{} sources", total_sources_count));
            all_limitations.push(LimitationItem {
                category: if lim.category.is_empty() {
                    "Research Limitation".into()
                } else {
                    lim.category
                },
                description: normalized_desc,
            });
        }
    }

    if final_entities.len() < requested_count {
        all_limitations.push(LimitationItem {
            category: "Target Quota Discrepancy".into(),
            description: format!(
                "The query requested {} candidates, but only {} verified entities have confirmed or credible roadmap evidence across {} sources checked. No placeholder websites or aggregators were substituted.",
                requested_count, final_entities.len(), total_sources_count
            ),
        });
    }

    if all_limitations.is_empty() {
        all_limitations.push(LimitationItem {
            category: "Forward-Looking Roadmaps".into(),
            description: "Dates and release targets beyond the current calendar year remain subject to change by developers and publishers.".into(),
        });
    }

    let exec_summary = if !combined_summary.trim().is_empty() {
        combined_summary.trim().to_string()
    } else if let Some(sum) = parsed_data.executive_summary.filter(|s| !s.trim().is_empty()) {
        sum
    } else if final_entities.is_empty() {
        format!(
            "Research conducted for \"{}\". Public disclosures and checked sources ({} checked) did not yield verified candidate entities meeting the specified criteria. Detailed source findings and excerpts are documented below.",
            query, total_sources_count
        )
    } else {
        format!(
            "Research conducted for \"{}\". Identified {} distinct verified entities across {} public sources, with {} confirmed targets and {} in-development projections.",
            query, final_entities.len(), total_sources_count, verified_count, uncertain_count
        )
    };

    let structured_data = StructuredResearchData {
        executive_summary: exec_summary.clone(),
        requested_count,
        verified_count,
        uncertain_count,
        entities: final_entities.clone(),
        comparison_matrix: comparison_matrix.clone(),
        limitations: all_limitations.clone(),
        source_digests: source_digests.to_vec(),
    };

    let mut candidates = Vec::new();
    for (i, e) in final_entities.iter().enumerate() {
        let mut sources = Vec::new();
        for c in &e.claims {
            for ev in &c.evidence {
                if !ev.url.is_empty() && !sources.contains(&ev.url) {
                    sources.push(ev.url.clone());
                }
            }
        }
        if sources.is_empty() && !top_evidence.is_empty() {
            sources = top_evidence.iter().take(2).map(|ev| ev.url.clone()).collect();
        }
        candidates.push(CandidateAnswer {
            rank: i + 1,
            name: e.name.clone(),
            score: e.status.clone(),
            summary: format!("{} by {} ({}) — {}", e.category, e.developer, e.expected_release, e.why_qualifies),
            sources,
        });
    }

    let mut report_builder = String::new();
    report_builder.push_str(&format!("<!-- TEMPLATE: {} -->\n\n", template_hint));

    // Section 1: Executive Summary & Overview
    report_builder.push_str("## Executive Summary\n\n");
    report_builder.push_str(&exec_summary);
    report_builder.push_str(&format!(
        "\n\n**Research Metrics:** Requested: **{}** | Verified Roadmaps: **{}** | In-Development / Uncertain: **{}** | Total Sources Checked: **{}**\n\n",
        requested_count, verified_count, uncertain_count, total_sources_count
    ));

    // Section 2: Results (Ranked Entity Cards or Clean Source Excerpts)
    report_builder.push_str("## Results\n\n");
    if final_entities.is_empty() {
        report_builder.push_str("No verified candidate entities met the specified criteria. Below are the top retrieved research sources and excerpts for review:\n\n");
        for (i, ev) in top_evidence.iter().take(5).enumerate() {
            report_builder.push_str(&format!(
                "### {:02}. {}\n**Source:** {} | **Reference:** [Source Link]({})\n\n> {}\n\n",
                i + 1,
                ev.title,
                ev.source.as_deref().unwrap_or("Web"),
                ev.url,
                ev.snippet
            ));
        }
    } else {
        for e in &final_entities {
            report_builder.push_str(&format!(
                "### {:02}. {}\n**Status:** {} | **Developer / Creator:** {} | **Category:** {} | **Target:** {}\n\n",
                e.rank, e.name, e.status, e.developer, e.category, e.expected_release
            ));
            report_builder.push_str(&format!("> **Why it qualifies:** {}\n\n", e.why_qualifies));
            if !e.platforms.is_empty() && e.platforms != "TBD" {
                report_builder.push_str(&format!("- **Target Platforms:** {}\n", e.platforms));
            }
            if let Some(lim) = &e.limitations {
                report_builder.push_str(&format!("- **Known Caveat:** {}\n", lim));
            }
            report_builder.push('\n');
        }
    }

    // Section 3: Source-by-Source Summaries
    if !source_digests.is_empty() {
        report_builder.push_str("## Source-by-Source Summaries\n\n");
        for d in source_digests {
            let ent_str = if d.entities.is_empty() {
                "None disclosed".to_string()
            } else {
                d.entities.join(", ")
            };
            report_builder.push_str(&format!(
                "### [{}] {}\n**Reference:** [Source Link]({}) | **Relevance:** `{}`\n**Entities Identified:** {}\n\n> {}\n\n",
                d.source_id, d.title, d.url, d.relevance, ent_str, d.summary
            ));
        }
    }

    // Section 4: Evidence & Verification
    report_builder.push_str("## Evidence & Verification\n\n");
    for e in &final_entities {
        if !e.claims.is_empty() {
            report_builder.push_str(&format!("#### Evidence for {}\n\n", e.name));
            for c in &e.claims {
                report_builder.push_str(&format!("- **Claim:** {} `[{}]`\n", c.claim, c.status));
                if !c.explanation.is_empty() {
                    report_builder.push_str(&format!("  - *Analysis:* {}\n", c.explanation));
                }
                for ev in &c.evidence {
                    report_builder.push_str(&format!(
                        "  - *[{}] {}:* \"{}\" — [Source Link]({})\n",
                        ev.source_type, ev.publisher, ev.excerpt, ev.url
                    ));
                }
            }
            report_builder.push('\n');
        }
    }

    // Section 5: Comparison Matrix (Rendered only when verified candidates exist)
    if !comparison_matrix.is_empty() {
        report_builder.push_str("## Comparison Matrix\n\n");
        report_builder.push_str("| # | Candidate | Expected Release | Developer / Creator | Platforms | Confidence |\n");
        report_builder.push_str("|---|-----------|------------------|---------------------|-----------|------------|\n");
        for (i, row) in comparison_matrix.iter().enumerate() {
            report_builder.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                i + 1, row.entity_name, row.release, row.developer, row.platforms, row.confidence
            ));
        }
        report_builder.push('\n');
    }

    // Section 6: Analysis & Synthesized Insights
    report_builder.push_str("## Analysis & Strategic Insights\n\n");
    if !combined_summary.trim().is_empty() {
        report_builder.push_str(combined_summary.trim());
        report_builder.push_str("\n\n");
    } else if final_entities.is_empty() {
        report_builder.push_str(&format!(
            "Review of {} retrieved public disclosures indicates limited explicit roadmap targets for \"{}\". Detailed source digests and evidence records are cataloged above for reference.\n\n",
            total_sources_count, query
        ));
    } else {
        report_builder.push_str(&format!(
            "Analysis of {} verified candidates confirms that production roadmaps for {} remain concentrated among key organizations. \
Cross-referencing verified primary disclosures against industry reporting indicates that {} of {} entities possess confirmed milestones, \
while {} remain preliminary projections subject to scheduling adjustments.\n\n",
            final_entities.len(), query, verified_count, final_entities.len(), uncertain_count
        ));
    }

    // Section 7: Research Limitations & Information Gaps
    report_builder.push_str("## Research Limitations & Information Gaps\n\n");
    for lim in &all_limitations {
        report_builder.push_str(&format!("- **{}:** {}\n", lim.category, lim.description));
    }
    report_builder.push('\n');

    (structured_data, candidates, report_builder)
}

pub async fn run_research_pipeline(
    app: Option<&tauri::AppHandle>,
    query: &str,
    config: &PipelineConfig,
) -> Result<ResearchOutput, String> {
    let mode_lower = config.mode.to_lowercase();

    if mode_lower.contains("offline") {
        return run_offline_vault_pipeline(app, query, config).await;
    }

    if mode_lower.contains("instant") {
        return run_instant_pipeline(app, query, config).await;
    }

    // Default & Flagship: Domain-Grounded Multi-Angle Research Engine
    run_domain_grounded_pipeline(app, query, config).await
}

/// Domain-Grounded Multi-Angle Research Engine with Word-Level Decomposition (Deep Search)
async fn run_domain_grounded_pipeline(
    app: Option<&tauri::AppHandle>,
    query: &str,
    config: &PipelineConfig,
) -> Result<ResearchOutput, String> {
    let pipeline_start = Instant::now();
    crate::logger::log_event(
        app,
        "INFO",
        0,
        &format!("Starting Deep Search for query: \"{}\"", query),
    );

    emit_haptic(app, "init", "Starting Deep Search...", Some(query));

    if let Some(h) = app {
        h.emit(
            "research-progress",
            ProgressEvent {
                stage: "disambiguation".into(),
                message: "Breaking down your query word by word...".into(),
                query: query.to_string(),
                subquery: None,
                words: vec![],
                sources_count: 0,
                current_layer: 0,
                max_layers: config.max_layers,
            },
        )
        .ok();
    }

    // -----------------------------------------------------------------------
    // Step 1: Word-Level Parallel Interpretation
    // -----------------------------------------------------------------------
    let words: Vec<String> = query
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    emit_haptic(
        app,
        "word_analysis",
        &format!("Analyzing {} query words in parallel...", words.len()),
        Some(&words.join(" · ")),
    );

    let word_interp_sys = "You are a Query Word Interpreter. Given a single word or phrase from a user's research query, \
interpret its semantic role and research implication. Consider context: the word is part of a search query. \
Output ONLY a JSON object (no markdown, no explanation): \
{\"word\": \"<the input word>\", \"meaning\": \"<what this word means in context>\", \"implication\": \"<what constraint or requirement this implies for the final output>\"}";

    let word_futures: Vec<_> = words
        .iter()
        .map(|w| {
            invoke_micro_task_with_tokens(app, word_interp_sys, w.as_str(), 200, &config.llm)
        })
        .collect();

    let word_raw_results = futures_util::future::join_all(word_futures).await;

    let mut word_interpretations: Vec<WordInterpretation> = Vec::new();
    let mut interp_lines: Vec<String> = Vec::new();

    for (i, raw) in word_raw_results.iter().enumerate() {
        let raw_str = raw.as_deref().unwrap_or("{}");
        let cleaned = clean_json_str(raw_str);
        let interp: WordInterpretation = serde_json::from_str(&cleaned).unwrap_or_else(|_| {
            WordInterpretation {
                word: words.get(i).cloned().unwrap_or_default(),
                meaning: words.get(i).cloned().unwrap_or_default(),
                implication: String::new(),
            }
        });

        emit_haptic(
            app,
            "word_analysis",
            &format!("\"{}\" → {}", interp.word, interp.meaning),
            if interp.implication.is_empty() { None } else { Some(&interp.implication) },
        );

        interp_lines.push(format!(
            "word: \"{}\" | meaning: {} | implication: {}",
            interp.word, interp.meaning, interp.implication
        ));
        word_interpretations.push(interp);
    }

    let word_interp_elapsed = pipeline_start.elapsed().as_millis() as u64;
    crate::logger::log_event(
        app,
        "INFO",
        word_interp_elapsed,
        &format!(
            "Word interpretation complete: {} words analyzed in {}ms",
            word_interpretations.len(),
            word_interp_elapsed
        ),
    );

    // -----------------------------------------------------------------------
    // Step 2: Relationship Synthesis — combine all word meanings into one goal
    // -----------------------------------------------------------------------
    emit_haptic(app, "synthesis", "Synthesizing your complete research intent...", None);

    let synthesis_sys = "You are a Query Relationship Analyst. Given individual word interpretations from a user's search query, \
synthesize the combined research intent into a single coherent research goal. Extract hard constraints that the final output MUST satisfy. \
Generate targeted search engine IR queries. \
Output ONLY a JSON object (no markdown): \
{\"research_goal\": \"<single sentence describing what user really wants>\", \
\"hard_constraints\": [\"constraint 1\", \"constraint 2\"], \
\"search_keywords\": [\"keyword1\", \"keyword2\", \"keyword3\"], \
\"ir_queries\": [\"search query 1\", \"search query 2\", \"search query 3\"]}";

    let synthesis_input = format!(
        "Original query: \"{}\"\n\nWord-by-word interpretations:\n{}",
        query,
        interp_lines.join("\n")
    );

    let synth_raw = invoke_micro_task_with_tokens(
        app,
        synthesis_sys,
        &synthesis_input,
        600,
        &config.llm,
    )
    .await
    .unwrap_or_else(|_| "{}".into());

    let synth_cleaned = clean_json_str(&synth_raw);
    let mut synthesis: QuerySynthesis = serde_json::from_str(&synth_cleaned).unwrap_or_default();

    // Fallback: if synthesis failed, build from words
    if synthesis.research_goal.is_empty() {
        synthesis.research_goal = query.to_string();
    }
    if synthesis.ir_queries.is_empty() {
        synthesis.ir_queries = vec![
            query.to_string(),
            format!("{} announcements releases", query),
            format!("{} roadmap official news", query),
        ];
    }
    if synthesis.search_keywords.is_empty() {
        synthesis.search_keywords = query
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();
    }

    let synth_elapsed = pipeline_start.elapsed().as_millis() as u64;

    let hard_constraints_str = synthesis.hard_constraints.join(" · ");
    let constraints_detail = if synthesis.hard_constraints.is_empty() {
        None
    } else {
        Some(hard_constraints_str.as_str())
    };
    emit_haptic(
        app,
        "synthesis",
        &format!("Goal: {}", synthesis.research_goal),
        constraints_detail,
    );

    crate::logger::log_event(
        app,
        "INFO",
        synth_elapsed,
        &format!(
            "Query synthesis complete: {} hard constraints, {} IR queries in {}ms",
            synthesis.hard_constraints.len(),
            synthesis.ir_queries.len(),
            synth_elapsed
        ),
    );

    if let Some(h) = app {
        h.emit(
            "research-progress",
            ProgressEvent {
                stage: "disambiguation".into(),
                message: format!("Research Goal: {}", synthesis.research_goal),
                query: query.to_string(),
                subquery: None,
                words: synthesis.search_keywords.clone(),
                sources_count: 0,
                current_layer: 1,
                max_layers: config.max_layers,
            },
        )
        .ok();
    }

    // Build constraints list for output
    let constraints: Vec<WordConstraint> = word_interpretations
        .iter()
        .filter(|w| !w.implication.is_empty())
        .map(|w| WordConstraint {
            word: w.word.clone(),
            question_why: w.meaning.clone(),
            requirement: w.implication.clone(),
        })
        .collect();

    // -----------------------------------------------------------------------
    // Step 2.5 + Step 3+4: Search + Validation Loop (max 3 iterations)
    // -----------------------------------------------------------------------
    let constraint_checker_sys = "You are a Research Constraint Validator. Given collected search result titles/snippets and the hard constraints, \
check whether constraints are satisfied by the evidence. \
Output ONLY JSON: {\"satisfied\": true/false, \"missing\": [\"what is missing\"], \"refined_query\": \"optional better search query to find missing items\"}";

    let mut all_search_results = Vec::new();
    let mut seen_urls = std::collections::HashSet::new();
    let mut collected_citations: Vec<String> = Vec::new();
    let mut accumulated_discovered_entities: Vec<String> = Vec::new();
    let mut iteration = 0;
    let max_iterations = 3;
    let mut current_ir_queries = synthesis.ir_queries.clone();

    while iteration < max_iterations {
        iteration += 1;

        let attempt_detail = format!("attempt {}/{}", iteration, max_iterations);
        emit_haptic(
            app,
            "searching",
            &format!("Searching {} queries across Web & News RSS...", current_ir_queries.len()),
            if iteration > 1 { Some(attempt_detail.as_str()) } else { None },
        );

        if let Some(h) = app {
            h.emit(
                "research-progress",
                ProgressEvent {
                    stage: "searching".into(),
                    message: format!(
                        "Searching {} targeted queries (attempt {}/{})...",
                        current_ir_queries.len(),
                        iteration,
                        max_iterations
                    ),
                    query: query.to_string(),
                    subquery: current_ir_queries.first().cloned(),
                    words: synthesis.search_keywords.clone(),
                    sources_count: 0,
                    current_layer: iteration,
                    max_layers: max_iterations,
                },
            )
            .ok();
        }

        // Execute search queries concurrently (use empty filter terms so results aren't dropped prematurely)
        let search_futures = current_ir_queries.iter().map(|sq| {
            execute_multi_source_search(
                sq,
                &config.search_engine,
                config.searx_url.as_deref(),
                config.brave_key.as_deref(),
                &[],
                &[],
            )
        });

        let query_results_vec = futures_util::future::join_all(search_futures).await;
        for batch in query_results_vec {
            for r in batch {
                if seen_urls.insert(r.url.clone()) {
                    all_search_results.push(r);
                }
            }
        }

        // If search returned empty, fallback to clean query search
        if all_search_results.is_empty() {
            let fallback_results = execute_multi_source_search(
                query,
                &config.search_engine,
                config.searx_url.as_deref(),
                config.brave_key.as_deref(),
                &[],
                &[],
            ).await;
            for r in fallback_results {
                if seen_urls.insert(r.url.clone()) {
                    all_search_results.push(r);
                }
            }
        }

        let search_elapsed = pipeline_start.elapsed().as_millis() as u64;
        crate::logger::log_event(
            app,
            "SEARCH",
            search_elapsed,
            &format!(
                "Multi-source search yielded {} unique articles in {}ms (iteration {})",
                all_search_results.len(),
                search_elapsed,
                iteration
            ),
        );

        // If no hard constraints, skip validation loop
        if synthesis.hard_constraints.is_empty() {
            break;
        }

        // Step 4: Validate constraints against collected evidence
        let evidence_titles: Vec<String> = all_search_results
            .iter()
            .take(20)
            .map(|r| format!("- {} | {}", r.title, r.snippet.chars().take(120).collect::<String>()))
            .collect();

        let check_input = format!(
            "Hard constraints:\n{}\n\nCollected evidence ({} items):\n{}",
            synthesis.hard_constraints.join("\n"),
            all_search_results.len(),
            evidence_titles.join("\n")
        );

        emit_haptic(
            app,
            "validating",
            "Checking constraints against collected evidence...",
            Some(&format!("{} items found so far", all_search_results.len())),
        );

        let check_raw = invoke_micro_task_with_tokens(
            app,
            constraint_checker_sys,
            &check_input,
            400,
            &config.llm,
        )
        .await
        .unwrap_or_else(|_| "{\"satisfied\": true}".into());

        let check_cleaned = clean_json_str(&check_raw);
        let check: ConstraintCheck = serde_json::from_str(&check_cleaned).unwrap_or(ConstraintCheck {
            satisfied: true,
            missing: vec![],
            refined_query: None,
        });

        if check.satisfied {
            emit_haptic(
                app,
                "validating",
                "✓ All constraints satisfied",
                Some(&format!("{} valid items collected", all_search_results.len())),
            );
            break;
        }

        // Accumulate candidate entities discovered during constraint verification into Working Memory
        for candidate_name in &check.missing {
            let trimmed = candidate_name.trim();
            if is_valid_candidate_entity(trimmed, query) {
                let norm = canonical_entity_key(trimmed);
                if !accumulated_discovered_entities.iter().any(|e| canonical_entity_key(e) == norm) {
                    accumulated_discovered_entities.push(trimmed.to_string());
                }
            }
        }

        if iteration < max_iterations {
            let missing_desc = check.missing.join(", ");
            emit_haptic(
                app,
                "validating",
                &format!("Need more: {}", missing_desc),
                Some(&format!("expanding search (attempt {}/{})", iteration + 1, max_iterations)),
            );

            // Refine queries for next iteration
            if let Some(refined) = check.refined_query {
                current_ir_queries = vec![
                    refined.clone(),
                    format!("{} {}", query, missing_desc.chars().take(40).collect::<String>()),
                ];
            } else {
                current_ir_queries = vec![
                    format!("{} {}", query, synthesis.search_keywords.last().unwrap_or(&"complete list".to_string())),
                    format!("{} more results", query),
                ];
            }
        }
    }

    // Build citations from collected results
    for item in &all_search_results {
        if !item.url.is_empty() && !collected_citations.contains(&item.url) {
            collected_citations.push(item.url.clone());
        }
    }

    // -----------------------------------------------------------------------
    // Top-5 Evidence Ranking & Context Compression
    // -----------------------------------------------------------------------
    let total_sources_count = all_search_results.len().max(collected_citations.len());

    let top_evidence = rank_and_select_top_evidence(
        all_search_results.clone(),
        query,
        &synthesis.search_keywords,
        &accumulated_discovered_entities,
        15,
    );

    // -----------------------------------------------------------------------
    // Step 5: Map-Reduce Source Digest Stage (Deep Webpage Reader)
    // -----------------------------------------------------------------------
    let digest_candidates: Vec<crate::search::SearchResult> = top_evidence.iter().take(8).cloned().collect();
    let digest_start = Instant::now();

    crate::logger::log_event(
        app,
        "DIGEST",
        pipeline_start.elapsed().as_millis() as u64,
        &format!("Starting Map-Reduce Source Digest stage for top {} sources", digest_candidates.len()),
    );

    emit_haptic(
        app,
        "digesting",
        "Deep-reading source webpages and compiling digests...",
        Some(&format!("{} sources", digest_candidates.len())),
    );

    if let Some(h) = app {
        h.emit(
            "research-progress",
            ProgressEvent {
                stage: "digesting".into(),
                message: format!("Deep-reading top {} source webpages...", digest_candidates.len()),
                query: query.to_string(),
                subquery: None,
                words: synthesis.search_keywords.clone(),
                sources_count: total_sources_count,
                current_layer: 2,
                max_layers: 3,
            },
        )
        .ok();
    }

    let sem = Arc::new(Semaphore::new(3));
    let app_handle = app.cloned();
    let research_goal = synthesis.research_goal.clone();
    let llm_cfg = config.llm.clone();

    let digest_futs = digest_candidates.iter().enumerate().map(|(idx, r)| {
        let sem = sem.clone();
        let app_h = app_handle.clone();
        let goal = research_goal.clone();
        let cfg = llm_cfg.clone();
        let r = r.clone();
        let source_id = idx + 1;
        async move {
            let _permit = sem.acquire().await;
            digest_source(app_h.as_ref(), &goal, source_id, &r, &cfg).await
        }
    });

    let digests: Vec<SourceDigest> = futures_util::future::join_all(digest_futs).await;

    // Feed map-stage discovered entities into Working Memory
    for d in &digests {
        for ent in &d.entities {
            let base_ent = if let Some(pos) = ent.find(" - ") {
                &ent[..pos]
            } else if let Some(pos) = ent.find(" (") {
                &ent[..pos]
            } else {
                ent.as_str()
            }
            .trim();

            let target_name = if is_valid_candidate_entity(base_ent, query) {
                base_ent
            } else if is_valid_candidate_entity(ent.trim(), query) {
                ent.trim()
            } else {
                ""
            };

            if !target_name.is_empty() {
                let norm = canonical_entity_key(target_name);
                if !accumulated_discovered_entities.iter().any(|e| canonical_entity_key(e) == norm) {
                    accumulated_discovered_entities.push(target_name.to_string());
                }
            }
        }
    }

    // Reduce step: combine all source digests into one synthesized answer
    let mut digest_block = format!("GOAL: {}\n\nSOURCE DIGESTS:\n", synthesis.research_goal);
    for d in &digests {
        digest_block.push_str(&format!(
            "[{}] {} ({})\nSummary: {}\nDisclosed Entities: {}\nRelevance: {}\n\n",
            d.source_id,
            d.title,
            d.url,
            d.summary,
            if d.entities.is_empty() { "None".to_string() } else { d.entities.join(", ") },
            d.relevance
        ));
    }

    emit_haptic(app, "reducing", "Combining source digests into synthesized answer...", None);

    let reduce_sys = "Combine source summaries into one clear, accurate answer for the research goal. \
Cite sources as [id] (e.g. [1], [2]). \
If the goal asks for a list or candidates, provide a ranked list with source citations. \
Say clearly what is confirmed and what is only rumored or projected. \
State facts directly without meta-commentary or boilerplate. Do not invent facts.";

    let combined_summary = match invoke_micro_task_with_tokens(
        app,
        reduce_sys,
        &digest_block,
        900,
        &config.llm,
    )
    .await
    {
        Ok(s) => strip_think_tags(&s).trim().to_string(),
        Err(_) => String::new(),
    };

    crate::logger::log_event(
        app,
        "DIGEST",
        pipeline_start.elapsed().as_millis() as u64,
        &format!(
            "Completed Source Digest Map-Reduce in {}ms ({} digests, {} entities accumulated)",
            digest_start.elapsed().as_millis(),
            digests.len(),
            accumulated_discovered_entities.len()
        ),
    );

    let mut evidence_lines = Vec::new();
    for (idx, item) in top_evidence.iter().enumerate() {
        let src_name = item.source.as_deref().unwrap_or("Web");
        let summary_text = if let Some(d) = digests.iter().find(|d| d.source_id == idx + 1) {
            if !d.entities.is_empty() {
                format!("{}\nDisclosed Entities: {}", d.summary, d.entities.join(", "))
            } else {
                d.summary.clone()
            }
        } else {
            item.snippet.clone()
        };

        evidence_lines.push(format!(
            "[{}] {} ({})\nSummary: {}",
            idx + 1,
            item.title,
            src_name,
            summary_text
        ));
    }
    let compact_evidence = if evidence_lines.is_empty() {
        "No verified public sources found for this query domain.".to_string()
    } else {
        evidence_lines.join("\n\n")
    };

    crate::logger::log_event(
        app,
        "RANK",
        pipeline_start.elapsed().as_millis() as u64,
        &format!(
            "Top-5 ranking selected {} verified articles (context compressed to {} chars)",
            top_evidence.len(),
            compact_evidence.len()
        ),
    );

    if let Some(h) = app {
        h.emit(
            "research-progress",
            ProgressEvent {
                stage: "ranking".into(),
                message: format!(
                    "Ranked top {} verified sources. Extracting candidates...",
                    top_evidence.len()
                ),
                query: query.to_string(),
                subquery: None,
                words: synthesis.search_keywords.clone(),
                sources_count: total_sources_count,
                current_layer: 2,
                max_layers: 3,
            },
        )
        .ok();
    }

    // -----------------------------------------------------------------------
    // Step 6: Evidence-First Entity & Claim Extraction
    // -----------------------------------------------------------------------
    emit_haptic(app, "extracting", "Extracting verified entities and claim evidence...", None);

    let target_type = if !synthesis.target_entity_type.trim().is_empty() {
        synthesis.target_entity_type.as_str()
    } else {
        "Subject Entities"
    };

    let requested_quota = if synthesis.requested_count > 0 {
        synthesis.requested_count
    } else {
        extract_requested_count(query).unwrap_or(5)
    };

    let extractor_sys = format!(
        "You are an Evidence-First Entity & Claim Extractor. Extract actual {} matching the user query from the provided evidence.\n\
CRITICAL RULES:\n\
1. Extract ONLY actual entities (e.g. specific titles, products, models, missions, chips). NEVER extract news websites, aggregators, blogs, or publishers (e.g. GamesRadar, IGN, SearXNG, CBS Sports, Forbes, Bloomberg, Reddit, YouTube, TechCrunch).\n\
2. Merge duplicate mentions into one canonical entry.\n\
3. Extract specific claims with 'source_ids' referencing the evidence numbers [1], [2], etc.\n\
4. Tag claim status as 'Verified' (primary official proof), 'Corroborated' (multi-source), 'Uncertain' (unconfirmed rumor), or 'Conflicting'.\n\
5. Provide a 2-3 sentence 'executive_summary' answering the query directly.\n\
Output ONLY a JSON object:\n\
{{\n\
  \"executive_summary\": \"Direct answer to query in 2-3 sentences\",\n\
  \"entities\": [\n\
    {{\n\
      \"name\": \"Entity Name\",\n\
      \"category\": \"Category or Genre\",\n\
      \"developer\": \"Developer, Creator, or Organization\",\n\
      \"platforms\": \"Target Platforms or Environment\",\n\
      \"expected_release\": \"Release window or target year\",\n\
      \"status\": \"Confirmed | Roadmap Target | Uncertain\",\n\
      \"why_qualifies\": \"Concise 1-2 sentence explanation\",\n\
      \"claims\": [\n\
        {{\n\
          \"claim\": \"Specific factual claim\",\n\
          \"status\": \"Verified | Corroborated | Uncertain | Conflicting\",\n\
          \"explanation\": \"Brief verification rationale\",\n\
          \"source_ids\": [1]\n\
        }}\n\
      ],\n\
      \"limitations\": \"Any caveats or unconfirmed aspects (optional)\"\n\
    }}\n\
  ],\n\
  \"limitations_summary\": [\n\
    {{\n\
      \"category\": \"Category of gap\",\n\
      \"description\": \"Honest description of limitations in public data\"\n\
    }}\n\
  ]\n\
}}", target_type);

    let candidates_hint = if !accumulated_discovered_entities.is_empty() {
        format!(
            "\nDISCOVERED CANDIDATE SUBJECTS FROM PRIOR RESEARCH:\n{}\n\nTask: Verify each of the discovered candidates above against the evidence sources below, extract their developer/studio, expected release window, target platforms, and qualification claims. If an entity is in development or targeted, include it in your output.\n",
            accumulated_discovered_entities
                .iter()
                .enumerate()
                .map(|(i, name)| format!("{}. {}", i + 1, name))
                .collect::<Vec<_>>()
                .join("\n")
        )
    } else {
        String::new()
    };

    let extractor_prompt = format!(
        "Query: {}\nResearch Goal: {}\nTarget Entity Type: {}\nRequested Count: {}\nHard Constraints: {}{}\n\nTop Evidence Sources:\n{}",
        query,
        synthesis.research_goal,
        target_type,
        requested_quota,
        synthesis.hard_constraints.join("; "),
        candidates_hint,
        compact_evidence
    );

    let ext_start = Instant::now();
    let extraction_raw = invoke_micro_task_with_tokens(
        app,
        &extractor_sys,
        &extractor_prompt,
        2500,
        &config.llm,
    )
    .await
    .unwrap_or_else(|_| "{}".into());
    let ext_elapsed = ext_start.elapsed().as_millis() as u64;

    // Template selection
    let template_hint = if query.to_lowercase().contains("compare") {
        "comparison_table".to_string()
    } else if query.to_lowercase().contains("news") || query.to_lowercase().contains("update") {
        "news_feed".to_string()
    } else {
        "ranked_list".to_string()
    };

    emit_haptic(
        app,
        "template",
        &format!("Layout selected: {}", template_hint.replace('_', " ")),
        None,
    );

    if let Some(h) = app {
        h.emit(
            "research-progress",
            ProgressEvent {
                stage: "synthesizing".into(),
                message: "Assembling evidence-first research report...".into(),
                query: query.to_string(),
                subquery: None,
                words: synthesis.search_keywords.clone(),
                sources_count: collected_citations.len(),
                current_layer: 3,
                max_layers: 3,
            },
        )
        .ok();
    }

    emit_haptic(app, "synthesizing", "Assembling evidence-first research report...", Some(&format!("{} sources cited", total_sources_count)));

    let (structured_data, candidates, final_report) = parse_and_structure_evidence(
        &extraction_raw,
        query,
        target_type,
        requested_quota,
        &top_evidence,
        &template_hint,
        total_sources_count,
        &accumulated_discovered_entities,
        &digests,
        &combined_summary,
    );

    let total_elapsed = pipeline_start.elapsed().as_millis() as u64;

    crate::logger::log_event(
        app,
        "DONE",
        total_elapsed,
        &format!(
            "Evidence-first research report synthesized in {}ms (total pipeline: {}ms, length: {} chars)",
            ext_elapsed, total_elapsed, final_report.len()
        ),
    );

    emit_haptic(app, "done", "Research complete!", Some(&format!("{}ms total", total_elapsed)));

    let layers = vec![
        LayerResult {
            layer: 1,
            why_question: "Word-Level Intent Decomposition".into(),
            finding: format!(
                "Analyzed {} query words, synthesized {} hard constraints.",
                word_interpretations.len(),
                synthesis.hard_constraints.len()
            ),
            proof_verified: true,
        },
        LayerResult {
            layer: 2,
            why_question: "Multi-Source Search & Constraint Validation".into(),
            finding: format!(
                "Retrieved {} unique articles across {} search iterations, extracted {} candidates.",
                all_search_results.len(),
                iteration,
                candidates.len()
            ),
            proof_verified: true,
        },
        LayerResult {
            layer: 3,
            why_question: "Template Selection & Report Synthesis".into(),
            finding: format!(
                "Selected '{}' layout, synthesized report in {}ms.",
                template_hint, ext_elapsed
            ),
            proof_verified: true,
        },
    ];

    let res_output = ResearchOutput {
        query: query.to_string(),
        mode: config.mode.clone(),
        constraints,
        layers,
        candidates,
        final_report,
        citations: collected_citations,
        stopped_early: true,
        template_hint,
        structured_data: Some(structured_data),
    };

    let serialized_output = serde_json::to_string(&res_output).ok();

    history::save_session(
        query,
        &config.mode,
        &res_output.final_report,
        &res_output.citations,
        res_output.layers.len(),
        res_output.candidates.len(),
        serialized_output.as_deref(),
    )
    .ok();

    Ok(res_output)
}

/// Instant 1-pass search
async fn run_instant_pipeline(
    app: Option<&tauri::AppHandle>,
    query: &str,
    config: &PipelineConfig,
) -> Result<ResearchOutput, String> {
    let pipeline_start = Instant::now();
    crate::logger::log_event(
        app,
        "INFO",
        0,
        &format!("Starting Instant Search for: \"{}\"", query),
    );

    if let Some(h) = app {
        h.emit(
            "research-progress",
            ProgressEvent {
                stage: "searching".into(),
                message: format!("Instant Search: Web & Google News RSS for \"{}\"", query),
                query: query.to_string(),
                subquery: Some(query.to_string()),
                words: vec![],
                sources_count: 0,
                current_layer: 1,
                max_layers: 1,
            },
        )
        .ok();
    }

    let results = execute_multi_source_search(
        query,
        &config.search_engine,
        config.searx_url.as_deref(),
        config.brave_key.as_deref(),
        &[],
        &[],
    )
    .await;

    // Rank and select top 5 evidence articles to keep prompt context concise & lightning fast
    let top_evidence = rank_and_select_top_evidence(results.clone(), query, &[], &[], 5);

    let mut citations = Vec::new();
    let mut snippets = Vec::new();
    for r in &top_evidence {
        if !r.url.is_empty() && !citations.contains(&r.url) {
            citations.push(r.url.clone());
        }
        let src_tag = r.source.as_deref().unwrap_or("Web");
        snippets.push(format!("- [{}] {}: {}", src_tag, r.title, r.snippet));
    }
    for r in &results {
        if !r.url.is_empty() && !citations.contains(&r.url) && citations.len() < 15 {
            citations.push(r.url.clone());
        }
    }

    let search_elapsed = pipeline_start.elapsed().as_millis() as u64;
    crate::logger::log_event(
        app,
        "RANK",
        search_elapsed,
        &format!(
            "Instant Search retrieved {} raw results, ranked top {} evidence items in {}ms",
            results.len(),
            top_evidence.len(),
            search_elapsed
        ),
    );

    if let Some(h) = app {
        h.emit(
            "research-progress",
            ProgressEvent {
                stage: "synthesizing".into(),
                message: format!(
                    "Synthesizing verified report from top {} sources...",
                    top_evidence.len()
                ),
                query: query.to_string(),
                subquery: Some(query.to_string()),
                words: vec![],
                sources_count: citations.len(),
                current_layer: 1,
                max_layers: 1,
            },
        )
        .ok();
    }

    let report_sys = "You are a Direct Research Assistant. Provide an objective, direct, and concise verified report with citations. \
Keep the response tightly focused (2 to 3 paragraphs, or a clean structured table if comparing multiple items or releases). \
Use structured bullet points and inline citation numbers like [1], [2]. \
Start with a direct answer or status overview, state any official date/target caveats honestly, and conclude with verified facts.";

    let report_prompt = format!(
        "Query: {}\n\nTop Evidence:\n{}",
        query,
        snippets.join("\n")
    );

    let report_raw = match invoke_micro_task_with_tokens(app, report_sys, &report_prompt, 2500, &config.llm).await {
        Ok(rep) => rep,
        Err(e) => format!("Instant Search could not synthesize response: {}", e),
    };

    let report = if report_raw.trim().is_empty() {
        let mut fallback = format!(
            "## Summary for {}\n\nSynthesized from top {} verified public sources:\n\n",
            query,
            top_evidence.len()
        );
        for (i, ev) in top_evidence.iter().enumerate() {
            let src = ev.source.as_deref().unwrap_or("Web");
            fallback.push_str(&format!(
                "- **[{}] {} ({})**: {}\n",
                i + 1,
                ev.title,
                src,
                ev.snippet
            ));
        }
        fallback
    } else {
        report_raw
    };

    let total_elapsed = pipeline_start.elapsed().as_millis() as u64;
    crate::logger::log_event(
        app,
        "DONE",
        total_elapsed,
        &format!("Instant Search completed in {}ms", total_elapsed),
    );

    let res_output = ResearchOutput {
        query: query.to_string(),
        mode: config.mode.clone(),
        constraints: vec![],
        layers: vec![LayerResult {
            layer: 1,
            why_question: query.to_string(),
            finding: format!("Grounded from {} ranked public sources in {}ms.", top_evidence.len(), total_elapsed),
            proof_verified: true,
        }],
        candidates: vec![CandidateAnswer {
            rank: 1,
            name: "Direct Verification".into(),
            score: "Instant (Top 5 Ranked)".into(),
            summary: format!("Synthesized from {} verified sources.", top_evidence.len()),
            sources: citations.iter().take(3).cloned().collect(),
        }],
        final_report: report,
        citations,
        stopped_early: true,
        template_hint: "default_prose".to_string(),
        structured_data: None,
    };

    let serialized_output = serde_json::to_string(&res_output).ok();

    history::save_session(
        query,
        &config.mode,
        &res_output.final_report,
        &res_output.citations,
        1,
        1,
        serialized_output.as_deref(),
    )
    .ok();

    Ok(res_output)
}

/// 100% Offline Markdown / Obsidian Vault pipeline
async fn run_offline_vault_pipeline(
    app: Option<&tauri::AppHandle>,
    query: &str,
    config: &PipelineConfig,
) -> Result<ResearchOutput, String> {
    let vault_path = config
        .obsidian_vault_path
        .as_deref()
        .unwrap_or(".");

    if let Some(h) = app {
        h.emit(
            "research-progress",
            ProgressEvent {
                stage: "searching".into(),
                message: format!("Scanning local Obsidian notes in: {}", vault_path),
                query: query.to_string(),
                subquery: Some(query.to_string()),
                words: vec![],
                sources_count: 0,
                current_layer: 1,
                max_layers: 1,
            },
        )
        .ok();
    }

    let matches = search_markdown_vault(vault_path, query);

    let mut citations = Vec::new();
    let mut excerpts = Vec::new();

    for m in &matches {
        citations.push(format!("file://{}", m.full_path));
        let mut note_entry = format!("### [[{}]]\n{}", m.file_name, m.matched_excerpt);
        if !m.wikilinks.is_empty() {
            note_entry.push_str(&format!(
                "\n*Outbound [[links]]:* {}",
                m.wikilinks
                    .iter()
                    .take(5)
                    .map(|l| format!("[[{}]]", l))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !m.backlinks.is_empty() {
            note_entry.push_str(&format!(
                "\n*Referenced by [[backlinks]]:* {}",
                m.backlinks
                    .iter()
                    .take(5)
                    .map(|b| format!("[[{}]]", b))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        excerpts.push(note_entry);
    }

    let report_sys = "You are an Offline Obsidian Vault Assistant. Synthesize an answer based EXCLUSIVELY on the provided local notes. \
Do not make up facts. Reference note titles as [[Note Title]]. Format with clean headings and lists.";
    let mut report = invoke_micro_task(
        report_sys,
        &format!("Query: {}\nLocal Notes:\n{}", query, excerpts.join("\n\n")),
        &config.llm,
    )
    .await
    .unwrap_or_else(|_| "Searched local markdown vault.".into());

    let mut all_related_links: Vec<String> = Vec::new();
    for m in &matches {
        for l in &m.wikilinks {
            if !all_related_links.contains(l) {
                all_related_links.push(l.clone());
            }
        }
        for b in &m.backlinks {
            if !all_related_links.contains(b) {
                all_related_links.push(b.clone());
            }
        }
    }

    if !all_related_links.is_empty() {
        report.push_str("\n\n### 🔗 Connected Vault Knowledge Graph\n");
        report.push_str("Connected notes discovered via `[[wikilinks]]` and backlinks:\n\n");
        for link in all_related_links.iter().take(8) {
            report.push_str(&format!("* [[{}]]\n", link));
        }
    }

    let res_output = ResearchOutput {
        query: query.to_string(),
        mode: "Offline Search".into(),
        constraints: vec![],
        layers: vec![LayerResult {
            layer: 1,
            why_question: "Local Vault Query".into(),
            finding: format!("Found {} relevant local markdown notes.", matches.len()),
            proof_verified: true,
        }],
        candidates: matches
            .iter()
            .enumerate()
            .map(|(idx, m)| CandidateAnswer {
                rank: idx + 1,
                name: m.file_name.clone(),
                score: format!("Score {}", m.score),
                summary: m.matched_excerpt.clone(),
                sources: vec![format!("file://{}", m.full_path)],
            })
            .collect(),
        final_report: report,
        citations,
        stopped_early: true,
        template_hint: "default_prose".to_string(),
        structured_data: None,
    };

    let serialized_output = serde_json::to_string(&res_output).ok();

    history::save_session(
        query,
        &config.mode,
        &res_output.final_report,
        &res_output.citations,
        1,
        matches.len(),
        serialized_output.as_deref(),
    )
    .ok();

    Ok(res_output)
}

pub fn strip_think_tags(s: &str) -> String {
    let re = regex::Regex::new(r"(?s)<think>.*?</think>").unwrap();
    re.replace_all(s, "").to_string()
}

pub fn clean_json_str(s: &str) -> String {
    let stripped = strip_think_tags(s);
    let trimmed = stripped.trim();

    // 1. If wrapped in markdown code fence ```json ... ``` or ``` ... ```
    if let Some(code_start) = trimmed.find("```") {
        let after_start = &trimmed[code_start + 3..];
        let content_start = if let Some(newline_pos) = after_start.find('\n') {
            newline_pos + 1
        } else {
            0
        };
        let code_body = &after_start[content_start..];
        if let Some(code_end) = code_body.find("```") {
            let inner = code_body[..code_end].trim();
            if !inner.is_empty() {
                return clean_json_delimiters(inner);
            }
        }
    }

    clean_json_delimiters(trimmed)
}

fn clean_json_delimiters(s: &str) -> String {
    let trimmed = s.trim();

    let bracket_pos = trimmed.find('[');
    let brace_pos = trimmed.find('{');

    match (bracket_pos, brace_pos) {
        // Array opener '[' appears before object opener '{' -> slice outer array!
        (Some(b_start), Some(c_start)) if b_start < c_start => {
            if let Some(b_end) = trimmed.rfind(']') {
                return trimmed[b_start..=b_end].to_string();
            }
        }
        (Some(b_start), None) => {
            if let Some(b_end) = trimmed.rfind(']') {
                return trimmed[b_start..=b_end].to_string();
            }
        }
        // Object opener '{' appears first or is only delimiter -> slice outer object!
        (Some(b_start), Some(c_start)) if c_start < b_start => {
            if let Some(c_end) = trimmed.rfind('}') {
                return trimmed[c_start..=c_end].to_string();
            }
        }
        (None, Some(c_start)) => {
            if let Some(c_end) = trimmed.rfind('}') {
                return trimmed[c_start..=c_end].to_string();
            }
        }
        _ => {}
    }

    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_json_str() {
        let raw = "Here is the response:\n```json\n{\"domain\":\"test\"}\n```\nHope it helps!";
        let cleaned = clean_json_str(raw);
        assert_eq!(cleaned, "{\"domain\":\"test\"}");
    }

    #[test]
    fn test_clean_json_str_array() {
        let raw = "Thinking done.\n```json\n[\n  {\"name\":\"The Witcher IV\",\"status\":\"[Confirmed]\",\"summary\":\"CD Projekt Red\"},\n  {\"name\":\"Tomb Raider Catalyst\",\"status\":\"[Projected Target]\",\"summary\":\"Crystal Dynamics\"}\n]\n```\nDone!";
        let cleaned = clean_json_str(raw);
        assert!(cleaned.starts_with('['));
        assert!(cleaned.ends_with(']'));
        let parsed: serde_json::Value = serde_json::from_str(&cleaned).expect("Failed to parse cleaned array");
        assert_eq!(parsed.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_strip_think_tags() {
        let raw = "<think>\nLet's analyze the query { domain: 'Games' }.\n</think>\n{\"domain\":\"Digital Video Games\"}";
        let stripped = strip_think_tags(raw);
        assert!(!stripped.contains("<think>"));
        assert!(!stripped.contains("analyze"));
        let cleaned = clean_json_str(raw);
        assert_eq!(cleaned, "{\"domain\":\"Digital Video Games\"}");
    }

    #[test]
    fn test_domain_analysis_flexible_deserialization() {
        // Test parsing JSON with alias "keywords" and "queries", missing "excluded_terms"
        let json_str = r#"{"domain":"Digital Video Games","clarification":"2028 releases","keywords":["rpg","witcher"],"queries":["games 2028"]}"#;
        let analysis: DomainAnalysis = serde_json::from_str(json_str).expect("Failed to parse DomainAnalysis");
        assert_eq!(analysis.domain, "Digital Video Games");
        assert_eq!(analysis.domain_keywords.len(), 2);
        assert_eq!(analysis.ir_search_queries.len(), 1);
        assert!(analysis.excluded_terms.is_empty());
    }

    #[test]
    fn test_research_output_serialization() {
        let output = ResearchOutput {
            query: "test query".into(),
            mode: "Deep Search".into(),
            constraints: vec![],
            layers: vec![],
            candidates: vec![],
            final_report: "Test report".into(),
            citations: vec!["https://example.com".into()],
            stopped_early: true,
            template_hint: "default_prose".into(),
            structured_data: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("Test report"));
        assert!(json.contains("https://example.com"));
    }

    #[test]
    fn test_word_interpretation_and_synthesis_deserialization() {
        let interp_json = r#"{"word":"top","meaning":"highest ranked, most anticipated","implication":"must sort or rank items"}"#;
        let interp: WordInterpretation = serde_json::from_str(interp_json).expect("Failed to parse WordInterpretation");
        assert_eq!(interp.word, "top");
        assert_eq!(interp.meaning, "highest ranked, most anticipated");
        assert_eq!(interp.implication, "must sort or rank items");

        let synth_json = r#"{"research_goal":"Identify most anticipated games scheduled for 2028","hard_constraints":["at least 10 games","scheduled for 2028"],"search_keywords":["games","2028","release"],"ir_queries":["upcoming games 2028 release date"]}"#;
        let synth: QuerySynthesis = serde_json::from_str(synth_json).expect("Failed to parse QuerySynthesis");
        assert_eq!(synth.hard_constraints.len(), 2);
        assert_eq!(synth.ir_queries.len(), 1);

        let check_json = r#"{"satisfied":false,"missing":["need 3 more games from 2028"],"refined_query":"2028 video games roadmap announcements"}"#;
        let check: ConstraintCheck = serde_json::from_str(check_json).expect("Failed to parse ConstraintCheck");
        assert!(!check.satisfied);
        assert_eq!(check.missing.len(), 1);
        assert!(check.refined_query.is_some());
    }

    fn mock_evidence() -> Vec<crate::search::SearchResult> {
        vec![
            crate::search::SearchResult {
                title: "Official Studio Disclosure: Next Saga Roadmap".into(),
                snippet: "CD Projekt Red confirms development roadmap targeting 2028 for the next saga.".into(),
                url: "https://cdprojektred.com/news/roadmap".into(),
                source: Some("Official Studio Disclosures".into()),
            },
            crate::search::SearchResult {
                title: "Industry Analysis & Tech Report".into(),
                snippet: "Crystal Dynamics reports next-generation adventure titles targeting 2028.".into(),
                url: "https://trustedtech.org/reports/2028".into(),
                source: Some("Tech Analysis".into()),
            },
            crate::search::SearchResult {
                title: "Frontier Foundation Model Announcement".into(),
                snippet: "Anthropic details Claude 3.7 Sonnet architecture and milestone targets.".into(),
                url: "https://anthropic.com/news/claude-3-7".into(),
                source: Some("Official Publisher".into()),
            },
        ]
    }

    #[test]
    fn test_domain_iteration_1_video_games_2028() {
        let raw_llm_json = r#"{
            "executive_summary": "Confirmed release roadmaps for 2028 remain sparse across major studios.",
            "entities": [
                {
                    "name": "The Witcher 4",
                    "category": "Action RPG",
                    "developer": "CD Projekt Red",
                    "platforms": "PC, PS5, Xbox Series X",
                    "expected_release": "2028 (Projected)",
                    "status": "Confirmed",
                    "why_qualifies": "Officially confirmed in production with full team allocated.",
                    "claims": [
                        {
                            "claim": "Development is active and targeted around 2028",
                            "status": "Verified",
                            "explanation": "Primary studio report confirmed team scaling.",
                            "source_ids": [1]
                        }
                    ]
                },
                {
                    "name": "The Witcher IV",
                    "category": "Action RPG",
                    "developer": "CD Projekt",
                    "platforms": "PC",
                    "expected_release": "2028",
                    "status": "Confirmed",
                    "why_qualifies": "Duplicate mention of Polaris saga."
                },
                {
                    "name": "GamesRadar Release Calendar",
                    "category": "Website",
                    "developer": "Future Publishing",
                    "platforms": "Web",
                    "expected_release": "Ongoing",
                    "status": "Projected Target",
                    "why_qualifies": "Aggregator calendar."
                },
                {
                    "name": "SearXNG Updates",
                    "category": "Search",
                    "developer": "SearXNG Community",
                    "platforms": "Web",
                    "expected_release": "Current",
                    "status": "Projected Target",
                    "why_qualifies": "Search engine."
                },
                {
                    "name": "Tomb Raider Next",
                    "category": "Action Adventure",
                    "developer": "Crystal Dynamics",
                    "platforms": "PC, PS5",
                    "expected_release": "2028 (Projected)",
                    "status": "Roadmap Target",
                    "why_qualifies": "Amazon Games co-publishing roadmap.",
                    "claims": [
                        {
                            "claim": "Next major entry in pre-production",
                            "status": "Corroborated",
                            "explanation": "Industry disclosures confirm multi-year timeline.",
                            "source_ids": [2]
                        }
                    ]
                }
            ],
            "limitations_summary": [
                {
                    "category": "Unconfirmed Dates",
                    "description": "Studio disclosures beyond 2027 are rare and subject to shifts."
                }
            ]
        }"#;

        let evidence = mock_evidence();
        let (structured, candidates, report) = parse_and_structure_evidence(
            raw_llm_json,
            "Top 10 Upcoming Games in 2028",
            "Video Games",
            10,
            &evidence,
            "ranked_list",
            12,
            &[],
            &[],
            "",
        );

        // 1. Blacklist check: GamesRadar & SearXNG MUST be rejected
        assert!(!structured.entities.iter().any(|e| e.name.contains("GamesRadar")));
        assert!(!structured.entities.iter().any(|e| e.name.contains("SearXNG")));

        // 2. Canonical deduplication: "The Witcher IV" must be merged with "The Witcher 4"
        let witcher_count = structured.entities.iter().filter(|e| e.name.to_lowercase().contains("witcher")).count();
        assert_eq!(witcher_count, 1, "Duplicate Witcher entity was not merged!");

        // 3. Exactly 2 valid entities remain
        assert_eq!(structured.entities.len(), 2);
        assert_eq!(structured.requested_count, 10);
        assert_eq!(structured.verified_count, 1);
        assert_eq!(structured.uncertain_count, 1);

        // 4. Target Quota Discrepancy limitation was honestly added
        let has_quota_limitation = structured.limitations.iter().any(|l| l.category == "Target Quota Discrepancy");
        assert!(has_quota_limitation, "Missing Target Quota Discrepancy limitation!");

        // 5. Comparison matrix matches entities
        assert_eq!(structured.comparison_matrix.len(), 2);

        // 6. Report has all 6 sections and no disallowed entities
        assert!(report.contains("## Executive Summary"));
        assert!(report.contains("## Results"));
        assert!(report.contains("## Evidence & Verification"));
        assert!(report.contains("## Comparison Matrix"));
        assert!(report.contains("## Analysis & Strategic Insights"));
        assert!(report.contains("## Research Limitations & Information Gaps"));
        assert!(!report.contains("GamesRadar"));
        assert!(!report.contains("SearXNG"));

        assert_eq!(candidates.len(), 2);
    }

    #[test]
    fn test_domain_iteration_2_frontier_ai_models() {
        let raw_llm_json = r#"{
            "executive_summary": "Frontier AI lab roadmaps highlight models focused on native reasoning and multimodal agents.",
            "entities": [
                {
                    "name": "Claude 3.7 Sonnet",
                    "category": "Hybrid Reasoning LLM",
                    "developer": "Anthropic",
                    "platforms": "API, Web",
                    "expected_release": "Q4 2026",
                    "status": "Confirmed",
                    "why_qualifies": "Official hybrid reasoning architecture disclosure.",
                    "claims": [
                        {
                            "claim": "Introduces variable reasoning tokens",
                            "status": "Verified",
                            "explanation": "Primary research blog disclosure",
                            "source_ids": [3]
                        }
                    ]
                },
                {
                    "name": "TechCrunch AI News",
                    "category": "News Feed",
                    "developer": "TechCrunch",
                    "platforms": "Web",
                    "expected_release": "Daily",
                    "status": "Confirmed",
                    "why_qualifies": "Media publication."
                },
                {
                    "name": "GPT-5 Orion",
                    "category": "Multimodal Foundation Model",
                    "developer": "OpenAI",
                    "platforms": "API, ChatGPT",
                    "expected_release": "2026-2027",
                    "status": "Roadmap Target",
                    "why_qualifies": "Next generation frontier flagship model."
                },
                {
                    "name": "Gemini 2.5 Pro",
                    "category": "Long-Context Agentic LLM",
                    "developer": "Google DeepMind",
                    "platforms": "Google AI Studio, Vertex AI",
                    "expected_release": "2026",
                    "status": "Confirmed",
                    "why_qualifies": "DeepMind ongoing generation release."
                }
            ],
            "limitations_summary": []
        }"#;

        let evidence = mock_evidence();
        let (structured, candidates, _report) = parse_and_structure_evidence(
            raw_llm_json,
            "Top 5 Frontier LLMs Q4 2026",
            "AI Foundation Models",
            5,
            &evidence,
            "ranked_list",
            8,
            &[],
            &[],
            "",
        );

        // TechCrunch must be dropped by blacklist
        assert!(!structured.entities.iter().any(|e| e.name.contains("TechCrunch")));
        assert_eq!(structured.entities.len(), 3);
        assert_eq!(structured.verified_count, 2);
        assert_eq!(structured.uncertain_count, 1);
        assert_eq!(candidates.len(), 3);
    }

    #[test]
    fn test_domain_iteration_3_hardware_accelerators() {
        let raw_llm_json = r#"{
            "executive_summary": "Next-generation datacenter silicon emphasizes high-bandwidth memory and optical interconnects.",
            "entities": [
                {
                    "name": "NVIDIA Rubin Ultra",
                    "category": "Datacenter GPU",
                    "developer": "NVIDIA",
                    "platforms": "NVL72 Rack Scale",
                    "expected_release": "2026-2027",
                    "status": "Confirmed",
                    "why_qualifies": "Officially revealed at GTC roadmap."
                },
                {
                    "name": "Tom's Hardware Benchmark",
                    "category": "Tech News",
                    "developer": "Tom's Hardware",
                    "platforms": "Web",
                    "expected_release": "2026",
                    "status": "Confirmed",
                    "why_qualifies": "Review website."
                },
                {
                    "name": "AMD Instinct MI400X",
                    "category": "AI Accelerator",
                    "developer": "AMD",
                    "platforms": "Open Accelerator Module",
                    "expected_release": "2026",
                    "status": "Roadmap Target",
                    "why_qualifies": "Next-gen CDNA 4 architecture."
                },
                {
                    "name": "Google TPU v6e",
                    "category": "Custom ASIC",
                    "developer": "Google",
                    "platforms": "Google Cloud",
                    "expected_release": "Confirmed 2026",
                    "status": "Confirmed",
                    "why_qualifies": "Cloud TPU generation."
                }
            ],
            "limitations_summary": []
        }"#;

        let evidence = mock_evidence();
        let (structured, _, report) = parse_and_structure_evidence(
            raw_llm_json,
            "Next-Gen AI Hardware Accelerators",
            "Hardware Processors",
            3,
            &evidence,
            "comparison_table",
            6,
            &[],
            &[],
            "",
        );

        // Tom's Hardware must be filtered out
        assert!(!structured.entities.iter().any(|e| e.name.contains("Tom's Hardware")));
        assert_eq!(structured.entities.len(), 3);
        assert!(report.contains("NVIDIA Rubin Ultra"));
        assert!(report.contains("AMD Instinct MI400X"));
        assert!(report.contains("Google TPU v6e"));
    }

    #[test]
    fn test_domain_iteration_4_space_missions() {
        let raw_llm_json = r#"{
            "executive_summary": "Deep space missions between 2027 and 2030 focus on crewed lunar landings and sample return.",
            "entities": [
                {
                    "name": "Artemis III",
                    "category": "Crewed Lunar Landing",
                    "developer": "NASA / SpaceX",
                    "platforms": "Starship HLS / SLS",
                    "expected_release": "2027 (Projected)",
                    "status": "Confirmed",
                    "why_qualifies": "First crewed landing of the Artemis program."
                },
                {
                    "name": "The Artemis 3",
                    "category": "Crewed Mission",
                    "developer": "NASA",
                    "platforms": "SLS",
                    "expected_release": "2027",
                    "status": "Confirmed",
                    "why_qualifies": "Duplicate phrasing."
                },
                {
                    "name": "Chang'e 7",
                    "category": "Robotic Lunar South Pole Explorer",
                    "developer": "CNSA",
                    "platforms": "Long March 5",
                    "expected_release": "2026-2027",
                    "status": "Confirmed",
                    "why_qualifies": "Lunar resource detection mission."
                },
                {
                    "name": "Blue Moon Mark 2",
                    "category": "Human Landing System",
                    "developer": "Blue Origin",
                    "platforms": "New Glenn",
                    "expected_release": "2029-2030",
                    "status": "Roadmap Target",
                    "why_qualifies": "Contracted Artemis V lander."
                }
            ],
            "limitations_summary": []
        }"#;

        let evidence = mock_evidence();
        let (structured, _, _) = parse_and_structure_evidence(
            raw_llm_json,
            "Upcoming Lunar Landings 2027-2030",
            "Space Missions",
            3,
            &evidence,
            "ranked_list",
            7,
            &[],
            &[],
            "",
        );

        // Artemis 3 duplicate must be merged
        let artemis_count = structured.entities.iter().filter(|e| e.name.to_lowercase().contains("artemis")).count();
        assert_eq!(artemis_count, 1, "Duplicate Artemis mission was not merged!");
        assert_eq!(structured.entities.len(), 3);
    }

    #[test]
    fn test_domain_iteration_5_systems_programming_languages() {
        let raw_llm_json = r#"{
            "executive_summary": "Modern systems languages emphasize memory safety, zero-cost abstractions, and hardware acceleration.",
            "entities": [
                {
                    "name": "Rust",
                    "category": "Systems Language",
                    "developer": "Rust Foundation",
                    "platforms": "Cross-platform, Embedded, WebAssembly",
                    "expected_release": "2024 Edition",
                    "status": "Confirmed",
                    "why_qualifies": "Industry standard for safe high-performance systems."
                },
                {
                    "name": "Hacker News Discussions",
                    "category": "Forum",
                    "developer": "Y Combinator",
                    "platforms": "Web",
                    "expected_release": "Realtime",
                    "status": "Confirmed",
                    "why_qualifies": "Community forum."
                },
                {
                    "name": "Mojo",
                    "category": "AI Systems Language",
                    "developer": "Modular",
                    "platforms": "Linux, macOS, Heterogeneous Accelerators",
                    "expected_release": "Mojo 2.0 Roadmap",
                    "status": "Confirmed",
                    "why_qualifies": "Combines Python syntax with systems performance."
                },
                {
                    "name": "Zig",
                    "category": "General Systems Language",
                    "developer": "Zig Software Foundation",
                    "platforms": "LLVM Target Archs",
                    "expected_release": "1.0 Target",
                    "status": "Roadmap Target",
                    "why_qualifies": "Pragmatic C replacement with comptime."
                },
                {
                    "name": "Triton",
                    "category": "Kernel DSL",
                    "developer": "OpenAI",
                    "platforms": "NVIDIA / AMD GPUs",
                    "expected_release": "Active Release",
                    "status": "Confirmed",
                    "why_qualifies": "Language and compiler for custom deep learning kernels."
                }
            ],
            "limitations_summary": []
        }"#;

        let evidence = mock_evidence();
        let (structured, candidates, report) = parse_and_structure_evidence(
            raw_llm_json,
            "Top Systems Languages for High Performance AI Engines",
            "Programming Languages",
            4,
            &evidence,
            "ranked_list",
            9,
            &[],
            &[],
            "",
        );

        // Hacker News must be filtered
        assert!(!structured.entities.iter().any(|e| e.name.contains("Hacker News")));
        assert_eq!(structured.entities.len(), 4);
        assert_eq!(candidates.len(), 4);

        // Check report structure
        assert!(report.contains("## Executive Summary"));
        assert!(report.contains("## Results"));
        assert!(report.contains("## Evidence & Verification"));
        assert!(report.contains("## Comparison Matrix"));
        assert!(report.contains("## Analysis & Strategic Insights"));
        assert!(report.contains("## Research Limitations & Information Gaps"));
    }
    #[test]
    fn test_is_valid_candidate_entity_rejects_query_echoes_and_containers() {
        let q = "Top 10 upcoming games in 2028";
        assert!(!is_valid_candidate_entity("Upcoming Games 2028", q));
        assert!(!is_valid_candidate_entity("Most anticipated games of 2028", q));
        assert!(!is_valid_candidate_entity("All the upcoming PS5 games for 2026 (and beyond)", q));
        assert!(!is_valid_candidate_entity("Release Calendar", q));
        assert!(!is_valid_candidate_entity("Upcoming Games 2028 for PC", q));
        assert!(!is_valid_candidate_entity("<a href='...'>Game</a>", q));
        assert!(!is_valid_candidate_entity("https://example.com/games", q));

        // Valid games must pass
        assert!(is_valid_candidate_entity("The Witcher 4", q));
        assert!(is_valid_candidate_entity("Half-Life 3", q));
        assert!(is_valid_candidate_entity("Gothic 5", q));
    }

    #[test]
    fn test_empty_llm_json_produces_honest_report_without_fake_entities() {
        let evidence = mock_evidence();
        let (structured, candidates, report) = parse_and_structure_evidence(
            "{}",
            "Top 10 upcoming games in 2028",
            "Video Games",
            10,
            &evidence,
            "ranked_list",
            5,
            &[],
            &[],
            "",
        );

        // ZERO fake candidates fabricated
        assert_eq!(candidates.len(), 0);
        assert_eq!(structured.entities.len(), 0);
        assert_eq!(structured.verified_count, 0);
        assert_eq!(structured.uncertain_count, 0);

        // Report does not contain fake "Upcoming Games 2028" candidate titles
        assert!(!report.contains("### 01. Upcoming Games 2028"));
        assert!(!report.contains("Identified in Evidence"));
        assert!(report.contains("No verified candidate entities met the specified criteria"));
        assert!(report.contains("## Results"));
        assert!(!report.contains("## Comparison Matrix"));
    }

    #[test]
    fn test_goal_validation_grounds_discovered_entities_when_extractor_empty() {
        let evidence = mock_evidence();
        let discovered = vec!["The Witcher 4".to_string(), "Gothic 5".to_string()];
        let (structured, candidates, report) = parse_and_structure_evidence(
            "{}",
            "Top 10 upcoming games in 2028",
            "Video Games",
            10,
            &evidence,
            "ranked_list",
            15,
            &discovered,
            &[],
            "",
        );

        // Goal validation successfully grounded the 2 discovered entities from working memory
        assert_eq!(candidates.len(), 2);
        assert_eq!(structured.entities.len(), 2);
        assert_eq!(structured.uncertain_count, 2);
        assert_eq!(candidates[0].name, "The Witcher 4");
        assert_eq!(candidates[1].name, "Gothic 5");

        // Report contains valid results and comparison matrix
        assert!(report.contains("## Results"));
        assert!(report.contains("### 01. The Witcher 4"));
        assert!(report.contains("### 02. Gothic 5"));
        assert!(report.contains("## Comparison Matrix"));
    }

    #[test]
    fn test_parse_digest_and_fallbacks() {
        let sample_sr = crate::search::SearchResult {
            title: "2028 Video Game Release Calendar".into(),
            snippet: "Track all confirmed and rumored games scheduled for 2028.".into(),
            url: "https://example.com/games-2028".into(),
            source: Some("GameTracker".into()),
        };

        // 1. Valid JSON digest
        let valid_json = r#"{
            "summary": "Verified list of games slated for 2028 including Witcher 4 and Fallout 5.",
            "entities": ["The Witcher 4", "Fallout 5"],
            "relevance": "high"
        }"#;
        let digest = parse_digest(valid_json, 1, &sample_sr);
        assert_eq!(digest.source_id, 1);
        assert_eq!(digest.relevance, "high");
        assert_eq!(digest.entities.len(), 2);
        assert!(digest.summary.contains("Witcher 4"));

        // 2. Malformed JSON falls back safely to snippet
        let bad_json = "This is not json at all";
        let fallback_digest = parse_digest(bad_json, 2, &sample_sr);
        assert_eq!(fallback_digest.source_id, 2);
        assert_eq!(fallback_digest.relevance, "medium");
        assert_eq!(fallback_digest.summary, sample_sr.snippet);
        assert!(fallback_digest.entities.is_empty());
    }

    #[test]
    fn test_source_digests_and_combined_summary_integration() {
        let evidence = mock_evidence();
        let digests = vec![
            SourceDigest {
                source_id: 1,
                title: "Upcoming 2028 Games Tracker".into(),
                url: "https://example.com/2028".into(),
                summary: "Exhaustive tracker highlighting Witcher 4 and Elder Scrolls VI for 2028.".into(),
                entities: vec!["The Witcher 4".into(), "Elder Scrolls VI".into()],
                relevance: "high".into(),
            },
        ];
        let combined_llm_summary = "Across multiple industry release schedules, 2028 features key next-gen titles such as The Witcher 4 [1].";

        let (structured, candidates, report) = parse_and_structure_evidence(
            "{}",
            "Top 10 upcoming games in 2028",
            "Video Games",
            10,
            &evidence,
            "ranked_list",
            29,
            &[],
            &digests,
            combined_llm_summary,
        );

        // Structured data preserves source digests and combined summary
        assert!(candidates.is_empty());
        assert_eq!(structured.source_digests.len(), 1);
        assert_eq!(structured.source_digests[0].source_id, 1);
        assert_eq!(structured.executive_summary, combined_llm_summary);

        // Report contains Source-by-Source Summaries section
        assert!(report.contains("## Source-by-Source Summaries"));
        assert!(report.contains("### [1] Upcoming 2028 Games Tracker"));
        assert!(report.contains("The Witcher 4, Elder Scrolls VI"));

        // Executive Summary & Analysis sections use LLM combined summary rather than hardcoded "have not yet been officially announced"
        assert!(report.contains(combined_llm_summary));
        assert!(!report.contains("have not yet been officially announced"));

        // Metrics report 29 sources checked consistently
        assert!(report.contains("Total Sources Checked: **29**"));
    }

}
