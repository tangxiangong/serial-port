import { createEffect, createSignal, For, onCleanup, Show } from "solid-js";
import {
  type BleFrameEvent,
  onBleFrame,
  onSerialFrame,
  type SerialFrameEvent,
} from "../lib/events";
import {
  activePortId,
  deviceType,
  pinBottom,
  readDisplayFormat,
  setReadDisplayFormat,
  setPinBottom,
} from "../stores/ui";
import {
  clearRecords,
  exportRecords,
  getRecords,
  type LogFormat,
  type ModbusFrame,
  parseModbusAscii,
  parseModbusRtu,
  startFileLogging,
  stopFileLogging,
} from "../lib/commands";
import {
  createRecordItem,
  getRecordDisplayData,
  type RecordItem,
  recordItemFromFormattedRecord,
} from "../lib/record-utils";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import {
  ArrowDownIcon,
  DownloadIcon,
  FileTextIcon,
  TrashIcon,
} from "../assets/icons";

type ModbusMode = "Off" | "RTU" | "ASCII";
type DisplayFormat = "hex" | "ascii" | "dec";

const modbusModes: ModbusMode[] = ["Off", "RTU", "ASCII"];
const logFormats: LogFormat[] = ["raw", "hex", "csv"];
const displayFormats: DisplayFormat[] = ["hex", "ascii", "dec"];

export default function MonitorTab() {
  const [records, setRecords] = createSignal<RecordItem[]>([]);
  const [modbusMode, setModbusMode] = createSignal<ModbusMode>("Off");
  const [isLogging, setIsLogging] = createSignal(false);
  const [logFormat, setLogFormat] = createSignal<LogFormat>("hex");
  let containerRef: HTMLDivElement | undefined;

  // ── Scroll ──────────────────────────────────────────────────────────────

  const scrollToBottom = () => {
    if (containerRef && pinBottom()) {
      requestAnimationFrame(() => {
        containerRef!.scrollTop = containerRef!.scrollHeight;
      });
    }
  };

  // ── Modbus parsing ───────────────────────────────────────────────────────

  const tryParseModbus = async (raw: number[]): Promise<ModbusFrame | null> => {
    const mode = modbusMode();
    if (mode === "Off" || raw.length === 0) return undefined as any;
    try {
      if (mode === "RTU") return await parseModbusRtu(raw);
      if (mode === "ASCII") return await parseModbusAscii(raw);
    } catch {
      return null;
    }
    return null;
  };

  // ── Record append ────────────────────────────────────────────────────────

  const appendRecord = async (
    payload: SerialFrameEvent | BleFrameEvent,
    recordType: "read" | "write",
  ) => {
    const modbus =
      modbusMode() !== "Off"
        ? await tryParseModbus(payload.raw ?? [])
        : undefined;

    setRecords((prev) => [
      ...prev,
      createRecordItem(
        {
          hex: payload.hex,
          ascii: payload.ascii,
          dec: payload.dec,
          raw: payload.raw ?? [],
          timestamp: payload.timestamp,
        },
        recordType,
        recordType === "write" ? "hex" : readDisplayFormat(),
        modbus,
      ),
    ]);
    scrollToBottom();
  };

  // ── Event listeners ──────────────────────────────────────────────────────

  const setupSerialListener = async () => {
    const unlisten = await onSerialFrame(async (e: SerialFrameEvent) => {
      if (e.portId !== activePortId()) return;
      await appendRecord(e, e.recordType ?? "read");
    });
    onCleanup(() => unlisten());
  };

  const setupBleListener = async () => {
    const unlisten = await onBleFrame(async (e: BleFrameEvent) => {
      if (activePortId() !== "ble") return;
      await appendRecord(e, e.recordType ?? "read");
    });
    onCleanup(() => unlisten());
  };

  setupSerialListener();
  setupBleListener();

  // ── Effects ──────────────────────────────────────────────────────────────

  // Sync readDisplayFormat changes to existing RX records
  createEffect(() => {
    const nextDisplay = readDisplayFormat();
    setRecords((prev) =>
      prev.map((record) =>
        record.recordType === "read" ? { ...record, display: nextDisplay } : record,
      ),
    );
  });

  // Load records when active port changes
  createEffect(() => {
    const portId = activePortId();
    if (!portId) {
      setRecords([]);
      return;
    }

    void getRecords(portId)
      .then((items) =>
        setRecords(
          items.map((item) => recordItemFromFormattedRecord(item, readDisplayFormat())),
        ),
      )
      .catch((error) => {
        console.error("Failed to load records:", error);
        setRecords([]);
      });
  });

  // Expose clear for external callers (ControlPanel compat)
  createEffect(() => {
    (window as any).__clearRecordPanel = () => setRecords([]);
  });

  // ── Toolbar actions ──────────────────────────────────────────────────────

  const handleClearRecords = async () => {
    try {
      await clearRecords(activePortId());
      setRecords([]);
    } catch (e) {
      console.error("Failed to clear records:", e);
    }
  };

  const handleExport = async () => {
    try {
      const data = await exportRecords(activePortId());
      const suffix = deviceType() === "ble" ? "ble" : "serial";
      const filePath = await save({
        title: "Export session records",
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
          title: "Choose log file location",
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

  const cycleModbusMode = () => {
    setModbusMode((prev) => {
      const idx = modbusModes.indexOf(prev);
      return modbusModes[(idx + 1) % modbusModes.length];
    });
  };

  const cycleLogFormat = () => {
    if (isLogging()) return;
    setLogFormat((prev) => {
      const idx = logFormats.indexOf(prev);
      return logFormats[(idx + 1) % logFormats.length];
    });
  };

  // ── Annotation helpers ───────────────────────────────────────────────────

  const getAnnotation = (record: RecordItem): string => {
    if (record.modbus != null) {
      const m = record.modbus!;
      const variant = modbusMode() === "RTU" ? "RTU" : "ASCII";
      const crcLabel = modbusMode() === "RTU" ? "CRC" : "LRC";
      return `Modbus ${variant} · Addr:${m.slaveId.toString().padStart(2, "0")} Func:${m.functionCode.toString(16).padStart(2, "0").toUpperCase()} ${crcLabel}:${m.crcValid ? "OK" : "FAIL"}`;
    }
    // In hex mode, show ASCII preview as annotation for RX frames
    if (record.recordType === "read" && readDisplayFormat() === "hex" && record.ascii) {
      const printable = record.ascii.replace(/[^\x20-\x7e]/g, "·");
      return printable.length > 0 ? printable : "";
    }
    return "";
  };

  // ── Render ───────────────────────────────────────────────────────────────

  return (
    <div class="flex flex-col h-full">
      {/* Toolbar */}
      <div class="flex items-center px-3 py-1.5 border-b border-base-300 bg-base-200/50 gap-2 text-xs flex-wrap">
        {/* Format button group */}
        <div class="join">
          <For each={displayFormats}>
            {(fmt) => (
              <button
                class="join-item btn btn-xs"
                classList={{
                  "btn-primary": readDisplayFormat() === fmt,
                  "btn-ghost": readDisplayFormat() !== fmt,
                }}
                onClick={() => setReadDisplayFormat(fmt)}
              >
                {fmt.toUpperCase()}
              </button>
            )}
          </For>
        </div>

        <span class="opacity-20 select-none">|</span>

        {/* Auto-scroll toggle */}
        <button
          class="btn btn-xs gap-1"
          classList={{
            "btn-primary": pinBottom(),
            "btn-ghost opacity-50": !pinBottom(),
          }}
          title="Auto-scroll"
          onClick={() => setPinBottom(!pinBottom())}
        >
          <img src={ArrowDownIcon} class="w-3 h-3" alt="" />
          <span class="hidden sm:inline">Auto-scroll</span>
        </button>

        <span class="opacity-20 select-none">|</span>

        {/* Modbus mode badge */}
        <button
          class="badge badge-sm badge-outline cursor-pointer select-none"
          classList={{
            "badge-ghost": modbusMode() === "Off",
            "badge-success": modbusMode() === "RTU",
            "badge-info": modbusMode() === "ASCII",
          }}
          onClick={cycleModbusMode}
          title="Cycle Modbus mode: Off / RTU / ASCII"
        >
          Modbus: {modbusMode()}
        </button>

        <div class="flex-1" />

        {/* Log format badge */}
        <button
          class="badge badge-sm badge-outline cursor-pointer select-none"
          classList={{
            "opacity-40 pointer-events-none": isLogging(),
            "badge-warning": logFormat() === "raw",
            "badge-info": logFormat() === "hex",
            "badge-success": logFormat() === "csv",
          }}
          onClick={cycleLogFormat}
          title="Cycle log format: raw / hex / csv"
        >
          {logFormat().toUpperCase()}
        </button>

        {/* Record / Stop logging */}
        <button
          class="btn btn-xs gap-1"
          classList={{
            "btn-error": isLogging(),
            "btn-ghost": !isLogging(),
          }}
          title={isLogging() ? "Stop logging" : "Start logging to file"}
          onClick={handleToggleLogging}
        >
          <img src={FileTextIcon} class="w-3 h-3" alt="" />
          <span
            class="inline-block w-1.5 h-1.5 rounded-full"
            classList={{
              "bg-error animate-pulse": isLogging(),
              "bg-base-content/40": !isLogging(),
            }}
          />
        </button>

        {/* Export */}
        <button
          class="btn btn-xs btn-ghost"
          title="Export records"
          onClick={handleExport}
        >
          <img src={DownloadIcon} class="w-3 h-3" alt="" />
        </button>

        {/* Clear */}
        <button
          class="btn btn-xs btn-ghost text-error/70 hover:text-error"
          title="Clear records"
          onClick={handleClearRecords}
        >
          <img src={TrashIcon} class="w-3 h-3" alt="" />
        </button>
      </div>

      {/* Data stream */}
      <div
        ref={containerRef}
        class="flex-1 overflow-y-auto custom-scrollbar p-1 font-mono text-xs"
      >
        <Show
          when={records().length > 0}
          fallback={
            <div class="flex items-center justify-center h-full opacity-40 text-sm select-none">
              No data yet
            </div>
          }
        >
          <For each={records()}>
            {(record) => {
              const annotation = getAnnotation(record);
              const hasModbus = record.modbus != null;

              return (
                <div
                  class="flex items-start gap-3 px-3 py-0.5 hover:bg-base-200/30 rounded"
                  classList={{
                    "border-l-2 border-warning bg-warning/5": hasModbus,
                  }}
                >
                  {/* Timestamp */}
                  <span class="opacity-40 min-w-[75px] shrink-0 tabular-nums">
                    {(() => {
                      const parts = record.timestamp.split(" ");
                      return parts.length > 1 ? parts[1] : record.timestamp;
                    })()}
                  </span>

                  {/* Direction badge */}
                  <span
                    class="badge badge-xs shrink-0 mt-0.5"
                    classList={{
                      "badge-success": record.recordType === "read",
                      "badge-info": record.recordType === "write",
                    }}
                  >
                    {record.recordType === "read" ? "RX" : "TX"}
                  </span>

                  {/* Data */}
                  <span class="flex-1 break-all tracking-wide leading-relaxed">
                    <Show
                      when={hasModbus && record.modbus !== null}
                      fallback={
                        <Show
                          when={modbusMode() !== "Off" && record.modbus === null}
                          fallback={<>{getRecordDisplayData(record)}</>}
                        >
                          <span class="opacity-40 italic mr-1">[parse failed]</span>
                          {getRecordDisplayData(record)}
                        </Show>
                      }
                    >
                      {getRecordDisplayData(record)}
                    </Show>
                  </span>

                  {/* Annotation */}
                  <Show when={annotation.length > 0}>
                    <span class="opacity-30 text-xs shrink-0 max-w-[200px] truncate mt-0.5">
                      {annotation}
                    </span>
                  </Show>
                </div>
              );
            }}
          </For>
        </Show>
      </div>
    </div>
  );
}
