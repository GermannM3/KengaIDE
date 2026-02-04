/** Безопасный invoke: в браузере (без Tauri) не падаем, а возвращаем ошибку. */
export type InvokeFn = (cmd: string, args?: unknown) => Promise<unknown>;

export type AiRequestType = "chat" | "explain" | "refactor" | "generate" | "agent";

export interface AiRequestPayload {
  request: {
    type: AiRequestType;
    message?: string;
    path?: string;
    selection?: string;
    instruction?: string;
    prompt?: string;
  };
  current_file_path?: string;
  current_file_content?: string;
  selection?: string;
}

export type LocalModelStatus = "not_available" | "not_loaded" | "ready";

export interface DownloadProgress {
  bytes_done: number;
  bytes_total: number;
  file_index: number;
  file_count: number;
}

export type AiChunkPayload =
  | { request_id: string; type: "start" }
  | { request_id: string; type: "token"; value: string }
  | { request_id: string; type: "end" }
  | { request_id: string; type: "error"; error: string };

export interface ProjectTreeNode {
  name: string;
  path: string;
  kind: string;
  children?: ProjectTreeNode[];
}

export interface CommandItem {
  id: string;
  label: string;
  keywords: string[];
}

export type AgentProgressPayload =
  | { request_id: string; kind: "session_started"; session_id: string }
  | { request_id: string; kind: "model_selected"; role: string; model_id: string }
  | { request_id: string; kind: "thinking" }
  | { request_id: string; kind: "tool_call"; name: string; path?: string }
  | { request_id: string; kind: "tool_result"; success: boolean; output: string }
  | { request_id: string; kind: "patch_apply_started"; path: string }
  | { request_id: string; kind: "patch_apply_success"; path: string }
  | { request_id: string; kind: "patch_apply_error"; path: string; message: string }
  | { request_id: string; kind: "patch_applied"; path: string; before: string; after: string }
  | { request_id: string; kind: "done"; message: string };

export type ThemeId = "light" | "dark" | "high-contrast";
