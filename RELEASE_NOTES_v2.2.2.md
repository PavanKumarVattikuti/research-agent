# 🔬 Release Notes — Research Agent v2.2.2

> **Tag:** `v2.2.2`  
> **Release Name:** `Research Agent v2.2.2 — Security Hardening, Format Validation & Zero-Hallucination Fallback`  
> **Target Commits:** Strictly Local (Ready for Manual Human Verification & Push)  

---

## 🌟 Executive Summary

**Research Agent v2.2.2** resolves critical quality and security defects uncovered in research query edge cases:
1. **Strict Candidate Entity Format Validation**: Eliminates pseudo-entities such as article headlines (*"Upcoming Games 2028"*, *"Most anticipated games of 2028"*, *"Release Calendar"*). Candidates must strictly represent verifiable products, models, missions, or titles. Generic container phrases, HTML tags, URLs, and prompt query echoes are systematically rejected.
2. **Zero-Hallucination Fallback**: Removed the legacy fallback heuristic that converted raw search result headlines into candidate cards with fabricated statuses (*"Roadmap Target"*) and placeholder developers (*"Identified in Evidence"*). When LLM extraction is unavailable or public data does not contain verified targets, the system honestly reports 0 candidates and displays the raw retrieved evidence sources directly.
3. **RSS & Search Feed HTML Stripping**: Cleaned up the Google News RSS and SearXNG ingestion pipelines by removing all raw and escaped HTML tags (`<a href...>`, `<font color...>`, `<p>`) and unescaping XML entities before passing snippets and titles to the research pipeline.
4. **IPC Security Hardening & Path Traversal Prevention**:
   - `save_markdown_export`: Rejects path traversal attempts (`../`, `/`, `\`, `:`) and strictly whitelists `.md`, `.html`, and `.txt` file extensions with a 15MB content size ceiling.
   - `save_vault_export`: Added path canonicalization and blocked saving to operating system root directories (`C:\`, `C:\Windows`, `/`, etc.).
   - `execute_research`: Validates query lengths (2–500 chars), blocks control characters (< 32), and enforces allowed search modes and LLM protocols.
5. **Full Test Suite Expansion**: Added unit tests for RSS HTML stripping, candidate entity format validation, zero-candidate honest reporting, query constraints, and path traversal rejection (28 passed, 0 failed).

---

## 🧪 Verification & Test Results

- **Rust Unit Tests**: `28 passed; 0 failed; 0 ignored`
  - `tests::test_validate_query_constraints` ✅
  - `tests::test_sanitize_export_filename_path_traversal` ✅
  - `pipeline::tests::test_is_valid_candidate_entity_rejects_query_echoes_and_containers` ✅
  - `pipeline::tests::test_empty_llm_json_produces_honest_report_without_fake_entities` ✅
  - `search::tests::test_parse_rss_xml_strips_html_and_escaped_tags` ✅
  - `vault::tests::test_export_to_obsidian_vault` ✅
  - `ssrf::tests::test_blocks_localhost` ✅
  - `ssrf::tests::test_allows_public_urls` ✅
  - All 5 domain verification tests (Games, AI, Hardware, Space, Languages) ✅
- **TypeScript Check**: `0 errors` (`npx tsc --noEmit`).
- **Frontend Build**: `0 errors` (`npm run build`).

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
git commit -m "feat(v2.2.2): security hardening, format validation, HTML stripping, and zero-hallucination fallback"
```

### Step 3: Tag the Release
```bash
git tag -a v2.2.2 -m "Release v2.2.2: Security Hardening & Zero-Hallucination Fallback"
```

### Step 4: Push to GitHub (User Choice)
```bash
git push origin main
git push origin v2.2.2
```

### Step 5: Create GitHub Release
1. Open your repository releases page on GitHub: `https://github.com/<owner>/research-agent/releases/new`
2. Select tag `v2.2.2`.
3. Paste the contents of this `RELEASE_NOTES_v2.2.2.md` file into the description.
4. Attach the Windows installers from:
   - `src-tauri/target/release/bundle/nsis/Research-Agent_2.2.2_x64-setup.exe`
   - `src-tauri/target/release/bundle/msi/Research-Agent_2.2.2_x64_en-US.msi`
