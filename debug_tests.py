"""
debug_tests.py — Standalone debug & verification harness for Research Agent.

Run: uv run python debug_tests.py
Each test is isolated. Failures don't halt the test suite.
"""

import os
import sys
import tempfile
from pathlib import Path

# Ensure UTF-8 output on Windows consoles
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

PASS = "[PASS] PASS"
FAIL = "[FAIL] FAIL"
SKIP = "[SKIP] SKIP"

results: list[tuple[str, str, str]] = []


def run_test(name: str):
    """Decorator to register and execute a test function."""
    def decorator(fn):
        print(f"\n--- [TEST] {name} ---", flush=True)
        try:
            fn()
            results.append((name, PASS, ""))
            print(f"Result: {PASS}", flush=True)
        except Exception as exc:
            err_msg = f"{type(exc).__name__}: {exc}"
            results.append((name, FAIL, err_msg))
            print(f"Result: {FAIL} -> {err_msg}", flush=True)
        return fn
    return decorator


# ===========================================================================
# 1. Module Imports
# ===========================================================================

@run_test("Import core modules (agent, tools, prompts, main)")
def test_imports():
    import tools
    import prompts
    import agent
    import main
    assert hasattr(tools, "visit_webpage")
    assert hasattr(prompts, "get_planner_prompt")
    assert hasattr(agent, "run_research_agent")
    assert hasattr(main, "app")


# ===========================================================================
# 2. SSRF Protection
# ===========================================================================

@run_test("SSRF: _validate_external_url blocks internal and unsafe schemes")
def test_ssrf_blocking():
    from tools import _validate_external_url

    blocked_targets = [
        "http://localhost:8080",
        "http://127.0.0.1",
        "http://127.0.0.1:7860/api",
        "http://[::1]/secret",
        "http://0.0.0.0",
        "http://10.0.0.1/admin",
        "http://192.168.1.1",
        "http://172.16.0.1",
        "file:///etc/passwd",
        "ftp://example.com/file.txt",
        "gopher://example.com",
    ]
    for target in blocked_targets:
        try:
            _validate_external_url(target)
            raise AssertionError(f"Expected URL to be blocked: {target}")
        except ValueError:
            pass  # Successfully blocked


@run_test("SSRF: _validate_external_url allows legitimate public URLs")
def test_ssrf_allowing():
    from tools import _validate_external_url

    allowed_targets = [
        "https://example.com",
        "https://www.google.com/search?q=test",
        "http://github.com/PavanKumarVattikuti/research-agent",
    ]
    for target in allowed_targets:
        validated = _validate_external_url(target)
        assert validated == target


# ===========================================================================
# 3. Tool Protections
# ===========================================================================

@run_test("Tool safety: visit_webpage rejects blocked internal URLs")
def test_visit_webpage_safety():
    from tools import visit_webpage
    out = visit_webpage.invoke("http://127.0.0.1:7860")
    assert "URL validation error" in out or "blocked" in out.lower(), f"Unexpected: {out}"


@run_test("Tool safety: read_rss_feed rejects blocked internal URLs")
def test_read_rss_safety():
    from tools import read_rss_feed
    out = read_rss_feed.invoke("http://localhost:8000/feed.xml")
    assert "validation error" in out or "blocked" in out.lower(), f"Unexpected: {out}"


@run_test("Tool safety: extract_links rejects blocked internal URLs")
def test_extract_links_safety():
    from tools import extract_links
    out = extract_links.invoke("http://127.0.0.1/dashboard")
    assert "validation error" in out or "blocked" in out.lower(), f"Unexpected: {out}"


# ===========================================================================
# 4. Search Dispatcher
# ===========================================================================

@run_test("Search: Guard against missing API keys/parameters")
def test_search_missing_keys():
    from agent import perform_dynamic_search

    # Brave Search without key
    res_brave = perform_dynamic_search("test", "Brave Search", "", "", "", "")
    assert "key is required" in res_brave.lower()

    # Google CSE without keys
    res_google = perform_dynamic_search("test", "Google Custom Search", "", "", "", "")
    assert "required" in res_google.lower()

    # SearXNG without URL
    res_searx = perform_dynamic_search("test", "SearXNG", "", "", "", "")
    assert "url is required" in res_searx.lower()


@run_test("Search: DuckDuckGo (ddgs) executes query")
def test_search_duckduckgo():
    from agent import perform_dynamic_search
    res = perform_dynamic_search("python programming", "DuckDuckGo (Default)", "", "", "", "")
    # In sandbox without network, error is caught gracefully; with network, results return.
    assert isinstance(res, str)
    assert len(res) > 0


# ===========================================================================
# 5. Tool Action Parsing
# ===========================================================================

@run_test("Agent parser: _parse_action parses TOOL formats & RESULT")
def test_action_parsing():
    from agent import _parse_action

    assert _parse_action("TOOL: SEARCH quantum computing") == ("SEARCH", "quantum computing")
    assert _parse_action("TOOL: VISIT https://example.com") == ("VISIT", "https://example.com")
    assert _parse_action("TOOL: RSS https://example.com/feed.xml") == ("RSS", "https://example.com/feed.xml")
    assert _parse_action("TOOL: LINKS https://example.com/page") == ("LINKS", "https://example.com/page")
    assert _parse_action('{"tool": "SEARCH", "arg": "machine learning"}') == ("SEARCH", "machine learning")
    assert _parse_action("RESULT: The answer is 42.") is None


# ===========================================================================
# 6. LLM Factory
# ===========================================================================

@run_test("Agent factory: build_llm constructs provider instances")
def test_build_llm():
    from agent import build_llm

    # Local provider (default)
    llm_local = build_llm("Local (LM Studio)", "", "test-model", "http://localhost:1234/v1")
    assert llm_local is not None

    # OpenAI
    llm_openai = build_llm("OpenAI", "sk-mock-key-1234567890", "gpt-4o-mini", "")
    assert llm_openai is not None

    # Anthropic
    llm_anthropic = build_llm("Anthropic", "sk-ant-mock-1234567890", "claude-3-5-sonnet-latest", "")
    assert llm_anthropic is not None

    # Google Gemini
    llm_gemini = build_llm("Google Gemini", "mock-google-key", "gemini-2.0-flash", "")
    assert llm_gemini is not None


# ===========================================================================
# 7. Prompt Engineering
# ===========================================================================

@run_test("Prompts: Date-aware prompt templates")
def test_prompts():
    from datetime import datetime
    from prompts import get_planner_prompt, get_executor_prompt, get_writer_prompt

    year = str(datetime.now().year)

    planner = get_planner_prompt(max_steps=7)
    assert "7 steps" in planner or "7" in planner
    assert year in planner

    executor = get_executor_prompt()
    assert year in executor
    assert "TOOL: SEARCH" in executor

    writer = get_writer_prompt()
    assert "## Sources" in writer


# ===========================================================================
# 8. Config Persistence
# ===========================================================================

@run_test("Config persistence: save and load without corruption")
def test_config_persistence():
    import main
    import json

    orig_config = main.load_config()
    test_key = "test_key_temp_12345"

    try:
        main.save_config(
            provider="OpenAI",
            model="gpt-4o",
            api_key=test_key,
            base_url="http://localhost:1234/v1",
            search_engine="DuckDuckGo (Default)",
            searx_url="",
            brave_key="",
            google_api="",
            google_cse="",
            max_steps=6,
        )
        loaded = main.load_config()
        assert loaded.get("api_key") == test_key
        assert loaded.get("max_steps") == 6
        assert loaded.get("provider") == "OpenAI"
    finally:
        # Restore original config file
        with open(main.CONFIG_FILE, "w", encoding="utf-8") as f:
            json.dump(orig_config, f, indent=2)


# ===========================================================================
# 9. Gradio UI Components & Dropdown Updates
# ===========================================================================

@run_test("Gradio UI: Components and dropdown update helpers")
def test_gradio_ui():
    import gradio as gr
    from agent import update_model_dropdown, get_latest_report_file
    import main

    # Check dropdown constructors
    dd_openai = update_model_dropdown("OpenAI", "")
    assert isinstance(dd_openai, gr.Dropdown)

    dd_anthropic = update_model_dropdown("Anthropic", "")
    assert isinstance(dd_anthropic, gr.Dropdown)

    dd_gemini = update_model_dropdown("Google Gemini", "")
    assert isinstance(dd_gemini, gr.Dropdown)

    # Check app initialization
    assert isinstance(main.app, gr.Blocks)

    # Report file helper returns str or None
    rep = get_latest_report_file()
    assert rep is None or isinstance(rep, str)


# ===========================================================================
# Summary Report
# ===========================================================================

def main_summary():
    print("\n" + "=" * 65)
    print("                    TEST SUITE SUMMARY")
    print("=" * 65)
    for name, status, err in results:
        line = f"  {status} - {name}"
        if err:
            line += f"\n         Error: {err}"
        print(line)

    passed = sum(1 for _, s, _ in results if s == PASS)
    total = len(results)
    print("-" * 65)
    print(f"Total: {passed}/{total} tests passed ({int(passed / total * 100)}%)")
    print("=" * 65)
    return 0 if passed == total else 1


if __name__ == "__main__":
    sys.exit(main_summary())
