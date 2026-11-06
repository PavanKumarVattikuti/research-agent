# 🔬 Research Agent (v2.2.4)

> **"Proof Over Hallucination."**  
> The Open-Source, Offline-First Perplexity Alternative.

Research Agent is a lightweight, memory-efficient native desktop application built with **Rust**, **Tauri v2**, and **SolidJS**. It eliminates LLM hallucinations and overthinking through an **Evidence-First Research Architecture**: entities are extracted as distinct verifiable subjects (never substituted with publisher websites or news aggregators), duplicates are canonically merged across Roman numerals and naming variants, claims are verified at the atomic level against primary and secondary sources, and output is presented via honest metric cards and structured comparative matrices.

---

## ✨ What's New in v2.2.4 (Map-Reduce Source Digest Stage & Deep Webpage Reader)

- **📖 Full Webpage Deep Reader & Source Digest Stage**:
  - Implements a parallel Map-Reduce stage between evidence ranking and candidate extraction.
  - Concurrently fetches full clean webpage text (up to 6,000 characters) via `fetch_clean_webpage` with 6-second timeout and snippet fallback.
  - Runs single-responsibility micro-tasks per source to extract an atomic direct summary (<= 80 words), candidate entities, and source relevance score (`high`, `medium`, `low`).
  - Concurrency is strictly bounded by a 3-permit Semaphore (`tokio::sync::Semaphore::new(3)`) to protect local GPUs and inference engines.
- **🔄 Synthesized Reduce Stage & Honest Dynamic Summaries**:
  - Automatically combines per-source digests into a unified answer with `[id]` citations and ranked candidate disclosures.
  - Eliminates hardcoded fallback text (*"formal roadmaps have not been announced"*); the report always reflects synthesized evidence directly.
  - Source-by-Source Summaries cards and markdown sections display full source provenance and disclosed entities.
- **📊 List Query Aggregator Ranking Boost**:
  - Removes the `-4.0` aggregator penalty for list/count-targeted queries (e.g. "top 10 games in 2028") and boosts release trackers and calendars (`+3.5`).
- **🛡️ Source Count Consistency**:
  - Unifies the `total_sources_count` metric across the entire pipeline, report headers, and limitations.

---

## ✨ What's New in v2.2.3 (Working Memory Pipeline & Goal Validation)

- **🧠 Persistent Candidate Working Memory**:
  - Eliminates information amnesia: candidates identified during intermediate search iterations and constraint validation passes (*"Need more: Gothic 5, Witcher 4, Elder Scrolls VI..."*) are preserved in working memory (`accumulated_discovered_entities`).
- **🎯 Candidate-Aware Evidence Selection (Expanded Context)**:
  - Boosts articles mentioning discovered candidate entities (`+15.0` in title, `+8.0` in snippet) and expands evidence context from 5 to 15 sources, ensuring specific studio disclosures rank above generic calendar aggregators.
- **🔗 Candidate Continuity to Final Extractor Agent**:
  - Injects discovered candidate subjects directly into the final extraction prompt, guiding the model to extract and cite specific details for the real entities identified during research.
- **🛡️ Output Goal Validation Layer**:
  - Automatically validates whether the final output satisfies the research goal. If the LLM extraction produced 0 entities despite candidates having been discovered in working memory, the pipeline grounds those candidates directly from the collected evidence corpus.

---

## ✨ What's New in v2.2.2 (Security Hardening, Format Validation & Zero-Hallucination Fallback)

- **🛡️ Strict Candidate Format & Quality Validation**:
  - Rejects generic container phrases (e.g., *"Upcoming Games"*, *"Most Anticipated"*, *"Release Calendar"*).
  - Rejects query-echo candidates that simply duplicate the user's search prompt.
  - Strictly blocks names containing HTML tags, unescaped entities, or URLs.
- **🚫 Zero-Hallucination Fallback**:
  - Completely removed the legacy heuristic that fabricated candidate entities with fake statuses (*"Roadmap Target"*) and placeholder developers (*"Identified in Evidence"*) when LLM extraction timed out or returned empty.
  - Honestly reports zero confirmed candidate targets and presents retrieved source excerpts directly without inventing cards.
- **🧹 RSS & Search Feed HTML Stripping**:
  - Strips all HTML formatting (`<a href...>`, `<font color...>`, `<p>`) and unescapes entities from RSS and search titles/descriptions.
- **🔒 IPC Input Hardening & Path Traversal Prevention**:
  - Guarded `save_markdown_export` against directory traversal (`../`, `/`, `\`, `:`) and enforced `.md`, `.html`, and `.txt` file extension whitelisting.
  - Guarded `save_vault_export` with path canonicalization and blocked saving to operating system root directories.
  - Enforced query length ceilings (2–500 chars) and prohibited control character injections.

---

## ✨ What's New in v2.2.1 (Icon Refresh, Launch Reliability & Obsidian Vault Export)

- **💎 Crystal Prism Visual Identity**:
  - Replaced legacy placeholders with the crystal prism app icon across Windows desktop taskbars, alt-tab switchers, NSIS/MSI installers, dock icons, and Web UI.
- **📁 Direct Obsidian Vault Export (.VAULT)**:
  - Added 1-click export saving verified research reports directly into your local Obsidian Vault directory under Research Agent/.
- **🛠️ Launch Reliability & Simplified Dev Workflow**:
  - Added dedicated npm scripts: 
pm run app:dev for live desktop development and 
pm run app:build for production bundling.
  - Eliminated ambiguity with Vite's client-side build commands.
- **🐧 Linux Desktop Target Readiness**:
  - Configured Tauri v2 bundle targets for Linux (deb, ppimage) and documented packaging requirements.
- **🔒 Security & Dependency Audit**:
  - Audited transitive glib dependency: confirmed safe and uninvoked on Windows (where native Edge WebView2 COM APIs are used instead of GTK).

---

## ✨ What's New in v2.2.0 (Evidence-First Architecture)

- **🏆 Evidence-First Research Workspace**:
  - Direct 2–3 sentence executive summary upfront without repetitive filler paragraphs.
  - Interactive **Metrics Bar**: Target Quota requested, Confirmed milestones, In-Development projections, and total sources checked.
  - Real **Entity Cards**: Only actual products, titles, missions, models, or languages are extracted (never news aggregators or calendars).
  - Multi-attribute grids: Developer/Studio, Category/Genre, Expected Release, and Target Platforms.
  - *"Why it qualifies"* concise rationale backed by explicit citations.
  - Expandable **Claim-Level Evidence Accordion**: Each claim is tagged as `Verified` (primary proof), `Corroborated` (independent outlets), `Uncertain` (rumor/estimate), or `Conflicting` (disputed), complete with exact quoted excerpts and publisher links.
  - Side-by-side **Comparative Matrix** with confidence ratings.
  - Honest **Research Limitations & Gaps**: Explicit callouts when public roadmap data is unconfirmed or fails to meet the requested quota (no fake entries invented to fill counts).
  - Single-click **[⚡ Evidence] / [📄 Report]** view toggle.
- **🛡️ Publisher Blacklist & Anti-Website Filter**:
  - Automatically filters out publisher sites (GamesRadar, SearXNG, IGN, CBS Sports, Forbes, Bloomberg, TechCrunch, Tom's Hardware, etc.) from candidate entities.
- **🔗 Canonical Deduplication Across Roman Numerals**:
  - Normalizes entities across naming variations and Roman numerals (e.g., *The Witcher 4* and *The Witcher IV*, *Artemis III* and *The Artemis 3* merge into a single canonical entry).
- **📝 Clean 6-Section Non-Repetitive Report Generation**:
  - Generates zero-redundancy reports covering Overview, Results, Evidence, Comparison, Strategic Analysis, and Limitations.

---

## ✨ Features

- **🛡️ Proof Over Hallucination**: Every claim and candidate is verified against cited public web proofs or local notes.
- **🧩 Word-Level Query Decomposition (Deep Search Flagship)**:
  - Splits input queries into individual words and interprets their semantic role in parallel micro-tasks (e.g., `top` $\rightarrow$ ranked, `10` $\rightarrow$ minimum count constraint, `2028` $\rightarrow$ future year target).
  - Synthesizes combined relational context into a unified research goal with extracted hard constraints.
  - Cross-checks collected evidence against constraints in an automated validation loop (up to 3 iterations) to ensure complete answers.
- **⚡ Live Haptic Progress Stream & Preserved Trace**:
  - Live milestone streaming during research without timestamp clutter.
  - Permanent step-by-step execution trace preserved in the **Process Disclosure Accordion** (`Show Steps` $\rightarrow$ `Live Trace`) even after report generation.
- **🎨 4 Smart Pre-Templated Layout Renderers**:
  - `ranked_list`: Numbered cards with rank badges (`1`, `2`, `3`), status tags (`Confirmed`, `Roadmap Target`, `Projected`), and concise highlights.
  - `news_feed`: Clean vertical card list with source domain favicons, summaries, and clickable proof links.
  - `comparison_table`: High-contrast side-by-side comparative matrices.
  - `flashcards`: Interactive 3D flip-cards with front/back tap-to-reveal interactions for educational and Q&A content.
  - `default_prose`: High-density Obsidian dark-mode prose typography.
- **📰 Bot-Free Multi-Source Search**:
  - Queries Web engines (DuckDuckGo, SearXNG, Brave) concurrently with **Google News RSS** and **Hacker News Algolia** to bypass anti-bot blocks.
- **⚡ 4 Investigation Modes**:
  1. `Deep Search`: Parallel word decomposition, constraint extraction, iterative validation loop, and candidate extraction.
  2. `Instant Search`: Sub-second 1-pass verification with cited proof points.
  3. `Compare Search`: Multi-angle comparative matrix across competing candidates.
  4. `Offline Search`: 100% offline querying of local Obsidian / Markdown vaults (`[[Note]]` citations, zero cloud leaks).
- **🔌 Multi-Protocol Local & Cloud LLM Support**:
  - OpenAI-compatible (LM Studio, vLLM, LocalAI, OpenAI, Groq)
  - Anthropic (Claude 3.5 Sonnet, Haiku)
  - Ollama native API
  - Custom REST endpoints
  - Integrated 1-click **[Test]** button with live latency badges (ms) and loaded model detection.
- **🔒 SSRF Protection**: Outbound requests strictly block loopback, link-local, private LAN subnets, and cloud metadata IPs (`169.254.169.254`, `localhost`, `127.0.0.1`, `::1`).
- **🗄️ Embedded SQLite History**: Local persistence of research sessions, layers, and citations with zero cloud sync.
- **📥 1-Click Native Markdown Export**: Direct file save to your system's `Downloads/` directory with clean sanitized file naming.
- **🪶 Ultra-Lightweight Binary**: Native standalone Windows `.exe` is only **~8.6 MB** (compared to >200MB Python bundles) and uses minimal RAM (~35MB idle).

---

## 🚀 Quick Start

### Prerequisites
- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (1.80+)

### Development
```bash
# Install frontend dependencies
npm install

# Run in desktop development mode (hot-reloading UI + Rust backend)
npm run tauri dev
```

### Production Build
```bash
# Build standalone native executable and installer
npm run tauri build
```

Compiled bundles are output to:
- Standalone `.exe`: `src-tauri/target/release/research-agent.exe`
- NSIS installer: `src-tauri/target/release/bundle/nsis/Research-Agent_2.2.4_x64-setup.exe`
- MSI installer: `src-tauri/target/release/bundle/msi/Research-Agent_2.2.4_x64_en-US.msi`

---

## 🏛️ Architecture & Verification

Read [`Agents.md`](./Agents.md) for architectural guardrails and [`project_structure.md`](./project_structure.md) for file indexing.

To run the Rust verification test suite:
```bash
cargo test --lib --manifest-path src-tauri/Cargo.toml
```

---

## 📄 License
MIT License — Copyright (c) 2026 Pavan Kumar
