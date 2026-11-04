import { createSignal, onMount, onCleanup, For, Show, Switch, Match } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { marked } from "marked";
import "./App.css";

// Configure marked with GitHub Flavored Markdown
marked.setOptions({
  gfm: true,
  breaks: true,
});

interface WordConstraint {
  word: string;
  question_why: string;
  requirement: string;
}

interface LayerResult {
  layer: number;
  why_question: string;
  finding: string;
  proof_verified: boolean;
}

interface CandidateAnswer {
  rank: number;
  name: string;
  score: string;
  summary: string;
  sources: string[];
}

interface ResearchOutput {
  query: string;
  mode: string;
  constraints: WordConstraint[];
  layers: LayerResult[];
  candidates: CandidateAnswer[];
  final_report: string;
  citations: string[];
  stopped_early: boolean;
  template_hint?: string;
}

interface HapticEvent {
  step: string;
  message: string;
  detail?: string;
}

interface SessionSummary {
  id: number;
  query: string;
  mode: string;
  created_at: string;
  layer_count: number;
  candidate_count: number;
}

interface FullSession {
  summary: SessionSummary;
  report_markdown: string;
  citations: string[];
}

interface TestResult {
  live: boolean;
  latency_ms: number;
  message: string;
  detected_models: string[];
}

interface LogEntry {
  timestamp: string;
  level: string; // "INFO" | "SEARCH" | "RANK" | "LLM" | "WARN" | "ERROR" | "DONE"
  elapsed_ms: number;
  message: string;
}

interface ProgressEvent {
  stage: string;
  message: string;
  query: string;
  subquery?: string;
  words: string[];
  sources_count: number;
  current_layer: number;
  max_layers: number;
}

export default function App() {
  // Navigation & View State
  const [sidebarOpen, setSidebarOpen] = createSignal(true);
  const [currentView, setCurrentView] = createSignal<"search" | "answer">("search");
  const [activeTab, setActiveTab] = createSignal<"answer" | "sources">("answer");
  const [settingsOpen, setSettingsOpen] = createSignal(false);
  const [settingsTab, setSettingsTab] = createSignal<"engine" | "terminal">("engine");
  const [isSearching, setIsSearching] = createSignal(false);
  const [searchError, setSearchError] = createSignal<string | null>(null);

  // Search Inputs
  const [query, setQuery] = createSignal("");
  const [followUpQuery, setFollowUpQuery] = createSignal("");
  const [mode, setMode] = createSignal("Deep Search");
  const [layers, setLayers] = createSignal(5);

  // Active Research Result & Session
  const [activeResult, setActiveResult] = createSignal<ResearchOutput | null>(null);
  const [activeSessionId, setActiveSessionId] = createSignal<number | null>(null);
  const [schematicOpen, setSchematicOpen] = createSignal(true);
  const [copyFeedback, setCopyFeedback] = createSignal(false);
  const [exportStatus, setExportStatus] = createSignal<string | null>(null);
  const [exportHtmlStatus, setExportHtmlStatus] = createSignal<string | null>(null);

  // Pipeline Execution Logs & Diagnostics
  const [terminalLogs, setTerminalLogs] = createSignal<LogEntry[]>([]);
  const [logCopyFeedback, setLogCopyFeedback] = createSignal(false);
  const [logExportStatus, setLogExportStatus] = createSignal<string | null>(null);
  let terminalScrollRef: HTMLDivElement | undefined;

  // Live Investigation Progress
  const [_liveStage, setLiveStage] = createSignal<string>("");
  const [liveMessage, setLiveMessage] = createSignal<string>("Initializing research engine...");
  const [liveWords, setLiveWords] = createSignal<string[]>([]);
  const [liveSubqueries, setLiveSubqueries] = createSignal<string[]>([]);
  const [liveSourcesCount, setLiveSourcesCount] = createSignal(0);
  const [elapsedSeconds, setElapsedSeconds] = createSignal(0);
  const [showAllSourcesInAccordion, setShowAllSourcesInAccordion] = createSignal(false);
  let timerInterval: any = null;

  // History List
  const [history, setHistory] = createSignal<SessionSummary[]>([]);

  // Settings State (persisted in localStorage)
  const [llmProtocol, setLlmProtocol] = createSignal(localStorage.getItem("llm_protocol") || "openai");
  const [llmBaseUrl, setLlmBaseUrl] = createSignal(localStorage.getItem("llm_base_url") || "http://localhost:1234/v1");
  const [llmModel, setLlmModel] = createSignal(localStorage.getItem("llm_model") || "local-model");
  const [llmApiKey, setLlmApiKey] = createSignal(localStorage.getItem("llm_api_key") || "");

  const [searchEngine, setSearchEngine] = createSignal(localStorage.getItem("search_engine") || "DuckDuckGo");
  const [searxUrl, setSearxUrl] = createSignal(localStorage.getItem("searx_url") || "http://localhost:8080");
  const [braveKey, setBraveKey] = createSignal(localStorage.getItem("brave_key") || "");

  const [vaultPath, setVaultPath] = createSignal(localStorage.getItem("vault_path") || "D:\\Notes\\Obsidian");
  const [vaultCount, setVaultCount] = createSignal(0);

  // Health Test Badges
  const [llmStatus, setLlmStatus] = createSignal<{ live: boolean; text: string } | null>(null);
  const [searchStatus, setSearchStatus] = createSignal<{ live: boolean; text: string } | null>(null);
  const [isTestingLlm, setIsTestingLlm] = createSignal(false);
  const [isTestingSearch, setIsTestingSearch] = createSignal(false);

  let unlistenProgress: (() => void) | null = null;
  let unlistenLogs: (() => void) | null = null;
  let unlistenHaptic: (() => void) | null = null;

  // Animated single rotating word for hero page
  const ROTATING_WORDS = [
    "verify",
    "investigate",
    "research",
    "fact-check",
    "uncover",
    "explore",
  ];
  const [wordIndex, setWordIndex] = createSignal(0);
  const [wordVisible, setWordVisible] = createSignal(true);

  // Accordion sub-view ("summary" | "trace")
  const [accordionView, setAccordionView] = createSignal<"summary" | "trace">("summary");

  // Haptic streaming events (live during search and preserved in accordion)
  const [hapticEvents, setHapticEvents] = createSignal<HapticEvent[]>([]);
  let hapticScrollRef: HTMLDivElement | undefined;

  onMount(async () => {
    loadHistory();
    checkVaultCount();
    loadLogs();

    // Single word rotation — 5 second cycle, 600ms smooth slide transition
    const wordInterval = setInterval(() => {
      setWordVisible(false);
      setTimeout(() => {
        setWordIndex((i) => (i + 1) % ROTATING_WORDS.length);
        setWordVisible(true);
      }, 600);
    }, 5000);
    onCleanup(() => clearInterval(wordInterval));

    // Listen to real-time research progress events from Rust
    try {
      unlistenProgress = await listen<ProgressEvent>("research-progress", (event) => {
        const payload = event.payload;
        setLiveStage(payload.stage);
        setLiveMessage(payload.message);
        if (payload.words && payload.words.length > 0) {
          setLiveWords(payload.words);
        }
        if (payload.subquery && !liveSubqueries().includes(payload.subquery)) {
          setLiveSubqueries([...liveSubqueries(), payload.subquery]);
        }
        if (payload.sources_count > 0) {
          setLiveSourcesCount(payload.sources_count);
        }
      });
    } catch (err) {
      console.warn("Progress event listener initialization skipped:", err);
    }

    // Listen to haptic streaming events from Rust pipeline
    try {
      unlistenHaptic = await listen<HapticEvent>("research-haptic", (event) => {
        setHapticEvents((prev) => [...prev.slice(-30), event.payload]);
        if (hapticScrollRef) {
          setTimeout(() => {
            if (hapticScrollRef) hapticScrollRef.scrollTop = hapticScrollRef.scrollHeight;
          }, 20);
        }
      });
    } catch (err) {
      console.warn("Haptic event listener initialization skipped:", err);
    }

    // Listen to real-time execution logs from Rust
    try {
      unlistenLogs = await listen<LogEntry>("pipeline-log", (event) => {
        setTerminalLogs((prev) => [...prev.slice(-400), event.payload]);
        if (terminalScrollRef) {
          setTimeout(() => {
            if (terminalScrollRef) terminalScrollRef.scrollTop = terminalScrollRef.scrollHeight;
          }, 20);
        }
      });
    } catch (err) {
      console.warn("Log event listener initialization skipped:", err);
    }
  });

  onCleanup(() => {
    if (unlistenProgress) unlistenProgress();
    if (unlistenLogs) unlistenLogs();
    if (unlistenHaptic) unlistenHaptic();
    if (timerInterval) clearInterval(timerInterval);
  });

  const loadHistory = async () => {
    try {
      const items = await invoke<SessionSummary[]>("get_history");
      setHistory(items);
    } catch {
      setHistory([]);
    }
  };

  const checkVaultCount = async () => {
    try {
      const count = await invoke<number>("count_vault_notes", { vaultPath: vaultPath() });
      setVaultCount(count);
    } catch {
      setVaultCount(0);
    }
  };

  const saveSettings = () => {
    localStorage.setItem("llm_protocol", llmProtocol());
    localStorage.setItem("llm_base_url", llmBaseUrl());
    localStorage.setItem("llm_model", llmModel());
    localStorage.setItem("llm_api_key", llmApiKey());
    localStorage.setItem("search_engine", searchEngine());
    localStorage.setItem("searx_url", searxUrl());
    localStorage.setItem("brave_key", braveKey());
    localStorage.setItem("vault_path", vaultPath());
    setSettingsOpen(false);
  };

  const handleTestLlm = async () => {
    setIsTestingLlm(true);
    setLlmStatus(null);
    try {
      const res = await invoke<TestResult>("test_llm", {
        protocol: llmProtocol(),
        baseUrl: llmBaseUrl(),
        apiKey: llmApiKey() || null,
      });
      if (res.live) {
        setLlmStatus({ live: true, text: `Live (${res.latency_ms}ms) — ${res.detected_models.length} models` });
      } else {
        setLlmStatus({ live: false, text: res.message });
      }
    } catch (err: any) {
      setLlmStatus({ live: false, text: String(err) });
    } finally {
      setIsTestingLlm(false);
    }
  };

  const handleTestSearch = async () => {
    setIsTestingSearch(true);
    setSearchStatus(null);
    try {
      const urlOrKey = searchEngine() === "SearXNG" ? searxUrl() : braveKey();
      const res = await invoke<TestResult>("test_search", {
        engine: searchEngine(),
        urlOrKey: urlOrKey,
      });
      if (res.live) {
        setSearchStatus({ live: true, text: `OK (${res.latency_ms}ms)` });
      } else {
        setSearchStatus({ live: false, text: res.message });
      }
    } catch (err: any) {
      setSearchStatus({ live: false, text: String(err) });
    } finally {
      setIsTestingSearch(false);
    }
  };

  const loadLogs = async () => {
    try {
      const logs = await invoke<LogEntry[]>("get_pipeline_logs");
      setTerminalLogs(logs);
      if (terminalScrollRef) {
        setTimeout(() => {
          if (terminalScrollRef) terminalScrollRef.scrollTop = terminalScrollRef.scrollHeight;
        }, 50);
      }
    } catch {
      setTerminalLogs([]);
    }
  };

  const clearLogs = async () => {
    try {
      await invoke("clear_pipeline_logs");
      setTerminalLogs([]);
    } catch (err) {
      console.error("Failed to clear logs", err);
    }
  };

  const copyLogs = async () => {
    const logs = terminalLogs();
    if (logs.length === 0) return;
    const text = logs
      .map((l) => `[${l.timestamp}] [${l.level}] (+${l.elapsed_ms}ms) ${l.message}`)
      .join("\n");
    await navigator.clipboard.writeText(text);
    setLogCopyFeedback(true);
    setTimeout(() => setLogCopyFeedback(false), 2000);
  };

  const exportLogsToFile = async () => {
    try {
      const savedPath = await invoke<string>("save_logs_export");
      const filename = savedPath.split(/[\/\\]/).pop() || "research-agent-logs.txt";
      setLogExportStatus(`Saved: ${filename}`);
      setTimeout(() => setLogExportStatus(null), 3500);
    } catch (err: any) {
      setLogExportStatus("Export failed");
      setTimeout(() => setLogExportStatus(null), 2500);
    }
  };

  const reportGitHubIssue = () => {
    const url = "https://github.com/12gblabs/research-agent/issues/new?title=Pipeline%20Latency%20or%20Synthesis%20Stall&body=%23%23%20Problem%20Description%0AWhat%20query%20were%20you%20running%3F%20How%20long%20did%20it%20take%3F%0A%0A%23%23%20Steps%20to%20Reproduce%0A1.%0A2.%0A%0A%23%23%20Execution%20Logs%0A%60%60%60%0APlease%20paste%20the%20logs%20from%20Settings%20-%3E%20Pipeline%20Logs%20here%0A%60%60%60";
    window.open(url, "_blank");
  };

  const handleRunSearch = async (targetQuery?: string) => {
    const q = (targetQuery || query()).trim();
    if (!q || isSearching()) return;

    setIsSearching(true);
    setSearchError(null);
    setCurrentView("answer");
    setActiveTab("answer");
    setSchematicOpen(true);
    setLiveSubqueries([]);
    setLiveWords([]);
    setLiveSourcesCount(0);
    setShowAllSourcesInAccordion(false);
    setLiveMessage("Deconstructing query intent & domain constraints...");
    setElapsedSeconds(0);
    setHapticEvents([]);

    // Initial placeholder result so UI instantly shifts to the Perplexity answer layout
    setActiveResult({
      query: q,
      mode: mode(),
      constraints: [],
      layers: [],
      candidates: [],
      final_report: "",
      citations: [],
      stopped_early: false,
    });

    const startTime = Date.now();
    if (timerInterval) clearInterval(timerInterval);
    timerInterval = setInterval(() => {
      setElapsedSeconds(Math.floor((Date.now() - startTime) / 1000));
    }, 1000);

    const config = {
      mode: mode(),
      max_layers: Number(layers()),
      search_engine: searchEngine(),
      searx_url: searxUrl() || null,
      brave_key: braveKey() || null,
      obsidian_vault_path: vaultPath() || null,
      llm: {
        protocol: llmProtocol(),
        base_url: llmBaseUrl(),
        model_name: llmModel(),
        api_key: llmApiKey() || null,
      },
    };

    try {
      const output = await invoke<ResearchOutput>("execute_research", {
        query: q,
        config: config,
      });
      setActiveResult(output);
      setFollowUpQuery("");
      loadHistory();
    } catch (err: any) {
      setSearchError(String(err));
    } finally {
      setIsSearching(false);
      if (timerInterval) clearInterval(timerInterval);
    }
  };

  const handleSelectSession = async (item: SessionSummary) => {
    try {
      const full = await invoke<FullSession | null>("get_history_detail", { sessionId: item.id });
      if (full) {
        setActiveResult({
          query: full.summary.query,
          mode: full.summary.mode,
          constraints: [],
          layers: [],
          candidates: [],
          final_report: full.report_markdown,
          citations: full.citations,
          stopped_early: true,
        });
        setActiveSessionId(item.id);
        setCurrentView("answer");
        setActiveTab("answer");
        setSchematicOpen(false);
      }
    } catch (err) {
      console.error("Failed to load session details", err);
    }
  };

  const handleDeleteSession = async (id: number, e: MouseEvent) => {
    e.stopPropagation();
    try {
      await invoke("delete_history_item", { sessionId: id });
      setHistory((prev) => prev.filter((item) => item.id !== id));
      if (activeSessionId() === id) {
        setActiveResult(null);
        setCurrentView("search");
      }
    } catch (err) {
      console.error("Failed to delete session", err);
    }
  };

  const exportMarkdown = async () => {
    const res = activeResult();
    if (!res || !res.final_report) {
      setExportStatus("No report yet");
      setTimeout(() => setExportStatus(null), 2500);
      return;
    }

    const cleanReport = res.final_report.replace(/^<!--\s*TEMPLATE:\s*\w+\s*-->\n?/, "");
    const citationsList = res.citations.length
      ? `\n\n## Sources\n${res.citations.map((c, i) => `${i + 1}. ${c}`).join("\n")}`
      : "";

    const content = `# ${res.query}\n\n*Investigated with Research Agent (${res.mode})*\n\n${cleanReport}${citationsList}\n`;
    const sanitizedTitle = res.query
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .slice(0, 40)
      .replace(/^-|-$/g, "");
    const filename = `${sanitizedTitle || "research-report"}.md`;

    try {
      // 1. Native desktop direct save to user's Downloads directory
      await invoke<string>("save_markdown_export", {
        filename: filename,
        content: content,
      });
      setExportStatus("✓ Saved to Downloads");
      setTimeout(() => setExportStatus(null), 3000);
    } catch {
      // 2. Fallback to DOM Blob with document.body append
      try {
        const blob = new Blob([content], { type: "text/markdown;charset=utf-8" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.style.display = "none";
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        setTimeout(() => {
          document.body.removeChild(a);
          URL.revokeObjectURL(url);
        }, 1000);
        setExportStatus("✓ Exported");
        setTimeout(() => setExportStatus(null), 2500);
      } catch {
        setExportStatus("Export failed");
        setTimeout(() => setExportStatus(null), 2500);
      }
    }
  };

  const exportHtml = async () => {
    const res = activeResult();
    if (!res || !res.final_report) {
      setExportHtmlStatus("No report yet");
      setTimeout(() => setExportHtmlStatus(null), 2500);
      return;
    }

    const { hint, body } = parseTemplateHint(res.final_report);
    const renderedBody = marked.parse(body) as string;

    const citationsHtml = res.citations.length
      ? `
        <section class="sources-section">
          <h2>Verified Sources & Proof Points</h2>
          <ol class="sources-list">
            ${res.citations.map((c) => {
              const urlMatch = c.match(/(https?:\/\/[^\s)]+)/);
              const url = urlMatch ? urlMatch[1] : null;
              return `<li>${url ? `<a href="${url}" target="_blank" rel="noopener noreferrer">${c}</a>` : c}</li>`;
            }).join("\n")}
          </ol>
        </section>`
      : "";

    const htmlContent = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${res.query.replace(/</g, "&lt;").replace(/>/g, "&gt;")} — Research Agent</title>
  <style>
    :root {
      --bg: #121314;
      --card-bg: #181C20;
      --card-border: #252B30;
      --accent: #38D9C0;
      --accent-muted: rgba(56, 217, 192, 0.12);
      --text: #E8EDF0;
      --text-muted: #929DA6;
      --text-dim: #64707D;
      --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: var(--bg);
      color: var(--text);
      font-family: var(--font-sans);
      line-height: 1.65;
      padding: 48px 24px;
      -webkit-font-smoothing: antialiased;
    }
    .container {
      max-width: 840px;
      margin: 0 auto;
    }
    header {
      border-bottom: 1px solid var(--card-border);
      padding-bottom: 24px;
      margin-bottom: 32px;
    }
    .badge {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      font-size: 11px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      color: var(--accent);
      background: var(--accent-muted);
      border: 1px solid rgba(56, 217, 192, 0.3);
      padding: 4px 10px;
      border-radius: 9999px;
      margin-bottom: 12px;
    }
    h1 {
      font-size: 30px;
      font-weight: 700;
      color: #FFFFFF;
      letter-spacing: -0.02em;
      margin-bottom: 8px;
    }
    .meta {
      font-size: 13px;
      color: var(--text-muted);
    }
    .content h2 {
      font-size: 20px;
      font-weight: 600;
      color: #FFFFFF;
      margin: 32px 0 14px 0;
      padding-bottom: 8px;
      border-bottom: 1px solid var(--card-border);
    }
    .content h3 {
      font-size: 16px;
      font-weight: 600;
      color: var(--accent);
      margin: 22px 0 8px 0;
    }
    .content p {
      margin-bottom: 16px;
      color: var(--text);
    }
    .content strong {
      color: #FFFFFF;
    }
    .content ul, .content ol {
      margin: 0 0 16px 24px;
    }
    .content li {
      margin-bottom: 6px;
    }
    .content table {
      width: 100%;
      border-collapse: collapse;
      margin: 20px 0;
      background: var(--card-bg);
      border-radius: 8px;
      overflow: hidden;
      border: 1px solid var(--card-border);
    }
    .content th, .content td {
      padding: 10px 14px;
      text-align: left;
      border-bottom: 1px solid var(--card-border);
      font-size: 13px;
    }
    .content th {
      background: rgba(255, 255, 255, 0.04);
      color: var(--text-muted);
      font-weight: 600;
      text-transform: uppercase;
      font-size: 11px;
      letter-spacing: 0.04em;
    }
    .content tr:last-child td {
      border-bottom: none;
    }
    .sources-section {
      margin-top: 40px;
      padding-top: 24px;
      border-top: 1px solid var(--card-border);
    }
    .sources-section h2 {
      font-size: 15px;
      font-weight: 600;
      color: var(--text-muted);
      text-transform: uppercase;
      letter-spacing: 0.05em;
      margin-bottom: 14px;
      border: none;
      padding: 0;
    }
    .sources-list {
      list-style-position: inside;
      color: var(--text-muted);
      font-size: 13px;
    }
    .sources-list li {
      margin-bottom: 8px;
      word-break: break-word;
    }
    a {
      color: var(--accent);
      text-decoration: none;
    }
    a:hover {
      text-decoration: underline;
    }
    footer {
      margin-top: 50px;
      padding-top: 20px;
      border-top: 1px solid var(--card-border);
      font-size: 12px;
      color: var(--text-dim);
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    @media print {
      body { background: #FFFFFF !important; color: #111111 !important; padding: 0 !important; }
      .container { max-width: 100% !important; margin: 0 !important; }
      header { border-color: #DDDDDD !important; }
      h1, .content h2, .content strong { color: #000000 !important; }
      .meta, .content p, .content td, .sources-list { color: #333333 !important; }
      .content table { border-color: #DDDDDD !important; background: #FFFFFF !important; }
      .content th { background: #F5F5F5 !important; color: #555555 !important; }
      .content td, .content th { border-color: #EEEEEE !important; }
      .badge { border-color: #38D9C0 !important; }
      footer { border-color: #EEEEEE !important; color: #888888 !important; }
    }
  </style>
</head>
<body>
  <div class="container">
    <header>
      <div class="badge">Research Agent • ${res.mode}</div>
      <h1>${res.query.replace(/</g, "&lt;").replace(/>/g, "&gt;")}</h1>
      <div class="meta">Verified with ${res.citations.length} independent proof points • Layout: ${hint}</div>
    </header>
    <main class="content">
      ${renderedBody}
      ${citationsHtml}
    </main>
    <footer>
      <span>Research Agent — Proof Over Hallucination</span>
      <span>${new Date().toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })}</span>
    </footer>
  </div>
</body>
</html>`;

    const sanitizedTitle = res.query
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .slice(0, 40)
      .replace(/^-|-$/g, "");
    const filename = `${sanitizedTitle || "research-report"}.html`;

    try {
      await invoke<string>("save_markdown_export", {
        filename: filename,
        content: htmlContent,
      });
      setExportHtmlStatus("✓ Saved to Downloads");
      setTimeout(() => setExportHtmlStatus(null), 3000);
    } catch {
      try {
        const blob = new Blob([htmlContent], { type: "text/html;charset=utf-8" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.style.display = "none";
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        setTimeout(() => {
          document.body.removeChild(a);
          URL.revokeObjectURL(url);
        }, 1000);
        setExportHtmlStatus("✓ Exported");
        setTimeout(() => setExportHtmlStatus(null), 2500);
      } catch {
        setExportHtmlStatus("Export failed");
        setTimeout(() => setExportHtmlStatus(null), 2500);
      }
    }
  };

  const copyReport = () => {
    const res = activeResult();
    if (!res || !res.final_report) return;
    const cleanReport = res.final_report.replace(/^<!--\s*TEMPLATE:\s*\w+\s*-->\n?/, "");
    navigator.clipboard.writeText(cleanReport);
    setCopyFeedback(true);
    setTimeout(() => setCopyFeedback(false), 2000);
  };

  // ---------------------------------------------------------------------------
  // Template Detection & Renderers
  // ---------------------------------------------------------------------------
  const parseTemplateHint = (md: string): { hint: string; body: string } => {
    const match = md.match(/^<!--\s*TEMPLATE:\s*([\w\-]+)\s*-->\n?/i);
    let hint = "default_prose";
    let body = md;
    if (match) {
      const raw = match[1].toLowerCase().replace(/-/g, "_");
      body = md.slice(match[0].length);
      if (raw.includes("rank") || raw.includes("list") || raw.includes("top")) {
        hint = "ranked_list";
      } else if (raw.includes("news") || raw.includes("feed") || raw.includes("update")) {
        hint = "news_feed";
      } else if (raw.includes("compare") || raw.includes("table") || raw.includes("matrix") || raw.includes("vs")) {
        hint = "comparison_table";
      } else if (raw.includes("card") || raw.includes("flash") || raw.includes("qa") || raw.includes("faq")) {
        hint = "flashcards";
      }
    } else {
      // Content-based intelligent heuristic if template hint was omitted
      if (/\|\s*#\s*\|\s*Title\s*\|/i.test(md) || /^\|\s*1\s*\|.+/m.test(md)) {
        hint = "ranked_list";
      } else if (/latest|weekly|today|announced/i.test(md) && /\[\d+\]/.test(md)) {
        hint = "news_feed";
      }
    }
    return { hint, body };
  };

  // Parse ranked list items from markdown — supports both ### headings and markdown tables
  const parseRankedItems = (md: string) => {
    // 1. Try parsing from ### headings first
    if (/^###\s+/m.test(md)) {
      const sections = md.split(/^###\s+/m).slice(1);
      const items = sections.map((section) => {
        const lines = section.split("\n");
        const rawTitle = lines[0]?.trim() || "";
        const title = rawTitle.replace(/^\d+[\.\)]\s*|\[\d+\]\s*/g, "").trim();
        const body = lines.slice(1).join("\n").trim();
        const badges: string[] = [];
        const badgeMatches = body.matchAll(/\[Confirmed\]|\[In-Development Roadmap\]|\[Projected Target\]/gi);
        for (const m of badgeMatches) badges.push(m[0].replace(/\[|\]/g, ""));
        const summary = body
          .replace(/\*\*(.*?)\*\*/g, "$1")
          .replace(/\[Confirmed\]|\[In-Development Roadmap\]|\[Projected Target\]/gi, "")
          .replace(/\[\d+\]/g, "")
          .replace(/\n+/g, " ")
          .trim()
          .slice(0, 260);
        return { title, summary, badges };
      }).filter((item) => item.title.length > 0);
      if (items.length > 0) return items;
    }

    // 2. Fallback: Parse from markdown table rows (| # | Title | Studio | ... |)
    const tableRows = md.match(/^\|\s*\d+\s*\|.+$/gm);
    if (tableRows && tableRows.length > 0) {
      return tableRows.map((row) => {
        const cells = row.split("|").map((c) => c.trim()).filter(Boolean);
        // Typical columns: # | Title | Studio / Developer | Platforms | Status | Basis
        const title = (cells[1] || "").replace(/\[|\]/g, "").trim();
        const studio = cells[2] || "";
        const platforms = cells[3] && cells[3] !== "—" && cells[3] !== "-" ? cells[3] : "";
        const status = cells[4] || "";
        const basis = cells[5] || "";
        const badges: string[] = [];
        if (status) badges.push(status.replace(/\[|\]/g, ""));
        const summaryParts = [studio, platforms, basis].filter(Boolean);
        const summary = summaryParts.join(" • ").replace(/\[\d+\]/g, "").trim();
        return { title, summary, badges };
      }).filter((item) => item.title.length > 0 && !item.title.startsWith("---"));
    }

    return [];
  };

  // Parse news feed items from markdown — supports ### headings, bullet points, and tables
  const parseNewsItems = (md: string) => {
    // 1. Try parsing from ### headings
    if (/^###\s+/m.test(md)) {
      const sections = md.split(/^###\s+/m).slice(1);
      const items = sections.map((section) => {
        const lines = section.split("\n");
        const title = lines[0]?.trim() || "";
        const body = lines.slice(1).join("\n").trim();
        const citationMatches = [...body.matchAll(/\[(\d+)\]/g)];
        const citationNums = [...new Set(citationMatches.map((m) => Number(m[1])))];
        const summary = body
          .replace(/\*\*(.*?)\*\*/g, "$1")
          .replace(/\[\d+\]/g, "")
          .replace(/\n+/g, " ")
          .trim()
          .slice(0, 260);
        return { title, summary, citationNums };
      }).filter((item) => item.title.length > 0);
      if (items.length > 0) return items;
    }

    // 2. Fallback: Parse from bullet points "- **Title**: summary [1]"
    const bulletMatches = md.match(/^[-*]\s+\*\*([^*]+)\*\*:\s*(.+)$/gm);
    if (bulletMatches && bulletMatches.length > 0) {
      return bulletMatches.map((b) => {
        const m = b.match(/^[-*]\s+\*\*([^*]+)\*\*:\s*(.+)$/);
        const title = m ? m[1].trim() : "";
        const body = m ? m[2].trim() : "";
        const citationMatches = [...body.matchAll(/\[(\d+)\]/g)];
        const citationNums = [...new Set(citationMatches.map((c) => Number(c[1])))];
        return { title, summary: body.replace(/\[\d+\]/g, "").trim(), citationNums };
      }).filter((item) => item.title.length > 0);
    }

    // 3. Fallback: Parse from table rows
    const tableRows = md.match(/^\|\s*\d+\s*\|.+$/gm);
    if (tableRows && tableRows.length > 0) {
      return tableRows.map((row) => {
        const cells = row.split("|").map((c) => c.trim()).filter(Boolean);
        const title = (cells[1] || "").trim();
        const summary = (cells[cells.length - 1] || cells[2] || "").trim();
        const citationMatches = [...summary.matchAll(/\[(\d+)\]/g)];
        const citationNums = [...new Set(citationMatches.map((c) => Number(c[1])))];
        return { title, summary: summary.replace(/\[\d+\]/g, "").trim(), citationNums };
      }).filter((item) => item.title.length > 0 && !item.title.startsWith("---"));
    }

    return [];
  };

  const getDomain = (urlStr: string) => {
    try {
      if (urlStr.startsWith("file://")) return "Local Note";
      const u = new URL(urlStr);
      return u.hostname.replace(/^www\./, "");
    } catch {
      return "web";
    }
  };

  const getFaviconUrl = (urlStr: string) => {
    try {
      if (urlStr.startsWith("file://")) return "";
      const domain = getDomain(urlStr);
      return `https://www.google.com/s2/favicons?domain=${domain}&sz=32`;
    } catch {
      return "";
    }
  };

  // Group history items by Today, Yesterday, and Older
  const groupedHistory = () => {
    const today: SessionSummary[] = [];
    const yesterday: SessionSummary[] = [];
    const older: SessionSummary[] = [];

    const now = new Date();
    const todayStr = now.toISOString().slice(0, 10);
    const yest = new Date(now.getTime() - 86400000);
    const yestStr = yest.toISOString().slice(0, 10);

    for (const item of history()) {
      const itemDate = item.created_at.slice(0, 10);
      if (itemDate === todayStr) {
        today.push(item);
      } else if (itemDate === yestStr) {
        yesterday.push(item);
      } else {
        older.push(item);
      }
    }

    return { today, yesterday, older };
  };

  // Transform markdown with status badges
  const renderEnrichedMarkdown = (md: string): string => {
    if (!md) return "";
    let enriched = md
      .replace(/\[Confirmed\]/gi, `<span class="badge-confirmed px-2 py-0.5 rounded text-[11px] font-medium inline-block">Confirmed</span>`)
      .replace(/\[In-Development Roadmap\]/gi, `<span class="badge-roadmap px-2 py-0.5 rounded text-[11px] font-medium inline-block">Roadmap Target</span>`)
      .replace(/\[Projected Target\]/gi, `<span class="badge-projected px-2 py-0.5 rounded text-[11px] font-medium inline-block">Projected</span>`);
    return marked.parse(enriched) as string;
  };

  const activeTemplate = () => parseTemplateHint(activeResult()?.final_report || "");

  return (
    <div class="h-screen w-screen overflow-hidden flex flex-col bg-[#0C0E10] text-[#E8EDF0]">

      {/* TOP HEADER NAVIGATION BAR */}
      <header class="h-13 border-b border-[#252B30] bg-[#121518] flex items-center justify-between px-4 z-40 shrink-0">
        <div class="flex items-center gap-3">
          <button
            onClick={() => setSidebarOpen(!sidebarOpen())}
            class="p-1.5 rounded-lg text-[#929DA6] hover:text-[#E8EDF0] hover:bg-white/5 transition"
            title="Toggle Sidebar"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          </button>

          <div
            onClick={() => {
              setQuery("");
              setActiveResult(null);
              setCurrentView("search");
            }}
            class="flex items-center gap-2 cursor-pointer group"
          >
            <span class="w-2.5 h-2.5 rounded-full bg-[#38D9C0] shadow-sm shadow-[#38D9C0]/50 group-hover:scale-110 transition"></span>
            <span class="font-semibold text-xs tracking-wider text-[#E8EDF0]">RESEARCH AGENT</span>
          </div>

          <span class="text-[#252B30] text-sm hidden sm:inline">|</span>
          <span class="text-xs text-[#38D9C0] font-medium hidden sm:inline">Proof Over Hallucination</span>
        </div>

        {/* CENTER TABS (ANSWER / SOURCES) WHEN IN ANSWER VIEW */}
        <Show when={currentView() === "answer"}>
          <div class="flex items-center gap-1 bg-[#181C20] p-1 rounded-xl border border-[#252B30]">
            <button
              onClick={() => setActiveTab("answer")}
              class={`px-3 py-1 rounded-lg text-xs font-medium transition flex items-center gap-1.5 ${
                activeTab() === "answer"
                  ? "bg-[#252B30] text-[#E8EDF0] shadow-sm"
                  : "text-[#929DA6] hover:text-[#E8EDF0]"
              }`}
            >
              <svg class="w-3.5 h-3.5 text-[#38D9C0]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <span>Answer</span>
            </button>
            <button
              onClick={() => setActiveTab("sources")}
              class={`px-3 py-1 rounded-lg text-xs font-medium transition flex items-center gap-1.5 ${
                activeTab() === "sources"
                  ? "bg-[#252B30] text-[#E8EDF0] shadow-sm"
                  : "text-[#929DA6] hover:text-[#E8EDF0]"
              }`}
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
              </svg>
              <span>Sources</span>
              <Show when={activeResult()?.citations.length}>
                <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-[#38D9C0]/20 text-[#38D9C0] font-mono">
                  {activeResult()?.citations.length}
                </span>
              </Show>
            </button>
          </div>
        </Show>

        {/* RIGHT CONTROLS */}
        <div class="flex items-center gap-2">
          {/* Sources Count Badge button (Clickable to switch tab) */}
          <Show when={activeResult()?.citations.length}>
            <button
              onClick={() => setActiveTab(activeTab() === "sources" ? "answer" : "sources")}
              class="px-2.5 py-1 rounded-lg bg-[#181C20] hover:bg-[#1F2429] border border-[#252B30] text-xs text-[#E8EDF0] font-medium flex items-center gap-2 transition"
              title="Toggle Sources List"
            >
              <span class="text-xs text-[#929DA6]">Sources</span>
              <div class="flex -space-x-1.5 items-center">
                <For each={activeResult()?.citations.slice(0, 3)}>
                  {(url) => (
                    <img
                      src={getFaviconUrl(url)}
                      class="w-3.5 h-3.5 rounded-full bg-white/10 border border-[#181C20]"
                      onError={(e) => (e.currentTarget.style.display = "none")}
                      alt=""
                    />
                  )}
                </For>
              </div>
              <span class="text-xs text-[#38D9C0] font-mono font-semibold">
                {activeResult()?.citations.length} &gt;
              </span>
            </button>
          </Show>

          {/* Share / Copy Menu */}
          <Show when={activeResult()?.final_report}>
            <button
              onClick={copyReport}
              class="px-2.5 py-1 rounded-lg bg-[#181C20] hover:bg-[#1F2429] border border-[#252B30] text-xs text-[#929DA6] hover:text-[#E8EDF0] flex items-center gap-1.5 transition"
              title="Copy answer to clipboard"
            >
              <Show when={copyFeedback()} fallback={
                <>
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z" />
                  </svg>
                  <span class="hidden sm:inline">Share</span>
                </>
              }>
                <span class="text-[#34D399] font-medium">✓ Copied</span>
              </Show>
            </button>
          </Show>

          {/* Settings Button */}
          <button
            onClick={() => setSettingsOpen(true)}
            class="p-1.5 rounded-lg text-[#929DA6] hover:text-[#E8EDF0] hover:bg-white/5 transition"
            title="Settings"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </button>
        </div>
      </header>

      {/* WORKSPACE CONTENT AREA */}
      <div class="flex-1 flex overflow-hidden relative">

        {/* HIDEABLE SIDEBAR (GROUPED BY TODAY, YESTERDAY, EARLIER) */}
        <Show when={sidebarOpen()}>
          <aside class="w-68 border-r border-[#252B30] bg-[#121518] flex flex-col justify-between shrink-0 transition-all select-none z-20">
            <div class="p-3 flex flex-col gap-3 overflow-hidden">
              <button
                onClick={() => {
                  setQuery("");
                  setActiveResult(null);
                  setCurrentView("search");
                }}
                class="w-full flex items-center justify-between py-2 px-3 rounded-xl bg-[#181C20] hover:bg-[#1F2429] text-[#E8EDF0] font-medium text-xs transition border border-[#252B30] shadow-sm hover:border-[#38D9C0]/30"
              >
                <div class="flex items-center gap-2">
                  <span class="text-[#38D9C0] text-sm">+</span>
                  <span>New Search</span>
                </div>
                <span class="text-[10px] text-[#64707D] font-mono">Shift+Enter</span>
              </button>

              <div class="flex-1 overflow-y-auto flex flex-col gap-3 pr-1">
                {/* TODAY */}
                <Show when={groupedHistory().today.length > 0}>
                  <div>
                    <div class="text-[11px] font-semibold text-[#64707D] uppercase tracking-wider px-2 py-1">
                      Today
                    </div>
                    <div class="flex flex-col gap-0.5">
                      <For each={groupedHistory().today}>
                        {(item) => (
                          <div
                            onClick={() => handleSelectSession(item)}
                            class={`group flex items-center justify-between p-2 rounded-lg cursor-pointer transition text-left ${
                              activeSessionId() === item.id
                                ? "bg-[#181C20] text-[#38D9C0] border border-[#38D9C0]/30"
                                : "hover:bg-[#181C20] text-[#929DA6] hover:text-[#E8EDF0]"
                            }`}
                          >
                            <div class="truncate flex-1 mr-2">
                              <div class="font-medium text-xs truncate">{item.query}</div>
                              <div class="text-[10px] text-[#64707D] mt-0.5 flex items-center gap-1.5">
                                <span>{item.mode}</span>
                                <span>•</span>
                                <span>{item.created_at.slice(11, 16)}</span>
                              </div>
                            </div>
                            <button
                              onClick={(e) => handleDeleteSession(item.id, e)}
                              class="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-rose-500/20 text-[#64707D] hover:text-rose-400 transition"
                              title="Delete search"
                            >
                              ✕
                            </button>
                          </div>
                        )}
                      </For>
                    </div>
                  </div>
                </Show>

                {/* YESTERDAY */}
                <Show when={groupedHistory().yesterday.length > 0}>
                  <div>
                    <div class="text-[11px] font-semibold text-[#64707D] uppercase tracking-wider px-2 py-1">
                      Yesterday
                    </div>
                    <div class="flex flex-col gap-0.5">
                      <For each={groupedHistory().yesterday}>
                        {(item) => (
                          <div
                            onClick={() => handleSelectSession(item)}
                            class={`group flex items-center justify-between p-2 rounded-lg cursor-pointer transition text-left ${
                              activeSessionId() === item.id
                                ? "bg-[#181C20] text-[#38D9C0] border border-[#38D9C0]/30"
                                : "hover:bg-[#181C20] text-[#929DA6] hover:text-[#E8EDF0]"
                            }`}
                          >
                            <div class="truncate flex-1 mr-2">
                              <div class="font-medium text-xs truncate">{item.query}</div>
                              <div class="text-[10px] text-[#64707D] mt-0.5">
                                {item.mode}
                              </div>
                            </div>
                            <button
                              onClick={(e) => handleDeleteSession(item.id, e)}
                              class="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-rose-500/20 text-[#64707D] hover:text-rose-400 transition"
                              title="Delete search"
                            >
                              ✕
                            </button>
                          </div>
                        )}
                      </For>
                    </div>
                  </div>
                </Show>

                {/* EARLIER */}
                <Show when={groupedHistory().older.length > 0}>
                  <div>
                    <div class="text-[11px] font-semibold text-[#64707D] uppercase tracking-wider px-2 py-1">
                      Earlier
                    </div>
                    <div class="flex flex-col gap-0.5">
                      <For each={groupedHistory().older}>
                        {(item) => (
                          <div
                            onClick={() => handleSelectSession(item)}
                            class={`group flex items-center justify-between p-2 rounded-lg cursor-pointer transition text-left ${
                              activeSessionId() === item.id
                                ? "bg-[#181C20] text-[#38D9C0] border border-[#38D9C0]/30"
                                : "hover:bg-[#181C20] text-[#929DA6] hover:text-[#E8EDF0]"
                            }`}
                          >
                            <div class="truncate flex-1 mr-2">
                              <div class="font-medium text-xs truncate">{item.query}</div>
                              <div class="text-[10px] text-[#64707D] mt-0.5">
                                {item.created_at.slice(0, 10)}
                              </div>
                            </div>
                            <button
                              onClick={(e) => handleDeleteSession(item.id, e)}
                              class="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-rose-500/20 text-[#64707D] hover:text-rose-400 transition"
                              title="Delete search"
                            >
                              ✕
                            </button>
                          </div>
                        )}
                      </For>
                    </div>
                  </div>
                </Show>

                {/* EMPTY FALLBACK */}
                <Show when={history().length === 0}>
                  <div class="text-[11px] text-[#64707D] px-2 py-6 italic text-center">
                    No recent searches yet.
                  </div>
                </Show>
              </div>
            </div>

            <div class="p-3 border-t border-[#252B30] flex items-center justify-between text-[11px] text-[#64707D]">
              <span class="flex items-center gap-1.5 text-[#34D399]">
                <span class="w-1.5 h-1.5 rounded-full bg-[#34D399] animate-pulse"></span> Engine Ready
              </span>
              <span class="font-mono text-[#64707D]">v2.1.5</span>
            </div>
          </aside>
        </Show>

        {/* MAIN CANVAS */}
        <main class="flex-1 overflow-y-auto flex flex-col relative bg-[#0C0E10]">

          {/* VIEW 1: HERO SEARCH START PAGE */}
          <Show when={currentView() === "search"}>
            <section class="flex-1 flex flex-col items-center justify-center p-6 max-w-3xl mx-auto w-full select-none">

              {/* REAL APP ICON */}
              <img
                src="/icon.svg"
                class="w-14 h-14 rounded-2xl mb-5 shadow-xl shadow-[#38D9C0]/10 hover:scale-105 transition-transform duration-300"
                alt="Research Agent"
                onError={(e) => {
                  e.currentTarget.style.display = "none";
                }}
              />

              {/* ANIMATED ROTATING TAGLINE (SINGLE KEYWORD ROTATION) */}
              <div class="mb-6 flex items-center justify-center w-full select-none">
                <h1 class="text-2xl md:text-3xl font-semibold text-[#E8EDF0] tracking-tight text-center flex items-center justify-center gap-2 flex-wrap">
                  <span>What do you want to</span>
                  <span class="rotating-word-slot">
                    <span
                      class={`rotating-word ${
                        wordVisible() ? "rotating-word-enter" : "rotating-word-exit"
                      }`}
                    >
                      {ROTATING_WORDS[wordIndex()]}
                    </span>
                  </span>
                  <span>today?</span>
                </h1>
              </div>

              {/* HERO SEARCH CONTAINER */}
              <div class="w-full bg-[#121518] border border-[#252B30] focus-within:border-[#38D9C0]/50 rounded-2xl p-4 shadow-2xl transition flex flex-col gap-3">
                <textarea
                  rows="3"
                  value={query()}
                  onInput={(e) => setQuery(e.currentTarget.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && !e.shiftKey) {
                      e.preventDefault();
                      handleRunSearch();
                    }
                  }}
                  placeholder="Ask a question or enter a topic to investigate... (Enter to verify, Shift+Enter for newline)"
                  class="w-full bg-transparent border-0 resize-none text-sm md:text-base text-[#E8EDF0] placeholder-[#64707D] focus:outline-none leading-relaxed"
                />

                <div class="flex items-center justify-between pt-3 border-t border-[#252B30]">
                  <div class="flex items-center gap-2">
                    <span class="text-xs text-[#929DA6]">Mode:</span>
                    <select
                      value={mode()}
                      onChange={(e) => setMode(e.currentTarget.value)}
                      class="bg-[#181C20] border border-[#252B30] text-[#38D9C0] text-xs rounded-lg px-2.5 py-1.5 focus:outline-none focus:border-[#38D9C0]/60 font-medium"
                    >
                      <option>Deep Search</option>
                      <option>Instant Search</option>
                      <option>Compare Search</option>
                      <option>Offline Search</option>
                    </select>

                    <Show when={mode() === "Deep Search"}>
                      <span class="text-xs text-[#929DA6] ml-2">Layers:</span>
                      <select
                        value={layers()}
                        onChange={(e) => setLayers(Number(e.currentTarget.value))}
                        class="bg-[#181C20] border border-[#252B30] text-[#E8EDF0] text-xs rounded-lg px-2 py-1.5 focus:outline-none"
                      >
                        <option value="3">3 Layers</option>
                        <option value="5">5 Layers</option>
                        <option value="10">10 Layers</option>
                      </select>
                    </Show>
                  </div>

                  <button
                    onClick={() => handleRunSearch()}
                    disabled={isSearching() || !query().trim()}
                    class="px-4 py-2 rounded-xl bg-[#38D9C0] hover:bg-[#2EC4AC] text-black font-semibold text-xs flex items-center gap-1.5 shadow-lg shadow-[#38D9C0]/20 transition disabled:opacity-40 disabled:cursor-not-allowed"
                  >
                    <span>Investigate</span>
                    <span>➔</span>
                  </button>
                </div>
              </div>

              <Show when={searchError()}>
                <div class="mt-4 text-xs text-rose-400 p-3 rounded-lg bg-rose-500/10 border border-rose-500/20 max-w-xl text-center">
                  {searchError()}
                </div>
              </Show>

              <div class="flex items-center gap-4 text-[11px] text-[#64707D] mt-6 text-center">
                <span>⚡ Enter to run</span>
                <span>•</span>
                <span>🛡️ SSRF Protected</span>
                <span>•</span>
                <span>📰 Google News RSS & Hacker News Enabled</span>
              </div>
            </section>
          </Show>

          {/* VIEW 2: PERPLEXITY-GRADE WORKSPACE VIEW */}
          <Show when={currentView() === "answer"}>
            <section class="flex-1 p-6 md:p-10 max-w-5xl mx-auto w-full flex flex-col gap-6">

              {/* USER QUERY PILL (TOP RIGHT) */}
              <div class="flex justify-end">
                <div class="bg-[#181C20] border border-[#252B30] text-[#E8EDF0] text-sm md:text-base font-medium px-4 py-2.5 rounded-2xl max-w-2xl shadow-lg leading-snug">
                  {activeResult()?.query}
                </div>
              </div>

              {/* SHOW ANSWER TAB */}
              <Show when={activeTab() === "answer"}>
                {/* RESEARCH ACTIVITY / PROCESS DISCLOSURE ACCORDION */}
                <div class="rounded-2xl border border-[#252B30] bg-[#121518] overflow-hidden shadow-lg">
                  <div
                    onClick={() => setSchematicOpen(!schematicOpen())}
                    class="flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-white/[0.02] transition"
                  >
                    <div class="flex items-center gap-3">
                      <Show
                        when={!isSearching()}
                        fallback={
                          <div class="flex items-center gap-2.5">
                            <span class="w-2.5 h-2.5 rounded-full bg-[#38D9C0] animate-ping"></span>
                            <span class="text-xs text-[#38D9C0] font-medium">{liveMessage()}</span>
                          </div>
                        }
                      >
                        <div class="flex items-center gap-2">
                          <svg class="w-4 h-4 text-[#34D399]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                          </svg>
                          <span class="text-xs text-[#929DA6] font-medium">
                            Finished in {elapsedSeconds() || 3}s • {activeResult()?.mode}
                          </span>
                        </div>
                        <Show when={activeResult()?.stopped_early}>
                          <span class="text-[10px] px-2 py-0.5 rounded bg-[#34D399]/15 text-[#34D399] border border-[#34D399]/30 font-medium">
                            Proof Grounded
                          </span>
                        </Show>
                      </Show>
                    </div>

                    <div class="flex items-center gap-2 text-xs text-[#929DA6]">
                      <Show when={hapticEvents().length > 0}>
                        <div class="flex items-center bg-[#181C20] rounded-lg p-0.5 border border-[#252B30] mr-1">
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              setAccordionView("summary");
                              setSchematicOpen(true);
                            }}
                            class={`px-2 py-0.5 rounded text-[11px] font-medium transition ${
                              accordionView() === "summary"
                                ? "bg-[#252B30] text-[#E8EDF0]"
                                : "text-[#929DA6] hover:text-[#E8EDF0]"
                            }`}
                          >
                            Summary
                          </button>
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              setAccordionView("trace");
                              setSchematicOpen(true);
                            }}
                            class={`px-2 py-0.5 rounded text-[11px] font-medium transition flex items-center gap-1.5 ${
                              accordionView() === "trace"
                                ? "bg-[#252B30] text-[#38D9C0]"
                                : "text-[#929DA6] hover:text-[#E8EDF0]"
                            }`}
                          >
                            <span>Live Trace</span>
                            <span class="text-[10px] px-1 rounded-full bg-[#38D9C0]/20 text-[#38D9C0] font-mono">
                              {hapticEvents().length}
                            </span>
                          </button>
                        </div>
                      </Show>
                      <span>{schematicOpen() ? "Hide Steps" : "Show Steps"}</span>
                      <span>{schematicOpen() ? "▴" : "▾"}</span>
                    </div>
                  </div>

                  <Show when={schematicOpen()}>
                    <div class="px-5 pb-5 pt-2 border-t border-[#252B30] flex flex-col gap-4 text-xs">

                      {/* SUB-VIEW 1: SUMMARY */}
                      <Show when={accordionView() === "summary"}>
                        {/* 1. DOMAIN GROUNDING */}
                        <Show when={liveWords().length || activeResult()?.constraints?.length}>
                          <div class="flex flex-col gap-1.5">
                            <div class="text-[11px] font-semibold text-[#929DA6] uppercase tracking-wider flex items-center gap-2">
                              <span>✦ Domain Keywords & Constraints:</span>
                            </div>
                            <div class="flex flex-wrap gap-1.5">
                              <For each={liveWords().length ? liveWords() : activeResult()?.constraints?.map((c) => c.word)}>
                                {(w) => (
                                  <span class="px-2 py-0.5 rounded-md bg-[#181C20] text-[#38D9C0] border border-[#252B30] font-mono text-[11px]">
                                    "{w}"
                                  </span>
                                )}
                              </For>
                            </div>
                          </div>
                        </Show>

                        {/* 2. SEARCH QUERIES & LIVE SOURCES CONSULTED */}
                        <div class="flex flex-col gap-2">
                          <div class="text-[11px] font-semibold text-[#929DA6] uppercase tracking-wider flex items-center justify-between">
                            <span>🌐 Targeted Search & Verification:</span>
                            <span class="text-[10px] text-[#64707D]">
                              {liveSourcesCount() || activeResult()?.citations?.length || 0} Sources Consulted
                            </span>
                          </div>

                          {/* Subqueries List */}
                          <div class="flex flex-col gap-1.5">
                            <For each={liveSubqueries().length ? liveSubqueries() : activeResult()?.layers?.map((l) => l.why_question)}>
                              {(sq) => (
                                <div class="flex items-center gap-2 text-[#E8EDF0] bg-[#181C20] px-3 py-1.5 rounded-lg border border-[#252B30]">
                                  <span class="text-[#38D9C0] font-mono text-[11px]">🔍</span>
                                  <span class="truncate font-mono text-[11.5px]">{sq}</span>
                                </div>
                              )}
                            </For>
                          </div>

                          {/* Top Sources Pill List */}
                          <Show when={activeResult()?.citations.length}>
                            <div class="pt-2 flex flex-col gap-1.5">
                              <div class="flex flex-wrap gap-2">
                                <For each={showAllSourcesInAccordion() ? activeResult()?.citations : activeResult()?.citations.slice(0, 5)}>
                                  {(url) => (
                                    <a
                                      href={url}
                                      target="_blank"
                                      class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-[#181C20] hover:bg-[#1F2429] border border-[#252B30] text-[#E8EDF0] text-[11px] transition max-w-xs truncate"
                                    >
                                      <img
                                        src={getFaviconUrl(url)}
                                        class="w-3.5 h-3.5 rounded-full"
                                        onError={(e) => (e.currentTarget.style.display = "none")}
                                        alt=""
                                      />
                                      <span class="truncate">{getDomain(url)}</span>
                                      <span class="text-[#64707D] font-mono text-[10px]">↗</span>
                                    </a>
                                  )}
                                </For>
                              </div>

                              <Show when={(activeResult()?.citations.length || 0) > 5}>
                                <button
                                  onClick={() => setShowAllSourcesInAccordion(!showAllSourcesInAccordion())}
                                  class="text-[11px] text-[#38D9C0] hover:underline self-start mt-1 font-medium"
                                >
                                  {showAllSourcesInAccordion()
                                    ? "Show fewer sources"
                                    : `+${(activeResult()?.citations.length || 0) - 5} more sources`}
                                </button>
                              </Show>
                            </div>
                          </Show>
                        </div>
                      </Show>

                      {/* SUB-VIEW 2: LIVE HAPTIC EXECUTION TRACE */}
                      <Show when={accordionView() === "trace"}>
                        <div class="flex flex-col gap-2">
                          <div class="text-[11px] font-semibold text-[#929DA6] uppercase tracking-wider flex items-center justify-between">
                            <span>⚡ Step-by-Step Execution Trace:</span>
                            <span class="text-[10px] text-[#64707D] font-mono">
                              {hapticEvents().length} milestones logged
                            </span>
                          </div>
                          <div class="haptic-panel rounded-xl border border-[#252B30] bg-[#0A0C0E] p-3 flex flex-col gap-1.5 overflow-y-auto max-h-80">
                            <For each={hapticEvents()}>
                              {(ev) => (
                                <div class="haptic-line flex items-start gap-2.5 py-1 hover:bg-white/[0.02] rounded px-1">
                                  <span class="w-1.5 h-1.5 rounded-full bg-[#38D9C0] mt-1.5 shrink-0" />
                                  <div class="flex-1 min-w-0">
                                    <span class="text-[#E8EDF0] text-xs">{ev.message}</span>
                                    <Show when={ev.detail}>
                                      <span class="text-[#64707D] ml-2 font-mono text-[11px] break-all">{ev.detail}</span>
                                    </Show>
                                  </div>
                                </div>
                              )}
                            </For>
                          </div>
                        </div>
                      </Show>

                    </div>
                  </Show>
                </div>

                {/* ERROR DIAGNOSTICS CARD */}
                <Show when={searchError()}>
                  <div class="rounded-2xl border border-rose-500/30 bg-rose-500/10 p-4 md:p-5 flex flex-col gap-3 shadow-lg">
                    <div class="flex items-center gap-2 text-rose-400 font-semibold text-xs md:text-sm">
                      <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                      </svg>
                      <span>Research Stalled or Interrupted</span>
                    </div>
                    <div class="text-xs text-rose-200/90 leading-relaxed font-mono bg-black/40 p-3 rounded-xl border border-rose-500/20 break-words">
                      {searchError()}
                    </div>
                    <div class="flex items-center gap-3 pt-1">
                      <button
                        onClick={() => handleRunSearch()}
                        class="px-3.5 py-1.5 rounded-xl bg-rose-500 hover:bg-rose-400 text-black font-semibold text-xs transition"
                      >
                        Retry Research
                      </button>
                      <button
                        onClick={() => setSettingsOpen(true)}
                        class="px-3.5 py-1.5 rounded-xl bg-white/10 hover:bg-white/15 text-xs text-gray-200 transition border border-white/10"
                      >
                        Check Settings / LLM URL
                      </button>
                    </div>
                  </div>
                </Show>

                {/* RENDERED MARKDOWN REPORT CANVAS — Smart Template Router */}
                <Show
                  when={activeResult()?.final_report}
                  fallback={
                    <Show when={!searchError()}>
                      {/* HAPTIC STREAMING PANEL — shown while searching */}
                      <div class="flex flex-col gap-4">
                        <div
                          ref={hapticScrollRef}
                          class="haptic-panel rounded-2xl border border-[#252B30] bg-[#0D0F11] p-4 flex flex-col gap-2.5 overflow-y-auto max-h-72"
                        >
                          <Show when={hapticEvents().length === 0}>
                            <div class="flex items-center gap-3 py-6 justify-center">
                              <span class="w-5 h-5 border-2 border-[#38D9C0] border-t-transparent rounded-full animate-spin" />
                              <span class="text-xs text-[#929DA6]">{liveMessage()}</span>
                            </div>
                          </Show>
                          <For each={hapticEvents()}>
                            {(ev, i) => (
                              <div
                                class={`haptic-line flex items-start gap-3 text-xs ${
                                  i() >= hapticEvents().length - 1
                                    ? "opacity-100"
                                    : i() >= hapticEvents().length - 4
                                    ? "opacity-50"
                                    : "opacity-25"
                                }`}
                              >
                                <span class="w-1.5 h-1.5 rounded-full bg-[#38D9C0] mt-1.5 shrink-0" />
                                <div class="flex-1 min-w-0">
                                  <span class="text-[#E8EDF0]">{ev.message}</span>
                                  <Show when={ev.detail}>
                                    <span class="text-[#64707D] ml-2 font-mono text-[11px] break-all">{ev.detail}</span>
                                  </Show>
                                </div>
                              </div>
                            )}
                          </For>
                          {/* Blinking cursor — only when searching */}
                          <Show when={isSearching()}>
                            <div class="flex items-center gap-2 mt-1">
                              <span class="w-1.5 h-1.5 rounded-full bg-[#38D9C0] animate-pulse shrink-0" />
                              <span class="text-[11px] text-[#38D9C0] font-mono">processing...</span>
                            </div>
                          </Show>
                        </div>
                      </div>
                    </Show>
                  }
                >
                  <Switch
                    fallback={
                      <article
                        class="prose-workspace"
                        innerHTML={renderEnrichedMarkdown(activeTemplate().body)}
                      />
                    }
                  >
                    {/* RANKED LIST TEMPLATE */}
                    <Match when={activeTemplate().hint === "ranked_list"}>
                      {(() => {
                        const items = parseRankedItems(activeTemplate().body);
                        if (items.length === 0) {
                          return (
                            <article
                              class="prose-workspace"
                              innerHTML={renderEnrichedMarkdown(activeTemplate().body)}
                            />
                          );
                        }
                        const summaryMatch = activeTemplate().body.match(/^([\s\S]*?)(?=^##\s+Candidate|^###\s|\Z)/m);
                        const summaryMd = summaryMatch ? summaryMatch[1].trim() : "";
                        return (
                          <div class="flex flex-col gap-3">
                            <Show when={summaryMd}>
                              <article class="prose-workspace" innerHTML={renderEnrichedMarkdown(summaryMd)} />
                            </Show>
                            <For each={items}>
                              {(item, i) => (
                                <div class="ranked-card flex gap-4 p-4 rounded-xl bg-[#121518] border border-[#252B30] hover:border-[#38D9C0]/30 transition-all duration-150">
                                  <div class="rank-badge w-9 h-9 rounded-xl bg-[#38D9C0]/10 border border-[#38D9C0]/30 flex items-center justify-center font-bold text-[#38D9C0] text-sm shrink-0">
                                    {i() + 1}
                                  </div>
                                  <div class="flex-1 min-w-0">
                                    <div class="font-semibold text-[#E8EDF0] text-sm leading-snug">{item.title}</div>
                                    <Show when={item.summary}>
                                      <div class="text-xs text-[#929DA6] mt-1.5 leading-relaxed">{item.summary}</div>
                                    </Show>
                                    <Show when={item.badges.length > 0}>
                                      <div class="flex gap-2 mt-2 flex-wrap">
                                        <For each={item.badges}>
                                          {(b) => (
                                            <span class={`px-2 py-0.5 rounded text-[11px] font-medium ${
                                              b.toLowerCase().includes("confirm") ? "badge-confirmed" :
                                              b.toLowerCase().includes("roadmap") ? "badge-roadmap" :
                                              "badge-projected"
                                            }`}>{b}</span>
                                          )}
                                        </For>
                                      </div>
                                    </Show>
                                  </div>
                                </div>
                              )}
                            </For>
                          </div>
                        );
                      })()}
                    </Match>

                    {/* NEWS FEED TEMPLATE */}
                    <Match when={activeTemplate().hint === "news_feed"}>
                      {(() => {
                        const items = parseNewsItems(activeTemplate().body);
                        if (items.length === 0) {
                          return (
                            <article
                              class="prose-workspace"
                              innerHTML={renderEnrichedMarkdown(activeTemplate().body)}
                            />
                          );
                        }
                        const summaryMatch = activeTemplate().body.match(/^([\s\S]*?)(?=^##\s+Candidate|^###\s|^[-*]\s+\*\*|\Z)/m);
                        const summaryMd = summaryMatch ? summaryMatch[1].trim() : "";
                        return (
                          <div class="flex flex-col gap-3">
                            <Show when={summaryMd}>
                              <article class="prose-workspace" innerHTML={renderEnrichedMarkdown(summaryMd)} />
                            </Show>
                            <For each={items}>
                              {(item) => (
                                <div class="news-card p-4 rounded-xl bg-[#121518] border border-[#252B30] hover:border-[#38D9C0]/20 transition-all duration-150 flex flex-col gap-2">
                                  <div class="font-semibold text-[#E8EDF0] text-sm leading-snug">{item.title}</div>
                                  <Show when={item.summary}>
                                    <p class="text-xs text-[#929DA6] leading-relaxed">{item.summary}</p>
                                  </Show>
                                  <Show when={item.citationNums.length > 0}>
                                    <div class="flex gap-1.5 flex-wrap mt-1">
                                      <For each={item.citationNums}>
                                        {(n) => (
                                          <Show when={activeResult()?.citations[n - 1]}>
                                            <a
                                              href={activeResult()!.citations[n - 1]}
                                              target="_blank"
                                              class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-[#181C20] border border-[#252B30] hover:border-[#38D9C0]/40 text-[11px] text-[#929DA6] hover:text-[#38D9C0] transition font-mono"
                                            >
                                              <img
                                                src={getFaviconUrl(activeResult()!.citations[n - 1])}
                                                class="w-3 h-3 rounded-full"
                                                onError={(e) => (e.currentTarget.style.display = "none")}
                                                alt=""
                                              />
                                              <span class="news-card-source">{getDomain(activeResult()!.citations[n - 1])}</span>
                                              <span class="text-[#64707D]">↗</span>
                                            </a>
                                          </Show>
                                        )}
                                      </For>
                                    </div>
                                  </Show>
                                </div>
                              )}
                            </For>
                          </div>
                        );
                      })()}
                    </Match>

                    {/* COMPARISON TABLE — enhanced prose with extra table styling */}
                    <Match when={activeTemplate().hint === "comparison_table"}>
                      <div class="flex flex-col gap-4">
                        <article
                          class="prose-workspace comparison-enhanced"
                          innerHTML={renderEnrichedMarkdown(activeTemplate().body)}
                        />
                      </div>
                    </Match>

                    {/* FLASHCARDS TEMPLATE */}
                    <Match when={activeTemplate().hint === "flashcards"}>
                      <div class="flex flex-col gap-3">
                        {(() => {
                          const summaryMatch = activeTemplate().body.match(/^([\s\S]*?)(?=^###\s|\Z)/m);
                          const summaryMd = summaryMatch ? summaryMatch[1].trim() : "";
                          return summaryMd ? (
                            <article class="prose-workspace" innerHTML={renderEnrichedMarkdown(summaryMd)} />
                          ) : null;
                        })()}
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                          <For each={parseRankedItems(activeTemplate().body)}>
                            {(item) => {
                              const [flipped, setFlipped] = createSignal(false);
                              return (
                                <div
                                  class={`flashcard cursor-pointer ${flipped() ? "flipped" : ""}`}
                                  onClick={() => setFlipped(!flipped())}
                                  style="perspective: 800px; min-height: 120px;"
                                >
                                  <div class="flashcard-inner relative w-full h-full" style="transform-style: preserve-3d; transition: transform 0.5s cubic-bezier(0.16,1,0.3,1); transform: rotateY(0deg);">
                                    {/* Front */}
                                    <div class="flashcard-front absolute inset-0 rounded-xl bg-[#121518] border border-[#252B30] p-4 flex flex-col justify-between" style="backface-visibility: hidden;">
                                      <div class="font-semibold text-[#E8EDF0] text-sm">{item.title}</div>
                                      <div class="text-[11px] text-[#38D9C0] mt-auto pt-2 font-mono">tap to reveal →</div>
                                    </div>
                                    {/* Back */}
                                    <div class="flashcard-back absolute inset-0 rounded-xl bg-[#38D9C0]/5 border border-[#38D9C0]/30 p-4 flex flex-col justify-center" style="backface-visibility: hidden; transform: rotateY(180deg);">
                                      <div class="text-xs text-[#929DA6] leading-relaxed">{item.summary || "No details available."}</div>
                                    </div>
                                  </div>
                                </div>
                              );
                            }}
                          </For>
                        </div>
                      </div>
                    </Match>
                  </Switch>
                </Show>
              </Show>

              {/* SHOW SOURCES TAB (WHEN CLICKED IN HEADER TABS) */}
              <Show when={activeTab() === "sources"}>
                <div class="flex flex-col gap-4">
                  <div class="flex items-center justify-between pb-2 border-b border-[#252B30]">
                    <div>
                      <h2 class="text-lg font-semibold text-[#E8EDF0]">Verified Proof Sources</h2>
                      <p class="text-xs text-[#929DA6]">
                        {activeResult()?.citations.length || 0} external citations inspected for this research
                      </p>
                    </div>
                    <button
                      onClick={() => setActiveTab("answer")}
                      class="text-xs text-[#38D9C0] hover:underline font-medium"
                    >
                      ← Back to Answer
                    </button>
                  </div>

                  <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <For each={activeResult()?.citations}>
                      {(url) => (
                        <a
                          href={url}
                          target="_blank"
                          class="p-3.5 rounded-xl bg-[#121518] hover:bg-[#181C20] border border-[#252B30] hover:border-[#38D9C0]/40 transition flex items-start gap-3 group"
                        >
                          <img
                            src={getFaviconUrl(url)}
                            class="w-4 h-4 mt-0.5 rounded-full"
                            onError={(e) => (e.currentTarget.style.display = "none")}
                            alt=""
                          />
                          <div class="flex-1 overflow-hidden">
                            <div class="text-xs font-medium text-[#E8EDF0] group-hover:text-[#38D9C0] transition truncate">
                              {getDomain(url)}
                            </div>
                            <div class="text-[11px] text-[#64707D] font-mono truncate mt-0.5">
                              {url}
                            </div>
                          </div>
                          <span class="text-[#64707D] group-hover:text-[#38D9C0] font-mono text-xs">↗</span>
                        </a>
                      )}
                    </For>
                  </div>
                </div>
              </Show>

              {/* STICKY BOTTOM ACTION COMPOSER (RAYCAST / LINEAR / PERPLEXITY STYLE) */}
              <div class="sticky bottom-4 mt-6 bg-[#121518]/95 backdrop-blur-md border border-[#252B30] rounded-2xl p-3 shadow-2xl flex flex-col gap-2.5 z-30">
                <div class="flex items-center gap-2">
                  <input
                    type="text"
                    value={followUpQuery()}
                    onInput={(e) => setFollowUpQuery(e.currentTarget.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && !e.shiftKey) {
                        e.preventDefault();
                        handleRunSearch(followUpQuery());
                      }
                    }}
                    placeholder="Ask a follow-up or explore another angle..."
                    class="flex-1 bg-transparent px-3 py-1.5 text-sm text-[#E8EDF0] placeholder-[#64707D] focus:outline-none"
                  />
                  <button
                    onClick={() => handleRunSearch(followUpQuery())}
                    disabled={isSearching() || !followUpQuery().trim()}
                    class="w-8 h-8 rounded-xl bg-[#38D9C0] hover:bg-[#2EC4AC] text-black font-semibold text-sm flex items-center justify-center transition disabled:opacity-40"
                    title="Send"
                  >
                    ↑
                  </button>
                </div>

                <div class="flex items-center justify-between pt-2 border-t border-[#252B30] text-xs text-[#929DA6]">
                  <div class="flex items-center gap-2">
                    <button
                      onClick={exportMarkdown}
                      class="px-2.5 py-1 rounded-lg bg-[#181C20] hover:bg-[#1F2429] hover:text-[#E8EDF0] transition flex items-center gap-1.5 border border-[#252B30]"
                      title="Export report as Markdown (.md)"
                    >
                      <svg class="w-3.5 h-3.5 text-[#38D9C0]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                      </svg>
                      <Show when={exportStatus()} fallback={<span>.MD</span>}>
                        {(status) => <span class="text-[#34D399] font-medium">{status()}</span>}
                      </Show>
                    </button>
                    <button
                      onClick={exportHtml}
                      class="px-2.5 py-1 rounded-lg bg-[#181C20] hover:bg-[#1F2429] hover:text-[#E8EDF0] transition flex items-center gap-1.5 border border-[#252B30]"
                      title="Export report as styled HTML with dark mode & citations (.html)"
                    >
                      <svg class="w-3.5 h-3.5 text-[#38D9C0]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                      </svg>
                      <Show when={exportHtmlStatus()} fallback={<span>.HTML</span>}>
                        {(status) => <span class="text-[#34D399] font-medium">{status()}</span>}
                      </Show>
                    </button>
                    <button
                      onClick={() => window.print()}
                      class="px-2.5 py-1 rounded-lg bg-[#181C20] hover:bg-[#1F2429] hover:text-[#E8EDF0] transition flex items-center gap-1.5 border border-[#252B30]"
                      title="Print or Save as PDF"
                    >
                      <svg class="w-3.5 h-3.5 text-[#38D9C0]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 17h2a2 2 0 002-2v-4a2 2 0 00-2-2H5a2 2 0 00-2 2v4a2 2 0 002 2h2m2 4h6a2 2 0 002-2v-4a2 2 0 00-2-2H9a2 2 0 00-2 2v4a2 2 0 002 2zm8-12V5a2 2 0 00-2-2H9a2 2 0 00-2 2v4h10z" />
                      </svg>
                      <span>.PDF</span>
                    </button>
                    <button
                      onClick={copyReport}
                      class="px-2.5 py-1 rounded-lg bg-[#181C20] hover:bg-[#1F2429] hover:text-[#E8EDF0] transition flex items-center gap-1.5 border border-[#252B30]"
                      title="Copy clean report to clipboard"
                    >
                      <Show when={copyFeedback()} fallback={<span>📋 Copy</span>}>
                        <span class="text-[#34D399] font-medium">✓ Copied</span>
                      </Show>
                    </button>
                  </div>

                  <div class="flex items-center gap-3">
                    <span class="text-[11px] text-[#64707D] hidden sm:inline">
                      Model: <span class="text-[#929DA6] font-mono">{llmModel()}</span>
                    </span>
                    <button
                      onClick={() => {
                        setQuery("");
                        setActiveResult(null);
                        setCurrentView("search");
                      }}
                      class="text-[#929DA6] hover:text-[#38D9C0] transition font-medium"
                    >
                      + New Search
                    </button>
                  </div>
                </div>
              </div>

            </section>
          </Show>

        </main>
      </div>

      {/* SETTINGS MODAL */}
      <Show when={settingsOpen()}>
        <div class="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div class="bg-[#121518] border border-[#252B30] rounded-2xl w-full max-w-2xl p-6 shadow-2xl flex flex-col gap-4 max-h-[90vh] overflow-hidden">
            {/* MODAL HEADER & TAB SWITCHER */}
            <div class="flex items-center justify-between pb-3 border-b border-[#252B30] shrink-0">
              <div class="flex items-center gap-2">
                <button
                  onClick={() => setSettingsTab("engine")}
                  class={`px-3 py-1.5 rounded-xl text-xs font-medium transition flex items-center gap-1.5 ${
                    settingsTab() === "engine"
                      ? "bg-[#181C20] text-[#E8EDF0] border border-[#252B30] shadow-sm"
                      : "text-[#929DA6] hover:text-[#E8EDF0]"
                  }`}
                >
                  <svg class="w-3.5 h-3.5 text-[#38D9C0]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                    <circle cx="12" cy="12" r="3" />
                  </svg>
                  <span>Engine Settings</span>
                </button>

                <button
                  onClick={() => {
                    setSettingsTab("terminal");
                    loadLogs();
                  }}
                  class={`px-3 py-1.5 rounded-xl text-xs font-medium transition flex items-center gap-1.5 ${
                    settingsTab() === "terminal"
                      ? "bg-[#181C20] text-[#E8EDF0] border border-[#252B30] shadow-sm"
                      : "text-[#929DA6] hover:text-[#E8EDF0]"
                  }`}
                >
                  <svg class="w-3.5 h-3.5 text-[#38BDF8]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                  </svg>
                  <span>Pipeline Logs & Diagnostics</span>
                  <Show when={terminalLogs().length > 0}>
                    <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-[#38D9C0]/20 text-[#38D9C0] font-mono">
                      {terminalLogs().length}
                    </span>
                  </Show>
                </button>
              </div>

              <button onClick={() => setSettingsOpen(false)} class="p-1 text-[#929DA6] hover:text-[#E8EDF0]">✕</button>
            </div>

            {/* TAB 1: ENGINE SETTINGS */}
            <Show when={settingsTab() === "engine"}>
              <div class="flex-1 overflow-y-auto flex flex-col gap-4 pr-1">
                {/* 1. LLM ENGINE */}
                <div class="flex flex-col gap-2.5">
                  <span class="text-xs font-semibold text-[#E8EDF0] uppercase tracking-wider">1. LLM Engine</span>
                  <div class="grid grid-cols-2 gap-3 text-xs">
                    <div>
                      <label class="block text-[#929DA6] mb-1">Protocol Format</label>
                      <select
                        value={llmProtocol()}
                        onChange={(e) => setLlmProtocol(e.currentTarget.value)}
                        class="w-full bg-[#181C20] border border-[#252B30] rounded-lg px-2.5 py-1.5 text-[#E8EDF0]"
                      >
                        <option value="openai">OpenAI / LM Studio / LocalAI</option>
                        <option value="anthropic">Anthropic Claude</option>
                        <option value="ollama">Ollama Native API</option>
                        <option value="custom">Custom REST Endpoint</option>
                      </select>
                    </div>
                    <div>
                      <label class="block text-[#929DA6] mb-1">Model Name</label>
                      <input
                        type="text"
                        value={llmModel()}
                        onInput={(e) => setLlmModel(e.currentTarget.value)}
                        class="w-full bg-[#181C20] border border-[#252B30] rounded-lg px-2.5 py-1.5 text-[#E8EDF0]"
                      />
                    </div>
                  </div>

                  <div class="text-xs">
                    <label class="block text-[#929DA6] mb-1">API Base URL</label>
                    <input
                      type="text"
                      value={llmBaseUrl()}
                      onInput={(e) => setLlmBaseUrl(e.currentTarget.value)}
                      class="w-full bg-[#181C20] border border-[#252B30] rounded-lg px-2.5 py-1.5 text-[#E8EDF0] font-mono text-[11px]"
                    />
                  </div>

                  <div class="text-xs">
                    <label class="block text-[#929DA6] mb-1">API Key (Optional for Local)</label>
                    <input
                      type="password"
                      value={llmApiKey()}
                      onInput={(e) => setLlmApiKey(e.currentTarget.value)}
                      placeholder="sk-..."
                      class="w-full bg-[#181C20] border border-[#252B30] rounded-lg px-2.5 py-1.5 text-[#E8EDF0]"
                    />
                  </div>

                  <div class="flex items-center justify-between pt-1">
                    <button
                      onClick={handleTestLlm}
                      disabled={isTestingLlm()}
                      class="px-3 py-1.5 rounded-lg bg-[#181C20] hover:bg-[#1F2429] text-xs text-[#E8EDF0] font-medium transition border border-[#252B30]"
                    >
                      <Show when={isTestingLlm()} fallback={<span>[ Test LLM Connection ]</span>}>
                        <span>Testing...</span>
                      </Show>
                    </button>
                    <Show when={llmStatus()}>
                      <span class={`text-xs px-2 py-0.5 rounded font-mono ${llmStatus()?.live ? "text-[#34D399] bg-[#34D399]/10" : "text-rose-400 bg-rose-500/10"}`}>
                        {llmStatus()?.text}
                      </span>
                    </Show>
                  </div>
                </div>

                {/* 2. SEARCH ENGINE */}
                <div class="flex flex-col gap-2.5 pt-3 border-t border-[#252B30]">
                  <span class="text-xs font-semibold text-[#E8EDF0] uppercase tracking-wider">2. Search Engine</span>
                  <div class="grid grid-cols-2 gap-3 text-xs">
                    <div>
                      <label class="block text-[#929DA6] mb-1">Primary Engine</label>
                      <select
                        value={searchEngine()}
                        onChange={(e) => setSearchEngine(e.currentTarget.value)}
                        class="w-full bg-[#181C20] border border-[#252B30] rounded-lg px-2.5 py-1.5 text-[#E8EDF0]"
                      >
                        <option>DuckDuckGo</option>
                        <option>SearXNG</option>
                        <option>Brave</option>
                      </select>
                    </div>
                    <div>
                      <Show when={searchEngine() === "SearXNG"}>
                        <label class="block text-[#929DA6] mb-1">SearXNG URL</label>
                        <input
                          type="text"
                          value={searxUrl()}
                          onInput={(e) => setSearxUrl(e.currentTarget.value)}
                          class="w-full bg-[#181C20] border border-[#252B30] rounded-lg px-2.5 py-1.5 text-[#E8EDF0] font-mono text-[11px]"
                        />
                      </Show>
                      <Show when={searchEngine() === "Brave"}>
                        <label class="block text-[#929DA6] mb-1">Brave API Key</label>
                        <input
                          type="password"
                          value={braveKey()}
                          onInput={(e) => setBraveKey(e.currentTarget.value)}
                          class="w-full bg-[#181C20] border border-[#252B30] rounded-lg px-2.5 py-1.5 text-[#E8EDF0]"
                        />
                      </Show>
                    </div>
                  </div>

                  <div class="text-[11px] text-[#64707D] pt-1">
                    * Google News RSS and Hacker News Algolia are enabled concurrently as bot-free sources.
                  </div>

                  <div class="flex items-center justify-between pt-1">
                    <button
                      onClick={handleTestSearch}
                      disabled={isTestingSearch()}
                      class="px-3 py-1.5 rounded-lg bg-[#181C20] hover:bg-[#1F2429] text-xs text-[#E8EDF0] font-medium transition border border-[#252B30]"
                    >
                      <Show when={isTestingSearch()} fallback={<span>[ Test Search ]</span>}>
                        <span>Testing...</span>
                      </Show>
                    </button>
                    <Show when={searchStatus()}>
                      <span class={`text-xs px-2 py-0.5 rounded font-mono ${searchStatus()?.live ? "text-[#34D399] bg-[#34D399]/10" : "text-rose-400 bg-rose-500/10"}`}>
                        {searchStatus()?.text}
                      </span>
                    </Show>
                  </div>
                </div>

                {/* 3. OBSIDIAN VAULT */}
                <div class="flex flex-col gap-2 pt-3 border-t border-[#252B30] text-xs">
                  <span class="font-semibold text-[#E8EDF0] uppercase tracking-wider">3. Offline Vault</span>
                  <label class="text-[#929DA6]">Obsidian / Markdown Directory Path</label>
                  <div class="flex gap-2">
                    <input
                      type="text"
                      value={vaultPath()}
                      onInput={(e) => setVaultPath(e.currentTarget.value)}
                      class="flex-1 bg-[#181C20] border border-[#252B30] rounded-lg px-2.5 py-1.5 text-[#E8EDF0] font-mono text-[11px]"
                    />
                    <button
                      onClick={checkVaultCount}
                      class="px-2.5 py-1.5 rounded-lg bg-[#181C20] hover:bg-[#1F2429] text-[#E8EDF0] text-xs border border-[#252B30]"
                    >
                      Scan
                    </button>
                  </div>
                  <span class="text-[11px] text-[#64707D]">
                    Found <strong class="text-[#38D9C0]">{vaultCount()}</strong> local markdown files.
                  </span>
                </div>

                <div class="flex justify-end gap-2 pt-3 border-t border-[#252B30] shrink-0">
                  <button
                    onClick={saveSettings}
                    class="px-4 py-2 rounded-xl bg-[#38D9C0] hover:bg-[#2EC4AC] text-black font-semibold text-xs transition"
                  >
                    Save Settings
                  </button>
                </div>
              </div>
            </Show>

            {/* TAB 2: PIPELINE LOGS & DIAGNOSTICS */}
            <Show when={settingsTab() === "terminal"}>
              <div class="flex-1 flex flex-col gap-3 overflow-hidden">
                {/* TOOLBAR CONTROLS */}
                <div class="flex flex-wrap items-center justify-between gap-2 shrink-0">
                  <div class="flex items-center gap-2">
                    <button
                      onClick={exportLogsToFile}
                      class="px-3 py-1.5 rounded-lg bg-[#181C20] hover:bg-[#1F2429] text-xs text-[#E8EDF0] font-medium border border-[#252B30] flex items-center gap-1.5 transition"
                      title="Save log file to Downloads folder"
                    >
                      <svg class="w-3.5 h-3.5 text-[#38D9C0]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                      </svg>
                      <Show when={logExportStatus()} fallback={<span>Export Log File</span>}>
                        {(st) => <span class="text-[#34D399] font-medium">{st()}</span>}
                      </Show>
                    </button>

                    <button
                      onClick={copyLogs}
                      class="px-3 py-1.5 rounded-lg bg-[#181C20] hover:bg-[#1F2429] text-xs text-[#E8EDF0] font-medium border border-[#252B30] flex items-center gap-1.5 transition"
                      title="Copy logs to clipboard"
                    >
                      <Show when={logCopyFeedback()} fallback={<span>📋 Copy</span>}>
                        <span class="text-[#34D399] font-medium">✓ Copied</span>
                      </Show>
                    </button>

                    <button
                      onClick={clearLogs}
                      class="px-2.5 py-1.5 rounded-lg bg-[#181C20] hover:bg-[#1F2429] text-xs text-[#929DA6] hover:text-rose-400 border border-[#252B30] transition"
                      title="Clear log buffer"
                    >
                      Clear
                    </button>
                  </div>

                  <button
                    onClick={reportGitHubIssue}
                    class="px-3 py-1.5 rounded-lg bg-emerald-500/10 hover:bg-emerald-500/20 text-[#34D399] text-xs font-semibold border border-emerald-500/30 flex items-center gap-1.5 transition"
                    title="Open GitHub issue with pre-filled report template"
                  >
                    <span>🐛 Report on GitHub</span>
                    <span class="text-[10px] opacity-70">↗</span>
                  </button>
                </div>

                {/* INSTRUCTIONS BANNER */}
                <div class="text-[11px] text-[#929DA6] bg-[#181C20]/80 p-2.5 rounded-xl border border-[#252B30] leading-relaxed shrink-0">
                  <span class="text-[#38D9C0] font-medium">Diagnostic Tracer:</span> Observes exact millisecond latency across search retrieval, top-5 context ranking, and local LLM token generation. If a stall occurs, export the log above and attach it to your issue.
                </div>

                {/* TERMINAL LOG OUTPUT CONTAINER */}
                <div
                  ref={terminalScrollRef}
                  class="flex-1 bg-[#0A0C0E] border border-[#252B30] rounded-xl p-3 font-mono text-[11px] overflow-y-auto flex flex-col gap-1 min-h-[260px] select-text shadow-inner"
                >
                  <Show when={terminalLogs().length === 0}>
                    <div class="h-full flex items-center justify-center text-[#64707D] italic py-10">
                      No pipeline events logged yet. Execute a search to see real-time latency milestones.
                    </div>
                  </Show>

                  <For each={terminalLogs()}>
                    {(entry) => (
                      <div class="flex items-start gap-2 py-0.5 hover:bg-white/[0.02] rounded px-1">
                        <span class="text-[#64707D] shrink-0">{entry.timestamp}</span>
                        <span
                          class={`px-1.5 py-0.2 rounded text-[10px] font-bold shrink-0 ${
                            entry.level === "ERROR"
                              ? "bg-rose-500/20 text-rose-400"
                              : entry.level === "WARN"
                              ? "bg-amber-500/20 text-amber-300"
                              : entry.level === "DONE"
                              ? "bg-emerald-500/20 text-emerald-400"
                              : entry.level === "LLM"
                              ? "bg-teal-500/20 text-[#38D9C0]"
                              : entry.level === "RANK"
                              ? "bg-purple-500/20 text-purple-300"
                              : entry.level === "SEARCH"
                              ? "bg-sky-500/20 text-sky-400"
                              : "bg-white/10 text-gray-300"
                          }`}
                        >
                          {entry.level}
                        </span>
                        <span class="text-[#64707D] shrink-0 font-mono">
                          +{entry.elapsed_ms}ms
                        </span>
                        <span class="text-[#E8EDF0] break-words flex-1">
                          {entry.message}
                        </span>
                      </div>
                    )}
                  </For>
                </div>
              </div>
            </Show>

          </div>
        </div>
      </Show>

    </div>
  );
}
