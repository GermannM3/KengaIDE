export const INVOKE_ERR = "Запустите приложение KengaIDE для работы с AI.";

export const DEFAULT_LEFT_PANEL_WIDTH = 220;
export const DEFAULT_RIGHT_PANEL_WIDTH = 320;
export const MIN_LEFT_PANEL = 160;
export const MAX_LEFT_PANEL = 500;
export const MIN_RIGHT_PANEL = 260;
export const MAX_RIGHT_PANEL = 600;
export const RESIZER_WIDTH = 4;

export const STORAGE_LEFT = "kengaide_left_panel_width";
export const STORAGE_RIGHT = "kengaide_right_panel_width";
export const STORAGE_WELCOME_DISMISSED = "kengaide_welcome_dismissed";
export const STORAGE_THEME = "kengaide_theme";

export const THEME_MONACO: Record<"light" | "dark" | "high-contrast", string> = {
  light: "vs",
  dark: "vs-dark",
  "high-contrast": "hc-black",
};

export const THEME_CSS: Record<
  "light" | "dark" | "high-contrast",
  {
    bg: string;
    fg: string;
    panel: string;
    panelElevated: string;
    sidebarBg: string;
    border: string;
    accent: string;
    muted: string;
    accentBg: string;
    borderSubtle: string;
    tabActive: string;
    tabInactive: string;
    error: string;
    success: string;
  }
> = {
  light: {
    bg: "#ffffff",
    fg: "#333333",
    panel: "#f3f3f3",
    panelElevated: "#ffffff",
    sidebarBg: "#f3f3f3",
    border: "#e5e5e5",
    accent: "#0078d4",
    muted: "#6e6e6e",
    accentBg: "rgba(0, 120, 212, 0.1)",
    borderSubtle: "#e8e8e8",
    tabActive: "#ffffff",
    tabInactive: "transparent",
    error: "#c62828",
    success: "#2e7d32",
  },
  dark: {
    bg: "#1e1e1e",
    fg: "#cccccc",
    panel: "#252526",
    panelElevated: "#2d2d30",
    sidebarBg: "#252526",
    border: "#3c3c3c",
    accent: "#0a84ff",
    muted: "#858585",
    accentBg: "rgba(10, 132, 255, 0.25)",
    borderSubtle: "#3c3c3c",
    tabActive: "#1e1e1e",
    tabInactive: "transparent",
    error: "#f14c4c",
    success: "#89d185",
  },
  "high-contrast": {
    bg: "#000000",
    fg: "#ffffff",
    panel: "#1c1c1e",
    panelElevated: "#2d2d30",
    sidebarBg: "#1c1c1e",
    border: "#ffffff",
    accent: "#64d2ff",
    muted: "#e5e5ea",
    accentBg: "rgba(100, 210, 255, 0.2)",
    borderSubtle: "#666666",
    tabActive: "#000000",
    tabInactive: "transparent",
    error: "#ff6b6b",
    success: "#4ec9b0",
  },
};
