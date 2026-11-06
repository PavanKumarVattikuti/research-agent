# 🔬 Release Notes — Research Agent v2.2.3

> **Tag:** `v2.2.3`  
> **Release Name:** `Research Agent v2.2.3 — Working Memory Pipeline, Candidate Continuity & Goal Validation`  
> **Target Commits:** Strictly Local (Ready for Manual Human Verification & Push)  

---

## 🌟 Executive Summary

**Research Agent v2.2.3** solves the critical information-continuity defect in Deep Search:
1. **Persistent Candidate Working Memory**: In prior versions, entities identified during iterative search and constraint verification (*"Need more: Gothic 5, Witcher 4, Elder Scrolls VI..."*) were only emitted to the live haptic UI stream and subsequently lost. In v2.2.3, all discovered candidate names are continuously accumulated into a persistent working memory pool (`accumulated_discovered_entities`).
2. **Candidate-Aware Evidence Selection (Expanded Context)**:
   - Expanded evidence selection from 5 to **15 sources**.
   - Added a `+15.0` title boost and `+8.0` snippet boost to articles mentioning discovered candidate entities, and penalized generic calendar aggregators (`-4.0`).
   - Ensures that detailed studio roadmap articles rank higher than generic calendar homepages.
3. **Candidate Continuity to Final Extractor Agent**:
   - The final entity extraction prompt is now explicitly supplied with the list of discovered candidate subjects from prior research.
   - Guiding the model to extract and cite specific details for the real entities identified during research.
4. **Output Goal Validation Layer**:
   - Automatically validates whether the final output satisfies the research goal. If the LLM extraction produced 0 entities despite candidates having been discovered in working memory, the pipeline grounds those candidates directly from the collected evidence corpus.
5. **Full Test Suite Verification**: All 30 unit tests pass, TypeScript compiler reports 0 errors, and standalone packages were compiled cleanly.

---

## 🧪 Verification & Test Results

- **Rust Unit Tests**: `30 passed; 0 failed; 0 ignored`
  - `search::tests::test_rank_and_select_top_evidence_boosts_discovered_entities` ✅
  - `pipeline::tests::test_goal_validation_grounds_discovered_entities_when_extractor_empty` ✅
  - `pipeline::tests::test_is_valid_candidate_entity_rejects_query_echoes_and_containers` ✅
  - `pipeline::tests::test_empty_llm_json_produces_honest_report_without_fake_entities` ✅
  - `search::tests::test_parse_rss_xml_strips_html_and_escaped_tags` ✅
  - `tests::test_sanitize_export_filename_path_traversal` ✅
  - `tests::test_validate_query_constraints` ✅
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
git commit -m "feat(v2.2.3): persistent working memory, candidate continuity, and output goal validation"
```

### Step 3: Tag the Release
```bash
git tag -a v2.2.3 -m "Release v2.2.3: Working Memory & Goal Validation"
```

### Step 4: Push to GitHub (User Choice)
```bash
git push origin main
git push origin v2.2.3
```

### Step 5: Create GitHub Release
1. Open your repository releases page on GitHub: `https://github.com/<owner>/research-agent/releases/new`
2. Select tag `v2.2.3`.
3. Paste the contents of this `RELEASE_NOTES_v2.2.3.md` file into the description.
4. Attach the Windows installers from:
   - `src-tauri/target/release/bundle/nsis/Research-Agent_2.2.3_x64-setup.exe`
   - `src-tauri/target/release/bundle/msi/Research-Agent_2.2.3_x64_en-US.msi`
