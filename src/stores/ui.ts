import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { setAppError } from "./serial";
import { createSignal } from "solid-js";

export type Theme = "light" | "dark";
export type DisplayMode = "text" | "hex" | "both";

const preferredTheme = (): Theme => {
  const saved = localStorage.getItem("serial-studio.theme") as Theme | null;
  if (saved === "light" || saved === "dark") return saved;
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
};

export const [theme, setTheme] = createSignal<Theme>("light");
export const [displayMode, setDisplayMode] = createSignal<DisplayMode>(
  (localStorage.getItem("serial-studio.display") as DisplayMode) || "both",
);
export const [autoScroll, setAutoScroll] = createSignal(true);

export function initAppearance() {
  applyTheme(preferredTheme());
}

export function toggleTheme() {
  applyTheme(theme() === "light" ? "dark" : "light");
}

export function chooseDisplayMode(mode: DisplayMode) {
  setDisplayMode(mode);
  localStorage.setItem("serial-studio.display", mode);
}

function applyTheme(nextTheme: Theme) {
  if (isTauri()) {
    void getCurrentWindow()
      .setTheme(nextTheme)
      .catch(() => {
        setAppError("窗口外观同步失败，请重新启动应用。");
      });
  }
  setTheme(nextTheme);
  document.documentElement.dataset.theme = nextTheme;
  localStorage.setItem("serial-studio.theme", nextTheme);
}
