import { createSignal, onCleanup, onMount, Show } from "solid-js";
import {
  activePortId,
  bleConnected,
  connectedSerialPorts,
  deviceType,
  serialConnected,
} from "../stores/ui";
import { getRecordCounts } from "../lib/commands";
import { onSerialStats, type StatsEvent } from "../lib/events";

function formatBytesPerSec(bps: number): string {
  if (bps >= 1024) return `${(bps / 1024).toFixed(1)} KB/s`;
  return `${bps.toFixed(0)} B/s`;
}

export default function StatusBar() {
  const [txCount, setTxCount] = createSignal(0);
  const [rxCount, setRxCount] = createSignal(0);
  const [stats, setStats] = createSignal<StatsEvent | null>(null);

  const isConnected = () => serialConnected() || bleConnected();

  let timer: ReturnType<typeof setInterval>;

  onMount(async () => {
    timer = setInterval(async () => {
      try {
        const counts = await getRecordCounts(activePortId());
        setTxCount(counts.tx);
        setRxCount(counts.rx);
      } catch {
        // Port may not exist yet
      }
    }, 1000);

    const unlisten = await onSerialStats((e: StatsEvent) => {
      if (e.portId !== activePortId()) return;
      setStats(e);
    });
    onCleanup(() => unlisten());
  });

  onCleanup(() => clearInterval(timer));

  return (
    <div class="bg-base-200 border-t border-base-300 flex justify-between items-center h-6 px-3">
      <div class="flex h-full items-center gap-2">
        <div
          class="badge badge-sm gap-1"
          classList={{
            "badge-success": isConnected(),
            "badge-info": !isConnected(),
          }}
        >
          {isConnected() ? "设备已连接" : "等待连接"}
        </div>
        <Show when={isConnected()}>
          <div class="badge badge-sm badge-outline">
            {deviceType() === "ble" ? "蓝牙" : "串口"}
          </div>
          <Show when={deviceType() === "ble"}>
            <div class="text-xs opacity-70">BLE 会话</div>
          </Show>
          <Show when={deviceType() === "serial"}>
            <div class="text-xs opacity-70">
              {activePortId() || `${connectedSerialPorts().length} 个串口`}
            </div>
          </Show>
        </Show>
      </div>

      <div class="flex h-full items-center gap-3 text-xs opacity-70">
        <span>Tx: {txCount()} Bytes</span>
        <span>Rx: {rxCount()} Bytes</span>

        <Show when={stats() !== null}>
          <span class="opacity-40">|</span>
          <span>
            ↑ {formatBytesPerSec(stats()!.txBytesPerSec)}
            <span class="opacity-50 ml-1">
              {stats()!.txFramesPerSec.toFixed(0)} fps
            </span>
          </span>
          <span>
            ↓ {formatBytesPerSec(stats()!.rxBytesPerSec)}
            <span class="opacity-50 ml-1">
              {stats()!.rxFramesPerSec.toFixed(0)} fps
            </span>
          </span>
          <Show when={stats()!.errorCount > 0}>
            <span class="text-error font-semibold">
              ERR: {stats()!.errorCount}
            </span>
          </Show>
        </Show>
      </div>
    </div>
  );
}
