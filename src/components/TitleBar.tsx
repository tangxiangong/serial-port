import { Show } from "solid-js";
import {
  activePortId,
  bleConnected,
  deviceType,
  serialConnected,
  setShowSettingsDrawer,
  showSettingsDrawer,
  theme,
  toggleTheme,
} from "../stores/ui";
import { closeSerialPort, bleDisconnect } from "../lib/commands";
import { SettingsIcon, SunIcon, MoonIcon, UnplugIcon } from "../assets/icons";

export default function TitleBar() {
  const isConnected = () => serialConnected() || bleConnected();

  const handleDisconnect = async () => {
    try {
      if (deviceType() === "ble") {
        await bleDisconnect();
      } else if (activePortId()) {
        await closeSerialPort(activePortId());
      }
    } catch (e) {
      console.error("Disconnect failed:", e);
    }
  };

  return (
    <div class="flex items-center px-3 py-1.5 bg-base-300 border-b border-base-300 gap-3 h-10">
      {/* Logo */}
      <span class="font-bold text-sm text-primary tracking-wide select-none">
        SERIAL PORT
      </span>

      {/* Connection capsule */}
      <Show when={isConnected()}>
        <div class="flex items-center gap-2 bg-base-100 px-2.5 py-0.5 rounded-md border border-base-content/10 text-xs font-mono">
          <div class="w-1.5 h-1.5 rounded-full bg-success" />
          <span class="opacity-80">{activePortId() || "BLE"}</span>
        </div>
      </Show>

      <Show when={isConnected()}>
        <button
          class="btn btn-ghost btn-xs gap-1 text-error"
          onClick={handleDisconnect}
        >
          <UnplugIcon class="w-3.5 h-3.5" />
          断开
        </button>
      </Show>

      {/* Spacer */}
      <div class="flex-1" />

      {/* Settings gear */}
      <button
        class="btn btn-ghost btn-xs btn-square"
        classList={{ "btn-active": showSettingsDrawer() }}
        onClick={() => setShowSettingsDrawer(!showSettingsDrawer())}
        title="设置"
      >
        <SettingsIcon class="w-4 h-4" />
      </button>

      {/* Theme toggle */}
      <button
        class="btn btn-ghost btn-xs btn-square"
        onClick={toggleTheme}
        title="切换主题"
      >
        <Show when={theme() === "dark"} fallback={<MoonIcon class="w-4 h-4" />}>
          <SunIcon class="w-4 h-4" />
        </Show>
      </button>
    </div>
  );
}
