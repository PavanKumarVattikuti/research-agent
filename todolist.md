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

### Legend
- ⬜ Todo — Not started
- 🔄 In Progress (agent-name) — Being worked on
- ✅ Done — Completed and verified
- ❌ Blocked — Waiting on something
- ⏭️ Skipped — Not needed
