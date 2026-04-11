import { createSignal, For, onCleanup, Show } from "solid-js";
import {
  parseModbusRtu,
  parseModbusAscii,
  type ModbusFrame,
} from "../lib/commands";
import {
  onSerialFrame,
  onBleFrame,
  type SerialFrameEvent,
  type BleFrameEvent,
} from "../lib/events";
import { activePortId } from "../stores/ui";
import { TrashIcon, DownloadIcon, FilterIcon } from "../assets/icons";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";

type ModbusMode = "RTU" | "ASCII";

interface ParsedEntry {
  timestamp: string;
  recordType: "read" | "write";
  frame: ModbusFrame;
  raw: number[];
}

function formatTime(timestamp: string): string {
  const parts = timestamp.split(" ");
  return parts.length > 1 ? parts[1] : timestamp;
}

function toHex2(n: number): string {
  return n.toString(16).padStart(2, "0").toUpperCase();
}

function dataToHexPreview(data: number[], limit = 10): string {
  const preview = data
    .slice(0, limit)
    .map((b) => toHex2(b))
    .join(" ");
  return data.length > limit ? `${preview} ...` : preview;
}

export default function ModbusTab() {
  const [modbusMode, setModbusMode] = createSignal<ModbusMode>("RTU");
  const [entries, setEntries] = createSignal<ParsedEntry[]>([]);

  const handleFrame = async (
    payload: SerialFrameEvent | BleFrameEvent,
    recordType: "read" | "write",
  ) => {
    const raw = payload.raw ?? [];
    if (raw.length === 0) return;
    try {
      const frame =
        modbusMode() === "RTU"
          ? await parseModbusRtu(raw)
          : await parseModbusAscii(raw);
      if (frame) {
        setEntries((prev) => [
          ...prev,
          { timestamp: payload.timestamp, recordType, frame, raw },
        ]);
      }
    } catch {
      // Not a valid Modbus frame, skip silently
    }
  };

  const setupSerialListener = async () => {
    const unlisten = await onSerialFrame(async (e: SerialFrameEvent) => {
      if (e.portId !== activePortId()) return;
      await handleFrame(e, e.recordType ?? "read");
    });
    onCleanup(() => unlisten());
  };

  const setupBleListener = async () => {
    const unlisten = await onBleFrame(async (e: BleFrameEvent) => {
      if (activePortId() !== "ble") return;
      await handleFrame(e, e.recordType ?? "read");
    });
    onCleanup(() => unlisten());
  };

  setupSerialListener();
  setupBleListener();

  const handleClear = () => setEntries([]);

  const handleExport = async () => {
    try {
      const rows = entries().map((e) => ({
        timestamp: e.timestamp,
        direction: e.recordType === "read" ? "RX" : "TX",
        slaveId: e.frame.slaveId,
        functionCode: `0x${toHex2(e.frame.functionCode)}`,
        functionName: e.frame.functionName,
        data: e.frame.data.map(toHex2).join(" "),
        crcValid: e.frame.crcValid,
        raw: e.raw.map(toHex2).join(" "),
      }));
      const text = JSON.stringify(rows, null, 2);
      const filePath = await save({
        title: "Export Modbus frames",
        defaultPath: "modbus_frames.json",
        filters: [
          { name: "JSON Files", extensions: ["json"] },
          { name: "All Files", extensions: ["*"] },
        ],
      });
      if (!filePath) return;
      await writeTextFile(filePath, text);
    } catch (e) {
      console.error("Failed to export Modbus frames:", e);
    }
  };

  return (
    <div class="flex flex-col h-full">
      {/* Toolbar */}
      <div class="flex items-center px-3 py-1.5 gap-2 border-b border-base-300 bg-base-200/50 text-xs">
        <span class="opacity-60">协议</span>

        <div class="join">
          <button
            class="join-item btn btn-xs"
            classList={{
              "btn-primary": modbusMode() === "RTU",
              "btn-ghost": modbusMode() !== "RTU",
            }}
            onClick={() => setModbusMode("RTU")}
          >
            RTU
          </button>
          <button
            class="join-item btn btn-xs"
            classList={{
              "btn-primary": modbusMode() === "ASCII",
              "btn-ghost": modbusMode() !== "ASCII",
            }}
            onClick={() => setModbusMode("ASCII")}
          >
            ASCII
          </button>
        </div>

        <span class="opacity-20 select-none">|</span>

        <button class="btn btn-ghost btn-xs gap-1" title="Filter (placeholder)">
          <img src={FilterIcon} class="w-3 h-3" alt="" />
        </button>

        <div class="flex-1" />

        <button
          class="btn btn-ghost btn-xs"
          title="Export frames"
          onClick={handleExport}
        >
          <img src={DownloadIcon} class="w-3 h-3" alt="" />
        </button>

        <button
          class="btn btn-ghost btn-xs text-error/70 hover:text-error"
          title="Clear frames"
          onClick={handleClear}
        >
          <img src={TrashIcon} class="w-3 h-3" alt="" />
        </button>
      </div>

      {/* Table area */}
      <div class="flex-1 overflow-auto custom-scrollbar">
        {/* Header row */}
        <div class="flex bg-base-200 border-b border-base-300 px-3 py-1.5 text-xs uppercase tracking-wider opacity-50 sticky top-0 z-10">
          <span class="w-[70px] shrink-0">时间</span>
          <span class="w-[30px] shrink-0">Dir</span>
          <span class="w-[50px] shrink-0">地址</span>
          <span class="w-[100px] shrink-0">功能码</span>
          <span class="flex-1 min-w-0">数据</span>
          <span class="w-[70px] shrink-0 text-right">校验</span>
        </div>

        <Show
          when={entries().length > 0}
          fallback={
            <div class="flex items-center justify-center h-32 opacity-40 text-sm select-none">
              暂无 Modbus 帧
            </div>
          }
        >
          <For each={entries()}>
            {(entry) => {
              const isException = entry.frame.functionCode >= 0x80;
              const isNormalRx =
                !isException &&
                entry.frame.crcValid &&
                entry.recordType === "read";

              return (
                <div
                  class="flex items-center px-3 py-1.5 border-b border-base-300/50 text-xs font-mono hover:bg-base-200/30"
                  classList={{
                    "bg-error/5 border-l-2 border-error": isException,
                    "bg-warning/5 border-l-2 border-warning":
                      !isException && !entry.frame.crcValid,
                    "bg-success/5":
                      isNormalRx && !isException && entry.frame.crcValid,
                  }}
                >
                  {/* Time */}
                  <span class="w-[70px] shrink-0 opacity-40 truncate">
                    {formatTime(entry.timestamp)}
                  </span>

                  {/* Direction */}
                  <span class="w-[30px] shrink-0">
                    <span
                      class="badge badge-xs"
                      classList={{
                        "badge-success": entry.recordType === "read",
                        "badge-info": entry.recordType === "write",
                      }}
                    >
                      {entry.recordType === "read" ? "RX" : "TX"}
                    </span>
                  </span>

                  {/* Slave ID */}
                  <span class="w-[50px] shrink-0">
                    0x{toHex2(entry.frame.slaveId)}
                  </span>

                  {/* Function code + name */}
                  <span
                    class="w-[100px] shrink-0 truncate"
                    title={entry.frame.functionName}
                  >
                    0x{toHex2(entry.frame.functionCode)}{" "}
                    <span class="opacity-60">{entry.frame.functionName}</span>
                  </span>

                  {/* Data bytes */}
                  <span class="flex-1 min-w-0 truncate opacity-80">
                    {dataToHexPreview(entry.frame.data)}
                  </span>

                  {/* CRC status */}
                  <span
                    class="w-[70px] shrink-0 text-right"
                    classList={{
                      "text-success": entry.frame.crcValid,
                      "text-warning": !entry.frame.crcValid,
                    }}
                  >
                    {entry.frame.crcValid ? "CRC OK" : "CRC ERR"}
                  </span>
                </div>
              );
            }}
          </For>
        </Show>
      </div>
    </div>
  );
}
