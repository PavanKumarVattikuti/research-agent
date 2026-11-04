# 🔬 Research Agent (v2.1.5)

> **"Proof Over Hallucination."**  
> The Open-Source, Offline-First Perplexity Alternative.

Research Agent is a lightweight, memory-efficient native desktop application built with **Rust**, **Tauri v2**, and **SolidJS**. It eliminates LLM overthinking by enforcing the **Single Micro-Task Principle**: constraint keywords are decomposed and verified independently in atomic micro-prompts, with iterative constraint validation, multi-source evidence cross-referencing, and intelligent pre-templated output layouts.

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
- NSIS installer: `src-tauri/target/release/bundle/nsis/Research-Agent_2.1.5_x64-setup.exe`
- MSI installer: `src-tauri/target/release/bundle/msi/Research-Agent_2.1.5_x64_en-US.msi`

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
