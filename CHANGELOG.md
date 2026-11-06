# 📋 Changelog — Research Agent

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [2.2.4] — 2026-11-06

### 🚀 Added
- **Full Webpage Deep Reader (Map-Reduce Source Digest Stage)**:
  - Concurrent web fetcher via `fetch_clean_webpage` with 6-second timeout safeguard and fallback to snippet.
  - Source-level atomic micro-task extracting <= 80-word direct summary, candidate entities with dates/facts, and relevance score (`high`, `medium`, `low`).
  - Bounded concurrency with `tokio::sync::Semaphore::new(3)` to protect local GPUs (12GB VRAM) and inference endpoints from saturation.
  - Reduce micro-task that combines per-source summaries into an objective, factual synthesis with `[id]` citations and ranked entity lists.
  - Continuous feedback loop injecting discovered entities from full webpages into `accumulated_discovered_entities` (Working Memory) and downstream extractor prompts.
- **Deep Source Digest Presentation Cards in UI**:
  - Added responsive source digest cards to the SolidJS Evidence workspace and Sources tab.
  - Interactive cards show citation ID badge, direct URL link, relevance tag, concise summary, and discovered entity tags.
  - Added a `🔍 Deep-Read` metric pill to the research metrics header.
- **Full History Session State Persistence & Instant Restoration**:
  - Added `output_json TEXT` column to SQLite `sessions` table with automatic, backward-compatible migration.
  - Persists full `ResearchOutput` (including `StructuredResearchData`, entity cards with claims and proof, comparative matrix, source digests, and limitations) across app sessions.
  - Implemented in-memory `sessionResultsCache` in SolidJS frontend for instant zero-latency switching between current search and historical sessions without losing complete views.

### ⚡ Changed
- **List Query Aggregator Ranking Overhaul**:
  - Detects count- and list-targeted queries (e.g. *"top 10 games in 2028"*, *"upcoming releases"*) via `extract_requested_count`.
  - Replaces the generic `-4.0` penalty on calendar/aggregator pages with a **`+3.5` relevance boost**, prioritizing release calendars and tracking databases when lists are requested.
- **Eliminated Hardcoded Fallback Boilerplate**:
  - Completely removed hardcoded assumptions (*"formal production roadmaps or confirmed release targets for ... have not yet been officially announced"*).
  - Reports dynamically reflect the LLM Reduce stage synthesis. If public evidence is sparse, the model directly states the exact status based on the retrieved sources.
- **Unified Source Accounting Metrics**:
  - Unified `total_sources_count` across search retrieval, progress events, prompt contexts, report metrics headers, and limitation descriptions, eliminating discrepancies (e.g. 29 vs 15).
- **Markdown Report Ordering**:
  - Ordered sections logically: Executive/Combined Summary -> Results (Entities or Excerpts) -> Source-by-Source Summaries -> Evidence & Verification -> Comparison Matrix -> Analysis & Synthesized Insights -> Research Limitations & Gaps.

### 🛡️ Fixed
- Fixed an issue where switching between historical sessions and the latest search result stripped `structured_data`, causing entity cards, metrics, and source summaries to disappear.
- Fixed linker failure (`LNK1108`) by relocating the build target directory via an NTFS directory junction to a high-capacity drive (`D:\research_agent_target`).

---

## [2.2.3] — 2026-11-04

### 🚀 Added
- **Persistent Candidate Working Memory**:
  - Implemented `accumulated_discovered_entities` to retain candidate subjects identified during intermediate search iterations.
- **Candidate Continuity to Extractor**:
  - Injected discovered candidate subjects directly into the final extraction prompt.
- **Output Goal Validation Layer**:
  - Grounded discovered candidates directly from the collected evidence corpus if the extraction step returned zero entities.

### ⚡ Changed
- Expanded evidence selection context from 5 to 15 sources in `rank_and_select_top_evidence`.
- Added a `+15.0` title boost and `+8.0` snippet boost to articles mentioning discovered candidate entities.

---

## [2.2.2] — 2026-11-04

### 🛡️ Security & Integrity
- Implemented `is_valid_candidate_entity` rejecting query-echoes, container phrases, and HTML/XML tags.
- Added path traversal sanitization in `sanitize_export_filename` (`..`, `/`, `\`, `:`).
- Enforced honest zero-candidate reporting without fabricating placeholder entities.

---

## [2.2.1] — 2026-11-04

### 🎨 Visual & Packaging
- Refreshed high-resolution desktop crystal prism icons (.ico, .icns, .png).
- Added `app:dev` and `app:build` scripts to `package.json`.
- Linux desktop bundle preparation.

---

## [2.2.0] — 2026-11-03

### 🏛️ Architecture
- Transitioned to the Evidence-First Research Architecture with publisher blacklists and canonical Roman numeral deduplication.
- Added academic preprint search (arXiv Atom XML parser).
- Added Obsidian Vault Export (`.VAULT`) and native wikilink graph traversal for offline search.
