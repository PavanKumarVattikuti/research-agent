# 🔬 Release Notes — Research Agent v2.2.4

> **Tag:** `v2.2.4`  
> **Release Name:** `Research Agent v2.2.4 — Map-Reduce Source Digest Stage & Deep Webpage Reader`  
> **Target Commits:** Strictly Local (Ready for Manual Human Verification & Push)  

---

## 🌟 Executive Summary

**Research Agent v2.2.4** addresses the core reasons why research reports on forward-looking or list-targeted queries (such as upcoming games or industry roadmaps) previously suffered from information starvation:

1. **Full Webpage Deep Reader (Map-Reduce Source Digest Stage)**:
   - In previous versions, the pipeline relied solely on 1-line search engine snippets or RSS titles. Primary release pages and calendar databases were never visited.
   - v2.2.4 introduces a high-speed parallel Map-Reduce stage between evidence ranking and extraction.
   - **Fetch**: Top 8 sources are fetched via `crate::search::fetch_clean_webpage` (with a 6-second timeout safeguard and snippet fallback), extracting up to 6,000 clean text characters per page.
   - **Map**: Executes atomic single-responsibility micro-tasks per source to produce structured JSON containing a concise summary (<=80 words), candidate entities found with dates or facts, and relevance categorization (`high`, `medium`, `low`).
   - **Bounded Concurrency**: Uses `tokio::sync::Semaphore::new(3)` to ensure local GPUs (e.g. 12GB VRAM) and inference endpoints are not overloaded.
   - **Reduce**: Synthesizes all per-source summaries into a unified, citation-grounded answer with `[id]` citations and ranked entity lists.
   - **Feedback Loop**: Entities discovered during source reading are fed into `accumulated_discovered_entities` (Working Memory) and forwarded to the final extractor prompt.

2. **List Query Aggregator Ranking Boost**:
   - For count/list queries (detected via `extract_requested_count(query).is_some()`), the ranker removes the generic `-4.0` aggregator penalty on calendar and tracking pages, granting them a `+3.5` relevance boost instead.

3. **Dynamic Synthesis & Honest Fallback Elimination**:
   - The hardcoded sentence (*"formal production roadmaps or confirmed release targets have not yet been officially announced"*) has been completely eliminated.
   - When evidence is present, the executive summary and strategic analysis are driven dynamically by the LLM Reduce stage.
   - Even if subsequent extraction yields zero candidate entities, the report preserves the combined synthesis and the complete Source-by-Source Summaries.

4. **Source Count Consistency**:
   - Unified `total_sources_count` across the entire pipeline. The discrepancy between reported metrics (e.g. 29 sources) and limitations (e.g. "none of the 15 sources") is completely resolved.

5. **Frontend Presentation & Deep Digest Workspace**:
   - Added `source_digests: Vec<SourceDigest>` to `StructuredResearchData`.
   - Built interactive Deep Source Digest cards in the SolidJS interface, displaying source link badges, relevance pills, clean summaries, and identified entity tags.
   - Added a `🔍 Deep-Read` metric pill to the research header.

---

## 🧪 Verification & Test Results

- **Rust Unit Tests**: `33 passed; 0 failed; 0 ignored`
  - `search::tests::test_rank_and_select_top_evidence_boosts_aggregators_for_list_queries` ✅
  - `pipeline::tests::test_parse_digest_and_fallbacks` ✅
  - `pipeline::tests::test_source_digests_and_combined_summary_integration` ✅
  - `pipeline::tests::test_goal_validation_grounds_discovered_entities_when_extractor_empty` ✅
  - `pipeline::tests::test_is_valid_candidate_entity_rejects_query_echoes_and_containers` ✅
  - `pipeline::tests::test_empty_llm_json_produces_honest_report_without_fake_entities` ✅
  - All 5 domain verification tests (Games, AI, Hardware, Space, Languages) ✅
- **TypeScript Check**: `0 errors` (`npx tsc --noEmit`).
- **Frontend Production Build**: `0 errors` (`npm run build` completed in 5.26s).

---

## 🚀 Manual Git & GitHub Release Instructions (For Human User)

In accordance with strict safety guardrails, **Antigravity never executes `git push`, tag pushes, or release creation**. All Git remotes remain strictly untouched. When you are ready to publish:

### Step 1: Review Git Status
```bash
git status
```

### Step 2: Stage and Commit Changes
```bash
git add .
git commit -m "feat(v2.2.4): map-reduce source digest stage, deep webpage reader, and list query ranking"
```

### Step 3: Tag the Release
```bash
git tag -a v2.2.4 -m "Release v2.2.4: Map-Reduce Source Digest Stage"
```

### Step 4: Push to GitHub (User Choice)
```bash
git push origin main
git push origin v2.2.4
```
