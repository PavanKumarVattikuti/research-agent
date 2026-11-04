# 🚀 Release v2.1.5 — The Native Desktop Architecture Overhaul

> **"Proof Over Hallucination"** — Research Agent v2.1.5 is a complete ground-up rewrite from the legacy Python prototype into a native, lightning-fast desktop application powered by **Rust 1.99+**, **Tauri v2**, and **SolidJS**.

---

## ⚡ Key Highlights: Legacy v1.x vs. Native v2.1.5

| Metric / Capability | Legacy v1.x (Python / Gradio) | Native v2.1.5 (Rust / Tauri / SolidJS) |
|---|---|---|
| **Binary Size** | ~220 MB (PyInstaller bundle) | **~8.6 MB** (native compiled `.exe`) |
| **Idle RAM Usage** | ~280 MB - 350 MB | **~35 MB** (native webview bridge) |
| **Startup Time** | ~4.5 - 7.0 seconds | **< 400 milliseconds** |
| **Query Deconstruction** | Monolithic prompt | **Word-level parallel micro-tasks** |
| **Search Retrieval** | Single DuckDuckGo scraper (fragile) | **Concurrent Multi-Source (Web + Google News RSS + Hacker News Algolia)** |
| **Anti-Bot Resistance** | Often blocked by CAPTCHAs | **Zero-CAPTCHA RSS & Algolia fallbacks** |
| **Presentation** | Raw markdown text dump | **4 dynamic layouts: Ranked Cards, News Feed, Comparison Table, 3D Flashcards** |
| **Local Privacy** | None (ephemeral) | **Embedded SQLite history + 100% Offline Obsidian Vault search** |

---

## 🌟 What's New in v2.1.5

### 1. 🧩 Single Micro-Task Word Decomposition
- Complex queries like *"Top 10 games coming in 2028"* are parsed word-by-word into parallel LLM micro-tasks.
- Extracts individual requirements:
  - `top` $\rightarrow$ ordered by anticipation / reviews
  - `10` $\rightarrow$ hard constraint: minimum 10 verified items
  - `games` $\rightarrow$ entity domain: digital video game software releases
  - `2028` $\rightarrow$ release calendar constraint
- Automated constraint validation loop checks evidence completeness and refines queries up to 3 iterations if more evidence is required.

### 2. ⚡ Live Haptic Feedback Stream & Preserved Trace
- Streams real-time pipeline actions with clean text (no millisecond or timestamp clutter).
- Visual hierarchy: newest actions at 100% opacity with a pulsing teal cursor; older entries fade softly.
- **Permanent Post-Synthesis Trace**: After the answer renders, the complete step-by-step trace remains permanently accessible in the **Process Disclosure Accordion** (`Show Steps` $\rightarrow$ `Live Trace` tab).

### 3. 🎨 4 Smart Pre-Templated Layout Renderers
The synthesis engine auto-selects and applies custom UI templates:
- **Ranked List Cards**: Numbered rank badges (`1`, `2`, `3`), title, developer disclosures, and status pills (`Confirmed`, `Roadmap Target`, `Projected`).
- **News Feed**: Modern headline cards with source domain favicons, bulleted takeaways, and verified proof citations.
- **Comparison Table**: High-contrast side-by-side matrices for multi-candidate queries.
- **3D Flashcards**: Interactive flip-cards with front/back tap-to-reveal interactions.
- **Prose Workspace**: Obsidian dark-mode typography for deep narrative analyses.

### 4. 📰 Anti-Bot Concurrent Search Engine
- Outbound search queries run concurrently across:
  - DuckDuckGo / SearXNG / Brave
  - **Google News RSS** (fresh publisher disclosures, zero bot blocking)
  - **Hacker News Algolia** (developer and engineering discussions)
- Hardened with SSRF protection blocking private subnets, loopback, and cloud metadata endpoints (`169.254.169.254`).

### 5. 🗄️ Offline Vault & Local Privacy
- Embedded SQLite database stores search sessions locally with zero cloud telemetry.
- Query local Obsidian and Markdown vaults with `[[Note]]` citations without any network connection.
- 1-click **Export to Downloads** automatically writes clean `.md` markdown files and styled `.html` web presentations with cards directly to your Downloads folder.

---

## 📦 Download & Installation

Download the Windows installer or standalone executable from the assets below:
- **`Research-Agent_2.1.5_x64-setup.exe`** (Standard Windows Installer)
- **`research-agent.exe`** (Portable standalone executable, zero install required)

*SHA256 checksums are attached with the release assets.*
