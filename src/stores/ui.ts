import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { setAppError } from "./serial";
import { createSignal } from "solid-js";
import { ENCODING_OPTIONS, type SendFormat } from "../lib/commands";

export type Theme = "light" | "dark";
export type DisplayMode = SendFormat;

const preferredTheme = (): Theme => {
  const saved = localStorage.getItem("serial-studio.theme") as Theme | null;
  if (saved === "light" || saved === "dark") return saved;
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
};

export const [theme, setTheme] = createSignal<Theme>("light");
export const [displayMode, setDisplayMode] = createSignal<DisplayMode>(
  ENCODING_OPTIONS.find(
    (option) => option.value === localStorage.getItem("serial-studio.display"),
  )?.value ?? "utf-8",
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
