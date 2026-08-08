import { MoonIcon, SerialIcon, SunIcon, TrashIcon } from "../assets/icons";
import { clearFrames, frames, serialStatus } from "../stores/serial";
import {
  chooseDisplayMode,
  displayMode,
  theme,
  toggleTheme,
  type DisplayMode,
} from "../stores/ui";

const displayOptions: Array<{ value: DisplayMode; label: string }> = [
  { value: "text", label: "文本" },
  { value: "hex", label: "HEX" },
  { value: "both", label: "组合" },
];

export default function AppToolbar() {
  return (
    <header class="app-toolbar" data-tauri-drag-region>
      <div class="app-identity" data-tauri-drag-region>
        <span class="app-mark">
          <SerialIcon />
        </span>
        <span class="app-title">Serial Studio</span>
        <span
          classList={{
            "connection-dot": true,
            connected: serialStatus().connected,
          }}
          aria-label={serialStatus().connected ? "已连接" : "未连接"}
        />
      </div>

      <div class="toolbar-actions">
        <div class="segmented-control compact" aria-label="数据显示格式">
          {displayOptions.map((option) => (
            <button
              classList={{ active: displayMode() === option.value }}
              onClick={() => chooseDisplayMode(option.value)}
            >
              {option.label}
            </button>
          ))}
        </div>
        <button
          class="icon-button"
          title="清空终端"
          disabled={frames().length === 0}
          onClick={clearFrames}
        >
          <TrashIcon />
        </button>
        <span class="toolbar-divider" />
        <button
          class="icon-button"
          title={theme() === "light" ? "切换到深色模式" : "切换到浅色模式"}
          onClick={toggleTheme}
        >
          {theme() === "light" ? <MoonIcon /> : <SunIcon />}
        </button>
      </div>
    </header>
  );
}
