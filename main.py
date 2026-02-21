import json
import os
import gradio as gr
import threading
import webbrowser
import pystray
from PIL import Image, ImageDraw
import sys

def create_icon_image():
    return Image.open("icon.png")

def on_open(icon, item):
    webbrowser.open("http://127.0.0.1:7860")

def on_exit(icon, item):
    icon.stop()
    os._exit(0)

from agent import run_research_agent, update_model_dropdown

config_file = "config.json"

# ---------------- CONFIG ----------------
def load_config():
    if os.path.exists(config_file):
        try:
            with open(config_file, "r", encoding="utf-8") as f:
                return json.load(f)
        except Exception:
            return {}
    return {}

def save_config(provider, model, api_key, base_url, search_engine, searx_url, brave_key, google_api, google_cse):
    config_data = {
        "provider": provider,
        "model": model,
        "api_key": api_key,
        "base_url": base_url,
        "search_engine": search_engine,
        "searx_url": searx_url,
        "brave_key": brave_key,
        "google_api": google_api,
        "google_cse": google_cse,
    }
    with open(config_file, "w", encoding="utf-8") as f:
        json.dump(config_data, f, indent=2)
        
    gr.Info("💾 Configuration saved locally!")

saved_config = load_config()

# ---------------- UI LAYOUT ----------------
with gr.Blocks(title="Autonomous Research Agent") as app:
    gr.Markdown("# 🤖 Research Agent")
    
    with gr.Row():
        
        # --- SIDEBAR (Settings) ---
        with gr.Column(scale=1):
            gr.Markdown("### ⚙️ Settings")
            
            provider_dd = gr.Dropdown(
                choices=["Local (LM Studio)", "OpenAI", "Anthropic", "Google Gemini"],
                value=saved_config.get("provider", "Local (LM Studio)"),
                label="LLM Provider"
            )
            
            model_dd = gr.Dropdown(
                choices=[saved_config.get("model", "local-model")], 
                value=saved_config.get("model", "local-model"),
                label="Model", 
                allow_custom_value=True
            )
            
            api_key_field = gr.Textbox(
                label="API Key", 
                value=saved_config.get("api_key", ""), 
                type="password"
            )
            
            base_url_field = gr.Textbox(
                label="Base URL", 
                value=saved_config.get("base_url", "http://localhost:1234/v1")
            )
            
            with gr.Accordion("🔍 Search Config", open=False):
                search_engine_dd = gr.Dropdown(
                    choices=["DuckDuckGo (Default)", "SearXNG", "Brave Search", "Google Custom Search"],
                    value=saved_config.get("search_engine", "DuckDuckGo (Default)"),
                    label="Primary Search Engine"
                )
                searx_url = gr.Textbox(label="SearXNG URL", value=saved_config.get("searx_url", ""))
                brave_key = gr.Textbox(label="Brave API Key", value=saved_config.get("brave_key", ""), type="password")
                google_api = gr.Textbox(label="Google API Key", value=saved_config.get("google_api", ""), type="password")
                google_cse = gr.Textbox(label="Google CSE ID", value=saved_config.get("google_cse", ""), type="password")
            
            save_btn = gr.Button("💾 Save Config")
            
        # --- MAIN AREA ---
        with gr.Column(scale=3):
            query_input = gr.Textbox(
                label="Research Query", 
                placeholder="What do you want to research?", 
                lines=2
            )
            
            with gr.Row():
                run_btn = gr.Button("🚀 Run", variant="primary")
                stop_btn = gr.Button("🛑 Stop", variant="stop")
                
            research_output = gr.Markdown(label="Agent Output")

    # ---------------- EVENT LISTENERS ----------------
    
    # Auto-update models if provider changes
    provider_dd.change(
        fn=update_model_dropdown, 
        inputs=[provider_dd, base_url_field], 
        outputs=model_dd
    )

    # Save Config
    save_btn.click(
        fn=save_config,
        inputs=[
            provider_dd, model_dd, api_key_field, base_url_field, 
            search_engine_dd, searx_url, brave_key, google_api, google_cse
        ]
    )
    
    # Run Research
    run_event = run_btn.click(
        fn=run_research_agent,
        inputs=[
            query_input, provider_dd, api_key_field, model_dd, base_url_field, 
            search_engine_dd, searx_url, brave_key, google_api, google_cse
        ],
        outputs=[research_output]
    )
    
    # Stop Research (Gradio natively cancels the generator)
    stop_btn.click(
        fn=lambda: gr.Info("🛑 Research stopped by user."),
        cancels=[run_event]
    )

    # --- NEW: Automatically fetch the full model list on startup ---
    app.load(
        fn=update_model_dropdown, 
        inputs=[provider_dd, base_url_field], 
        outputs=model_dd
    )



if __name__ == "__main__":
    # 1. Start Gradio in a background daemon thread
    # prevent_thread_lock=True is crucial here!
    server_thread = threading.Thread(
        target=lambda: app.launch(server_port=7860, prevent_thread_lock=True, theme=gr.themes.Soft()),
        daemon=True
    )
    server_thread.start()

    # 2. Automatically open the browser on first launch
    webbrowser.open("http://127.0.0.1:7860")

    # 3. Create and run the System Tray Icon (This keeps the app alive)
    menu = pystray.Menu(
        pystray.MenuItem("Open Research Agent", on_open, default=True),
        pystray.MenuItem("Quit", on_exit)
    )
    
    tray_icon = pystray.Icon("Research_Agent", create_icon_image(), "Research Agent", menu)
    tray_icon.run()