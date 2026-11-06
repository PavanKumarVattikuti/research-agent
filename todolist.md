# 📋 Todolist — Research Agent

> **Workflow**: Pick a step → mark `🔄 In Progress` → do the work → mark `✅ Done` → next step.
> **Rule**: Only one agent works on a step at a time. Write your name next to `🔄`.

---

## Task: Update & Modernize All Dependencies (Nov 2026)

| # | Step | Status | Agent |
|---|------|--------|-------|
| 1 | Create `todolist.md` with all steps | ✅ Done | Antigravity |
| 2 | Update `Agents.md` with todolist workflow section | ✅ Done | Antigravity |
| 3 | Update `.gitignore` to whitelist `todolist.md` | ✅ Done | Antigravity |
| 4 | Update `pyproject.toml` — bump versions, remove dead deps, rename ddgs | ✅ Done | Antigravity |
| 5 | Run `uv sync` — install updated packages | ✅ Done | Antigravity |
| 6 | Rewrite `perform_dynamic_search()` in `agent.py` — remove `langchain-community` | ✅ Done | Antigravity |
| 7 | Replace `gr.update()` calls in `agent.py` — use component constructors | ✅ Done | Antigravity |
| 8 | Fix Gradio `theme=` in `main.py` — move to `.launch()` | ✅ Done | Antigravity |
| 9 | Replace `gr.update()` calls in `main.py` — use component constructors | ✅ Done | Antigravity |
| 10 | Create `debug_tests.py` — full verification harness | ✅ Done | Antigravity |
| 11 | Run `debug_tests.py` — verify all functions | ✅ Done | Antigravity |
| 12 | Update `project_structure.md` — add new files | ✅ Done | Antigravity |
| 13 | Final manual smoke test — UI + search | ✅ Done | Antigravity |

---

## Task: Release v1.3.0 Preparation & Packaging

| # | Step | Status | Agent |
|---|------|--------|-------|
| 14 | Bump version to `1.3.0` in `pyproject.toml` and update `SECURITY.md` | ✅ Done | Antigravity |
| 15 | Add `multiprocessing.freeze_support()` in `main.py` entry point | ✅ Done | Antigravity |
| 16 | Configure git user identity locally | ✅ Done | Antigravity |
| 17 | Compile `.exe` with PyInstaller & Gradio 6 asset hooks | ✅ Done | Antigravity |
| 18 | Stage and commit all changes for v1.3.0 | ✅ Done | Antigravity |
| 19 | Guide Git authentication & GitHub Release upload | ✅ Done | Antigravity |

---

## Task: Version 2.0 — Rust + Tauri + SolidJS Conversion

| # | Step | Status | Agent |
|---|------|--------|-------|
| 20 | Backup v1 Python files into `legacy/v1_backup/` | ✅ Done | Antigravity |
| 21 | Install Rust toolchain (`rustup` / `cargo`) | ✅ Done | Antigravity |
| 22 | Scaffold Tauri v2 + SolidJS project structure | ✅ Done | Antigravity |
| 23 | Implement Rust backend core (SSRF, Multi-protocol LLM client, Search, Health tester) | ✅ Done | Antigravity |
| 24 | Implement Obsidian / Local Markdown Vault parser in Rust | ✅ Done | Antigravity |
| 25 | Implement 5-Whys word-level questioning & candidate re-ranking pipeline in Rust | ✅ Done | Antigravity |
| 26 | Implement SQLite history store in Rust | ✅ Done | Antigravity |
| 27 | Build SolidJS minimalist UI (Hero search, Mode dropdown, hideable history, Settings with live test) | ✅ Done | Antigravity |
| 28 | Verify & test all functions end-to-end | ✅ Done | Antigravity |
| 29 | Build standalone cross-platform release binary | ✅ Done | Antigravity |

---

## Task: UI Redesign, Perplexity Parity, & UX Upgrades (v2.1.0)

| # | Step | Status | Agent |
|---|------|--------|-------|
| 30 | Isolate `history.rs` unit tests, add `delete_session`, and implement welcome seed chat | ✅ Done | Antigravity |
| 31 | Expose delete commands in `lib.rs` and add `research-progress` event emission in `pipeline.rs` | ✅ Done | Antigravity |
| 32 | Configure `marked` and style `.prose-obsidian` typography in `src/App.css` | ✅ Done | Antigravity |
| 33 | Redesign `App.tsx` for Perplexity answer layout, live progress, schematic timeline, Shift+Enter, Export/Copy | ✅ Done | Antigravity |
| 34 | Add sidebar chat deletion (`✕`), session details loader, and clean local test database | ✅ Done | Antigravity |
| 35 | Verify end-to-end with unit tests, frontend build, and release executable | ✅ Done | Antigravity |

---

## Task: Version 2.1.1–v2.1.3 — Formula Overhaul, Anti-Bot Sources & Premium Visual Identity

| # | Step | Status | Agent |
|---|------|--------|-------|
| 36 | Implement Google News RSS, Hacker News Algolia, and Domain Relevance Filter in `src-tauri/src/search.rs` | ✅ Done | Antigravity |
| 37 | Overhaul `src-tauri/src/pipeline.rs` with Relational Intent Disambiguation & Candidate Status Tagging | ✅ Done | Antigravity |
| 38 | Implement Premium Visual Identity Design Tokens & Typography in `src/App.css` (Geist scale, layered charcoal, #38D9C0 cyan) | ✅ Done | Antigravity |
| 39 | Redesign `src/App.tsx` with Linear/Raycast Research Workspace (activity timeline, interactive citations, grouped history) | ✅ Done | Antigravity |
| 40 | Verify end-to-end with unit tests, frontend build, and real research verification | ✅ Done | Antigravity |
| 41 | Compile optimized standalone production release binary | ✅ Done | Antigravity |
| 42 | Implement top-5 keyword article ranking & snippet compression in `src-tauri/src/search.rs` | ✅ Done | Antigravity |
| 43 | Fix local LLM temperature (0.2), base URL normalization & system prompt fallback in `src-tauri/src/llm.rs` | ✅ Done | Antigravity |
| 44 | Streamline research pipeline stages & eliminate redundant local LLM roundtrips in `src-tauri/src/pipeline.rs` | ✅ Done | Antigravity |
| 45 | Add native `save_markdown_export` Tauri command in `src-tauri/src/lib.rs` | ✅ Done | Antigravity |
| 46 | Rename export button to 'Export', fix download handling & add error diagnostics in `src/App.tsx` | ✅ Done | Antigravity |
| 47 | Verify end-to-end with tests, build, and release packaging | ✅ Done | Antigravity |
| 48 | Create `src-tauri/src/logger.rs` with timestamped step logging & `pipeline-log` event emission | ✅ Done | Antigravity |
| 49 | Wrap LLM calls with strict 60s `tokio::time::timeout` & token ceilings in `src-tauri/src/llm.rs` | ✅ Done | Antigravity |
| 50 | Convert multi-source retrieval to concurrent `tokio::join!` with 6s search limits in `src-tauri/src/search.rs` | ✅ Done | Antigravity |
| 51 | Instrument `run_instant_pipeline` & `run_domain_grounded_pipeline` with top-5 ranking & step timers in `src-tauri/src/pipeline.rs` | ✅ Done | Antigravity |
| 52 | Expose `get_pipeline_logs`, `clear_pipeline_logs`, and `save_logs_export` in `src-tauri/src/lib.rs` | ✅ Done | Antigravity |
| 53 | Add Terminal Logs tab in Settings with live view, download logs, and GitHub issue reporter in `src/App.tsx` | ✅ Done | Antigravity |
| 54 | Verify end-to-end with unit tests, frontend build, and release binary packaging | ✅ Done | Antigravity |

---

## Task: Version 2.1.4 — Thinking Model Compatibility, Array Slicing Fix & Robust Extraction

| # | Step | Status | Agent |
|---|------|--------|-------|
| 55 | Fix `clean_json_str`, add `strip_think_tags`, and add flexible Serde defaults & aliases in `src-tauri/src/pipeline.rs` | ✅ Done | Antigravity |
| 56 | Support `reasoning_content` / `reasoning` / `thinking` and pass `app` for `[LLM]` logs in `src-tauri/src/llm.rs` | ✅ Done | Antigravity |
| 57 | Add candidate & synthesis fallbacks and realign token ceilings in `src-tauri/src/pipeline.rs` | ✅ Done | Antigravity |
| 58 | Add comprehensive unit tests for JSON arrays, think tags & deserialization, run `cargo test --lib` | ✅ Done | Antigravity |
| 59 | Frontend build, production release binary packaging, and walkthrough update | ✅ Done | Antigravity |

---

## Task: Version 2.1.5 — Word-Level Decomposition, Pre-Templated Layouts, Animated Tagline & Live Haptic Streaming

| # | Step | Status | Agent |
|---|------|--------|-------|
| 60 | Add animated rotating tagline & logo styling in `src/App.tsx` and `src/App.css` | ✅ Done | Antigravity |
| 61 | Implement parallel word-level decomposition, relationship synthesis & constraint check in `src-tauri/src/pipeline.rs` | ✅ Done | Antigravity |
| 62 | Implement real-time `HapticEvent` stream and frontend live streaming panel in `src/App.tsx` | ✅ Done | Antigravity |
| 63 | Implement template detection & pre-templated renderers (`ranked_list`, `news_feed`, `comparison_table`, `flashcards`) in `src/App.tsx` | ✅ Done | Antigravity |
| 64 | Verify with `cargo test --lib` (15 passing tests), `npx tsc --noEmit` and production `npm run build` | ✅ Done | Antigravity |
| 65 | SVG app icon, 5s single-word rotating tagline, permanent haptic trace in accordion | ✅ Done | Antigravity |
| 66 | Version alignment v2.1.5, .gitignore legacy exclusion, README update & release notes | ✅ Done | Antigravity |
| 67 | Standalone styled HTML export with dark theme, citations & clean markdown/clipboard in `src/App.tsx` & `src-tauri/src/lib.rs` | ✅ Done | Antigravity |
| 68 | Normalize template routing synonyms & add auto-detection table fallback for cards in `src/App.tsx` & `src-tauri/src/pipeline.rs` | ✅ Done | Antigravity |
| 69 | Upload updated production binary, NSIS setup installer & MSI bundle to GitHub Release v2.1.5 | ✅ Done | Antigravity |
| 70 | Implement @media print stylesheets and clean document typography in `src/App.css` | ✅ Done | Antigravity |
| 71 | Add Print / Save to PDF (`.PDF`) button in toolbar and HTML export in `src/App.tsx` | ✅ Done | Antigravity |
| 72 | Verify frontend build, compile release binary, package installer & launch app | ✅ Done | Antigravity |
| 73 | Replace hardcoded gaming taxonomy with dynamic domain overview instructions in `src-tauri/src/pipeline.rs` | ✅ Done | Antigravity |

---

## Task: Version 2.2.0 — Evidence-First Research Architecture & Domain Verification

| # | Step | Status | Agent |
|---|------|--------|-------|
| 73 | Define Evidence-First Data Contract & Publisher Blacklist in `src-tauri/src/pipeline.rs` | ✅ Done | Antigravity |
| 74 | Implement Entity Extraction, Canonical Deduplication & Claim Verification in `src-tauri/src/pipeline.rs` | ✅ Done | Antigravity |
| 75 | Build Evidence-First Presentation Workspace (Metrics, Cards, Claims, Matrix, Limitations) in `src/App.tsx` & `src/App.css` | ✅ Done | Antigravity |
| 76 | Add 5-Domain Verification Fixtures (Games, AI, Chips, Space, Languages) & Test Suite in `src-tauri/src/pipeline.rs` | ✅ Done | Antigravity |
| 77 | Align version to `2.2.0` across Cargo, Tauri, package.json, App.tsx, README, todolist & generate release notes | ✅ Done | Antigravity |
| 78 | Full verification: `cargo test --lib`, `tsc`, `npm run build`, and walkthrough documentation | ✅ Done | Antigravity |

---

## Task: Version 2.2.0 (Extension A) — Academic Preprint Retrieval & Direct Obsidian Vault Export

| # | Step | Status | Agent |
|---|------|--------|-------|
| 79 | Implement `fetch_arxiv_search` & Atom XML parser in `src-tauri/src/search.rs` | ✅ Done | Antigravity |
| 80 | Integrate arXiv into `execute_multi_source_search` with scientific query detection | ✅ Done | Antigravity |
| 81 | Implement `save_to_obsidian_vault` in `src-tauri/src/vault.rs` and expose IPC command in `src-tauri/src/lib.rs` | ✅ Done | Antigravity |
| 82 | Add "Save to Vault" (`.VAULT`) button in `src/App.tsx` follow-up toolbar | ✅ Done | Antigravity |
| 83 | Add unit tests for arXiv XML parser & vault note generation in `src-tauri/src/search.rs` and `src-tauri/src/vault.rs` | ✅ Done | Antigravity |
| 84 | Full verification: `cargo test --lib`, `npx tsc --noEmit`, and `npm run build` | ✅ Done | Antigravity |

---

## Task: Version 2.2.0 (Extension B) — Obsidian Wikilink Extraction & Backlink Discovery for Offline Search

| # | Step | Status | Agent |
|---|------|--------|-------|
| 85 | Implement `extract_wikilinks` and `find_backlinks_in_vault` in `src-tauri/src/vault.rs` | ✅ Done | Antigravity |
| 86 | Include linked notes and backlinks in `VaultNoteMatch` struct in `src-tauri/src/vault.rs` | ✅ Done | Antigravity |
| 87 | Integrate wikilinks into `run_offline_vault_pipeline` in `src-tauri/src/pipeline.rs` | ✅ Done | Antigravity |
| 88 | Display connected vault backlinks in `src/App.tsx` Sources & Evidence views | ✅ Done | Antigravity |
| 89 | Add unit tests for `extract_wikilinks` and backlink discovery in `src-tauri/src/vault.rs` | ✅ Done | Antigravity |
| 90 | Verify full test suite (`cargo test --lib`), `tsc`, and `npm run build` | ✅ Done | Antigravity |

---

## Task: Version 2.2.1 — Crystal Prism App Icon, Launch Fix & Linux Bundle Preparation

| # | Step | Status | Agent |
|---|------|--------|-------|
| 91 | Copy Downloads/Icon.png and generate multi-resolution desktop icons (.ico, .icns, .png for Windows & Linux) via npx @tauri-apps/cli icon | ✅ Done | Antigravity |
| 92 | Update Web UI logos in index.html, src/App.tsx (Hero, header, sidebar footer) and copy to public/icon.png | ✅ Done | Antigravity |
| 93 | Add convenience npm scripts (app:dev, app:build) in package.json to prevent Vite CLI entry errors | ✅ Done | Antigravity |
| 94 | Update version to 2.2.1 in Cargo.toml, tauri.conf.json, package.json, src/App.tsx, README.md, project_structure.md | ✅ Done | Antigravity |
| 95 | Verify .gitignore comprehensively excludes legacy files, temporary assets, and target artifacts | ✅ Done | Antigravity |
| 96 | Compile standalone release executable, run full test suite (cargo test --lib), and compile RELEASE_NOTES_v2.2.1.md | ✅ Done | Antigravity |

---

---

## Task: Version 2.2.2 — Strict Security Checks, Format Validation & Robust Entity Fallback

| # | Step | Status | Agent |
|---|------|--------|-------|
| 97 | Implement HTML tag stripping and publisher extraction in parse_rss_xml in src-tauri/src/search.rs | ✅ Done | Antigravity |
| 98 | Implement is_valid_candidate_entity, query-echo rejection & container phrase filtering in src-tauri/src/pipeline.rs | ✅ Done | Antigravity |
| 99 | Replace fake-entity fallback with honest zero-candidate reporting in src-tauri/src/pipeline.rs | ✅ Done | Antigravity |
| 100 | Implement IPC input validation, path traversal guards & format whitelisting in src-tauri/src/lib.rs & src-tauri/src/vault.rs | ✅ Done | Antigravity |
| 101 | Bump version to 2.2.2 in Cargo.toml, tauri.conf.json, package.json, App.tsx, README.md, project_structure.md | ✅ Done | Antigravity |
| 102 | Add unit tests for RSS HTML stripping, entity validation & path traversal, run cargo test --lib, tsc, npm run build | ✅ Done | Antigravity |

---

## Task: Version 2.2.3 — Persistent Working Memory, Candidate Continuity & Goal Validation

| # | Step | Status | Agent |
|---|------|--------|-------|
| 103 | Update rank_and_select_top_evidence with candidate entity relevance boost in src-tauri/src/search.rs | ✅ Done | Antigravity |
| 104 | Implement accumulated_discovered_entities working memory accumulator in src-tauri/src/pipeline.rs | ✅ Done | Antigravity |
| 105 | Update extractor prompt with candidate continuity and expand evidence context to 15 sources in src-tauri/src/pipeline.rs | ✅ Done | Antigravity |
| 106 | Implement Output Goal Validation fallback in src-tauri/src/pipeline.rs | ✅ Done | Antigravity |
| 107 | Bump version to 2.2.3 in Cargo.toml, tauri.conf.json, package.json, App.tsx, README.md, project_structure.md | ✅ Done | Antigravity |
| 108 | Add unit tests for working memory and goal validation, run cargo test --lib, tsc, npm run build, and verify release | ✅ Done | Antigravity |

---

## Task: Comprehensive Codebase Architecture & Function-by-Function Engineering Guide

| # | Step | Status | Agent |
|---|------|--------|-------|
| 109 | Generate single-file comprehensive codebase architecture, function breakdown & improvement roadmap (`CODEBASE_ARCHITECTURE_AND_FUNCTION_GUIDE.md`) | ✅ Done | Antigravity |

---

## Task: Version 2.2.4 — Map-Reduce Source Digest Stage & List Aggregator Ranking

| # | Step | Status | Agent |
|---|------|--------|-------|
| 110 | Update `rank_and_select_top_evidence` to boost list/calendar aggregators for list queries | ✅ Done | Antigravity |
| 111 | Implement `SourceDigest` data structure, `digest_source`, and JSON parsing in `pipeline.rs` | ✅ Done | Antigravity |
| 112 | Implement Semaphore-bounded concurrent fetch + Map-Reduce pipeline stage in `pipeline.rs` | ✅ Done | Antigravity |
| 113 | Unify sources count variable and eliminate hardcoded "not yet announced" fallback text | ✅ Done | Antigravity |
| 114 | Update `StructuredResearchData`, Markdown report builder & frontend SolidJS Source Digest cards in `App.tsx` & `App.css` | ✅ Done | Antigravity |
| 115 | Bump version to 2.2.4 across Cargo.toml, tauri.conf.json, package.json, App.tsx, README.md, project_structure.md | ✅ Done | Antigravity |
| 116 | Comprehensive test suite & verification (`cargo test --lib`, `npx tsc --noEmit`, `npm run build`) | ✅ Done | Antigravity |
| 117 | Persist full `output_json` in SQLite `history.rs` and update `get_session_details` to return complete `ResearchOutput` | ✅ Done | Antigravity |
| 118 | Implement frontend session caching and complete result restoration in `App.tsx` | ✅ Done | Antigravity |
| 119 | Create comprehensive `CHANGELOG.md` for v2.2.4 release | ✅ Done | Antigravity |
| 120 | Recompile `.exe`, run tests, and verify end-to-end | ✅ Done | Antigravity |

### Legend
- ⬜ Todo — Not started
- 🔄 In Progress (agent-name) — Being worked on
- ✅ Done — Completed and verified
- ❌ Blocked — Waiting on something
- ⏭️ Skipped — Not needed
