import { createSignal } from "solid-js";
import {
  activePortId,
  showBridgePanel,
  setShowBridgePanel,
} from "../stores/ui";
import {
  clearRecords,
  exportRecords,
  startFileLogging,
  stopFileLogging,
  type LogFormat,
} from "../lib/commands";
import { save } from "@tauri-apps/plugin-dialog";

type DisplayFormat = "hex" | "ascii" | "dec";

export default function ControlPanel() {
  const [readType, setReadType] = createSignal<DisplayFormat>("hex");
  const [sendType, setSendType] = createSignal<DisplayFormat>("hex");
  const [pinBottom, setPinBottom] = createSignal(true);
  const [isLogging, setIsLogging] = createSignal(false);
  const [logFormat, setLogFormat] = createSignal<LogFormat>("hex");

  const cycleType = (current: DisplayFormat): DisplayFormat => {
    const order: DisplayFormat[] = ["hex", "ascii", "dec"];
    return order[(order.indexOf(current) + 1) % order.length];
  };

  const cycleLogFormat = (current: LogFormat): LogFormat => {
    const order: LogFormat[] = ["raw", "hex", "csv"];
    return order[(order.indexOf(current) + 1) % order.length];
  };

  const handleClearRecords = async () => {
    try {
      await clearRecords(activePortId());
      (window as any).__clearRecordPanel?.();
    } catch (e) {
      console.error("Failed to clear records:", e);
    }
  };

  const handleExport = async () => {
    try {
      await exportRecords(activePortId());
    } catch (e) {
      console.error("Failed to export records:", e);
    }
  };

  const handleToggleLogging = async () => {
    if (isLogging()) {
      try {
        await stopFileLogging(activePortId());
      } catch (e) {
        console.error("Failed to stop file logging:", e);
      }
      setIsLogging(false);
    } else {
      try {
        const ext = logFormat() === "csv" ? "csv" : "txt";
        const filePath = await save({
          title: "选择录制文件保存位置",
          defaultPath: `serial_log.${ext}`,
          filters: [
            { name: "Log Files", extensions: [ext, "log"] },
            { name: "All Files", extensions: ["*"] },
          ],
        });
        if (!filePath) return;
        await startFileLogging(activePortId(), filePath, logFormat());
        setIsLogging(true);
      } catch (e) {
        console.error("Failed to start file logging:", e);
      }
    }
  };

  return (
    <div class="flex items-center px-3 space-x-2 py-2 flex-wrap gap-y-1">
      {/* Read format toggle */}
      <button
        class="badge badge-sm badge-outline badge-info cursor-pointer"
        onClick={() => setReadType(cycleType(readType()))}
      >
        接收:{readType().toUpperCase()}
      </button>

      {/* Send format toggle */}
      <button
        class="badge badge-sm badge-outline badge-secondary cursor-pointer"
        onClick={() => setSendType(cycleType(sendType()))}
      >
        发送:{sendType().toUpperCase()}
      </button>

      {/* Pin bottom toggle */}
      <button
        class="badge badge-sm badge-outline badge-ghost cursor-pointer"
        onClick={() => setPinBottom(!pinBottom())}
      >
        {pinBottom() ? "自动滚动" : "锁定滚动"}
      </button>

      {/* Export */}
      <button
        class="badge badge-sm badge-outline badge-ghost cursor-pointer"
        onClick={handleExport}
      >
        导出
      </button>

      {/* Log format cycle */}
      <button
        class="badge badge-sm badge-outline cursor-pointer"
        classList={{
          "opacity-40 pointer-events-none": isLogging(),
          "badge-warning": logFormat() === "raw",
          "badge-info": logFormat() === "hex",
          "badge-success": logFormat() === "csv",
        }}
        onClick={() => setLogFormat(cycleLogFormat(logFormat()))}
      >
        {logFormat().toUpperCase()}
      </button>

      {/* Record / Stop recording toggle */}
      <button
        class="badge badge-sm badge-outline cursor-pointer"
        classList={{
          "badge-error": isLogging(),
          "badge-ghost": !isLogging(),
        }}
        onClick={handleToggleLogging}
      >
        <span
          class="inline-block w-2 h-2 rounded-full"
          classList={{
            "bg-error animate-pulse": isLogging(),
            "bg-base-content/40": !isLogging(),
          }}
        />
        {isLogging() ? "停止录制" : "录制"}
      </button>

      {/* Bridge button */}
      <button
        class="badge badge-sm badge-outline cursor-pointer"
        classList={{
          "badge-secondary": showBridgePanel(),
          "badge-ghost": !showBridgePanel(),
        }}
        onClick={() => setShowBridgePanel(!showBridgePanel())}
      >
        桥接
      </button>

      <div class="flex-1" />

      {/* Clear records */}
      <button
        class="badge badge-sm badge-outline badge-ghost cursor-pointer"
        onClick={handleClearRecords}
      >
        清空接收
      </button>
    </div>
  );
}
