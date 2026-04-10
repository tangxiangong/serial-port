import { createSignal } from "solid-js";
import type { DisplayFormat } from "../lib/record-utils";

// Panel visibility
export const [showSettingPanel, setShowSettingPanel] = createSignal(true);
export const [showSendPanel, setShowSendPanel] = createSignal(true);
export const [showQuickInputPanel, setShowQuickInputPanel] = createSignal(
  false,
);
export const [showFullScreen, setShowFullScreen] = createSignal(true);

// Theme
export const [theme, setTheme] = createSignal<"light" | "dark">("dark");

export function toggleTheme() {
  const newTheme = theme() === "dark" ? "light" : "dark";
  setTheme(newTheme);
  document.documentElement.setAttribute("data-theme", newTheme);
}

export function initTheme() {
  document.documentElement.setAttribute("data-theme", "dark");
  setTheme("dark");
}

// Device type
export const [deviceType, setDeviceType] = createSignal<"serial" | "ble">(
  "serial",
);

// Active port (for multi-port support)
export const [activePortId, setActivePortId] = createSignal<string>("");
export const [connectedSerialPorts, setConnectedSerialPorts] = createSignal<
  string[]
>([]);

// Connection state
export const [serialConnected, setSerialConnected] = createSignal(false);
export const [bleConnected, setBleConnected] = createSignal(false);

// Bridge panel modal
export const [showBridgePanel, setShowBridgePanel] = createSignal(false);

// Shared display controls
export const [readDisplayFormat, setReadDisplayFormat] = createSignal<
  DisplayFormat
>("hex");
export const [sendDisplayFormat, setSendDisplayFormat] = createSignal<
  DisplayFormat
>("hex");
export const [pinBottom, setPinBottom] = createSignal(true);
