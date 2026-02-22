# Research Agent 🤖

An autonomous AI research assistant that browses the web, reads RSS feeds, and visits webpages to generate comprehensive research reports. The app runs silently in your system tray and is accessible via a sleek web interface.

---

## 🚀 Getting Started

### Option 1: Simple Download & Run (Recommended)
No Python installation is required. Just download, extract, and run.

1. Download the latest version from the [**Releases**](https://github.com/PavanKumarVattikuti/research-agent/releases) page.
2. Extract the ZIP folder.
3. Double-click `main.exe`.
4. The app will open in your default browser, and a research icon will appear in your system tray.

### Option 2: Clone & Run (For Developers)
If you want to run the source code or modify the agent:

1. **Prerequisites:** Ensure you have the [**uv** package manager](https://github.com/astral-sh/uv) installed.
2. **Clone the Repo:**

   ```bash
   git clone https://github.com/PavanKumarVattikuti/research-agent.git
   cd research-agent
   ```
4. **Sync Environment**:
   ```bash
   uv sync
   ```
5. **Run the App**
   ```bash
   uv run python main.py
   ```
⚙️ **Requirements**

LLM API (Mandatory)
To use this agent, you must provide an API key for one of the following providers:
- OpenAI: (GPT-4o or GPT-4o-mini recommended)
- Anthropic: (Claude 3.5 Sonnet recommended)
- Google Gemini: (Gemini 1.5 Pro or Flash)
- Local (LM Studio): * Ensure LM Studio is running its local server.
  Recommended Models: Mistral-3-3B-Instruct, Qwen-3-VL-4B, or any large Instruct tuned models.

🛠️ **Features**
- System Tray Integration: Runs in the background without cluttering your taskbar.
- Autonomous Planning: The agent breaks down your query into a multi-step research plan.
- Dynamic Search: Supports DuckDuckGo, SearXNG, Brave Search, and Google Search.
- Persistent Config: Saves your API keys and model preferences locally for quick access.

