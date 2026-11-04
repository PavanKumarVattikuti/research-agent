# 🚀 Release Notes — Research Agent v2.2.0

> **"Proof Over Hallucination."**  
> Major Release: Evidence-First Research Architecture & Honest Verification

Release **v2.2.0** represents a transformative architectural leap for Research Agent. In response to feedback on duplicate candidates, news websites incorrectly listed as entities, and repetitive AI prose, v2.2.0 completely replaces traditional AI report dumps with an **Evidence-First Research Architecture**.

---

## 🌟 Major Highlights & Architectural Upgrades

### 1. Evidence-First Presentation Workspace
- **Executive Summary & Honest Metrics Bar**:
  - Direct 2–3 sentence answer to the research query.
  - Interactive metric pills:
    - 🎯 **Target Quota**: Explicit number parsed from query (e.g. "top 10" $\rightarrow$ 10).
    - 🟢 **Verified**: Confirmed targets supported by official roadmap disclosures.
    - 🟡 **In-Development / Projected**: Preliminary roadmap estimates or targets.
    - 📚 **Sources**: Total external citations verified.
- **Real Entity Result Cards**:
  - Extracted items represent actual entities (e.g., games, models, missions, hardware silicon, software languages).
  - Includes attribute grid: Creator / Studio, Category / Genre, Expected Target, Platforms / Environment.
  - *"Why it qualifies"* concise rationale backed by evidence.
- **Claim-Level Evidence Accordions**:
  - Expandable claim verification cards for each entity.
  - 4 distinct verification badge states:
    - `Verified`: Primary official studio/publisher disclosures.
    - `Corroborated`: Multiple independent publications agreeing on facts.
    - `Uncertain`: Speculation, preliminary roadmap targets, or unconfirmed rumors.
    - `Conflicting`: Sources or industry reports disagree on dates or details.
  - Direct supporting quote excerpts, publisher names, and clickable source links with favicons.
- **Comparative Matrix**:
  - Structured side-by-side comparison table (`#`, `Candidate`, `Release Target`, `Developer / Creator`, `Platforms`, `Confidence`).
- **Research Limitations & Information Gaps**:
  - Honest disclosure of unconfirmed dates, missing studio disclosures, and quota discrepancies when fewer verified targets exist than requested.
- **View Toggle Pill**:
  - One-click toggle between **[⚡ Evidence View]** (interactive structured workspace) and **[📄 Report View]** (structured markdown).

### 2. Publisher Website Blacklist & Anti-Website Filter
- Implemented `is_disallowed_entity_name()` to strictly reject publisher domains, news outlets, and release calendar aggregators from ever being extracted as research candidates:
  - Excluded: *GamesRadar, SearXNG, IGN, CBS Sports, Forbes, Bloomberg, YouTube, Reddit, Hacker News, TechCrunch, The Verge, Polygon, Kotaku, Wired, Reuters, Tom's Hardware, PC Gamer, Eurogamer*, etc.

### 3. Canonical Deduplication Across Roman Numerals
- Implemented `canonical_entity_key()` to merge duplicate naming variants:
  - Normalizes Roman numerals and numbers (e.g. *The Witcher 4* and *The Witcher IV*, *Artemis III* and *The Artemis 3* merge seamlessly into a single canonical entry).

### 4. Non-Repetitive 6-Section Report Synthesis
- Output is generated in 6 clean, non-repetitive sections:
  1. Executive Summary & Overview
  2. Results (Numbered Entity Cards)
  3. Evidence & Verification (Claim-level excerpts and URLs)
  4. Comparison Matrix
  5. Analysis & Strategic Insights
  6. Research Limitations & Information Gaps

### 5. Multi-Domain Verification Harness
- Verified and regression-tested across 5 distinct domains:
  1. Video Games & Roadmaps (*Top 10 Upcoming Games in 2028*)
  2. Frontier AI Models (*Top 5 Frontier LLMs Q4 2026*)
  3. Datacenter Silicon & AI Accelerators (*Next-Gen AI Hardware Accelerators*)
  4. Deep Space Missions (*Upcoming Lunar Landings 2027-2030*)
  5. Systems Programming Languages (*Top Systems Languages for High Performance AI Engines*)

---

## 📦 Binary Downloads
- `Research-Agent_2.2.0_x64-setup.exe` (NSIS Installer)
- `Research-Agent_2.2.0_x64_en-US.msi` (Windows Installer Package)
- Standalone executable in `src-tauri/target/release/`
