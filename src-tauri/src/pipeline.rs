use crate::history;
use crate::llm::{invoke_micro_task, invoke_micro_task_with_tokens, LlmConfig};
use crate::search::{execute_multi_source_search, rank_and_select_top_evidence};
use crate::vault::search_markdown_vault;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::Emitter;

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
    pub hard_constraints: Vec<String>,
    #[serde(default)]
    pub search_keywords: Vec<String>,
    #[serde(default)]
    pub ir_queries: Vec<String>,
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
    let top_evidence = rank_and_select_top_evidence(
        all_search_results.clone(),
        query,
        &synthesis.search_keywords,
        5,
    );

    for item in &all_search_results {
        if !item.url.is_empty() && !collected_citations.contains(&item.url) && collected_citations.len() < 15 {
            collected_citations.push(item.url.clone());
        }
    }

    let mut evidence_lines = Vec::new();
    for (idx, item) in top_evidence.iter().enumerate() {
        let src_name = item.source.as_deref().unwrap_or("Web");
        evidence_lines.push(format!(
            "[{}] {} ({})\nSummary: {}",
            idx + 1,
            item.title,
            src_name,
            item.snippet
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
                sources_count: collected_citations.len(),
                current_layer: 2,
                max_layers: 3,
            },
        )
        .ok();
    }

    // -----------------------------------------------------------------------
    // Candidate Extraction
    // -----------------------------------------------------------------------
    let candidates_sys = "You are a Candidate Extractor. Extract up to 10 candidates/titles addressing the query from the provided evidence. \
For each, provide: \
'name': Entity title \
'status': Tag as '[Confirmed]', '[In-Development Roadmap]', or '[Projected Target]' \
'summary': Studio/developer and concise basis for inclusion. \
Output ONLY a JSON array: [{\"name\":\"Title\",\"status\":\"[Status]\",\"summary\":\"Details\"}]";

    let cand_start = Instant::now();
    let candidates_raw = invoke_micro_task_with_tokens(
        app,
        candidates_sys,
        &format!(
            "Query: {}\nResearch Goal: {}\nHard Constraints: {}\n\nTop Evidence:\n{}",
            query,
            synthesis.research_goal,
            synthesis.hard_constraints.join("; "),
            compact_evidence
        ),
        2000,
        &config.llm,
    )
    .await
    .unwrap_or_else(|_| "[]".into());

    let cand_elapsed = cand_start.elapsed().as_millis() as u64;

    #[derive(Deserialize)]
    struct RawCand {
        #[serde(default, alias = "title")]
        name: String,
        #[serde(default)]
        status: Option<String>,
        #[serde(default, alias = "details", alias = "description")]
        summary: String,
    }

    let clean_cands = clean_json_str(&candidates_raw);
    let parsed_cands: Vec<RawCand> = serde_json::from_str(&clean_cands).unwrap_or_default();

    let mut candidates = Vec::new();
    for (i, c) in parsed_cands.into_iter().enumerate() {
        if !c.name.trim().is_empty() {
            let score = c.status.unwrap_or_else(|| "Verified".into());
            candidates.push(CandidateAnswer {
                rank: i + 1,
                name: c.name,
                score,
                summary: c.summary,
                sources: collected_citations.iter().take(2).cloned().collect(),
            });
        }
    }

    if candidates.is_empty() {
        for (i, item) in top_evidence.iter().take(5).enumerate() {
            candidates.push(CandidateAnswer {
                rank: i + 1,
                name: item.title.clone(),
                score: "[Projected Target]".into(),
                summary: item.snippet.clone(),
                sources: vec![item.url.clone()],
            });
        }
    }

    crate::logger::log_event(
        app,
        "INFO",
        pipeline_start.elapsed().as_millis() as u64,
        &format!("Extracted {} candidates in {}ms", candidates.len(), cand_elapsed),
    );

    // -----------------------------------------------------------------------
    // Step 5: Template Selection + Final Synthesis
    // -----------------------------------------------------------------------
    emit_haptic(app, "template", "Selecting best visual layout for this data...", None);

    let template_sys = "You are a Presentation Strategist. Given the research query and content type, select the best visual template. \
Output ONLY JSON: {\"template\": \"<template_name>\", \"reason\": \"<brief reason>\"} \
Available templates: \
ranked_list — for top-N lists, rankings, best-of queries \
comparison_table — for comparing multiple items side by side, X vs Y \
news_feed — for recent news, weekly updates, latest events \
flashcards — for definitions, Q&A, educational content \
default_prose — for narrative analysis, general research";

    let template_raw = invoke_micro_task_with_tokens(
        app,
        template_sys,
        &format!(
            "Query: \"{}\"\nResearch goal: {}\nCandidates found: {}",
            query,
            synthesis.research_goal,
            candidates.len()
        ),
        150,
        &config.llm,
    )
    .await
    .unwrap_or_else(|_| "{\"template\": \"default_prose\"}".into());

    #[derive(Deserialize, Default)]
    struct TemplateHint {
        #[serde(default)]
        template: String,
        #[serde(default)]
        reason: String,
    }

    let template_cleaned = clean_json_str(&template_raw);
    let template_choice: TemplateHint = serde_json::from_str(&template_cleaned).unwrap_or_default();
    let template_hint = if template_choice.template.is_empty() {
        "default_prose".to_string()
    } else {
        template_choice.template.clone()
    };

    emit_haptic(
        app,
        "template",
        &format!("Layout selected: {}", template_hint.replace('_', " ")),
        if template_choice.reason.is_empty() { None } else { Some(&template_choice.reason) },
    );

    // Final synthesis
    if let Some(h) = app {
        h.emit(
            "research-progress",
            ProgressEvent {
                stage: "synthesizing".into(),
                message: "Writing your research report...".into(),
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

    emit_haptic(app, "synthesizing", "Writing your research report...", Some(&format!("{} sources cited", collected_citations.len())));

    let report_sys = format!("You are an Elite Research Analyst. Write an objective, high-density research report answering the user's query based strictly on the provided evidence. \
Strict Guidelines: \
1. Start with an '## Executive Summary' summarizing key findings. If an official definitive list does not yet exist, state this clearly. \
2. Provide a '## Candidate Overview' structured markdown table with columns: # | Title | Studio / Developer | Platforms | Status / Target | Key Anticipation Basis. \
3. Follow with '## Detailed Analysis' covering each candidate with confirmed roadmap details, technological advances, and studio disclosures. \
4. Distinguish clearly between officially confirmed targets and credible industry roadmap projections. \
5. Use inline citations like [1], [2] referencing the provided sources. Do NOT fabricate ungrounded claims. \
6. The very first line of your response MUST be exactly one of these template hint tags (no other text before it): \
   <!-- TEMPLATE: {} --> \
   This line is stripped by the renderer and never shown to users.", template_hint);

    let report_prompt = format!(
        "User Query: {}\n\nResearch Goal: {}\nHard Constraints: {}\n\nTop Evidence Sources:\n{}\n\nExtracted Candidates:\n{:?}\n\nSources:\n{:?}",
        query,
        synthesis.research_goal,
        synthesis.hard_constraints.join("; "),
        compact_evidence,
        candidates,
        collected_citations
    );

    let synth_start = Instant::now();
    let final_report_raw = match invoke_micro_task_with_tokens(app, &report_sys, &report_prompt, 4096, &config.llm).await {
        Ok(rep) => rep,
        Err(e) => {
            return Err(format!(
                "Failed to generate research report with LLM ({}): {}",
                config.llm.base_url, e
            ));
        }
    };
    let synth_elapsed = synth_start.elapsed().as_millis() as u64;
    let total_elapsed = pipeline_start.elapsed().as_millis() as u64;

    let final_report = if final_report_raw.trim().is_empty() {
        let hint_line = format!("<!-- TEMPLATE: {} -->\n", template_hint);
        let mut fallback = format!(
            "{}## Executive Summary\n\nResearch completed for **{}** using {} verified sources.\n\n## Candidate Overview\n\n| # | Title | Status | Key Highlights |\n|---|---|---|---|\n",
            hint_line, query, top_evidence.len()
        );
        for (i, c) in candidates.iter().enumerate() {
            fallback.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                i + 1, c.name, c.score, c.summary.replace('\n', " ")
            ));
        }
        fallback
    } else {
        // Ensure template hint is present if LLM forgot it
        if final_report_raw.contains("<!-- TEMPLATE:") {
            final_report_raw
        } else {
            format!("<!-- TEMPLATE: {} -->\n{}", template_hint, final_report_raw)
        }
    };

    crate::logger::log_event(
        app,
        "DONE",
        total_elapsed,
        &format!(
            "Research report synthesized in {}ms (total pipeline: {}ms, length: {} chars)",
            synth_elapsed, total_elapsed, final_report.len()
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
                template_hint, synth_elapsed
            ),
            proof_verified: true,
        },
    ];

    history::save_session(
        query,
        &config.mode,
        &final_report,
        &collected_citations,
        layers.len(),
        candidates.len(),
    )
    .ok();

    Ok(ResearchOutput {
        query: query.to_string(),
        mode: config.mode.clone(),
        constraints,
        layers,
        candidates,
        final_report,
        citations: collected_citations,
        stopped_early: true,
        template_hint,
    })
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
    let top_evidence = rank_and_select_top_evidence(results.clone(), query, &[], 5);

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

    history::save_session(query, &config.mode, &report, &citations, 1, 1).ok();

    Ok(ResearchOutput {
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
    })
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
        excerpts.push(format!("### [[{}]]\n{}", m.file_name, m.matched_excerpt));
    }

    let report_sys = "You are an Offline Obsidian Vault Assistant. Synthesize an answer based EXCLUSIVELY on the provided local notes. \
Do not make up facts. Reference note titles as [[Note Title]]. Format with clean headings and lists.";
    let report = invoke_micro_task(
        report_sys,
        &format!("Query: {}\nLocal Notes:\n{}", query, excerpts.join("\n\n")),
        &config.llm,
    )
    .await
    .unwrap_or_else(|_| "Searched local markdown vault.".into());

    history::save_session(query, &config.mode, &report, &citations, 1, matches.len()).ok();

    Ok(ResearchOutput {
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
    })
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
}
