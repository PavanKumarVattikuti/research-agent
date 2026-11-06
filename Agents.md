# 🤖 Agents.md — AI Assistant Guide for Research Agent (v2.0)

> This document defines the role, context, and guardrails for any AI agent
> (Copilot, Antigravity, ChatGPT, etc.) assisting with this project.

---

## Navigation

📂 **See [`project_structure.md`](./project_structure.md) for the full project map, file index, and dependency graph.**

---

## Tagline & Core Mission

> **"Proof Over Hallucination."**
> The Open-Source, Offline-First Perplexity Alternative.

The primary objective is to stop LLM overthinking and produce verified answers with cited public or local proofs.
To eliminate hallucinations, the architecture follows the **Single Micro-Task Principle**:
- Never give an LLM multiple responsibilities at once.
- Constraint keywords (e.g., "this week", "open-source", "best") are decomposed and verified individually in atomic micro-prompts.
- Research iterates through recursive 5-Whys layers with early stopping as soon as ground truth proof is confirmed.
- 5 candidate answers are extracted and re-ranked using familiarity, relatedness, and benchmark evidence.

---

## Technology Stack

| Component | Technology | Role |
|-----------|------------|------|
| **Core & Backend** | Rust 1.99+ | High-performance, memory-safe backend engine |
| **Desktop Framework** | Tauri v2 | Lightweight native webview bridge with minimal RAM |
| **Frontend UI** | SolidJS + TypeScript | High-speed fine-grained reactive interface |
| **Styling** | Tailwind CSS v4 | Obsidian dark mode palette (`#121314`), teal accent (`#14b8a6`) |
| **Embedded DB** | SQLite (`rusqlite`) | Local search history, sessions, layers, and citations |
| **Offline Vault** | Walkdir + Scraper | Native Obsidian / Markdown folder scanner and extractor |

---

## 4 Investigation Modes

1. **`Deep Search`** — 5-Whys recursive subtask investigation tree, keyword constraint deconstruction, and 5-candidate ranking.
2. **`Instant Search`** — Fast 1-pass verification (~2-3s) with 3 cited proof points.
3. **`Compare Search`** — Multi-angle comparative matrix across competing candidates.
4. **`Offline Search`** — 100% offline querying of local Obsidian / Markdown vaults (`[[Note]]` citations, zero cloud leaks).

---

## Security Guardrails

### SSRF (Server-Side Request Forgery) Protection
All outbound web fetch and search operations pass through `ssrf::validate_external_url()`:
- Whitelist only `http://` and `https://`.
- Explicitly block `localhost`, `127.0.0.1`, `::1`, `0.0.0.0`, `host.docker.internal`, `metadata.google.internal`, and `169.254.169.254`.
- Block private IPv4 ranges (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, loopback, link-local, multicast).
- Block private IPv6 ranges (unique local `fc00::/7`, link-local `fe80::/10`).

### Secrets & Privacy
- API keys are masked in UI and stored locally.
- In `Offline Search` mode, zero outbound network requests are made.

---

## Agent Pair-Programming Workflow

1. **Check [`todolist.md`](./todolist.md)** before starting work.
2. Mark your current task as `🔄 In Progress (agent-name)`.
3. Complete and verify work with unit/integration tests (`cargo test --lib`).
4. Mark task as `✅ Done`.
5. Maintain legacy compatibility: never destroy or overwrite files in `legacy/v1_backup/`.
6. **Strict Git & GitHub Rule (ABSOLUTE)**:
   - **NEVER run `git push` or push releases/tags to GitHub under any circumstances.**
   - All Git commits, tags, and remote pushes are strictly reserved for the human user.
   - AI agents must NEVER execute `git push`, `gh release create`, or create/push tags.
   - Keep all changes strictly local on the machine. Provide clear verification instructions and let the user decide when and what to push manually.
