import type { InvokeFn } from "../types";

export async function getInvoke(): Promise<InvokeFn | null> {
  try {
    const core = await import("@tauri-apps/api/core");
    if (typeof core.invoke !== "function") return null;
    return ((cmd: string, args?: unknown) =>
      core.invoke(cmd, args as Record<string, unknown>)) as InvokeFn;
  } catch {
    return null;
  }
}
