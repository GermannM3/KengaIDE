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
  { bg: string; fg: string; panel: string; border: string; accent: string }
> = {
  light: { bg: "#ffffff", fg: "#333333", panel: "#f5f5f5", border: "#e0e0e0", accent: "#1e88e5" },
  dark: { bg: "#1e1e1e", fg: "#d4d4d4", panel: "#252526", border: "#3c3c3c", accent: "#0e639c" },
  "high-contrast": { bg: "#000000", fg: "#ffffff", panel: "#1a1a1a", border: "#ffffff", accent: "#ffff00" },
};
