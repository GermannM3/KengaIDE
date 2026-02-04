import Editor from "@monaco-editor/react";
import { useEffect, useRef, useState } from "react";
import { FileTreeList } from "./components/FileTree";
import { getInvoke } from "./lib/invoke";
import {
  DEFAULT_LEFT_PANEL_WIDTH,
  DEFAULT_RIGHT_PANEL_WIDTH,
  INVOKE_ERR,
  MAX_LEFT_PANEL,
  MAX_RIGHT_PANEL,
  MIN_LEFT_PANEL,
  MIN_RIGHT_PANEL,
  RESIZER_WIDTH,
  STORAGE_LEFT,
  STORAGE_RIGHT,
  STORAGE_THEME,
  STORAGE_WELCOME_DISMISSED,
  THEME_CSS,
  THEME_MONACO,
} from "./constants";
import type {
  AiChunkPayload,
  AiRequestPayload,
  AiRequestType,
  AgentProgressPayload,
  CommandItem,
  DownloadProgress,
  InvokeFn,
  LocalModelStatus,
  ProjectTreeNode,
  ThemeId,
} from "./types";

function App() {
  const [code, setCode] = useState("// KengaIDE\nfn main() {\n    println!(\"Hello\");\n}\n");
  const [aiInput, setAiInput] = useState("");
  const [aiResponse, setAiResponse] = useState("");
  const [modelStatus, setModelStatus] = useState<LocalModelStatus>("not_available");
  const [modelInfo, setModelInfo] = useState<{ size_gb: number; display_name: string } | null>(null);
  const [showDownloadDialog, setShowDownloadDialog] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState<DownloadProgress | null>(null);
  const [downloadError, setDownloadError] = useState<string | null>(null);
  const invokeRef = useRef<InvokeFn | null>(null);
  const [tauriReady, setTauriReady] = useState(false);
  const [inTauri, setInTauri] = useState(false);
  const [streamingRequestId, setStreamingRequestId] = useState<string | null>(null);
  const [agentRequestId, setAgentRequestId] = useState<string | null>(null);
  const [modelSelection, setModelSelection] = useState<{ role: string; model_id: string } | null>(null);
  const [toolTimeline, setToolTimeline] = useState<
    Array<{
      kind: string;
      name?: string;
      path?: string;
      success?: boolean;
      output?: string;
      message?: string;
      before?: string;
      after?: string;
    }>
  >([]);
  const [toolTimelineExpanded, setToolTimelineExpanded] = useState(true);
  const [appliedPatches, setAppliedPatches] = useState<Array<{ path: string; before: string; after: string }>>([]);
  const [lastAgentMessage, setLastAgentMessage] = useState("");
  const [lastSessionId, setLastSessionId] = useState<string | null>(null);
  const [splitActive, setSplitActive] = useState(false);
  const [splitFilePath, setSplitFilePath] = useState<string | null>(null);
  const [splitCode, setSplitCode] = useState("");
  const responseEndRef = useRef<HTMLPreElement | null>(null);
  const editorSelectionRef = useRef<string | null>(null);

  const [projectPath, setProjectPath] = useState<string | null>(null);
  const [projectTree, setProjectTree] = useState<ProjectTreeNode[] | null>(null);
  const [projectTreeLoading, setProjectTreeLoading] = useState(false);
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(new Set());
  const [openFiles, setOpenFiles] = useState<string[]>([]);
  const [fileContents, setFileContents] = useState<Record<string, string>>({});
  const [currentFilePath, setCurrentFilePath] = useState<string | null>(null);

  const [leftPanelWidth, setLeftPanelWidth] = useState(() => {
    try {
      const w = parseInt(localStorage.getItem(STORAGE_LEFT) ?? "", 10);
      return Number.isFinite(w) && w >= MIN_LEFT_PANEL && w <= MAX_LEFT_PANEL ? w : DEFAULT_LEFT_PANEL_WIDTH;
    } catch {
      return DEFAULT_LEFT_PANEL_WIDTH;
    }
  });
  const [rightPanelWidth, setRightPanelWidth] = useState(() => {
    try {
      const w = parseInt(localStorage.getItem(STORAGE_RIGHT) ?? "", 10);
      return Number.isFinite(w) && w >= MIN_RIGHT_PANEL && w <= MAX_RIGHT_PANEL ? w : DEFAULT_RIGHT_PANEL_WIDTH;
    } catch {
      return DEFAULT_RIGHT_PANEL_WIDTH;
    }
  });
  const [resizing, setResizing] = useState<"left" | "right" | null>(null);
  const dragStartRef = useRef({ x: 0, left: 0, right: 0 });

  const [showCreateModal, setShowCreateModal] = useState(false);
  const [createTemplate, setCreateTemplate] = useState("rust");
  const [createName, setCreateName] = useState("");
  const [createParentDir, setCreateParentDir] = useState<string | null>(null);
  const [createError, setCreateError] = useState<string | null>(null);

  const [showCommandPalette, setShowCommandPalette] = useState(false);
  const [commandPaletteQuery, setCommandPaletteQuery] = useState("");
  const [commandPaletteSelected, setCommandPaletteSelected] = useState(0);
  const commandPaletteInputRef = useRef<HTMLInputElement | null>(null);

  const [showAgentPrompt, setShowAgentPrompt] = useState(false);
  const [agentPromptInput, setAgentPromptInput] = useState("");
  const [showAddProviderModal, setShowAddProviderModal] = useState(false);
  const [showSwitchModelModal, setShowSwitchModelModal] = useState(false);
  const [addProviderApiKey, setAddProviderApiKey] = useState("");
  const [addProviderError, setAddProviderError] = useState<string | null>(null);
  const [aiProviders, setAiProviders] = useState<{ id: string; name: string; available: boolean }[]>([]);
  const [activeProviderId, setActiveProviderId] = useState<string | null>(null);
  const [systemInfo, setSystemInfo] = useState<{ ram_gb: number; cpu_cores: number } | null>(null);
  const [welcomeDismissed, setWelcomeDismissed] = useState(() => {
    try {
      return !!localStorage.getItem(STORAGE_WELCOME_DISMISSED);
    } catch {
      return false;
    }
  });

  const [theme, setTheme] = useState<ThemeId>(() => {
    try {
      const t = localStorage.getItem(STORAGE_THEME) as ThemeId | null;
      return t && (t === "light" || t === "dark" || t === "high-contrast") ? t : "light";
    } catch {
      return "light";
    }
  });

  const [gitStatus, setGitStatus] = useState<{ branch: string; changes: number } | null>(null);
  const [appVersion, setAppVersion] = useState<{ name: string; version: string } | null>(null);

  useEffect(() => {
    if (resizing === null) return;
    const prevCursor = document.body.style.cursor;
    const prevSelect = document.body.style.userSelect;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    const onMove = (e: MouseEvent) => {
      const { x, left, right } = dragStartRef.current;
      if (resizing === "left") {
        const delta = e.clientX - x;
        setLeftPanelWidth(Math.min(MAX_LEFT_PANEL, Math.max(MIN_LEFT_PANEL, left + delta)));
      } else {
        const delta = x - e.clientX;
        setRightPanelWidth(Math.min(MAX_RIGHT_PANEL, Math.max(MIN_RIGHT_PANEL, right + delta)));
      }
    };
    const onUp = () => setResizing(null);
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
    return () => {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
      document.body.style.cursor = prevCursor;
      document.body.style.userSelect = prevSelect;
    };
  }, [resizing]);

  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_LEFT, String(leftPanelWidth));
      localStorage.setItem(STORAGE_RIGHT, String(rightPanelWidth));
    } catch {
      /* ignore */
    }
  }, [leftPanelWidth, rightPanelWidth]);

  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_THEME, theme);
    } catch {
      /* ignore */
    }
    const css = THEME_CSS[theme];
    document.documentElement.style.setProperty("--kenga-bg", css.bg);
    document.documentElement.style.setProperty("--kenga-fg", css.fg);
    document.documentElement.style.setProperty("--kenga-panel", css.panel);
    document.documentElement.style.setProperty("--kenga-border", css.border);
    document.documentElement.style.setProperty("--kenga-accent", css.accent);
    document.documentElement.style.setProperty("--kenga-muted", css.muted);
    document.documentElement.style.setProperty("--kenga-accent-bg", css.accentBg);
  }, [theme]);

  useEffect(() => {
    if (!inTauri || !invokeRef.current) return;
    invokeRef.current("git_status")
      .then((v) => setGitStatus((v as { branch: string; changes: number } | null) ?? null))
      .catch(() => setGitStatus(null));
  }, [inTauri, projectPath]);

  useEffect(() => {
    if (!inTauri || !invokeRef.current) return;
    invokeRef.current("get_app_version")
      .then((v) => {
        const arr = v as [string, string];
        setAppVersion(arr ? { name: arr[0], version: arr[1] } : null);
      })
      .catch(() => {});
  }, [inTauri]);

  const toggleTreePath = (path: string) => {
    setExpandedPaths((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  };

  useEffect(() => {
    getInvoke().then((fn) => {
      invokeRef.current = fn;
      setInTauri(!!fn);
      setTauriReady(true);
      const splashEl = document.getElementById("splash");
      if (splashEl) {
        window.setTimeout(() => splashEl.remove(), 1500);
      }
    });
  }, []);

  const refreshProjectPath = () => {
    const inv = invokeRef.current;
    if (!inv) return;
    inv("get_project_path")
      .then((v) => setProjectPath((v as string | null) ?? null))
      .catch(() => setProjectPath(null));
  };

  const refreshProjectTree = () => {
    const inv = invokeRef.current;
    if (!inv) return;
    setProjectTreeLoading(true);
    inv("get_project_tree")
      .then((v) => setProjectTree((v as ProjectTreeNode[] | null) ?? null))
      .catch(() => setProjectTree(null))
      .finally(() => setProjectTreeLoading(false));
  };

  useEffect(() => {
    if (!inTauri) return;
    refreshProjectPath();
  }, [inTauri]);

  useEffect(() => {
    if (inTauri && invokeRef.current) {
      invokeRef.current("get_system_info")
        .then((v) => setSystemInfo(v as { ram_gb: number; cpu_cores: number }))
        .catch(() => {});
    }
  }, [inTauri]);

  useEffect(() => {
    if (modelStatus === "ready") {
      try {
        localStorage.setItem(STORAGE_WELCOME_DISMISSED, "1");
        setWelcomeDismissed(true);
      } catch {
        /* ignore */
      }
    }
  }, [modelStatus]);

  useEffect(() => {
    if (inTauri && modelStatus === "not_loaded" && !welcomeDismissed) {
      loadAiProviders();
    }
  }, [inTauri, modelStatus, welcomeDismissed]);

  useEffect(() => {
    if (inTauri) loadAiProviders();
  }, [inTauri]);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "p" && e.ctrlKey && e.shiftKey) {
        e.preventDefault();
        setShowCommandPalette((prev) => {
          if (!prev) {
            setCommandPaletteQuery("");
            setCommandPaletteSelected(0);
            setTimeout(() => commandPaletteInputRef.current?.focus(), 50);
          }
          return !prev;
        });
      }
      if (showCommandPalette) {
        if (e.key === "Escape") {
          e.preventDefault();
          setShowCommandPalette(false);
        }
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [showCommandPalette]);

  useEffect(() => {
    if (!projectPath || !invokeRef.current) {
      setProjectTree(null);
      setExpandedPaths(new Set());
      setOpenFiles([]);
      setFileContents({});
      setCurrentFilePath(null);
      setCode("");
      setSplitFilePath(null);
      setSplitCode("");
      return;
    }
    refreshProjectTree();
  }, [projectPath]);

  const handleOpenFile = async (relativePath: string, inSplit?: boolean) => {
    const inv = invokeRef.current;
    if (!inv) return;
    try {
      const content = (await inv("read_project_file", { relativePath })) as string;
      setOpenFiles((prev) => (prev.includes(relativePath) ? prev : [...prev, relativePath]));
      setFileContents((prev) => ({ ...prev, [relativePath]: content }));
      if (inSplit) {
        setSplitActive(true);
        setSplitFilePath(relativePath);
        setSplitCode(content);
      } else {
        setCurrentFilePath(relativePath);
        setCode(content);
      }
    } catch (e) {
      setAiResponse((prev) => prev + `\nОшибка открытия ${relativePath}: ${String(e)}\n`);
    }
  };

  const handleSwitchTab = (path: string) => {
    if (path === currentFilePath) return;
    if (currentFilePath) {
      setFileContents((prev) => ({ ...prev, [currentFilePath]: code }));
    }
    setCurrentFilePath(path);
    setCode(fileContents[path] ?? "");
  };

  const handleCloseTab = (path: string) => {
    const idx = openFiles.indexOf(path);
    if (idx < 0) return;
    const next = openFiles.filter((p) => p !== path);
    setOpenFiles(next);
    setFileContents((prev) => {
      const c = { ...prev };
      delete c[path];
      return c;
    });
    if (path === currentFilePath) {
      const newCurrent = next[idx] ?? next[idx - 1] ?? null;
      setCurrentFilePath(newCurrent);
      setCode(newCurrent ? (fileContents[newCurrent] ?? "") : "");
    }
  };

  const editorLanguage = (() => {
    if (!currentFilePath) return "plaintext";
    const ext = currentFilePath.replace(/^.*\./, "").toLowerCase();
    const map: Record<string, string> = {
      rs: "rust",
      ts: "typescript",
      tsx: "typescriptreact",
      js: "javascript",
      jsx: "javascript",
      json: "json",
      md: "markdown",
      py: "python",
      html: "html",
      css: "css",
    };
    return map[ext] ?? "plaintext";
  })();

  const refreshStatus = () => {
    const inv = invokeRef.current;
    if (!inv) {
      setModelStatus("not_available");
      setModelInfo(null);
      return;
    }
    inv("local_model_status")
      .then((v) => setModelStatus(v as LocalModelStatus))
      .catch(() => setModelStatus("not_available"));
    inv("local_model_info")
      .then((info) =>
        setModelInfo({
          size_gb: (info as { size_gb: number }).size_gb,
          display_name: (info as { display_name: string }).display_name,
        })
      )
      .catch(() => setModelInfo(null));
  };

  useEffect(() => {
    if (!tauriReady) return;
    refreshStatus();
  }, [tauriReady, aiResponse]);

  const unlistenRef = useRef<(() => void) | null>(null);
  useEffect(() => {
    if (!inTauri) return;
    import("@tauri-apps/api/event").then(({ listen }) => {
      listen<DownloadProgress>("model_download_progress", (ev) => {
        setDownloadProgress(ev.payload);
      }).then((fn) => {
        unlistenRef.current = fn;
      });
    });
    return () => {
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }
    };
  }, [inTauri]);

  useEffect(() => {
    if (!inTauri) return;
    let unlisten: (() => void) | undefined;
    import("@tauri-apps/api/event").then(({ listen }) => {
      listen<{ request_id: string; role: string; model_id: string }>("ai_model_selected", (ev) => {
        setModelSelection({ role: ev.payload.role, model_id: ev.payload.model_id });
      }).then((fn) => {
        unlisten = fn;
      });
    });
    return () => {
      unlisten?.();
    };
  }, [inTauri]);

  useEffect(() => {
    if (!inTauri || !streamingRequestId) return;
    let unlisten: (() => void) | undefined;
    import("@tauri-apps/api/event").then(({ listen }) => {
      listen<AiChunkPayload>("ai_chunk", (ev) => {
        const { request_id, type } = ev.payload;
        if (request_id !== streamingRequestId) return;
        switch (type) {
          case "start":
            setAiResponse("");
            break;
          case "token":
            setAiResponse((prev) => prev + (ev.payload as { value: string }).value);
            break;
          case "end":
            setStreamingRequestId(null);
            break;
          case "error":
            setAiResponse((prev) => prev + "\n[Ошибка: " + (ev.payload as { error: string }).error + "]");
            setStreamingRequestId(null);
            break;
        }
      }).then((fn) => {
        unlisten = fn;
      });
    });
    return () => {
      unlisten?.();
    };
  }, [inTauri, streamingRequestId]);

  useEffect(() => {
    if (aiResponse && responseEndRef.current) {
      responseEndRef.current.scrollTop = responseEndRef.current.scrollHeight;
    }
  }, [aiResponse]);

  useEffect(() => {
    if (!inTauri || !agentRequestId) return;
    let unlisten: (() => void) | undefined;
    import("@tauri-apps/api/event").then(({ listen }) => {
      listen<AgentProgressPayload>("agent_progress", (ev) => {
        const { request_id, kind } = ev.payload;
        if (request_id !== agentRequestId) return;
        setToolTimeline((prev) => {
          const entry = { ...ev.payload } as {
            kind: string;
            name?: string;
            path?: string;
            success?: boolean;
            output?: string;
            message?: string;
          };
          return [...prev, entry];
        });
        switch (kind) {
          case "session_started":
            setLastSessionId((ev.payload as { session_id: string }).session_id);
            break;
          case "model_selected":
            setModelSelection({
              role: (ev.payload as { role: string }).role,
              model_id: (ev.payload as { model_id: string }).model_id,
            });
            break;
          case "thinking":
            break;
          case "tool_call":
          case "tool_result":
          case "patch_apply_started":
            break;
          case "patch_apply_success": {
            const patchedPath = (ev.payload as { path: string }).path;
            if (invokeRef.current && patchedPath === currentFilePath) {
              invokeRef
                .current("read_project_file", { relativePath: patchedPath })
                .then((content) => setCode(content as string))
                .catch(() => {});
            }
            break;
          }
          case "patch_apply_error":
            break;
          case "patch_applied": {
            const p = ev.payload as { path: string; before: string; after: string };
            setAppliedPatches((prev) => [...prev, { path: p.path, before: p.before, after: p.after }]);
            break;
          }
          case "done":
            setAiResponse((prev) => prev + "\n\n" + (ev.payload as { message: string }).message);
            setAgentRequestId(null);
            break;
        }
      }).then((fn) => {
        unlisten = fn;
      });
    });
    return () => {
      unlisten?.();
    };
  }, [inTauri, agentRequestId, currentFilePath]);

  useEffect(() => {
    if (modelStatus === "not_loaded" && modelInfo && !showDownloadDialog && !downloading) {
      setShowDownloadDialog(true);
    }
  }, [modelStatus, modelInfo, showDownloadDialog, downloading]);

  const handleConfirmDownload = async () => {
    const inv = invokeRef.current;
    if (!inv) {
      setAiResponse(INVOKE_ERR);
      return;
    }
    setDownloading(true);
    setDownloadProgress(null);
    setDownloadError(null);
    try {
      await inv("start_model_download");
      setShowDownloadDialog(false);
      refreshStatus();
      loadAiProviders();
    } catch (e) {
      setDownloadError(String(e));
    } finally {
      setDownloading(false);
      setDownloadProgress(null);
    }
  };

  const handleAiRequest = async (type: AiRequestType) => {
    const inv = invokeRef.current;
    if (!inv) {
      setAiResponse(INVOKE_ERR);
      return;
    }
    if (streamingRequestId) return;
    const path = currentFilePath ?? "";
    if ((type === "explain" || type === "refactor" || type === "generate") && !path) {
      setAiResponse("Откройте файл в редакторе для Explain / Refactor / Generate.\n");
      return;
    }
    const selection = editorSelectionRef.current ?? code;
    const payload: AiRequestPayload = {
      request:
        type === "chat"
          ? { type: "chat", message: aiInput }
          : type === "agent"
            ? { type: "agent", message: aiInput }
            : type === "explain"
              ? { type: "explain", path, selection: selection || undefined }
              : type === "refactor"
                ? { type: "refactor", path, selection: selection || code, instruction: aiInput }
                : type === "generate"
                  ? { type: "generate", path, prompt: aiInput }
                  : { type: "chat", message: aiInput },
      current_file_path: currentFilePath ?? undefined,
      current_file_content: code,
      selection: selection || undefined,
    };
    try {
      const requestId = (await inv("ai_request_stream", { payload })) as string;
      setStreamingRequestId(requestId);
    } catch (e) {
      setAiResponse(`Error: ${String(e)}`);
    }
  };

  const handleStopGeneration = () => {
    const inv = invokeRef.current;
    if (!inv || !streamingRequestId) return;
    inv("ai_cancel", { requestId: streamingRequestId }).catch(() => {});
    setStreamingRequestId(null);
  };

  const handleOpenProject = async () => {
    const inv = invokeRef.current;
    if (!inv) return;
    try {
      const path = (await inv("open_project_dialog")) as string | null | undefined;
      if (path) {
        refreshProjectPath();
        refreshProjectTree();
        setAiResponse(`Проект открыт: ${path}\n`);
      }
    } catch (e) {
      setAiResponse(`Ошибка: ${String(e)}`);
    }
  };

  const handleCreateProject = async () => {
    const inv = invokeRef.current;
    if (!inv) return;
    setCreateError(null);
    try {
      const path = (await inv("create_project", {
        payload: {
          template: createTemplate,
          parentDir: createParentDir ?? undefined,
          name: createName.trim() || undefined,
        },
      })) as string;
      setShowCreateModal(false);
      setCreateName("");
      setCreateParentDir(null);
      refreshProjectPath();
      refreshProjectTree();
      setAiResponse(`Проект создан и открыт: ${path}\n`);
    } catch (e) {
      setCreateError(String(e));
    }
  };

  const handlePickFolderForCreate = async () => {
    const inv = invokeRef.current;
    if (!inv) return;
    try {
      const path = (await inv("pick_folder")) as string | null | undefined;
      if (path) setCreateParentDir(path);
    } catch {
      /* ignore */
    }
  };

  const COMMANDS: CommandItem[] = [
    { id: "open_audit", label: "Открыть папку аудита", keywords: ["audit", "аудит", "логи"] },
    { id: "new_project", label: "Новый проект", keywords: ["new", "create", "проект"] },
    { id: "open_folder", label: "Открыть папку", keywords: ["open", "folder", "открыть", "папка"] },
    { id: "run_agent", label: "Запустить агента", keywords: ["agent", "run", "запустить", "агент"] },
    { id: "mcp_settings", label: "Настройки MCP", keywords: ["mcp", "settings", "настройки"] },
    { id: "add_provider", label: "Добавить AI провайдер", keywords: ["add", "provider", "ai", "провайдер"] },
    { id: "switch_model", label: "Сменить модель", keywords: ["switch", "model", "модель"] },
    { id: "theme_light", label: "Тема: Светлая", keywords: ["theme", "светлая", "light"] },
    { id: "theme_dark", label: "Тема: Тёмная", keywords: ["theme", "тёмная", "dark"] },
    { id: "theme_hc", label: "Тема: Высокий контраст", keywords: ["theme", "контраст", "high"] },
    { id: "open_logs", label: "Открыть папку логов", keywords: ["logs", "log", "логи", "debug"] },
  ];

  const filteredCommands = COMMANDS.filter(
    (c) =>
      commandPaletteQuery === "" ||
      c.label.toLowerCase().includes(commandPaletteQuery.toLowerCase()) ||
      c.keywords.some((k) => k.toLowerCase().includes(commandPaletteQuery.toLowerCase()))
  );

  useEffect(() => {
    if (!showCommandPalette) return;
    setCommandPaletteSelected((prev) => Math.min(prev, Math.max(0, filteredCommands.length - 1)));
  }, [showCommandPalette, filteredCommands.length, commandPaletteQuery]);

  const loadAiProviders = async () => {
    const inv = invokeRef.current;
    if (!inv) return;
    try {
      const list = (await inv("list_ai_providers")) as { id: string; name: string; available: boolean }[];
      setAiProviders(list);
      const active = (await inv("get_active_provider")) as string | null | undefined;
      setActiveProviderId(active ?? null);
    } catch {
      /* ignore */
    }
  };

  const runCommand = (id: string) => {
    setShowCommandPalette(false);
    switch (id) {
      case "new_project":
        setShowCreateModal(true);
        setCreateError(null);
        break;
      case "open_folder":
        handleOpenProject();
        break;
      case "run_agent":
        setShowAgentPrompt(true);
        setAgentPromptInput("");
        break;
      case "mcp_settings":
        invokeRef.current?.("open_mcp_config_folder").catch(() => {});
        break;
      case "add_provider":
        setShowAddProviderModal(true);
        setAddProviderApiKey("");
        setAddProviderError(null);
        break;
      case "switch_model":
        setShowSwitchModelModal(true);
        loadAiProviders();
        break;
      case "open_logs":
        invokeRef.current?.("open_logs_folder").catch(() => {});
        break;
      case "open_audit":
        invokeRef.current?.("open_audit_folder").catch(() => {});
        break;
      case "theme_light":
        setTheme("light");
        break;
      case "theme_dark":
        setTheme("dark");
        break;
      case "theme_hc":
        setTheme("high-contrast");
        break;
      default:
        break;
    }
  };

  const handleAgentRequest = async (messageOverride?: string) => {
    const inv = invokeRef.current;
    if (!inv) {
      setAiResponse(INVOKE_ERR);
      return;
    }
    if (!projectPath) {
      setAiResponse("Откройте проект (кнопка «Открыть папку» в шапке), затем нажмите Agent.\n");
      return;
    }
    if (streamingRequestId || agentRequestId) return;
    const msg = (messageOverride ?? aiInput).trim();
    if (!msg) {
      setAiResponse("Введите задачу для агента.\n");
      return;
    }
    const payload: AiRequestPayload = {
      request: { type: "agent", message: msg },
      current_file_path: currentFilePath ?? undefined,
      current_file_content: code,
      selection: undefined,
    };
    try {
      setAiResponse("Агент запущен…\n");
      setToolTimeline([]);
      setAppliedPatches([]);
      setLastAgentMessage(msg);
      setLastSessionId(null);
      const requestId = (await inv("ai_agent_request", { payload })) as string;
      setAgentRequestId(requestId);
    } catch (e) {
      setAiResponse(`Error: ${String(e)}`);
    }
  };

  const progressPct =
    downloadProgress && downloadProgress.bytes_total > 0
      ? Math.round((downloadProgress.bytes_done / downloadProgress.bytes_total) * 100)
      : 0;

  const [menuOpen, setMenuOpen] = useState<"file" | "view" | "ai" | "tools" | "help" | null>(null);
  const [showSaveAsModal, setShowSaveAsModal] = useState(false);
  const [showAboutModal, setShowAboutModal] = useState(false);
  const [saveAsPath, setSaveAsPath] = useState("");
  const menuContainerRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!menuOpen) return;
    const onDocClick = (e: MouseEvent) => {
      if (menuContainerRef.current && !menuContainerRef.current.contains(e.target as Node)) {
        setMenuOpen(null);
      }
    };
    document.addEventListener("mousedown", onDocClick);
    return () => document.removeEventListener("mousedown", onDocClick);
  }, [menuOpen]);

  const handleSave = async () => {
    if (!currentFilePath) {
      setAiResponse((prev) => prev + "\nНет открытого файла для сохранения.\n");
      return;
    }
    const inv = invokeRef.current;
    if (!inv) return;
    try {
      await inv("write_project_file", { relativePath: currentFilePath, content: code });
      setFileContents((prev) => ({ ...prev, [currentFilePath]: code }));
      setAiResponse((prev) => prev + `\nСохранено: ${currentFilePath}\n`);
    } catch (e) {
      setAiResponse((prev) => prev + `\nОшибка сохранения: ${String(e)}\n`);
    }
  };

  const handleSaveAs = async () => {
    if (!saveAsPath.trim()) return;
    const inv = invokeRef.current;
    if (!inv) return;
    try {
      await inv("write_project_file", { relativePath: saveAsPath.trim(), content: code });
      setShowSaveAsModal(false);
      setSaveAsPath("");
      setOpenFiles((prev) => (prev.includes(saveAsPath.trim()) ? prev : [...prev, saveAsPath.trim()]));
      setCurrentFilePath(saveAsPath.trim());
      setFileContents((prev) => ({ ...prev, [saveAsPath.trim()]: code }));
      refreshProjectTree();
      setAiResponse((prev) => prev + `\nСохранено как: ${saveAsPath.trim()}\n`);
    } catch (e) {
      setAiResponse((prev) => prev + `\nОшибка: ${String(e)}\n`);
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", background: "var(--kenga-bg)", color: "var(--kenga-fg)" }}>
      {inTauri && (
        <div
          ref={menuContainerRef}
          style={{
            display: "flex",
            alignItems: "center",
            padding: "2px 8px",
            background: "var(--kenga-panel)",
            borderBottom: "1px solid var(--kenga-border)",
            fontSize: 13,
            gap: 4,
          }}
        >
          <div style={{ position: "relative" }}>
            <button
              type="button"
              onClick={() => setMenuOpen((prev) => (prev === "file" ? null : "file"))}
              style={{
                padding: "4px 10px",
                border: "none",
                background: menuOpen === "file" ? "var(--kenga-panel)" : "transparent",
                cursor: "pointer",
                borderRadius: 4,
              }}
            >
              Файл
            </button>
            {menuOpen === "file" && (
              <div
                style={{
                  position: "absolute",
                  top: "100%",
                  left: 0,
                  marginTop: 0,
                  background: "var(--kenga-bg)",
                  border: "1px solid var(--kenga-border)",
                  borderRadius: 4,
                  boxShadow: "0 4px 12px rgba(0,0,0,0.15)",
                  minWidth: 180,
                  zIndex: 2000,
                }}
              >
                <button
                  type="button"
                  onClick={() => {
                    setMenuOpen(null);
                    setShowCreateModal(true);
                    setCreateError(null);
                  }}
                  style={{
                    display: "block",
                    width: "100%",
                    padding: "8px 12px",
                    textAlign: "left",
                    border: "none",
                    background: "transparent",
                    cursor: "pointer",
                    fontSize: 13,
                  }}
                >
                  Новый проект
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setMenuOpen(null);
                    handleOpenProject();
                  }}
                  style={{
                    display: "block",
                    width: "100%",
                    padding: "8px 12px",
                    textAlign: "left",
                    border: "none",
                    background: "transparent",
                    cursor: "pointer",
                    fontSize: 13,
                  }}
                >
                  Открыть папку
                </button>
                <div style={{ borderTop: "1px solid var(--kenga-border)", margin: "2px 0" }} />
                <button
                  type="button"
                  onClick={() => {
                    setMenuOpen(null);
                    handleSave();
                  }}
                  style={{
                    display: "block",
                    width: "100%",
                    padding: "8px 12px",
                    textAlign: "left",
                    border: "none",
                    background: "transparent",
                    cursor: "pointer",
                    fontSize: 13,
                  }}
                >
                  Сохранить
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setMenuOpen(null);
                    setShowSaveAsModal(true);
                    setSaveAsPath(currentFilePath ?? "");
                  }}
                  style={{
                    display: "block",
                    width: "100%",
                    padding: "8px 12px",
                    textAlign: "left",
                    border: "none",
                    background: "transparent",
                    cursor: "pointer",
                    fontSize: 13,
                  }}
                >
                  Сохранить как…
                </button>
              </div>
            )}
          </div>
          <div style={{ position: "relative" }}>
            <button
              type="button"
              onClick={() => setMenuOpen((prev) => (prev === "view" ? null : "view"))}
              style={{
                padding: "4px 10px",
                border: "none",
                background: menuOpen === "view" ? "var(--kenga-panel)" : "transparent",
                cursor: "pointer",
                borderRadius: 4,
              }}
            >
              Вид
            </button>
            {menuOpen === "view" && (
              <div
                style={{
                  position: "absolute",
                  top: "100%",
                  left: 0,
                  marginTop: 2,
                  background: "var(--kenga-bg)",
                  border: "1px solid var(--kenga-border)",
                  borderRadius: 4,
                  boxShadow: "0 4px 12px rgba(0,0,0,0.15)",
                  minWidth: 180,
                  zIndex: 2000,
                }}
              >
                <button type="button" onClick={() => { setMenuOpen(null); setLeftPanelWidth((w) => projectPath ? (w < MIN_LEFT_PANEL + 20 ? DEFAULT_LEFT_PANEL_WIDTH : MIN_LEFT_PANEL) : w); }} style={{ display: "block", width: "100%", padding: "8px 12px", textAlign: "left", border: "none", background: "transparent", cursor: "pointer", fontSize: 13 }}>
                  Панель файлов
                </button>
                <button type="button" onClick={() => { setMenuOpen(null); setSplitActive((prev) => !prev); }} style={{ display: "block", width: "100%", padding: "8px 12px", textAlign: "left", border: "none", background: "transparent", cursor: "pointer", fontSize: 13 }}>
                  Split Editor
                </button>
                <button type="button" onClick={() => { setMenuOpen(null); setShowCommandPalette(true); setCommandPaletteQuery(""); setCommandPaletteSelected(0); setTimeout(() => commandPaletteInputRef.current?.focus(), 50); }} style={{ display: "block", width: "100%", padding: "8px 12px", textAlign: "left", border: "none", background: "transparent", cursor: "pointer", fontSize: 13 }}>
                  Command Palette
                </button>
                <div style={{ borderTop: "1px solid var(--kenga-border)", margin: "2px 0" }} />
                <div style={{ padding: "4px 12px", fontSize: 11, color: "var(--kenga-muted)" }}>Тема</div>
                <button type="button" onClick={() => { setMenuOpen(null); setTheme("light"); }} style={{ display: "block", width: "100%", padding: "6px 12px 6px 24px", textAlign: "left", border: "none", background: theme === "light" ? "var(--kenga-accent-bg)" : "transparent", cursor: "pointer", fontSize: 13 }}>
                  {theme === "light" ? "● " : ""}Светлая
                </button>
                <button type="button" onClick={() => { setMenuOpen(null); setTheme("dark"); }} style={{ display: "block", width: "100%", padding: "6px 12px 6px 24px", textAlign: "left", border: "none", background: theme === "dark" ? "var(--kenga-accent-bg)" : "transparent", cursor: "pointer", fontSize: 13 }}>
                  {theme === "dark" ? "● " : ""}Тёмная
                </button>
                <button type="button" onClick={() => { setMenuOpen(null); setTheme("high-contrast"); }} style={{ display: "block", width: "100%", padding: "6px 12px 6px 24px", textAlign: "left", border: "none", background: theme === "high-contrast" ? "var(--kenga-accent-bg)" : "transparent", cursor: "pointer", fontSize: 13 }}>
                  {theme === "high-contrast" ? "● " : ""}Высокий контраст
                </button>
              </div>
            )}
          </div>
          <div style={{ position: "relative" }}>
            <button
              type="button"
              onClick={() => setMenuOpen((prev) => (prev === "ai" ? null : "ai"))}
              style={{
                padding: "4px 10px",
                border: "none",
                background: menuOpen === "ai" ? "var(--kenga-panel)" : "transparent",
                cursor: "pointer",
                borderRadius: 4,
              }}
            >
              AI
            </button>
            {menuOpen === "ai" && (
              <div
                style={{
                  position: "absolute",
                  top: "100%",
                  left: 0,
                  marginTop: 2,
                  background: "var(--kenga-bg)",
                  border: "1px solid var(--kenga-border)",
                  borderRadius: 4,
                  boxShadow: "0 4px 12px rgba(0,0,0,0.15)",
                  minWidth: 180,
                  zIndex: 2000,
                }}
              >
                <button type="button" onClick={() => { setMenuOpen(null); setShowAgentPrompt(true); setAgentPromptInput(""); }} style={{ display: "block", width: "100%", padding: "8px 12px", textAlign: "left", border: "none", background: "transparent", cursor: "pointer", fontSize: 13 }}>
                  Запустить агента
                </button>
                <button type="button" onClick={() => { setMenuOpen(null); if (agentRequestId) invokeRef.current?.("ai_cancel", { requestId: agentRequestId }); }} disabled={!agentRequestId} style={{ display: "block", width: "100%", padding: "8px 12px", textAlign: "left", border: "none", background: "transparent", cursor: agentRequestId ? "pointer" : "not-allowed", fontSize: 13, opacity: agentRequestId ? 1 : 0.5 }}>
                  Остановить агента
                </button>
                <div style={{ borderTop: "1px solid var(--kenga-border)", margin: "2px 0" }} />
                <button type="button" onClick={() => { setMenuOpen(null); setShowSwitchModelModal(true); loadAiProviders(); }} style={{ display: "block", width: "100%", padding: "8px 12px", textAlign: "left", border: "none", background: "transparent", cursor: "pointer", fontSize: 13 }}>
                  Выбор модели
                </button>
                <button type="button" onClick={() => { setMenuOpen(null); setShowAddProviderModal(true); setAddProviderApiKey(""); setAddProviderError(null); }} style={{ display: "block", width: "100%", padding: "8px 12px", textAlign: "left", border: "none", background: "transparent", cursor: "pointer", fontSize: 13 }}>
                  Добавить провайдер (API)
                </button>
                <button type="button" onClick={() => { setMenuOpen(null); invokeRef.current?.("open_audit_folder").catch(() => {}); }} style={{ display: "block", width: "100%", padding: "8px 12px", textAlign: "left", border: "none", background: "transparent", cursor: "pointer", fontSize: 13 }}>
                  Папка аудита
                </button>
              </div>
            )}
          </div>
          <div style={{ position: "relative" }}>
            <button
              type="button"
              onClick={() => setMenuOpen((prev) => (prev === "tools" ? null : "tools"))}
              style={{
                padding: "4px 10px",
                border: "none",
                background: menuOpen === "tools" ? "var(--kenga-panel)" : "transparent",
                cursor: "pointer",
                borderRadius: 4,
              }}
            >
              Инструменты
            </button>
            {menuOpen === "tools" && (
              <div
                style={{
                  position: "absolute",
                  top: "100%",
                  left: 0,
                  marginTop: 2,
                  background: "var(--kenga-bg)",
                  border: "1px solid var(--kenga-border)",
                  borderRadius: 4,
                  boxShadow: "0 4px 12px rgba(0,0,0,0.15)",
                  minWidth: 180,
                  zIndex: 2000,
                }}
              >
                <button type="button" onClick={() => { setMenuOpen(null); invokeRef.current?.("open_mcp_config_folder").catch(() => {}); }} style={{ display: "block", width: "100%", padding: "8px 12px", textAlign: "left", border: "none", background: "transparent", cursor: "pointer", fontSize: 13 }}>
                  MCP Servers (mcp.json)
                </button>
              </div>
            )}
          </div>
          <div style={{ position: "relative" }}>
            <button
              type="button"
              onClick={() => setMenuOpen((prev) => (prev === "help" ? null : "help"))}
              style={{
                padding: "4px 10px",
                border: "none",
                background: menuOpen === "help" ? "var(--kenga-panel)" : "transparent",
                cursor: "pointer",
                borderRadius: 4,
              }}
            >
              Справка
            </button>
            {menuOpen === "help" && (
              <div
                style={{
                  position: "absolute",
                  top: "100%",
                  left: 0,
                  marginTop: 2,
                  background: "var(--kenga-bg)",
                  border: "1px solid var(--kenga-border)",
                  borderRadius: 4,
                  boxShadow: "0 4px 12px rgba(0,0,0,0.15)",
                  minWidth: 180,
                  zIndex: 2000,
                }}
              >
                <button type="button" onClick={() => { setMenuOpen(null); invokeRef.current?.("open_logs_folder").catch(() => {}); }} style={{ display: "block", width: "100%", padding: "8px 12px", textAlign: "left", border: "none", background: "transparent", cursor: "pointer", fontSize: 13 }}>
                  Открыть логи
                </button>
                <button type="button" onClick={() => { setMenuOpen(null); invokeRef.current?.("open_audit_folder").catch(() => {}); }} style={{ display: "block", width: "100%", padding: "8px 12px", textAlign: "left", border: "none", background: "transparent", cursor: "pointer", fontSize: 13 }}>
                  Папка аудита
                </button>
                <div style={{ borderTop: "1px solid var(--kenga-border)", margin: "2px 0" }} />
                <button type="button" onClick={() => { setMenuOpen(null); setShowAboutModal(true); }} style={{ display: "block", width: "100%", padding: "8px 12px", textAlign: "left", border: "none", background: "transparent", cursor: "pointer", fontSize: 13 }}>
                  О программе
                </button>
              </div>
            )}
          </div>
        </div>
      )}
      {showSaveAsModal && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.4)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 3000,
          }}
          onClick={() => setShowSaveAsModal(false)}
        >
          <div
            style={{
              background: "var(--kenga-bg)",
              padding: 20,
              borderRadius: 8,
              minWidth: 400,
              boxShadow: "0 4px 20px rgba(0,0,0,0.2)",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 style={{ margin: "0 0 12px 0" }}>Сохранить как</h3>
            <input
              type="text"
              value={saveAsPath}
              onChange={(e) => setSaveAsPath(e.target.value)}
              placeholder="путь/к/файлу.txt"
              style={{
                width: "100%",
                padding: "8px 12px",
                fontSize: 13,
                border: "1px solid #ccc",
                borderRadius: 4,
                boxSizing: "border-box",
              }}
            />
            <div style={{ display: "flex", gap: 8, marginTop: 16, justifyContent: "flex-end" }}>
              <button
                type="button"
                onClick={() => setShowSaveAsModal(false)}
                style={{ padding: "8px 16px", fontSize: 13 }}
              >
                Отмена
              </button>
              <button
                type="button"
                onClick={handleSaveAs}
                style={{
                  padding: "8px 16px",
                  fontSize: 13,
                  background: "#1e88e5",
                  color: "#fff",
                  border: "none",
                  borderRadius: 4,
                  cursor: "pointer",
                }}
              >
                Сохранить
              </button>
            </div>
          </div>
        </div>
      )}
      <header
        style={{
          padding: "8px 16px",
          borderBottom: "1px solid var(--kenga-border)",
          background: "var(--kenga-panel)",
          display: "flex",
          alignItems: "center",
          gap: 12,
          flexWrap: "wrap",
        }}
      >
        <strong>KengaIDE</strong>
        {inTauri && (
          <>
            <button
              type="button"
              onClick={handleOpenProject}
              style={{ fontSize: 12, padding: "4px 10px" }}
            >
              Открыть папку
            </button>
            <button
              type="button"
              onClick={() => {
                setShowCreateModal(true);
                setCreateError(null);
              }}
              style={{ fontSize: 12, padding: "4px 10px" }}
            >
              Новый проект
            </button>
            <button
              type="button"
              onClick={() => setSplitActive((prev) => !prev)}
              style={{
                fontSize: 12,
                padding: "4px 10px",
                background: splitActive ? "var(--kenga-accent-bg)" : "transparent",
              }}
              title="Разделить вид (Ctrl+клик по файлу — открыть во втором)"
            >
              Split
            </button>
            <button
              type="button"
              onClick={async () => {
                const inv = invokeRef.current;
                if (inv) {
                  try {
                    await inv("open_mcp_config_folder");
                  } catch (e) {
                    setAiResponse((prev) => prev + `\nMCP: ${String(e)}\n`);
                  }
                }
              }}
              style={{ fontSize: 12, padding: "4px 10px" }}
              title="Открыть папку настроек MCP (~/.kengaide, файл mcp.json)"
            >
              MCP
            </button>
          </>
        )}
        {projectPath && (
          <span
            style={{
              fontSize: 11,
              color: "var(--kenga-muted)",
              maxWidth: 280,
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
            }}
            title={projectPath}
          >
            {projectPath}
          </span>
        )}
        {inTauri && (
          <>
            <button
              type="button"
              onClick={() => {
                setShowSwitchModelModal(true);
                loadAiProviders();
              }}
              style={{ fontSize: 11, padding: "4px 8px" }}
              title="Сменить модель / провайдер"
            >
              Модель
            </button>
            <button
              type="button"
              onClick={() => {
                setShowAddProviderModal(true);
                setAddProviderApiKey("");
                setAddProviderError(null);
              }}
              style={{ fontSize: 11, padding: "4px 8px" }}
              title="Добавить AI через API (OpenAI и др.)"
            >
              + API
            </button>
          </>
        )}
        {tauriReady && !inTauri && (
          <span style={{ fontSize: 11, color: "var(--kenga-muted)" }}>
            Запустите приложение KengaIDE
          </span>
        )}
        {(modelStatus === "ready" || modelSelection || activeProviderId) && (
          <span
            style={{
              fontSize: 11,
              background: modelSelection ? "var(--kenga-accent-bg)" : "var(--kenga-panel)",
              padding: "2px 8px",
              borderRadius: 4,
            }}
            title={modelSelection ? `Role: ${modelSelection.role}` : activeProviderId ? aiProviders.find((p) => p.id === activeProviderId)?.name : undefined}
          >
            {modelSelection
              ? `${modelSelection.role} · ${modelSelection.model_id}`
              : activeProviderId
                ? aiProviders.find((p) => p.id === activeProviderId)?.name ?? activeProviderId
                : "LOCAL"}
          </span>
        )}
        {downloading && (
          <span style={{ fontSize: 11, color: "var(--kenga-muted)" }}>
            Downloading… {progressPct}%
          </span>
        )}
        {modelStatus === "not_loaded" && !downloading && !showDownloadDialog && (
          <span style={{ fontSize: 11, color: "var(--kenga-muted)" }}>
            Local model: not loaded
          </span>
        )}
      </header>

      {showDownloadDialog && modelInfo && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.5)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 1000,
          }}
        >
          <div
            style={{
              background: "white",
              padding: 24,
              borderRadius: 8,
              maxWidth: 400,
              boxShadow: "0 4px 20px rgba(0,0,0,0.2)",
            }}
          >
            <h3 style={{ marginTop: 0 }}>Офлайн-модель</h3>
            <p>
              Для офлайн-работы будет загружена российская нейромодель{" "}
              {modelInfo.display_name} (~{Math.round(modelInfo.size_gb)} ГБ).
              Продолжить?
            </p>
            {downloading && (
              <p style={{ fontSize: 13, color: "var(--kenga-muted)", marginBottom: 12 }}>
                {downloadProgress && downloadProgress.bytes_total > 0
                  ? `Загрузка… ${progressPct}%`
                  : "Загрузка началась…"}
              </p>
            )}
            {downloadError && (
              <p style={{ fontSize: 13, color: "#c62828", marginBottom: 12 }}>
                Ошибка: {downloadError}
              </p>
            )}
            <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
              <button
                onClick={() => {
                  setShowDownloadDialog(false);
                  setDownloadError(null);
                }}
              >
                Отмена
              </button>
              <button onClick={handleConfirmDownload} disabled={downloading}>
                Продолжить
              </button>
            </div>
          </div>
        </div>
      )}

      <div style={{ display: "flex", flex: 1, minHeight: 0 }}>
        {projectPath && (
          <>
            <aside
              style={{
                width: leftPanelWidth,
                minWidth: MIN_LEFT_PANEL,
                flexShrink: 0,
                borderRight: "1px solid var(--kenga-border)",
                display: "flex",
                flexDirection: "column",
                overflow: "hidden",
                background: "var(--kenga-panel)",
              }}
            >
              <div style={{ padding: "8px 10px", borderBottom: "1px solid var(--kenga-border)", fontSize: 12, fontWeight: 600 }}>
                Файлы
              </div>
              <div style={{ flex: 1, overflow: "auto", padding: "6px 0" }}>
                {projectTreeLoading && <div style={{ padding: 8, fontSize: 11, color: "var(--kenga-muted)" }}>Загрузка…</div>}
                {!projectTreeLoading && projectTree && projectTree.length === 0 && (
                  <div style={{ padding: 8, fontSize: 11, color: "var(--kenga-muted)" }}>Папка пуста</div>
                )}
                {!projectTreeLoading && projectTree && projectTree.length > 0 && (
                  <FileTreeList
                    nodes={projectTree}
                    expanded={expandedPaths}
                    onToggle={toggleTreePath}
                    onFileClick={(path, e) => handleOpenFile(path, e?.ctrlKey)}
                    currentFilePath={currentFilePath}
                  />
                )}
              </div>
            </aside>
            <div
              role="separator"
              aria-label="Изменить ширину панели файлов"
              style={{
                width: RESIZER_WIDTH,
                minWidth: RESIZER_WIDTH,
                flexShrink: 0,
                cursor: "col-resize",
                background: resizing === "left" ? "#1e88e5" : "#e0e0e0",
                borderLeft: "1px solid #ccc",
                borderRight: "1px solid #ccc",
              }}
              onMouseDown={(e) => {
                if (e.button !== 0) return;
                dragStartRef.current = { x: e.clientX, left: leftPanelWidth, right: rightPanelWidth };
                setResizing("left");
              }}
            />
          </>
        )}
        <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column" }}>
          {!projectPath && inTauri && (
            <div
              style={{
                flex: 1,
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "center",
                background: "var(--kenga-bg, #fff)",
                padding: 32,
              }}
            >
              <h2 style={{ margin: "0 0 8px 0", fontSize: 20, color: "var(--kenga-fg, #333)" }}>Добро пожаловать в KengaIDE</h2>
              <p style={{ margin: "0 0 24px 0", fontSize: 14, color: "var(--kenga-muted)" }}>Что хотите сделать?</p>
              <div style={{ display: "flex", gap: 12, flexWrap: "wrap", justifyContent: "center" }}>
                <button
                  type="button"
                  onClick={() => { setShowCreateModal(true); setCreateError(null); }}
                  style={{ padding: "12px 24px", fontSize: 14, background: "var(--kenga-accent, #1e88e5)", color: "#fff", border: "none", borderRadius: 8, cursor: "pointer", fontWeight: 600 }}
                >
                  Новый проект
                </button>
                <button
                  type="button"
                  onClick={handleOpenProject}
                  style={{ padding: "12px 24px", fontSize: 14, background: "var(--kenga-panel, #f5f5f5)", color: "var(--kenga-fg, #333)", border: "1px solid var(--kenga-border)", borderRadius: 8, cursor: "pointer" }}
                >
                  Открыть папку
                </button>
              </div>
            </div>
          )}
          {openFiles.length > 0 && (
            <div
              style={{
                display: "flex",
                alignItems: "center",
                background: "var(--kenga-panel, #f5f5f5)",
                borderBottom: "1px solid var(--kenga-border, #ddd)",
                minHeight: 32,
                overflowX: "auto",
              }}
            >
              {openFiles.map((path) => {
                const name = path.split(/[/\\]/).pop() ?? path;
                const isActive = path === currentFilePath;
                return (
                  <div
                    key={path}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      padding: "4px 8px 4px 12px",
                      background: isActive ? "var(--kenga-bg)" : "transparent",
                      borderRight: "1px solid #ddd",
                      cursor: "pointer",
                      maxWidth: 180,
                      minWidth: 60,
                    }}
                    onClick={() => handleSwitchTab(path)}
                    title={path}
                  >
                    <span
                      style={{
                        flex: 1,
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                        fontSize: 12,
                      }}
                    >
                      {name}
                    </span>
                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleCloseTab(path);
                      }}
                      style={{
                        marginLeft: 4,
                        padding: "0 4px",
                        border: "none",
                        background: "transparent",
                        cursor: "pointer",
                        fontSize: 14,
                        lineHeight: 1,
                      }}
                    >
                      ×
                    </button>
                  </div>
                );
              })}
            </div>
          )}
          <div style={{ flex: 1, minHeight: 0, display: "flex", flexDirection: splitActive ? "row" : "column" }}>
            {projectPath && (
            <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column" }}>
              <Editor
                height="100%"
                language={editorLanguage}
                value={code}
                onChange={(v) => setCode(v ?? "")}
                theme={THEME_MONACO[theme]}
                onMount={(editor) => {
                  editor.onDidChangeCursorSelection(() => {
                    const sel = editor.getSelection();
                    if (sel) {
                      const text = editor.getModel()?.getValueInRange({
                        startLineNumber: sel.startLineNumber,
                        startColumn: sel.startColumn,
                        endLineNumber: sel.endLineNumber,
                        endColumn: sel.endColumn,
                      });
                      editorSelectionRef.current = text ?? null;
                    } else {
                      editorSelectionRef.current = null;
                    }
                  });
                }}
                options={{ minimap: { enabled: false } }}
              />
            </div>
            )}
            {splitActive && projectPath && (
              <>
                <div
                  style={{
                    width: 4,
                    background: "#ddd",
                    cursor: "col-resize",
                    flexShrink: 0,
                  }}
                />
                <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column" }}>
                  {splitFilePath && (
                    <div
                      style={{
                        padding: "4px 8px",
                        fontSize: 11,
                        color: "var(--kenga-muted)",
                        background: "#f0f0f0",
                        borderBottom: "1px solid #ddd",
                      }}
                    >
                      Split: {splitFilePath}
                    </div>
                  )}
                  <Editor
                    height="100%"
                    language={
                      splitFilePath
                        ? (() => {
                            const ext = splitFilePath.replace(/^.*\./, "").toLowerCase();
                            const map: Record<string, string> = {
                              rs: "rust",
                              ts: "typescript",
                              tsx: "typescriptreact",
                              js: "javascript",
                              py: "python",
                            };
                            return map[ext] ?? "plaintext";
                          })()
                        : "plaintext"
                    }
                    value={splitCode}
                    onChange={(v) => setSplitCode(v ?? "")}
                    theme={THEME_MONACO[theme]}
                    options={{ minimap: { enabled: false } }}
                  />
                </div>
              </>
            )}
          </div>
        </div>
        <div
          role="separator"
          aria-label="Изменить ширину панели AI"
          style={{
            width: RESIZER_WIDTH,
            minWidth: RESIZER_WIDTH,
            flexShrink: 0,
            cursor: "col-resize",
            background: resizing === "right" ? "#1e88e5" : "#e0e0e0",
            borderLeft: "1px solid #ccc",
            borderRight: "1px solid #ccc",
          }}
          onMouseDown={(e) => {
            if (e.button !== 0) return;
            dragStartRef.current = { x: e.clientX, left: leftPanelWidth, right: rightPanelWidth };
            setResizing("right");
          }}
        />
        <aside
          style={{
            width: rightPanelWidth,
            minWidth: MIN_RIGHT_PANEL,
            flexShrink: 0,
            borderLeft: "1px solid var(--kenga-border)",
            padding: 16,
            overflow: "auto",
            background: "var(--kenga-bg)",
          }}
        >
          <h3 style={{ marginTop: 0 }}>AI</h3>
          {(agentRequestId || streamingRequestId || lastAgentMessage || toolTimeline.length > 0) && (
            <div
              style={{
                display: "flex",
                gap: 6,
                marginBottom: 12,
                flexWrap: "wrap",
                padding: "8px 0",
                borderBottom: "1px solid var(--kenga-border, #eee)",
              }}
            >
              <button
                type="button"
                onClick={() => handleAgentRequest()}
                disabled={!!streamingRequestId || !!agentRequestId || !projectPath}
                style={{ padding: "4px 10px", fontSize: 11, background: projectPath && !agentRequestId ? "var(--kenga-accent)" : "var(--kenga-muted)", color: "#fff", border: "none", borderRadius: 4, cursor: projectPath ? "pointer" : "not-allowed" }}
                title="Запустить агента"
              >
                ▶ Run
              </button>
              <button
                type="button"
                onClick={() => { if (agentRequestId) invokeRef.current?.("ai_cancel", { requestId: agentRequestId }); if (streamingRequestId) handleStopGeneration(); }}
                disabled={!agentRequestId && !streamingRequestId}
                style={{ padding: "4px 10px", fontSize: 11, background: agentRequestId || streamingRequestId ? "#c62828" : "var(--kenga-muted)", color: "#fff", border: "none", borderRadius: 4, cursor: agentRequestId || streamingRequestId ? "pointer" : "not-allowed" }}
                title="Остановить"
              >
                ⏹ Stop
              </button>
              {lastAgentMessage && !agentRequestId && !streamingRequestId && (
                <>
                  <button type="button" onClick={() => handleAgentRequest(lastAgentMessage)} style={{ padding: "4px 10px", fontSize: 11 }} title="Повторить">↺ Retry</button>
                  <button type="button" onClick={() => handleAgentRequest(`${lastAgentMessage}\n\nКонтекст: ${currentFilePath ?? ""}\n${code.slice(0, 2000)}`)} style={{ padding: "4px 10px", fontSize: 11 }} title="С контекстом">🧠 Retry+</button>
                </>
              )}
              {appliedPatches.length > 0 && !agentRequestId && (
                <button
                  type="button"
                  onClick={async () => {
                    const inv = invokeRef.current;
                    if (!inv) return;
                    try {
                      const n = (await inv("rollback_patches", { patches: appliedPatches })) as number;
                      setAiResponse((prev) => prev + `\nОткачено ${n} патчей.\n`);
                      setAppliedPatches([]);
                      if (currentFilePath) {
                        inv("read_project_file", { relativePath: currentFilePath })
                          .then((c) => setCode(c as string))
                          .catch(() => {});
                      }
                    } catch (e) {
                      setAiResponse((prev) => prev + `\nОшибка отката: ${String(e)}\n`);
                    }
                  }}
                  style={{ padding: "4px 10px", fontSize: 11, color: "#c62828" }}
                  title="Откатить изменения"
                >
                  ⏪ Rollback
                </button>
              )}
            </div>
          )}
          {toolTimeline.length > 0 && (
            <div style={{ marginBottom: 16 }}>
              <button
                type="button"
                onClick={() => setToolTimelineExpanded(!toolTimelineExpanded)}
                style={{
                  width: "100%",
                  padding: "6px 0",
                  textAlign: "left",
                  border: "none",
                  background: "transparent",
                  cursor: "pointer",
                  fontSize: 12,
                  fontWeight: 600,
                }}
              >
                {toolTimelineExpanded ? "▼" : "▶"} Tool timeline ({toolTimeline.length})
              </button>
              {toolTimelineExpanded && (
                <div
                  style={{
                    maxHeight: 200,
                    overflow: "auto",
                    fontSize: 11,
                    background: "#f9f9f9",
                    padding: 8,
                    borderRadius: 4,
                  }}
                >
                  {toolTimeline
                    .filter(
                      (t) =>
                        t.kind !== "patch_applied" &&
                        t.kind !== "model_selected" &&
                        t.kind !== "session_started"
                    )
                    .map((t, i) => (
                    <div key={i} style={{ marginBottom: 6, display: "flex", alignItems: "flex-start", gap: 6 }}>
                      {t.kind === "thinking" && (
                        <span style={{ color: "#7e57c2", fontSize: 12 }} title="Модель генерирует ответ">
                          🧠 Thinking…
                        </span>
                      )}
                      {t.kind === "tool_call" && (
                        <>
                          <span style={{ color: "var(--kenga-accent)", flexShrink: 0 }}>🛠</span>
                          <span style={{ color: "var(--kenga-accent)" }}>
                            {t.name}
                            {t.path && (
                              <button
                                type="button"
                                onClick={() => handleOpenFile(t.path!)}
                                style={{
                                  marginLeft: 4,
                                  padding: "0 4px",
                                  fontSize: 10,
                                  background: "var(--kenga-accent-bg)",
                                  border: "none",
                                  borderRadius: 2,
                                  cursor: "pointer",
                                  textDecoration: "underline",
                                }}
                                title={`Открыть ${t.path}`}
                              >
                                {t.path}
                              </button>
                            )}
                          </span>
                        </>
                      )}
                      {t.kind === "tool_result" && (
                        <>
                          <span style={{ color: t.success ? "#2e7d32" : "#c62828", flexShrink: 0 }}>
                            {t.success ? "✓" : "⚠"}
                          </span>
                          <span style={{ color: t.success ? "#2e7d32" : "#c62828", fontSize: 11 }}>
                            {t.output?.slice(0, 100)}
                            {(t.output?.length ?? 0) > 100 ? "…" : ""}
                          </span>
                        </>
                      )}
                      {t.kind === "patch_apply_success" && (
                        <>
                          <span style={{ color: "#2e7d32", flexShrink: 0 }}>📄</span>
                          <span style={{ color: "#2e7d32" }}>
                            Patched{" "}
                            <button
                              type="button"
                              onClick={() => handleOpenFile(t.path!)}
                              style={{
                                padding: "0 4px",
                                fontSize: 10,
                                background: "rgba(46,125,50,0.15)",
                                border: "none",
                                borderRadius: 2,
                                cursor: "pointer",
                                textDecoration: "underline",
                              }}
                            >
                              {t.path}
                            </button>
                          </span>
                        </>
                      )}
                      {t.kind === "patch_apply_error" && (
                        <>
                          <span style={{ color: "#c62828", flexShrink: 0 }}>⚠</span>
                          <span style={{ color: "#c62828" }}>
                            {t.path}: {t.message}
                          </span>
                        </>
                      )}
                      {t.kind === "done" && (
                        <>
                          <span style={{ color: "#2e7d32", flexShrink: 0 }}>✅</span>
                          <span style={{ color: "var(--kenga-muted)", fontSize: 11 }}>{t.message?.slice(0, 80)}{(t.message?.length ?? 0) > 80 ? "…" : ""}</span>
                        </>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
          {downloading && downloadProgress && (
            <div style={{ marginBottom: 16 }}>
              <div
                style={{
                  height: 8,
                  background: "var(--kenga-panel)",
                  borderRadius: 4,
                  overflow: "hidden",
                }}
              >
                <div
                  style={{
                    height: "100%",
                    width: `${progressPct}%`,
                    background: "#4caf50",
                    transition: "width 0.2s",
                  }}
                />
              </div>
              <small>
                {Math.round(downloadProgress.bytes_done / 1024 / 1024)} /{" "}
                {Math.round(downloadProgress.bytes_total / 1024 / 1024)} MB
              </small>
            </div>
          )}
          <input
            type="text"
            placeholder="Сообщение..."
            value={aiInput}
            onChange={(e) => setAiInput(e.target.value)}
            style={{ width: "100%", marginBottom: 8, padding: 8 }}
          />
          <div style={{ display: "flex", gap: 8, marginBottom: 16, flexWrap: "wrap" }}>
            <button
              onClick={() => handleAiRequest("chat")}
              disabled={!!streamingRequestId}
            >
              Chat
            </button>
            <button
              onClick={() => handleAiRequest("explain")}
              disabled={!!streamingRequestId}
            >
              Explain
            </button>
            <button
              onClick={() => handleAiRequest("refactor")}
              disabled={!!streamingRequestId}
            >
              Refactor
            </button>
            <button
              onClick={() => handleAiRequest("generate")}
              disabled={!!streamingRequestId}
            >
              Generate
            </button>
            <button
              onClick={() => handleAgentRequest()}
              disabled={!!streamingRequestId || !!agentRequestId || !projectPath}
              style={{ background: projectPath ? "var(--kenga-accent)" : "var(--kenga-muted)", color: "#fff" }}
              title={projectPath ? "План → инструменты (create_file, read_file, …) → проверка." : "Сначала откройте папку проекта (кнопка в шапке)."}
            >
              Agent
            </button>
            {streamingRequestId && (
              <button
                onClick={handleStopGeneration}
                style={{ marginLeft: "auto", background: "#c62828", color: "#fff" }}
              >
                STOP
              </button>
            )}
          </div>
          {lastSessionId && !streamingRequestId && !agentRequestId && (
            <button
              type="button"
              onClick={() => invokeRef.current?.("open_audit_folder").catch(() => {})}
              style={{ fontSize: 11, padding: "4px 8px", marginBottom: 8 }}
              title="Открыть папку аудита"
            >
              📂 Аудит
            </button>
          )}
          {streamingRequestId && (
            <div style={{ fontSize: 11, color: "var(--kenga-muted)", marginBottom: 8 }}>
              Генерация…
            </div>
          )}
          <pre
            ref={responseEndRef}
            style={{
              background: "var(--kenga-panel)",
              padding: 12,
              fontSize: 12,
              overflow: "auto",
              flex: 1,
              minHeight: 120,
            }}
          >
            {aiResponse || "Ответ появится здесь"}
          </pre>
        </aside>
      </div>

      {inTauri && (
        <footer
          style={{
            display: "flex",
            alignItems: "center",
            gap: 16,
            padding: "4px 12px",
            fontSize: 11,
            background: "var(--kenga-panel, #f5f5f5)",
            color: "var(--kenga-fg, #333)",
            borderTop: "1px solid var(--kenga-border, #e0e0e0)",
            flexShrink: 0,
          }}
        >
          <span title="Проект">{projectPath ? (projectPath.split(/[/\\]/).pop() ?? projectPath) : "—"}</span>
          <span style={{ color: "var(--kenga-border)" }}>|</span>
          <span title="Git">{gitStatus ? `${gitStatus.branch}${gitStatus.changes > 0 ? ` • ${gitStatus.changes} изменений` : ""}` : "—"}</span>
          <span style={{ color: "var(--kenga-border)" }}>|</span>
          <span title="Режим AI">{agentRequestId ? "Agent" : streamingRequestId ? "Chat" : "—"}</span>
          <span style={{ color: "var(--kenga-border)" }}>|</span>
          <span title="Модель">{modelSelection ? `${modelSelection.role} · ${modelSelection.model_id}` : modelStatus === "ready" ? "LOCAL" : "—"}</span>
          <span style={{ color: "var(--kenga-border)" }}>|</span>
          <span title="Аудит">{lastSessionId ? "● Recording" : "—"}</span>
          <span style={{ color: "var(--kenga-border)" }}>|</span>
          <span title="Система">{systemInfo ? `RAM ${systemInfo.ram_gb.toFixed(0)}GB` : "—"}</span>
          <span style={{ marginLeft: "auto" }} title="Версия">{appVersion ? `${appVersion.name} ${appVersion.version}` : ""}</span>
        </footer>
      )}

      {showCreateModal && inTauri && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.4)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 1000,
          }}
          onClick={() => setShowCreateModal(false)}
        >
          <div
            style={{
              background: "var(--kenga-bg)",
              padding: 20,
              borderRadius: 8,
              minWidth: 360,
              boxShadow: "0 4px 20px rgba(0,0,0,0.2)",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 style={{ margin: "0 0 16px 0", fontSize: 16 }}>Новый проект</h3>
            <div style={{ marginBottom: 12 }}>
              <label htmlFor="create-template" style={{ display: "block", fontSize: 12, marginBottom: 4 }}>Шаблон</label>
              <select
                id="create-template"
                aria-label="Шаблон для нового проекта"
                value={createTemplate}
                onChange={(e) => setCreateTemplate(e.target.value)}
                style={{ width: "100%", padding: 8 }}
              >
                <option value="empty">Пустой</option>
                <option value="rust">Rust</option>
                <option value="python">Python</option>
                <option value="node">Node.js</option>
              </select>
            </div>
            <div style={{ marginBottom: 12 }}>
              <label style={{ display: "block", fontSize: 12, marginBottom: 4 }}>Имя папки (опционально)</label>
              <input
                type="text"
                value={createName}
                onChange={(e) => setCreateName(e.target.value)}
                placeholder={`По умолчанию: ${createTemplate}-project`}
                style={{ width: "100%", padding: 8, boxSizing: "border-box" }}
              />
            </div>
            <div style={{ marginBottom: 12 }}>
              <label style={{ display: "block", fontSize: 12, marginBottom: 4 }}>Родительская папка</label>
              <div style={{ display: "flex", gap: 8 }}>
                <input
                  type="text"
                  value={createParentDir ?? ""}
                  onChange={(e) => setCreateParentDir(e.target.value.trim() || null)}
                  placeholder={projectPath ? "Пусто = текущий проект" : "Пусто = домашняя папка"}
                  style={{ flex: 1, padding: 8 }}
                />
                <button type="button" onClick={handlePickFolderForCreate} style={{ padding: "8px 12px" }}>
                  Обзор…
                </button>
              </div>
            </div>
            {createError && (
              <div style={{ color: "#c62828", fontSize: 12, marginBottom: 12 }}>{createError}</div>
            )}
            <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
              <button type="button" onClick={() => setShowCreateModal(false)}>
                Отмена
              </button>
              <button type="button" onClick={handleCreateProject} style={{ background: "var(--kenga-accent)", color: "#fff" }}>
                Создать
              </button>
            </div>
          </div>
        </div>
      )}

      {showCommandPalette && inTauri && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.3)",
            display: "flex",
            alignItems: "flex-start",
            justifyContent: "center",
            paddingTop: "15vh",
            zIndex: 2000,
          }}
          onClick={() => setShowCommandPalette(false)}
        >
          <div
            style={{
              background: "var(--kenga-bg)",
              borderRadius: 8,
              boxShadow: "0 8px 32px rgba(0,0,0,0.2)",
              minWidth: 480,
              maxWidth: 560,
              overflow: "hidden",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <input
              ref={commandPaletteInputRef}
              type="text"
              value={commandPaletteQuery}
              onChange={(e) => {
                setCommandPaletteQuery(e.target.value);
                setCommandPaletteSelected(0);
              }}
              onKeyDown={(e) => {
                if (e.key === "ArrowDown") {
                  e.preventDefault();
                  setCommandPaletteSelected((prev) => Math.min(prev + 1, filteredCommands.length - 1));
                } else if (e.key === "ArrowUp") {
                  e.preventDefault();
                  setCommandPaletteSelected((prev) => Math.max(prev - 1, 0));
                } else if (e.key === "Enter") {
                  e.preventDefault();
                  const cmd = filteredCommands[commandPaletteSelected];
                  if (cmd) runCommand(cmd.id);
                }
              }}
              placeholder="Введите команду..."
              style={{
                width: "100%",
                padding: "12px 16px",
                fontSize: 14,
                border: "none",
                borderBottom: "1px solid var(--kenga-border)",
                outline: "none",
                boxSizing: "border-box",
              }}
            />
            <div style={{ maxHeight: 280, overflow: "auto" }}>
              {filteredCommands.length === 0 ? (
                <div style={{ padding: 16, color: "var(--kenga-muted)", fontSize: 13 }}>Нет совпадений</div>
              ) : (
                filteredCommands.map((cmd, i) => (
                  <div
                    key={cmd.id}
                    role="button"
                    tabIndex={0}
                    onClick={() => runCommand(cmd.id)}
                    onKeyDown={(e) => e.key === "Enter" && runCommand(cmd.id)}
                    style={{
                      padding: "10px 16px",
                      fontSize: 13,
                      cursor: "pointer",
                      background: i === commandPaletteSelected ? "var(--kenga-accent-bg)" : "transparent",
                    }}
                  >
                    {cmd.label}
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      )}

      {inTauri &&
        tauriReady &&
        modelStatus === "not_loaded" &&
        !welcomeDismissed && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "linear-gradient(135deg, #1a237e 0%, #0d47a1 100%)",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 1500,
            color: "#fff",
            padding: 24,
          }}
        >
          <h1 style={{ margin: "0 0 8px 0", fontSize: 28 }}>KengaIDE</h1>
          <p style={{ margin: "0 0 16px 0", fontSize: 14, opacity: 0.9 }}>
            IDE с AI-ассистентом
          </p>
          <p style={{ margin: "0 0 20px 0", fontSize: 12, opacity: 0.8 }}>
            Что хочешь сделать?
          </p>
          <div style={{ display: "flex", gap: 12, marginBottom: 24, flexWrap: "wrap", justifyContent: "center" }}>
            <button
              type="button"
              onClick={() => {
                setShowCreateModal(true);
                setCreateError(null);
              }}
              style={{
                padding: "12px 24px",
                fontSize: 14,
                background: "var(--kenga-bg)",
                color: "var(--kenga-accent)",
                border: "none",
                borderRadius: 8,
                cursor: "pointer",
                fontWeight: 600,
              }}
            >
              Новый проект
            </button>
            <button
              type="button"
              onClick={() => {
                handleOpenProject();
              }}
              style={{
                padding: "12px 24px",
                fontSize: 14,
                background: "rgba(255,255,255,0.2)",
                color: "#fff",
                border: "1px solid rgba(255,255,255,0.6)",
                borderRadius: 8,
                cursor: "pointer",
              }}
            >
              Открыть папку
            </button>
          </div>
          <p style={{ margin: "0 0 12px 0", fontSize: 12, opacity: 0.8 }}>
            Для офлайн-режима загрузите модель. Рекомендуем DeepSeek-Coder для программирования.
          </p>
          {systemInfo && systemInfo.ram_gb > 0 && (
            <p style={{ margin: "0 0 24px 0", fontSize: 12, opacity: 0.8 }}>
              RAM: {systemInfo.ram_gb.toFixed(1)} ГБ · CPU: {systemInfo.cpu_cores} ядер
            </p>
          )}
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: 12,
              marginBottom: 24,
              minWidth: 280,
            }}
          >
            {aiProviders
              .filter((p) => p.id.startsWith("local-") && !p.available)
              .map((p) => (
                <div
                  key={p.id}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between",
                    padding: "12px 16px",
                    background: "rgba(255,255,255,0.1)",
                    borderRadius: 8,
                  }}
                >
                  <span style={{ fontSize: 14 }}>{p.name}</span>
                  <button
                    type="button"
                    onClick={async () => {
                      const inv = invokeRef.current;
                      if (!inv || downloading) return;
                      setDownloading(true);
                      setDownloadProgress(null);
                      setDownloadError(null);
                      try {
                        await inv("start_model_download_provider", { providerId: p.id });
                        refreshStatus();
                        try {
                          localStorage.setItem(STORAGE_WELCOME_DISMISSED, "1");
                          setWelcomeDismissed(true);
                        } catch {
                          /* ignore */
                        }
                      } catch (e) {
                        setDownloadError(String(e));
                      } finally {
                        setDownloading(false);
                        setDownloadProgress(null);
                      }
                    }}
                    disabled={downloading}
                    style={{
                      padding: "8px 16px",
                      fontSize: 13,
                      background: "var(--kenga-bg)",
                      color: "var(--kenga-accent)",
                      border: "none",
                      borderRadius: 6,
                      cursor: downloading ? "not-allowed" : "pointer",
                      fontWeight: 600,
                    }}
                  >
                    Загрузить
                  </button>
                </div>
              ))}
            {aiProviders.filter((p) => p.id.startsWith("local-") && !p.available).length === 0 && (
              <p style={{ fontSize: 13, opacity: 0.9 }}>
                Все локальные модели загружены или недоступны.
              </p>
            )}
          </div>
          {downloading && downloadProgress && downloadProgress.bytes_total > 0 && (
            <div style={{ marginBottom: 16, width: 280 }}>
              <div
                style={{
                  height: 8,
                  background: "rgba(255,255,255,0.3)",
                  borderRadius: 4,
                  overflow: "hidden",
                }}
              >
                <div
                  style={{
                    height: "100%",
                    width: `${progressPct}%`,
                    background: "var(--kenga-bg)",
                    transition: "width 0.2s",
                  }}
                />
              </div>
              <small style={{ fontSize: 11, opacity: 0.9 }}>
                {Math.round(downloadProgress.bytes_done / 1024 / 1024)} /{" "}
                {Math.round(downloadProgress.bytes_total / 1024 / 1024)} MB
              </small>
            </div>
          )}
          {downloadError && (
            <p style={{ marginBottom: 16, fontSize: 12, color: "#ffcdd2" }}>Ошибка: {downloadError}</p>
          )}
          <div style={{ display: "flex", gap: 12, flexWrap: "wrap", justifyContent: "center", marginBottom: 8 }}>
            <button
              type="button"
              onClick={async () => {
                const inv = invokeRef.current;
                if (!inv) return;
                try {
                  await inv("create_project", {
                    payload: { template: "rust", name: "my-first-project" },
                  });
                  try {
                    localStorage.setItem(STORAGE_WELCOME_DISMISSED, "1");
                    setWelcomeDismissed(true);
                  } catch {
                    /* ignore */
                  }
                  refreshProjectPath();
                  refreshProjectTree();
                } catch (e) {
                  setAiResponse((prev) => prev + `\nОшибка: ${String(e)}\n`);
                }
              }}
              style={{
                padding: "10px 20px",
                fontSize: 13,
                background: "rgba(76, 175, 80, 0.9)",
                color: "#fff",
                border: "none",
                borderRadius: 8,
                cursor: "pointer",
                fontWeight: 600,
              }}
            >
              Создать первый проект
            </button>
            <button
              type="button"
              onClick={() => {
                try {
                  localStorage.setItem(STORAGE_WELCOME_DISMISSED, "1");
                  setWelcomeDismissed(true);
                } catch {
                  /* ignore */
                }
              }}
              style={{
                padding: "12px 24px",
                fontSize: 14,
                background: "transparent",
                color: "#fff",
                border: "1px solid rgba(255,255,255,0.5)",
                borderRadius: 8,
                cursor: "pointer",
              }}
            >
              Пропустить
            </button>
          </div>
          <p style={{ marginTop: 24, fontSize: 11, opacity: 0.7 }}>
            Можно добавить OpenAI по API key (Ctrl+Shift+P → Добавить AI провайдер)
          </p>
        </div>
      )}

      {showAgentPrompt && inTauri && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.4)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 2100,
          }}
          onClick={() => setShowAgentPrompt(false)}
        >
          <div
            style={{
              background: "var(--kenga-bg)",
              padding: 20,
              borderRadius: 8,
              minWidth: 400,
              boxShadow: "0 4px 20px rgba(0,0,0,0.2)",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 style={{ margin: "0 0 12px 0", fontSize: 16 }}>Запустить агента</h3>
            <p style={{ margin: "0 0 12px 0", fontSize: 12, color: "var(--kenga-muted)" }}>
              Введите задачу для агента (создать файл, рефакторинг, исправить ошибки и т.п.)
            </p>
            <input
              type="text"
              value={agentPromptInput}
              onChange={(e) => setAgentPromptInput(e.target.value)}
              placeholder="Например: добавь функцию validate в main.rs"
              style={{ width: "100%", padding: 10, marginBottom: 16, boxSizing: "border-box" }}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  handleAgentRequest(agentPromptInput);
                  setShowAgentPrompt(false);
                }
                if (e.key === "Escape") setShowAgentPrompt(false);
              }}
              autoFocus
            />
            <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
              <button type="button" onClick={() => setShowAgentPrompt(false)}>
                Отмена
              </button>
              <button
                type="button"
                onClick={() => {
                  handleAgentRequest(agentPromptInput);
                  setShowAgentPrompt(false);
                }}
                disabled={!agentPromptInput.trim() || !projectPath}
                style={{ background: "var(--kenga-accent)", color: "#fff" }}
              >
                Запустить
              </button>
            </div>
          </div>
        </div>
      )}

      {showAddProviderModal && inTauri && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.4)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 2100,
          }}
          onClick={() => setShowAddProviderModal(false)}
        >
          <div
            style={{
              background: "var(--kenga-bg)",
              padding: 20,
              borderRadius: 8,
              minWidth: 400,
              boxShadow: "0 4px 20px rgba(0,0,0,0.2)",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 style={{ margin: "0 0 12px 0", fontSize: 16 }}>Добавить OpenAI провайдер</h3>
            <p style={{ margin: "0 0 12px 0", fontSize: 12, color: "var(--kenga-muted)" }}>
              Введите API key от OpenAI (sk-...). Ключ сохраняется в ~/.kengaide/ai_config.json
            </p>
            <input
              type="password"
              value={addProviderApiKey}
              onChange={(e) => {
                setAddProviderApiKey(e.target.value);
                setAddProviderError(null);
              }}
              placeholder="sk-..."
              style={{ width: "100%", padding: 10, marginBottom: 12, boxSizing: "border-box" }}
            />
            {addProviderError && (
              <div style={{ color: "#c62828", fontSize: 12, marginBottom: 12 }}>{addProviderError}</div>
            )}
            <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
              <button type="button" onClick={() => setShowAddProviderModal(false)}>
                Отмена
              </button>
              <button
                type="button"
                onClick={async () => {
                  const inv = invokeRef.current;
                  if (!inv) return;
                  setAddProviderError(null);
                  try {
                    await inv("add_openai_provider", { api_key: addProviderApiKey });
                    setShowAddProviderModal(false);
                    setAddProviderApiKey("");
                    setAiResponse((prev) => prev + "\nOpenAI провайдер добавлен.\n");
                  } catch (e) {
                    setAddProviderError(String(e));
                  }
                }}
                disabled={!addProviderApiKey.trim()}
                style={{ background: "var(--kenga-accent)", color: "#fff" }}
              >
                Добавить
              </button>
            </div>
          </div>
        </div>
      )}

      {showSwitchModelModal && inTauri && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.4)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 2100,
          }}
          onClick={() => setShowSwitchModelModal(false)}
        >
          <div
            style={{
              background: "var(--kenga-bg)",
              padding: 20,
              borderRadius: 8,
              minWidth: 360,
              maxHeight: 400,
              overflow: "auto",
              boxShadow: "0 4px 20px rgba(0,0,0,0.2)",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 style={{ margin: "0 0 12px 0", fontSize: 16 }}>Сменить провайдер</h3>
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
              {aiProviders.map((p) => (
                <div
                  key={p.id}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between",
                    padding: 12,
                    background: activeProviderId === p.id ? "var(--kenga-accent-bg)" : "var(--kenga-panel)",
                    border: "1px solid #ddd",
                    borderRadius: 4,
                  }}
                >
                  <button
                    type="button"
                    onClick={async () => {
                      const inv = invokeRef.current;
                      if (!inv) return;
                      try {
                        await inv("set_active_provider", { providerId: p.id });
                        setActiveProviderId(p.id);
                        setAiResponse((prev) => prev + `\nАктивный провайдер: ${p.name}\n`);
                      } catch {
                        /* ignore */
                      }
                    }}
                    style={{
                      flex: 1,
                      textAlign: "left",
                      background: "none",
                      border: "none",
                      cursor: "pointer",
                      padding: 0,
                    }}
                  >
                    <span style={{ fontWeight: 600 }}>{p.name}</span>
                    <span style={{ fontSize: 11, color: "var(--kenga-muted)", marginLeft: 8 }}>
                      {p.available ? "✓" : "—"}
                    </span>
                  </button>
                  {!p.available && (p.id.startsWith("local-")) && (
                    <button
                      type="button"
                      onClick={async () => {
                        const inv = invokeRef.current;
                        if (!inv) return;
                        try {
                          await inv("start_model_download_provider", { providerId: p.id });
                          loadAiProviders();
                        } catch (e) {
                          setAiResponse((prev) => prev + `\nОшибка загрузки: ${String(e)}\n`);
                        }
                      }}
                      style={{ fontSize: 11, padding: "4px 8px" }}
                    >
                      Загрузить
                    </button>
                  )}
                </div>
              ))}
            </div>
            <div style={{ marginTop: 16 }}>
              <button type="button" onClick={() => setShowSwitchModelModal(false)}>
                Закрыть
              </button>
            </div>
          </div>
        </div>
      )}

      {showAboutModal && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.4)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 3000,
          }}
          onClick={() => setShowAboutModal(false)}
        >
          <div
            style={{
              background: "var(--kenga-bg, #fff)",
              padding: 24,
              borderRadius: 8,
              minWidth: 320,
              boxShadow: "0 4px 20px rgba(0,0,0,0.2)",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 style={{ margin: "0 0 8px 0", fontSize: 18 }}>KengaIDE</h3>
            <p style={{ margin: 0, fontSize: 13, color: "var(--kenga-fg, #333)" }}>
              {appVersion ? `Версия ${appVersion.version}` : "IDE с AI-ассистентом"}
            </p>
            <p style={{ margin: "12px 0 0 0", fontSize: 12, color: "var(--kenga-muted)" }}>
              Tauri + React + Monaco + локальные и API-модели
            </p>
            <button type="button" onClick={() => setShowAboutModal(false)} style={{ marginTop: 16, padding: "8px 16px" }}>
              Закрыть
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
