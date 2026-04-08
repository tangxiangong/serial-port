import { createSignal } from "solid-js";

// Panel visibility
export const [showSettingPanel, setShowSettingPanel] = createSignal(true);
export const [showSendPanel, setShowSendPanel] = createSignal(true);
export const [showQuickInputPanel, setShowQuickInputPanel] =
  createSignal(false);
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
export const [activePortId, setActivePortId] = createSignal<string>("default");

// Connection state
export const [serialConnected, setSerialConnected] = createSignal(false);
export const [bleConnected, setBleConnected] = createSignal(false);

// Bridge panel modal
export const [showBridgePanel, setShowBridgePanel] = createSignal(false);
