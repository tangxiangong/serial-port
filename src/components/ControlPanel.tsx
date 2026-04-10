import { createSignal } from "solid-js";
import {
  activePortId,
  pinBottom,
  readDisplayFormat,
  sendDisplayFormat,
  setPinBottom,
  setReadDisplayFormat,
  setSendDisplayFormat,
  setShowBridgePanel,
  showBridgePanel,
} from "../stores/ui";
import {
  clearRecords,
  exportRecords,
  type LogFormat,
  startFileLogging,
  stopFileLogging,
} from "../lib/commands";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { deviceType } from "../stores/ui";

type DisplayFormat = "hex" | "ascii" | "dec";

export default function ControlPanel() {
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
      const data = await exportRecords(activePortId());
      const suffix = deviceType() === "ble" ? "ble" : "serial";
      const filePath = await save({
        title: "导出会话记录",
        defaultPath: `${suffix}_records.json`,
        filters: [
          { name: "JSON Files", extensions: ["json"] },
          { name: "All Files", extensions: ["*"] },
        ],
      });
      if (!filePath) return;
      await writeTextFile(filePath, data);
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
        onClick={() =>
          setReadDisplayFormat(cycleType(readDisplayFormat() as DisplayFormat))}
      >
        接收:{readDisplayFormat().toUpperCase()}
      </button>

      {/* Send format toggle */}
      <button
        class="badge badge-sm badge-outline badge-secondary cursor-pointer"
        onClick={() =>
          setSendDisplayFormat(cycleType(sendDisplayFormat() as DisplayFormat))}
      >
        发送:{sendDisplayFormat().toUpperCase()}
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
        清空记录
      </button>
    </div>
  );
}
