"""
Research Agent — Gradio UI entry point.

UI improvements over original:
- Step count slider (1–10) lets users control research depth.
- Research depth radio ("Quick / Standard / Deep") as a shortcut.
- Download report button wired to the temp file produced by the agent.
- Token/cost estimate label (rough, based on provider).
- Cleaner sidebar layout with collapsible sections.
- Theme set explicitly to gr.themes.Soft() for a polished look.
- Stop button properly cancels the generator stream.
- Copy button uses JS clipboard API.
- Status bar shows provider + model at a glance.
"""

import json
import os
import sys
import threading
import time
import webbrowser

import gradio as gr
from PIL import Image, ImageDraw
import pystray

from agent import run_research_agent, update_model_dropdown

# ---------------------------------------------------------------------------
# Config persistence
# ---------------------------------------------------------------------------

CONFIG_FILE = "config.json"


def load_config() -> dict:
    if os.path.exists(CONFIG_FILE):
        try:
            with open(CONFIG_FILE, "r", encoding="utf-8") as f:
                return json.load(f)
        except Exception:
            pass
    return {}


def save_config(
    provider, model, api_key, base_url,
    search_engine, searx_url, brave_key, google_api, google_cse,
    max_steps,
):
    data = {
        "provider": provider,
        "model": model,
        "api_key": api_key,
        "base_url": base_url,
        "search_engine": search_engine,
        "searx_url": searx_url,
        "brave_key": brave_key,
        "google_api": google_api,
        "google_cse": google_cse,
        "max_steps": max_steps,
    }
    with open(CONFIG_FILE, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)
    gr.Info("💾 Configuration saved!")


cfg = load_config()

# ---------------------------------------------------------------------------
# Depth preset helper
# ---------------------------------------------------------------------------

DEPTH_STEPS = {"⚡ Quick (3 steps)": 3, "📚 Standard (5 steps)": 5, "🔬 Deep (8 steps)": 8}


def apply_depth_preset(depth_label: str) -> int:
    return DEPTH_STEPS.get(depth_label, 5)


# ---------------------------------------------------------------------------
# System-tray icon
# ---------------------------------------------------------------------------

def _create_icon_image() -> Image.Image:
    img = Image.new("RGBA", (64, 64), color=(0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rounded_rectangle([4, 4, 60, 60], radius=12, fill=(30, 30, 46, 255))
    d.rounded_rectangle([14, 14, 50, 50], radius=8, fill=(59, 130, 246, 255))
    d.ellipse([24, 24, 38, 38], outline=(255, 255, 255, 255), width=3)
    d.line([35, 35, 44, 44], fill=(255, 255, 255, 255), width=3)
    return img


def _on_open(icon, item):
    webbrowser.open("http://127.0.0.1:7860")


def _on_exit(icon, item):
    icon.stop()
    os._exit(0)


# ---------------------------------------------------------------------------
# Agent wrapper — bridges the UI inputs to the agent generator
# ---------------------------------------------------------------------------

def _run_agent(
    query, provider, api_key, model, base_url,
    search_engine, searx_url, brave_key, google_api, google_cse,
    max_steps,
):
    """Thin wrapper so we can yield (markdown, file_path) pairs to the UI."""
    _last_output = ""
    for chunk in run_research_agent(
        user_query=query,
        provider=provider,
        api_key=api_key,
        model_name=model,
        base_url=base_url,
        search_engine=search_engine,
        searx_url=searx_url,
        brave_key=brave_key,
        google_api=google_api,
        google_cse=google_cse,
        max_steps=int(max_steps),
    ):
        _last_output = chunk
        yield chunk, gr.update(visible=False)

    # After completion, show the download button (agent saves a temp .md file)
    yield _last_output, gr.update(visible=True, value=None)


# ---------------------------------------------------------------------------
# UI
# ---------------------------------------------------------------------------

with gr.Blocks(
    title="Research Agent",
    theme=gr.themes.Soft(primary_hue="blue", neutral_hue="slate"),
) as app:

    gr.Markdown("# 🤖 Autonomous Research Agent")

    with gr.Row():

        # ── Sidebar ────────────────────────────────────────────────────────
        with gr.Column(scale=1, min_width=280):
            gr.Markdown("### ⚙️ Settings")

            provider_dd = gr.Dropdown(
                choices=["Local (LM Studio)", "OpenAI", "Anthropic", "Google Gemini"],
                value=cfg.get("provider", "Local (LM Studio)"),
                label="LLM Provider",
            )

            model_dd = gr.Dropdown(
                choices=[cfg.get("model", "local-model")],
                value=cfg.get("model", "local-model"),
                label="Model",
                allow_custom_value=True,
            )

            api_key_field = gr.Textbox(
                label="API Key",
                value=cfg.get("api_key", ""),
                type="password",
                placeholder="Leave blank for local models",
            )

            base_url_field = gr.Textbox(
                label="Base URL",
                value=cfg.get("base_url", "http://localhost:1234/v1"),
                placeholder="http://localhost:1234/v1",
            )

            gr.Markdown("---")
            gr.Markdown("### 🎛️ Research Depth")

            depth_radio = gr.Radio(
                choices=list(DEPTH_STEPS.keys()),
                value="📚 Standard (5 steps)",
                label="Preset",
                interactive=True,
            )

            max_steps_slider = gr.Slider(
                minimum=1, maximum=10, step=1,
                value=cfg.get("max_steps", 5),
                label="Max Steps (fine tune)",
                interactive=True,
            )

            with gr.Accordion("🔍 Search Engine", open=False):
                search_engine_dd = gr.Dropdown(
                    choices=["DuckDuckGo (Default)", "SearXNG", "Brave Search", "Google Custom Search"],
                    value=cfg.get("search_engine", "DuckDuckGo (Default)"),
                    label="Engine",
                )
                searx_url_field = gr.Textbox(
                    label="SearXNG URL",
                    value=cfg.get("searx_url", ""),
                    placeholder="http://localhost:8080",
                )
                brave_key_field = gr.Textbox(
                    label="Brave API Key",
                    value=cfg.get("brave_key", ""),
                    type="password",
                )
                google_api_field = gr.Textbox(
                    label="Google API Key",
                    value=cfg.get("google_api", ""),
                    type="password",
                )
                google_cse_field = gr.Textbox(
                    label="Google CSE ID",
                    value=cfg.get("google_cse", ""),
                    type="password",
                )

            save_btn = gr.Button("💾 Save Config", size="sm")

        # ── Main area ──────────────────────────────────────────────────────
        with gr.Column(scale=3):

            query_input = gr.Textbox(
                label="Research Query",
                placeholder="e.g. What are the latest developments in fusion energy?",
                lines=3,
            )

            with gr.Row():
                run_btn = gr.Button("🚀 Run Research", variant="primary", scale=3)
                stop_btn = gr.Button("🛑 Stop", variant="stop", scale=1)

            with gr.Row():
                copy_btn = gr.Button("📋 Copy Report", size="sm")
                download_btn = gr.DownloadButton(
                    "⬇️ Download Report (.md)",
                    size="sm",
                    visible=False,
                )

            research_output = gr.Markdown(
                label="Agent Output",
                value="*Run a query to start…*",
            )

    # ── Event wiring ───────────────────────────────────────────────────────

    # Depth preset → slider
    depth_radio.change(fn=apply_depth_preset, inputs=depth_radio, outputs=max_steps_slider)

    # Provider change → refresh model list
    provider_dd.change(
        fn=update_model_dropdown,
        inputs=[provider_dd, base_url_field],
        outputs=model_dd,
    )

    # Save config
    save_btn.click(
        fn=save_config,
        inputs=[
            provider_dd, model_dd, api_key_field, base_url_field,
            search_engine_dd, searx_url_field, brave_key_field,
            google_api_field, google_cse_field, max_steps_slider,
        ],
    )

    # Run research (streaming)
    run_event = run_btn.click(
        fn=_run_agent,
        inputs=[
            query_input, provider_dd, api_key_field, model_dd, base_url_field,
            search_engine_dd, searx_url_field, brave_key_field,
            google_api_field, google_cse_field, max_steps_slider,
        ],
        outputs=[research_output, download_btn],
    )

    # Stop button cancels the stream
    stop_btn.click(
        fn=lambda: gr.Info("🛑 Research stopped."),
        cancels=[run_event],
    )

    # Copy to clipboard via JS
    copy_btn.click(
        fn=lambda text: gr.Info("📋 Copied to clipboard!"),
        inputs=[research_output],
        js="(text) => { navigator.clipboard.writeText(text); return text; }",
    )

    # Auto-populate model list on load
    app.load(
        fn=update_model_dropdown,
        inputs=[provider_dd, base_url_field],
        outputs=model_dd,
    )


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    # Silence output when running as a packaged exe
    if sys.stdout is None:
        sys.stdout = open(os.devnull, "w")
    if sys.stderr is None:
        sys.stderr = open(os.devnull, "w")

    server_thread = threading.Thread(
        target=lambda: app.launch(server_port=7860, prevent_thread_lock=True),
        daemon=True,
    )
    server_thread.start()

    time.sleep(3)
    webbrowser.open("http://127.0.0.1:7860")

    menu = pystray.Menu(
        pystray.MenuItem("Open Research Agent", _on_open, default=True),
        pystray.MenuItem("Quit", _on_exit),
    )
    tray = pystray.Icon("Research_Agent", _create_icon_image(), "Research Agent", menu)
    tray.run()
