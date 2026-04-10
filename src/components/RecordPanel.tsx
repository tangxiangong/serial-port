import { createEffect, createSignal, For, onCleanup, Show } from "solid-js";
import {
  type BleFrameEvent,
  onBleFrame,
  onSerialFrame,
  type SerialFrameEvent,
} from "../lib/events";
import { activePortId, pinBottom, readDisplayFormat } from "../stores/ui";
import {
  getRecords,
  type ModbusFrame,
  parseModbusAscii,
  parseModbusRtu,
} from "../lib/commands";
import {
  createRecordItem,
  getRecordDisplayData,
  type RecordItem,
  recordItemFromFormattedRecord,
} from "../lib/record-utils";

type ModbusMode = "Off" | "RTU" | "ASCII";

function formatFunctionCode(fc: number, name: string): string {
  return `0x${fc.toString(16).padStart(2, "0")} – ${name}`;
}

function ModbusView(props: { frame: ModbusFrame }) {
  return (
    <div class="text-xs font-mono space-y-0.5">
      <div class="flex gap-3 flex-wrap">
        <span>
          <span class="opacity-60">Slave:</span>{" "}
          <span class="text-info">{props.frame.slaveId}</span>
        </span>
        <span>
          <span class="opacity-60">FC:</span>{" "}
          <span class="text-warning">
            {formatFunctionCode(
              props.frame.functionCode,
              props.frame.functionName,
            )}
          </span>
        </span>
        <span>
          <span class="opacity-60">Data:</span>{" "}
          <span>
            [
            {props.frame.data
              .map((b) => `0x${b.toString(16).padStart(2, "0")}`)
              .join(", ")}
            ]
          </span>
        </span>
        <span
          class="badge badge-xs"
          classList={{
            "badge-success": props.frame.crcValid,
            "badge-error": !props.frame.crcValid,
          }}
        >
          CRC {props.frame.crcValid ? "OK" : "FAIL"}
        </span>
      </div>
    </div>
  );
}

export default function RecordPanel() {
  const [records, setRecords] = createSignal<RecordItem[]>([]);
  const [showFullDate, setShowFullDate] = createSignal(false);
  const [modbusMode, setModbusMode] = createSignal<ModbusMode>("Off");
  let containerRef: HTMLDivElement | undefined;

  const scrollToBottom = () => {
    if (containerRef && pinBottom()) {
      requestAnimationFrame(() => {
        containerRef!.scrollTop = containerRef!.scrollHeight;
      });
    }
  };

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

  const appendRecord = async (
    payload: SerialFrameEvent | BleFrameEvent,
    recordType: "read" | "write",
  ) => {
    const modbus = modbusMode() !== "Off"
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

  const cycleDisplay = (index: number) => {
    const order = ["hex", "ascii", "dec"] as const;
    setRecords((prev) => {
      const updated = [...prev];
      const current = updated[index];
      const currentIdx = order.indexOf(current.display);
      updated[index] = {
        ...current,
        display: order[(currentIdx + 1) % order.length],
      };
      return updated;
    });
  };

  const formatTime = (timestamp: string) => {
    if (showFullDate()) return timestamp;
    const parts = timestamp.split(" ");
    return parts.length > 1 ? parts[1] : timestamp;
  };

  const clearRecords = () => setRecords([]);

  createEffect(() => {
    (window as any).__clearRecordPanel = clearRecords;
  });

  createEffect(() => {
    const nextDisplay = readDisplayFormat();
    setRecords((prev) =>
      prev.map((record) =>
        record.recordType === "read" ? { ...record, display: nextDisplay } : record
      ),
    );
  });

  createEffect(() => {
    const portId = activePortId();
    if (!portId) {
      setRecords([]);
      return;
    }

    void getRecords(portId)
      .then((items) =>
        setRecords(items.map((item) => recordItemFromFormattedRecord(item, readDisplayFormat()))))
      .catch((error) => {
        console.error("Failed to load records:", error);
        setRecords([]);
      });
  });

  const modbusModes: ModbusMode[] = ["Off", "RTU", "ASCII"];

  const cycleModbusMode = () => {
    setModbusMode((prev) => {
      const idx = modbusModes.indexOf(prev);
      return modbusModes[(idx + 1) % modbusModes.length];
    });
  };

  return (
    <div class="flex flex-col h-full">
      <div class="flex items-center gap-2 px-3 py-1 border-b border-base-300 bg-base-200/50">
        <span class="text-xs opacity-60">Modbus:</span>
        <button
          class="badge badge-sm badge-outline cursor-pointer"
          classList={{
            "badge-ghost": modbusMode() === "Off",
            "badge-success": modbusMode() === "RTU",
            "badge-info": modbusMode() === "ASCII",
          }}
          onClick={cycleModbusMode}
        >
          {modbusMode()}
        </button>
        <Show when={modbusMode() !== "Off"}>
          <span class="text-xs opacity-50 ml-1">(解析失败时显示原始数据)</span>
        </Show>
      </div>

      <div
        ref={containerRef}
        class="flex-1 overflow-y-auto scroll-smooth p-2 relative pb-10 custom-scrollbar"
      >
        <Show
          when={records().length > 0}
          fallback={
            <div class="flex items-center justify-center h-full opacity-50 text-sm">
              暂无收发记录
            </div>
          }
        >
          <For each={records()}>
            {(record, index) => (
              <div
                class="chat"
                classList={{
                  "chat-start": record.recordType === "read",
                  "chat-end": record.recordType === "write",
                }}
              >
                <div class="chat-header mx-2 flex gap-4">
                  <div
                    class="text-sm opacity-70 cursor-pointer hover:opacity-100 transition-opacity"
                    onClick={() => setShowFullDate(!showFullDate())}
                  >
                    {formatTime(record.timestamp)}
                  </div>
                  <div
                    class="text-sm cursor-pointer hover:text-primary"
                    onClick={() => cycleDisplay(index())}
                  >
                    {record.display.toUpperCase()}
                  </div>
                  <div class="text-xs opacity-50 uppercase">
                    {record.recordType}
                  </div>
                </div>
                <div class="chat-bubble wrap-break-word text-sm font-mono">
                  <Show
                    when={modbusMode() !== "Off" &&
                      record.modbus != null &&
                      record.modbus !== undefined}
                    fallback={
                      <Show
                        when={modbusMode() !== "Off" && record.modbus === null}
                        fallback={<span>{getRecordDisplayData(record)}</span>}
                      >
                        <span class="opacity-40 italic text-xs mr-1">
                          [Modbus解析失败]
                        </span>
                        <span>{getRecordDisplayData(record)}</span>
                      </Show>
                    }
                  >
                    <ModbusView frame={record.modbus!} />
                  </Show>
                </div>
              </div>
            )}
          </For>
        </Show>
      </div>
    </div>
  );
}
