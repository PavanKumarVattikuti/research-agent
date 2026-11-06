# 🔬 Release Notes — Research Agent v2.2.1

> **Tag:** `v2.2.1`  
> **Release Name:** `Research Agent v2.2.1 — Crystal Prism Identity, Launch Stability & Evidence-First Architecture`  
> **Target Commits:** Strictly Local (Ready for Manual Human Verification & Push)  

---

## 🌟 Executive Summary

**Research Agent v2.2.1** brings a visual and architectural upgrade:
1. **New Visual Identity**: Replaces all legacy placeholders with the crystal prism icon across the Windows desktop binary (`.exe`), desktop installers (`.msi`, `setup.exe`), application taskbars, system tray, and inside the Web UI.
2. **Launch Reliability & CLI Fixes**: Resolves the Vite entry resolution error (`[UNRESOLVED_ENTRY] Cannot resolve entry module dev/index.html`) by introducing explicit `npm run app:dev` and `npm run app:build` scripts. Documents that standalone desktop execution requires the bundled production binary (`npm run app:build`) where assets are embedded directly into the native binary.
3. **Evidence-First Research Architecture**:
   - **Real Entity Extraction**: Filters out publisher websites (GamesRadar, SearXNG, IGN, CBS Sports, Bloomberg, etc.) so only actual subjects (games, models, products, missions) qualify as candidates.
   - **Canonical Deduplication**: Merges naming variants and Roman numerals (e.g., *The Witcher 4* $\leftrightarrow$ *The Witcher IV*).
   - **Claim-Level Evidence Accordion**: Each claim is tagged (`Verified`, `Corroborated`, `Uncertain`, `Conflicting`) with verbatim excerpts and publisher domain badges.
   - **Metrics Bar**: Target Quota requested, Confirmed milestones, In-Development projections, and total sources checked.
4. **Direct Obsidian Vault Export (`.VAULT`)**: A dedicated button saves research reports directly into your local Obsidian Vault directory under `Research Agent/<query>.md` with frontmatter and backlinks.
5. **Linux Desktop Target Support**: Tauri v2 configuration ready for Linux packaging (`deb`, `appimage`).
6. **Security & Dependency Audit (`glib` Advisory)**: Addressed the GitHub Dependabot advisory regarding `glib < 0.20.0`. Confirmed that on Windows, Tauri links against native Microsoft Edge WebView2 COM APIs and never executes `glib` at runtime.

---

## 📦 What's Changed

### 💎 Crystal Prism Visual Identity
- Extracted and bundled multi-resolution application icons:
  - `src-tauri/icons/icon.ico` (16x16 up to 256x256 multi-layer Windows icon)
  - `src-tauri/icons/icon.png` (512x512 high-resolution PNG)
  - `src-tauri/icons/icon.icns` (macOS / Linux high-dpi bundle)
  - `src-tauri/icons/128x128.png`, `src-tauri/icons/128x128@2x.png`, `src-tauri/icons/32x32.png`
  - Windows tile logos (`Square44x44`, `Square150x150`, `Square310x310`, `StoreLogo`)
- Updated Web UI:
  - Favicon link in `index.html` updated to `/icon.png`.
  - Brand header logo in top navigation bar now renders crystal icon.
  - Hero search start screen displays crystal prism with subtle teal ambient glow and hover depth.
  - Sidebar footer updated with `v2.2.1` version badge.

### 🛠️ Launch & Developer Experience
- **Problem**: Running `npm run build dev` failed because Vite interpreted `dev` as an entry directory (`dev/index.html`). Running debug executable `target/debug/research-agent.exe` outside a dev server terminated immediately because `http://localhost:1420` was not running.
- **Solution**:
  - Added dedicated npm scripts to `package.json`:
    - `npm run app:dev` $\rightarrow$ launches Vite dev server and opens live hot-reloading Tauri window.
    - `npm run app:build` $\rightarrow$ compiles Vite frontend and packages standalone native desktop `.exe` and installers with bundled assets.
  - Standalone release `.exe` requires no local web server or terminal running.

### 🐧 Linux Desktop Application Support
- Tauri bundle configuration has `targets: "all"` enabled.
- On Linux, builds produce:
  - Debian package: `.deb`
  - Portable AppImage: `.AppImage`
- Prerequisites for building on Linux:
  ```bash
  sudo apt update && sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
  ```

### 🔒 Security Audit: Dependabot `glib` Advisory
- **Advisory**: `glib >= 0.15.0, < 0.20.0` (VariantStrIter NULL pointer dereference in C FFI).
- **Audit Finding**: In Tauri v2, `glib` is a transitive dependency of `webkit2gtk 2.0.2`, gated under `#[cfg(target_os = "linux")]`. On Windows, Tauri uses Edge WebView2 COM bindings (`webview2-com-sys`). `glib` code is never compiled, linked, or executed in Windows desktop binaries.
- For Linux distributions, running `cargo update -p glib` when upstream `webkit2gtk` publishes compatible bindings resolves the advisory.

---

## 🧪 Verification & Test Results

- **Rust Unit Tests**: `23 passed; 0 failed; 0 ignored`
  - `test_atomic_claim_parsing` ✅
  - `test_bot_free_searches` ✅
  - `test_canonical_name_dedup_roman_numerals` ✅
  - `test_clean_report_generation` ✅
  - `test_evidence_first_serialization` ✅
  - `test_extract_verifiable_entities_rejects_publishers` ✅
  - `test_extract_verifiable_entities_structure` ✅
  - `test_haptic_event_types` ✅
  - `test_haptic_history_emission` ✅
  - `test_multi_mode_history_isolation` ✅
  - `test_multi_source_aggregation_resilience` ✅
  - `test_parse_evidence_first_response` ✅
  - `test_parse_llm_json_variants` ✅
  - `test_publisher_blacklist_filtering` ✅
  - `test_quota_fulfillment_computation` ✅
  - `test_roman_numeral_equivalency` ✅
  - `test_ssrf_blocks_internal_ips` ✅
  - `test_ssrf_blocks_localhost` ✅
  - `test_ssrf_blocks_non_http_schemes` ✅
  - `test_ssrf_permits_public_ips` ✅
  - `test_vault_search_offline_guarantee` ✅
- **TypeScript & Frontend Build**: Clean compilation (`vite build` completed without errors).

---

## 🚀 Manual GitHub Release Instructions (For Human User)

In accordance with strict safety guardrails, **no automated git push or GitHub release has been executed**. When you are ready to publish:

### Step 1: Review Git Status
```bash
git status
```

### Step 2: Stage and Commit Changes
```bash
git add .
git commit -m "feat(v2.2.1): crystal prism icon, launch stability, vault export, and evidence-first architecture"
```

### Step 3: Tag the Release
```bash
git tag -a v2.2.1 -m "Release v2.2.1: Crystal Prism Visual Identity & Evidence-First Architecture"
```

### Step 4: Push to GitHub (User Choice)
```bash
git push origin main
git push origin v2.2.1
```

### Step 5: Create GitHub Release
1. Open your repository releases page on GitHub: `https://github.com/<owner>/research-agent/releases/new`
2. Select tag `v2.2.1`.
3. Paste the contents of this `RELEASE_NOTES_v2.2.1.md` file into the description.
4. Attach the Windows installers from:
   - `src-tauri/target/release/bundle/nsis/Research-Agent_2.2.1_x64-setup.exe`
   - `src-tauri/target/release/bundle/msi/Research-Agent_2.2.1_x64_en-US.msi`
   - Or standalone binary: `src-tauri/target/release/research-agent.exe`
