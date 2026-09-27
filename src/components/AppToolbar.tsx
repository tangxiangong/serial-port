import { MoonIcon, SerialIcon, SunIcon } from "../assets/icons";
import { serialStatus } from "../stores/serial";
import { theme, toggleTheme } from "../stores/ui";

export default function AppToolbar() {
  return (
    <header class="app-toolbar" data-tauri-drag-region>
      <div class="app-identity" data-tauri-drag-region>
        <span class="app-mark" aria-hidden="true">
          <SerialIcon />
        </span>
        <div class="window-title" data-tauri-drag-region>
          <span class="app-title" data-tauri-drag-region>
            Serial Studio
          </span>
          <span class="app-subtitle" data-tauri-drag-region>
            串口调试工作台
          </span>
        </div>
      </div>
      <div class="toolbar-actions">
        <span class="connection-state">
          <span
            classList={{
              "connection-dot": true,
              connected: serialStatus().connected,
            }}
          />
          {serialStatus().connected ? "已连接" : "未连接"}
        </span>
        <button
          class="icon-button glass-control"
          title={theme() === "light" ? "切换到深色模式" : "切换到浅色模式"}
          aria-label={theme() === "light" ? "切换到深色模式" : "切换到浅色模式"}
          onClick={toggleTheme}
        >
          {theme() === "light" ? <MoonIcon /> : <SunIcon />}
        </button>
      </div>
    </header>
  );
}
