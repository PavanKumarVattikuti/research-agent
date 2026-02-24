"""
Research agent core logic.

Key improvements over original:
- Retry with exponential backoff for rate-limit / quota errors (helps Gemini free tier).
- Provider-aware delay instead of hardcoded time.sleep(4) everywhere.
- Accumulated research notes passed into each executor call (avoids redundant searches).
- JSON-based tool action parsing with regex fallback.
- Temp report file is properly cleaned up via atexit.
- Dead `search_web` import removed.
- Planner JSON is extracted safely even if the LLM wraps it in markdown fences.
- Step count is now a parameter (wired up from the UI slider).
"""

import atexit
import json
import os
import re
import tempfile
import time
import datetime
from pathlib import Path

import gradio as gr
import requests

from langchain_core.messages import HumanMessage, SystemMessage
from langchain_openai import ChatOpenAI

try:
    from langchain_anthropic import ChatAnthropic
except ImportError:
    ChatAnthropic = None

try:
    from langchain_google_genai import ChatGoogleGenerativeAI
except ImportError:
    ChatGoogleGenerativeAI = None

from tools import read_rss_feed, visit_webpage, extract_links
from prompts import get_planner_prompt, get_executor_prompt, get_writer_prompt

# ---------------------------------------------------------------------------
# Temp-file cleanup
# ---------------------------------------------------------------------------

_TEMP_FILES: list[str] = []


def _cleanup_temp_files() -> None:
    for path in _TEMP_FILES:
        try:
            Path(path).unlink(missing_ok=True)
        except Exception:
            pass


atexit.register(_cleanup_temp_files)

# ---------------------------------------------------------------------------
# Provider-aware delays (seconds between LLM calls)
# Free-tier providers need breathing room; local/paid don't.
# ---------------------------------------------------------------------------

_PROVIDER_DELAY: dict[str, float] = {
    "Google Gemini": 6.0,   # free tier: ~15 RPM
    "Local (LM Studio)": 0.5,
    "OpenAI": 0.5,
    "Anthropic": 0.5,
}


def _inter_call_delay(provider: str) -> None:
    delay = _PROVIDER_DELAY.get(provider, 1.0)
    if delay > 0:
        time.sleep(delay)


# ---------------------------------------------------------------------------
# Model dropdown helper
# ---------------------------------------------------------------------------

def update_model_dropdown(provider: str, base_url: str):
    if provider == "Local (LM Studio)":
        try:
            url = f"{base_url.rstrip('/')}/models"
            response = requests.get(url, timeout=2)
            if response.status_code == 200:
                models = [m["id"] for m in response.json().get("data", [])]
                if models:
                    return gr.update(choices=models, value=models[0])
        except Exception:
            pass
        return gr.update(choices=["local-model"], value="local-model")

    elif provider == "OpenAI":
        choices = ["gpt-4o-mini", "gpt-4o", "gpt-4-turbo", "gpt-3.5-turbo"]
        return gr.update(choices=choices, value="gpt-4o-mini")

    elif provider == "Anthropic":
        choices = ["claude-3-5-sonnet-latest", "claude-3-opus-latest", "claude-3-haiku-20240307"]
        return gr.update(choices=choices, value="claude-3-5-sonnet-latest")

    elif provider == "Google Gemini":
        choices = ["gemini-2.0-flash", "gemini-1.5-flash", "gemini-1.5-pro", "gemini-2.5-pro", "gemini-2.5-flash"]
        # Default to flash — much higher free quota than pro
        return gr.update(choices=choices, value="gemini-2.0-flash")

    return gr.update(choices=[], value="")


# ---------------------------------------------------------------------------
# LLM factory
# ---------------------------------------------------------------------------

def build_llm(provider: str, api_key: str, model_name: str, base_url: str):
    provider = (provider or "Local (LM Studio)").strip()

    if provider == "OpenAI":
        key = (api_key or os.getenv("OPENAI_API_KEY", "")).strip()
        model = (model_name or os.getenv("OPENAI_MODEL", "gpt-4o-mini")).strip()
        return ChatOpenAI(model=model, api_key=key, temperature=0, timeout=120)

    if provider == "Anthropic":
        if ChatAnthropic is None:
            raise ImportError("langchain-anthropic is not installed.")
        key = (api_key or os.getenv("ANTHROPIC_API_KEY", "")).strip()
        model = (model_name or os.getenv("ANTHROPIC_MODEL", "claude-3-5-sonnet-latest")).strip()
        return ChatAnthropic(model=model, api_key=key, temperature=0, timeout=120)

    if provider == "Google Gemini":
        if ChatGoogleGenerativeAI is None:
            raise ImportError("langchain-google-genai is not installed.")
        key = (api_key or os.getenv("GOOGLE_API_KEY", "")).strip()
        model = (model_name or "gemini-2.0-flash").strip()
        return ChatGoogleGenerativeAI(model=model, google_api_key=key, temperature=0, timeout=120)

    # Local (LM Studio) or any custom OpenAI-compatible endpoint
    base = (base_url or os.getenv("LMSTUDIO_BASE_URL", "http://localhost:1234/v1")).strip()
    model = (model_name or os.getenv("LMSTUDIO_MODEL", "local-model")).strip()
    key = (api_key or os.getenv("LMSTUDIO_API_KEY", "lm-studio")).strip()
    return ChatOpenAI(base_url=base, api_key=key, model=model, temperature=0, max_tokens=2000, timeout=600)


# ---------------------------------------------------------------------------
# LLM invocation with retry / backoff
# ---------------------------------------------------------------------------

def _invoke_with_retry(llm, messages, max_retries: int = 3):
    """Call llm.invoke with exponential backoff on rate-limit errors."""
    last_exc: Exception | None = None
    for attempt in range(max_retries):
        try:
            return llm.invoke(messages)
        except Exception as exc:
            last_exc = exc
            err_str = str(exc).lower()
            is_rate_limit = any(
                kw in err_str for kw in ("429", "quota", "rate limit", "resource exhausted", "too many requests")
            )
            if is_rate_limit and attempt < max_retries - 1:
                wait = 2 ** attempt * 12  # 12s, 24s, 48s
                raise RetryableError(f"Rate limit hit — will retry in {wait}s…", wait=wait) from exc
            raise
    raise last_exc  # type: ignore[misc]


class RetryableError(Exception):
    def __init__(self, message: str, wait: float = 12):
        super().__init__(message)
        self.wait = wait


# ---------------------------------------------------------------------------
# Search dispatcher
# ---------------------------------------------------------------------------

def perform_dynamic_search(
    query: str,
    search_engine: str,
    searx_url: str,
    brave_key: str,
    google_api: str,
    google_cse: str,
) -> str:
    try:
        if search_engine == "Brave Search":
            from langchain_community.tools import BraveSearch
            return BraveSearch.from_api_key(api_key=brave_key, search_kwargs={"count": 5}).run(query)

        elif search_engine == "Google Custom Search":
            from langchain_community.utilities import GoogleSearchAPIWrapper
            return GoogleSearchAPIWrapper(google_api_key=google_api, google_cse_id=google_cse).run(query)

        elif search_engine == "SearXNG":
            from langchain_community.utilities import SearxSearchWrapper
            return SearxSearchWrapper(searx_host=searx_url).run(query)

        else:
            from langchain_community.tools import DuckDuckGoSearchRun
            return DuckDuckGoSearchRun().run(query)

    except Exception as exc:
        return f"Search Error ({search_engine}): {exc}"


# ---------------------------------------------------------------------------
# Tool action parser
# ---------------------------------------------------------------------------

_ACTION_RE = re.compile(
    r"TOOL\s*:\s*(SEARCH|VISIT|RSS|LINKS)\s+(.+)",
    re.IGNORECASE,
)
_RESULT_RE = re.compile(r"RESULT\s*:\s*(.+)", re.IGNORECASE | re.DOTALL)


def _parse_action(text: str) -> tuple[str, str] | None:
    """
    Try to extract (TOOL_NAME, argument) from the LLM response.
    Returns None if only a RESULT: line is found (caller uses text directly).
    """
    # First try JSON in case we migrated to structured output later
    try:
        data = json.loads(text)
        if isinstance(data, dict) and "tool" in data and "arg" in data:
            return data["tool"].upper(), data["arg"].strip()
    except (json.JSONDecodeError, ValueError):
        pass

    for line in text.splitlines():
        m = _ACTION_RE.search(line)
        if m:
            tool_name = m.group(1).upper()
            arg = m.group(2).strip().strip("\"'")
            return tool_name, arg

    return None  # RESULT or unparseable


# ---------------------------------------------------------------------------
# Main agent entry point
# ---------------------------------------------------------------------------

def run_research_agent(
    user_query: str,
    provider: str,
    api_key: str,
    model_name: str,
    base_url: str,
    search_engine: str,
    searx_url: str,
    brave_key: str,
    google_api: str,
    google_cse: str,
    max_steps: int = 5,
):
    """Generator that yields incremental Markdown output as research progresses."""

    history = f"🧠 **Researching:** {user_query}\n\n"
    yield history

    # --- Build LLM ---
    try:
        llm = build_llm(provider, api_key, model_name, base_url)
    except Exception as exc:
        yield history + f"❌ **Configuration error:** {exc}"
        return

    today = datetime.datetime.now().strftime("%A, %B %d, %Y")
    system_ctx = f"You are a live research agent. Today's current date is {today}."

    # ── Phase 1: Planning ──────────────────────────────────────────────────
    history += "📋 **Phase 1 — Planning…**\n"
    yield history

    plan_messages = [
        SystemMessage(content=system_ctx + "\n\n" + get_planner_prompt(max_steps)),
        HumanMessage(content=user_query),
    ]

    plan: list[str] = []
    for attempt in range(3):
        try:
            plan_response = _invoke_with_retry(llm, plan_messages)
            raw = plan_response.content.strip()
            # Strip markdown fences if present
            raw = re.sub(r"^```[a-zA-Z]*\n?", "", raw)
            raw = re.sub(r"\n?```$", "", raw)
            plan = json.loads(raw.strip())
            if not isinstance(plan, list):
                raise ValueError("Plan is not a list.")
            break
        except RetryableError as exc:
            history += f"  ⏳ Rate limit — waiting {exc.wait}s…\n"
            yield history
            time.sleep(exc.wait)
        except Exception as exc:
            history += f"  ⚠️ Planning error (attempt {attempt + 1}/3): {exc}\n"
            yield history
            _inter_call_delay(provider)
    else:
        plan = [f"Search for {user_query}"]
        history += "  ⚠️ Using fallback single-step plan.\n"
        yield history

    plan_text = "\n".join(f"  {i + 1}. {step}" for i, step in enumerate(plan))
    history += f"✅ **Plan ({len(plan)} steps):**\n{plan_text}\n\n"
    yield history

    # ── Phase 2: Execution ─────────────────────────────────────────────────
    history += "🕵️ **Phase 2 — Executing…**\n"
    yield history

    research_notes: list[str] = []

    for idx, step in enumerate(plan, start=1):
        history += f"\n> **Step {idx}/{len(plan)}:** {step}\n"
        yield history

        # Pass accumulated context so the LLM can skip redundant searches
        already_found = (
            "\n\n".join(research_notes[-3:]) if research_notes else "Nothing yet."
        )
        executor_messages = [
            SystemMessage(content=system_ctx + "\n\n" + get_executor_prompt()),
            HumanMessage(
                content=(
                    f"Current Task: {step}\n\n"
                    f"Already found (do NOT repeat these searches):\n{already_found}"
                )
            ),
        ]

        # Retry loop for this step
        action_response: str | None = None
        for attempt in range(3):
            try:
                action_response = _invoke_with_retry(llm, executor_messages).content
                break
            except RetryableError as exc:
                history += f"  ⏳ Rate limit — waiting {exc.wait}s…\n"
                yield history
                time.sleep(exc.wait)
            except Exception as api_err:
                history += f"  ❌ API error (attempt {attempt + 1}/3): {api_err}\n"
                yield history
                _inter_call_delay(provider)

        if action_response is None:
            history += "  ⏭️ Skipping step after repeated failures.\n"
            yield history
            continue

        # --- Execute tool ---
        parsed = _parse_action(action_response)
        result: str

        if parsed:
            tool_name, tool_arg = parsed

            if tool_name == "SEARCH":
                history += f"  🔍 Searching ({search_engine}): `{tool_arg}`\n"
                yield history
                result = perform_dynamic_search(
                    tool_arg, search_engine, searx_url, brave_key, google_api, google_cse
                )

            elif tool_name == "VISIT":
                history += f"  🌐 Visiting: `{tool_arg}`\n"
                yield history
                try:
                    result = visit_webpage.invoke(tool_arg)
                except Exception as exc:
                    result = f"Failed to visit page: {exc}"

            elif tool_name == "RSS":
                history += f"  📰 Reading RSS: `{tool_arg}`\n"
                yield history
                try:
                    result = read_rss_feed.invoke(tool_arg)
                except Exception as exc:
                    result = f"Failed to read feed: {exc}"

            elif tool_name == "LINKS":
                history += f"  🔗 Extracting links from: `{tool_arg}`\n"
                yield history
                try:
                    result = extract_links.invoke(tool_arg)
                except Exception as exc:
                    result = f"Failed to extract links: {exc}"

            else:
                result = action_response

        else:
            # RESULT: or unparseable — treat the LLM output itself as the finding
            result_match = _RESULT_RE.search(action_response)
            result = result_match.group(1).strip() if result_match else action_response.strip()

        research_notes.append(f"Task: {step}\nFindings: {result[:1_000]}")
        history += "  ✅ Info collected.\n"
        yield history

        _inter_call_delay(provider)

    # ── Phase 3: Writing ───────────────────────────────────────────────────
    history += "\n✍️ **Phase 3 — Writing final report…**\n"
    yield history

    all_notes = "\n\n---\n\n".join(research_notes)

    writer_messages = [
        SystemMessage(content=system_ctx + "\n\n" + get_writer_prompt()),
        HumanMessage(content=f"Original Request: {user_query}\n\nResearch Notes:\n{all_notes}"),
    ]

    final_content: str = ""
    for attempt in range(3):
        try:
            final_response = _invoke_with_retry(llm, writer_messages)
            final_content = final_response.content
            break
        except RetryableError as exc:
            history += f"  ⏳ Rate limit — waiting {exc.wait}s…\n"
            yield history
            time.sleep(exc.wait)
        except Exception as exc:
            history += f"  ❌ Writer error (attempt {attempt + 1}/3): {exc}\n"
            yield history
            _inter_call_delay(provider)
    else:
        final_content = "⚠️ Could not generate a final report due to repeated errors."

    # Save report to a temp file (cleaned up on exit)
    try:
        with tempfile.NamedTemporaryFile(
            delete=False, mode="w", suffix=".md", encoding="utf-8"
        ) as f:
            f.write(f"# Research Report: {user_query}\n\n{final_content}")
            _TEMP_FILES.append(f.name)
    except Exception:
        pass  # Non-critical

    yield history + f"\n---\n## 📝 Final Report\n\n{final_content}"
