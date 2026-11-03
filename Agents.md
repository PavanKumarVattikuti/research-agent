# 🤖 Agents.md — AI Assistant Guide for Research Agent

> This document defines the role, context, and guardrails for any AI agent
> (Copilot, Antigravity, ChatGPT, etc.) assisting with this project.

---

## Navigation

📂 **See [`project_structure.md`](./project_structure.md) for the full project map, file index, and dependency graph.**

---

## Agent Role

You are a **code assistant and pair-programmer** for the **Research Agent** project.
Your responsibilities:

1. **Understand before changing** — Read `project_structure.md` first to orient yourself.
2. **Preserve architecture** — The codebase follows a clean 4-file split: `main.py` (UI), `agent.py` (logic), `tools.py` (tools), `prompts.py` (prompts). Keep it that way.
3. **Respect conventions** — Python 3.12, `uv` package manager, type hints, docstrings at module/function level.
4. **Guard security** — Never weaken SSRF protections, never expose API keys, never bypass URL validation.
5. **Test locally** — The app runs via `uv run python main.py` on `http://127.0.0.1:7860`. Verify changes work end-to-end.

---

## About the Project

**Research Agent** is an autonomous AI research assistant that runs silently from the system tray. It:

- Takes a natural-language research query from the user
- **Plans** a multi-step investigation (Phase 1 — Planner LLM)
- **Executes** each step using web tools (Phase 2 — Executor LLM + tools)
- **Writes** a polished Markdown report with sources (Phase 3 — Writer LLM)
- Delivers everything through a **Gradio web interface** at `localhost:7860`

| Attribute | Value |
|-----------|-------|
| **Author** | Pavan Kumar |
| **License** | MIT |
| **Python** | ≥ 3.11, < 3.13 (pinned to 3.12) |
| **Package Manager** | `uv` |
| **UI Framework** | Gradio (Soft theme) |
| **LLM Framework** | LangChain |
| **Repo** | [github.com/PavanKumarVattikuti/research-agent](https://github.com/PavanKumarVattikuti/research-agent) |

---

## Features

### 🧠 Multi-Step Reasoning Pipeline
The agent follows a **Plan → Execute → Write** pipeline. The Planner LLM decomposes a user query into up to 10 discrete research steps (JSON array). The Executor LLM picks the right tool for each step. The Writer LLM synthesizes all findings into a cited report.

### 🔌 Multi-Provider LLM Support
Supports 4 LLM backends — switch in the UI dropdown:
- **Local (LM Studio)** — Free, private, any GGUF model via OpenAI-compatible API
- **OpenAI** — GPT-4o, GPT-4o-mini, GPT-4-turbo, GPT-3.5-turbo
- **Anthropic** — Claude 3.5 Sonnet, Claude 3 Opus, Claude 3 Haiku
- **Google Gemini** — Gemini 2.5 Pro/Flash, Gemini 2.0 Flash, Gemini 1.5 Pro/Flash

### 🔍 Multi-Engine Search
4 search backends, selectable at runtime:
- **DuckDuckGo** (default — zero config)
- **Brave Search** (API key)
- **Google Custom Search** (API key + CSE ID)
- **SearXNG** (self-hosted URL)

### 🛠️ Dynamic Tool Use
The Executor LLM chooses from 4 tools per step:
- `SEARCH` — Web search via the selected engine
- `VISIT` — Fetch and clean a webpage (HTML → plain text)
- `RSS` — Read headlines from an RSS/Atom feed
- `LINKS` — Extract hyperlinks from a page for follow-up visits

### 🎛️ Research Depth Control
- **Presets**: Quick (3 steps), Standard (5), Deep (8)
- **Fine-tune slider**: 1–10 steps

### 💾 Persistent Configuration
Settings (provider, model, API keys, search engine, depth) are saved to `config.json` and restored on restart.

### 🖥️ System Tray Integration
Runs as a background app via `pystray`. Right-click the tray icon to open the UI or quit.

### 📋 Report Export
- **Copy** to clipboard (JS Clipboard API)
- **Download** as `.md` file
- Temp report files are auto-cleaned on exit via `atexit`

### 📡 Streaming Output
The agent **yields** incremental Markdown as each phase/step completes — the user sees real-time progress in the Gradio UI.

---

## Security Methods

### SSRF (Server-Side Request Forgery) Protection
All outbound HTTP tools (`visit_webpage`, `read_rss_feed`, `extract_links`) pass through **`_validate_external_url()`** before making any request:

| Check | What It Blocks |
|-------|---------------|
| **Scheme whitelist** | Only `http://` and `https://` — blocks `file://`, `ftp://`, `gopher://`, etc. |
| **Blocked hosts list** | Explicit deny list: `localhost`, `127.0.0.1`, `::1`, `0.0.0.0`, `host.docker.internal`, `metadata.google.internal` |
| **Private IP detection** | Uses `ipaddress.ip_address()` to reject private, loopback, link-local, reserved, and multicast IPs |
| **Empty hostname guard** | Rejects URLs with missing or empty hostnames |
| **IPv6 bracket stripping** | Handles `[::1]`-style IPv6 literals correctly |

> **Location:** [`tools.py` L24–L68](file:///f:/projects/research-agent/tools.py#L24-L68)

### API Key Protection
- API keys are entered as `type="password"` fields in the UI (masked)
- Keys are stored in `config.json` which is **git-ignored** (`.gitignore` line 31)
- `.env` files are also git-ignored
- No keys are ever logged, printed, or included in research output

### Content Length Guard
- `visit_webpage` truncates page content to **8,000 characters** (`MAX_CONTENT_CHARS`) — prevents memory exhaustion from massive pages

### Graceful Import Guarding
- `langchain-anthropic` and `langchain-google-genai` are imported inside `try/except` — the app runs even if optional providers are missing

---

## Performance Methods

### Retry with Exponential Backoff
**`_invoke_with_retry()`** catches rate-limit errors (HTTP 429, quota exhausted, etc.) and retries with exponential delays: 12s → 24s → 48s. Critical for free-tier Gemini usage.

> **Location:** [`agent.py` L149–L165](file:///f:/projects/research-agent/agent.py#L149-L165)

### Provider-Aware Inter-Call Delays
Different LLM providers have different rate limits. The agent applies **calibrated delays** between calls:

| Provider | Delay |
|----------|-------|
| Google Gemini (free tier) | 6.0s |
| Local / OpenAI / Anthropic | 0.5s |

> **Location:** [`agent.py` L65–L77](file:///f:/projects/research-agent/agent.py#L65-L77)

### Accumulated Context Passing
Each executor step receives the **last 3 research notes** as context. This prevents redundant searches and lets the LLM skip steps it can already answer from prior findings.

> **Location:** [`agent.py` L321–L324](file:///f:/projects/research-agent/agent.py#L321-L324)

### Streaming Generator Architecture
`run_research_agent()` is a **Python generator** that `yield`s incremental output. The Gradio UI consumes it in real-time — no blocking, no buffering the entire report.

### HTML Boilerplate Removal
`visit_webpage` strips `<script>`, `<style>`, `<nav>`, `<footer>`, `<aside>`, `<header>`, `<form>`, `<noscript>` tags before extracting text — reduces noise, keeps token usage low.

> **Location:** [`tools.py` L128–L138](file:///f:/projects/research-agent/tools.py#L128-L138)

### JSON Parsing with Regex Fallback
The planner output parser tries `json.loads()` first, then falls back to regex extraction. This handles LLMs that wrap JSON in markdown fences.

> **Location:** [`agent.py` L218–L238](file:///f:/projects/research-agent/agent.py#L218-L238)

### Temp File Cleanup
Report files are created in the system temp directory and registered with `atexit` for automatic cleanup on process exit.

> **Location:** [`agent.py` L47–L58](file:///f:/projects/research-agent/agent.py#L47-L58)

### Daemon Thread Architecture
The Gradio server runs in a **daemon thread** so the main thread can own the system-tray event loop. Clean exit is handled via `os._exit(0)` from the tray menu.

---

## Task Tracking Workflow

All changes to this project follow a **todolist-driven** workflow:

1. **Plan** — Break the task into numbered steps in [`todolist.md`](./todolist.md)
2. **Claim** — Mark a step as `🔄 In Progress (agent-name)` before starting
3. **Execute** — Work on one step at a time
4. **Update** — Mark the step `✅ Done` when complete, then move to next
5. **Resume** — Any agent can pick up where the last one left off

This prevents conflicts when multiple agents or sessions work on the project.

📋 **See [`todolist.md`](./todolist.md) for current tasks and progress.**

---

## Quick Commands

```bash
# Install / sync dependencies
uv sync

# Run the app
uv run python main.py

# Build standalone exe (Windows)
uv run pyinstaller main.py --name "Research-Agent" --onefile --windowed --icon=icon.png --collect-all gradio --collect-all gradio_client --collect-all safehttpx --collect-all groovy --clean
```
