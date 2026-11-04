# 📂 Project Structure — Research Agent (v2.2.0)

> Quick-reference index for every file and directory in Version 2.2.0.
> Evidence-First Research Architecture with claim-level verification and structured data.

---

```
research-agent/
│
├── src/                          # SolidJS Frontend (Linear / Raycast / Perplexity Workspace)
│   ├── App.tsx                   # Main reactive UI — hero search, activity timeline, sources drawer, grouped history
│   ├── App.css                   # Premium workspace tokens, Geist typography scale, status badges, obsidian prose
│   ├── index.tsx                 # SolidJS DOM bootstrap
│   └── assets/                   # Frontend assets & icons
│
├── src-tauri/                    # Rust Tauri v2 Backend Core
│   ├── Cargo.toml                # Rust dependencies (Tauri 2, reqwest, rusqlite, scraper, walkdir)
│   ├── tauri.conf.json           # Tauri v2 configuration (window size, CSP, bundle)
│   ├── capabilities/             # Tauri v2 window security capabilities
│   └── src/                      # Rust modules
│       ├── main.rs               # Desktop executable entry point
│       ├── lib.rs                # Tauri command registry (test_llm, test_search, execute_research, delete_history_item)
│       ├── pipeline.rs           # Relational intent disambiguation, domain grounding, status tagging, multi-angle reasoning
│       ├── llm.rs                # Multi-protocol micro-task LLM client (OpenAI, Anthropic, Ollama, Custom REST)
│       ├── tester.rs             # Live health tester for LLM & Search endpoints (latency & models)
│       ├── search.rs             # Multi-source engine (DuckDuckGo, SearXNG, Brave, Google News RSS, Hacker News) + domain filter
│       ├── vault.rs              # 100% Offline Obsidian & Markdown vault scanner & paragraph extractor
│       ├── history.rs            # Embedded SQLite history store with in-memory test isolation & welcome seed
│       └── ssrf.rs               # SSRF protection & private/loopback IP validation
│
├── package.json                  # Node.js dependencies (Vite, SolidJS, Tauri CLI, Tailwind v4, marked)
├── vite.config.ts                # Vite configuration with Solid & Tailwind plugins
│
├── legacy/                       # Preserved Legacy Versions
│   └── v1_backup/                # Fully preserved v1 Python implementation
│       ├── main.py               # v1 Gradio entry point & system tray
│       ├── agent.py              # v1 LangChain pipeline
│       ├── tools.py              # v1 Python web tools & SSRF protection
│       ├── prompts.py            # v1 Prompt templates
│       ├── debug_tests.py        # v1 Verification suite
│       ├── pyproject.toml        # v1 Python dependencies (uv)
│       └── uv.lock               # v1 Lockfile
│
├── README.md                     # User-facing documentation & quick start
├── SECURITY.md                   # Security & SSRF disclosure policy
├── CODE_OF_CONDUCT.md            # Contributor standards
├── LICENSE                       # MIT License
├── Agents.md                     # AI assistant guide & architecture rules
├── todolist.md                   # Step-by-step task tracking
└── project_structure.md          # ← You are here
```

---

## Backend Modules (`src-tauri/src/`)

| File | Purpose |
|------|---------|
| [`main.rs`](file:///f:/projects/research-agent/src-tauri/src/main.rs) | Launches Tauri application runtime. |
| [`lib.rs`](file:///f:/projects/research-agent/src-tauri/src/lib.rs) | Exposes `invoke` IPC commands and events to SolidJS frontend. |
| [`pipeline.rs`](file:///f:/projects/research-agent/src-tauri/src/pipeline.rs) | Executes relational intent disambiguation, multi-angle evidence evaluation, and candidate extraction with status tags. |
| [`llm.rs`](file:///f:/projects/research-agent/src-tauri/src/llm.rs) | Streams and invokes single-task LLM queries across 4 protocols. |
| [`tester.rs`](file:///f:/projects/research-agent/src-tauri/src/tester.rs) | Tests endpoints, measures latency (ms), and detects available models. |
| [`search.rs`](file:///f:/projects/research-agent/src-tauri/src/search.rs) | Queries primary search + Google News RSS + Hacker News Algolia, cleans HTML, and filters domain relevance. |
| [`vault.rs`](file:///f:/projects/research-agent/src-tauri/src/vault.rs) | Scans Obsidian Markdown vaults offline and extracts scored excerpts. |
| [`history.rs`](file:///f:/projects/research-agent/src-tauri/src/history.rs) | Saves and queries research sessions in embedded SQLite database with in-memory test isolation. |
| [`ssrf.rs`](file:///f:/projects/research-agent/src-tauri/src/ssrf.rs) | Rejects private, loopback, link-local, and non-HTTP(S) addresses. |

---

## Frontend Modules (`src/`)

| File | Purpose |
|------|---------|
| [`App.tsx`](file:///f:/projects/research-agent/src/App.tsx) | Complete SolidJS UI: research workspace, activity timeline, sources drawer, grouped history, Enter-to-run. |
| [`App.css`](file:///f:/projects/research-agent/src/App.css) | Premium visual tokens, Geist/Inter scale, layered charcoal palette, status badges, markdown tables. |
| [`index.tsx`](file:///f:/projects/research-agent/src/index.tsx) | Mounts SolidJS application into DOM. |
