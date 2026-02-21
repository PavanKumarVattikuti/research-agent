import json
import os
import tempfile
import requests
import gradio as gr
import time

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

from tools import read_rss_feed, visit_webpage
from prompts import PLANNER_PROMPT, EXECUTOR_PROMPT, WRITER_PROMPT

def update_model_dropdown(provider, base_url):
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
        choices = ["gemini-2.5-pro", "gemini-2.5-flash", "gemini-2.0-flash", "gemini-1.5-pro"]
        return gr.update(choices=choices, value="gemini-2.5-pro")
                
    return gr.update(choices=[], value="")

def build_llm(provider: str, api_key: str, model_name: str, base_url: str):
    provider = (provider or "Local (LM Studio)").strip()

    if provider == "OpenAI":
        key = (api_key or os.getenv("OPENAI_API_KEY", "")).strip()
        model = (model_name or os.getenv("OPENAI_MODEL", "gpt-4o-mini")).strip()
        return ChatOpenAI(model=model, api_key=key, temperature=0, timeout=120)

    if provider == "Anthropic":
        key = (api_key or os.getenv("ANTHROPIC_API_KEY", "")).strip()
        model = (model_name or os.getenv("ANTHROPIC_MODEL", "claude-3-5-sonnet-latest")).strip()
        return ChatAnthropic(model=model, api_key=key, temperature=0, timeout=120)
        
    if provider == "Google Gemini":
        key = (api_key or os.getenv("GOOGLE_API_KEY", "")).strip()
        model = (model_name or "gemini-2.5-pro").strip() 
        return ChatGoogleGenerativeAI(model=model, api_key=key, temperature=0, timeout=120)

    base = (base_url or os.getenv("LMSTUDIO_BASE_URL", "http://localhost:1234/v1")).strip()
    model = (model_name or os.getenv("LMSTUDIO_MODEL", "local-model")).strip()
    key = (api_key or os.getenv("LMSTUDIO_API_KEY", "lm-studio")).strip()
    return ChatOpenAI(base_url=base, api_key=key, model=model, temperature=0, max_tokens=2000, timeout=600)

def perform_dynamic_search(query, search_engine, searx_url, brave_key, google_api, google_cse):
    try:
        if search_engine == "Brave Search":
            from langchain_community.tools import BraveSearch
            return BraveSearch.from_api_key(api_key=brave_key, search_kwargs={"count": 5}).run(query)
            
        elif search_engine == "Google Custom Search":
            from langchain_community.utilities import GoogleSearchAPIWrapper
            search = GoogleSearchAPIWrapper(google_api_key=google_api, google_cse_id=google_cse)
            return search.run(query)
            
        elif search_engine == "SearXNG":
            from langchain_community.utilities import SearxSearchWrapper
            search = SearxSearchWrapper(searx_host=searx_url)
            return search.run(query)
            
        else:
            from langchain_community.tools import DuckDuckGoSearchRun
            return DuckDuckGoSearchRun().run(query)
    except Exception as e:
        return f"Search Error ({search_engine}): {e}"

def run_research_agent(user_query, provider, api_key, model_name, base_url, 
                       search_engine, searx_url, brave_key, google_api, google_cse):
    
    full_history = f"🧠 **Starting research on:** {user_query}\n\n"
    yield full_history

    try:
        llm = build_llm(provider, api_key, model_name, base_url)
    except Exception as exc:
        yield full_history + f"❌ Configuration error: {exc}"
        return

    full_history += "📋 **Phase 1: Planning...**\n"
    yield full_history

    plan_messages = [SystemMessage(content=PLANNER_PROMPT), HumanMessage(content=user_query)]
    try:
        plan_response = llm.invoke(plan_messages)
        clean_json = plan_response.content.replace("```json", "").replace("```", "").strip()
        plan = json.loads(clean_json)
        plan_text = "\n".join([f"{i + 1}. {step}" for i, step in enumerate(plan)])
        full_history += f"✅ **Plan created:**\n{plan_text}\n\n"
        yield full_history
        time.sleep(4)
        
    except Exception as exc:
        full_history += f"⚠️ Error in planning: {exc}. Using fallback.\n"
        plan = [f"Search for {user_query}"]
        yield full_history

    research_notes = []
    full_history += "🕵️ **Phase 2: Executing plan...**\n"
    yield full_history

    for index, step in enumerate(plan, start=1):
        full_history += f"> *Step {index}: {step}*\n"
        yield full_history
        
        executor_messages = [SystemMessage(content=EXECUTOR_PROMPT), HumanMessage(content=f"Current Task: {step}")]
        
        try:
            action_response = llm.invoke(executor_messages).content
            time.sleep(4)
        except Exception as api_err:
            full_history += f"  - ❌ API Error during task execution: {api_err}\n"
            yield full_history
            time.sleep(15)
            continue # Skip this failed step and move to the next one in the plan

        if "TOOL: SEARCH" in action_response:
            query = action_response.split("SEARCH", 1)[1].strip().strip('"')
            full_history += f"  - 🔍 Searching ({search_engine}): {query}\n"
            yield full_history
            
            result = perform_dynamic_search(query, search_engine, searx_url, brave_key, google_api, google_cse)
                
        elif "TOOL: VISIT" in action_response:
            url = action_response.split("VISIT", 1)[1].strip().strip('"')
            full_history += f"  - 🌐 Visiting URL...\n"
            yield full_history
            try:
                result = visit_webpage.invoke(url)
            except Exception as e:
                result = f"Failed to visit: {e}"
                
        elif "TOOL: RSS" in action_response:
            url = action_response.split("RSS", 1)[1].strip().strip('"')
            full_history += "  - 📰 Reading RSS...\n"
            yield full_history
            try:
                result = read_rss_feed.invoke(url)
            except Exception as e:
                result = f"Failed to read feed: {e}"
        else:
            result = action_response.replace("RESULT:", "").strip()

        research_notes.append(f"Task: {step}\nResult: {result[:800]}") 
        full_history += "  - ✅ Info collected.\n"
        yield full_history

    full_history += "\n✍️ **Phase 3: Writing final report...**\n"
    yield full_history
    
    time.sleep(4)

    all_notes = "\n\n".join(research_notes)
    writer_messages = [
        SystemMessage(content=WRITER_PROMPT),
        HumanMessage(content=f"Original Request: {user_query}\n\nResearch Notes:\n{all_notes}"),
    ]

    final_response = llm.invoke(writer_messages)
    final_markdown = full_history + f"\n---\n## 📝 Final Report\n{final_response.content}"
    
    # We still create the temp file in case you want to wire up a Download button later!
    with tempfile.NamedTemporaryFile(delete=False, mode="w", suffix=".md", encoding="utf-8") as f:
        f.write(f"# Research Report: {user_query}\n\n{final_response.content}")
        file_path = f.name

    yield final_markdown