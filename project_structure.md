# 📂 Project Structure — Research Agent

> Quick-reference index for every file and directory.
> Use one-line descriptions to locate what you need fast.

---

```
research-agent/
│
├── main.py                  # Entry point — Gradio UI, system tray, config persistence
├── agent.py                 # Core logic — planner → executor → writer pipeline
├── tools.py                 # LangChain tools — search, visit, RSS, link extraction
├── prompts.py               # Prompt templates — planner, executor, writer roles
├── debug_tests.py           # Standalone debug & verification test harness
│
├── pyproject.toml           # Project metadata & dependencies (uv / pip)
├── uv.lock                  # Locked dependency versions (uv)
├── .python-version          # Pinned Python version (3.12)
│
├── config.json              # Runtime settings saved by the UI (git-ignored)
├── .gitignore               # Ignore rules — secrets, caches, temp reports
├── .env                     # Environment variables / API keys (git-ignored)
│
├── README.md                # User-facing docs — setup, features, screenshots
├── SECURITY.md              # Vulnerability reporting policy
├── CODE_OF_CONDUCT.md       # Contributor Covenant v2.0
├── LICENSE                  # MIT License
├── Agents.md                # Agent assistant guide for this project
├── todolist.md              # Step-by-step task tracker across sessions
├── project_structure.md     # ← You are here
│
├── icon.png                 # App icon (raster)
├── icon.svg                 # App icon (vector)
│
└── assets/                  # Screenshot gallery
    ├── run0.png             # UI screenshot — idle state
    ├── run1.png             # UI screenshot — planning phase
    ├── run2.png             # UI screenshot — execution phase
    └── run3.png             # UI screenshot — final report
```

---

## File Details

### Source Code

| File | Role | Key Responsibility |
|------|------|--------------------|
| [`main.py`](file:///f:/projects/research-agent/main.py) | **UI & Orchestration** | Gradio interface, system-tray icon, config save/load, depth presets, agent wrapper |
| [`agent.py`](file:///f:/projects/research-agent/agent.py) | **Agent Core** | 3-phase pipeline (Plan → Execute → Write), LLM factory, retry/backoff, search dispatch |
| [`tools.py`](file:///f:/projects/research-agent/tools.py) | **Tool Definitions** | `visit_webpage`, `read_rss_feed`, `extract_links`, SSRF protection, URL validation |
| [`prompts.py`](file:///f:/projects/research-agent/prompts.py) | **Prompt Engineering** | Date-aware prompt templates for planner, executor, and writer roles |
| [`debug_tests.py`](file:///f:/projects/research-agent/debug_tests.py) | **Testing & Verification** | Standalone test suite covering all tools, security checks, LLMs, and UI helpers |

### Config & Build

| File | Purpose |
|------|---------|
| [`pyproject.toml`](file:///f:/projects/research-agent/pyproject.toml) | Package name, version, Python constraint (≥3.11, <3.13), all dependencies |
| [`uv.lock`](file:///f:/projects/research-agent/uv.lock) | Reproducible lock file for the `uv` package manager |
| [`.python-version`](file:///f:/projects/research-agent/.python-version) | Pins runtime to Python 3.12 |
| [`.gitignore`](file:///f:/projects/research-agent/.gitignore) | Ignores secrets, caches, build artifacts, temp reports, OS cruft |

### Documentation & Community

| File | Purpose |
|------|---------|
| [`README.md`](file:///f:/projects/research-agent/README.md) | Getting started, features, screenshots, license |
| [`SECURITY.md`](file:///f:/projects/research-agent/SECURITY.md) | How to report vulnerabilities (GitHub Discussions) |
| [`CODE_OF_CONDUCT.md`](file:///f:/projects/research-agent/CODE_OF_CONDUCT.md) | Contributor Covenant v2.0 community standards |
| [`LICENSE`](file:///f:/projects/research-agent/LICENSE) | MIT License — Copyright (c) 2026 Pavan Kumar |
| [`Agents.md`](file:///f:/projects/research-agent/Agents.md) | Agent role definition and project knowledge base |
| [`todolist.md`](file:///f:/projects/research-agent/todolist.md) | Step-by-step task tracking, claimed tasks, and resumption history |

### Assets

| File | Content |
|------|---------|
| [`icon.png`](file:///f:/projects/research-agent/icon.png) / [`icon.svg`](file:///f:/projects/research-agent/icon.svg) | App icon used for system tray and branding |
| [`assets/run0–3.png`](file:///f:/projects/research-agent/assets) | Sequential screenshots showing the full research workflow |

---

## Dependency Map

```
main.py
 └── agent.py
      ├── tools.py        (visit_webpage, read_rss_feed, extract_links)
      └── prompts.py      (get_planner_prompt, get_executor_prompt, get_writer_prompt)
```

**External libraries:**

| Library | Used For |
|---------|----------|
| `gradio` | Web UI framework (v6.30+) |
| `pystray` + `pillow` | System-tray icon |
| `langchain` / `langchain-openai` | LLM orchestration (OpenAI-compatible) |
| `langchain-anthropic` | Claude models (optional) |
| `langchain-google-genai` | Gemini models (optional) |
| `beautifulsoup4` | HTML parsing for webpage visits |
| `feedparser` | RSS/Atom feed parsing |
| `requests` | HTTP client for web tools, Brave, Google CSE, SearXNG |
| `ddgs` | DuckDuckGo search backend (replaces deprecated `duckduckgo-search`) |
| `pyinstaller` | Packaging as standalone `.exe` |
