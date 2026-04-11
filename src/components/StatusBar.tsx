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
    <div class="bg-base-200 border-t border-base-300 flex items-center h-6 px-3 text-xs">
      {/* Left: connection indicator */}
      <div class="flex items-center gap-2">
        <div
          class="w-1.5 h-1.5 rounded-full"
          classList={{
            "bg-success": isConnected(),
            "bg-base-content/30": !isConnected(),
          }}
        />
        <Show
          when={isConnected()}
          fallback={<span class="opacity-40">未连接</span>}
        >
          <span class="opacity-70">
            {deviceType() === "ble"
              ? "BLE"
              : activePortId() ||
                `${connectedSerialPorts().length} 个串口`}
          </span>
        </Show>
      </div>

      {/* Center: TX/RX counts */}
      <div class="flex items-center gap-3 ml-6 opacity-60">
        <span>TX: {txCount()} B</span>
        <span>RX: {rxCount()} B</span>

        <Show when={stats() !== null}>
          <span class="opacity-40">|</span>
          <span>
            ↑ {formatBytesPerSec(stats()!.txBytesPerSec)}
          </span>
          <span>
            ↓ {formatBytesPerSec(stats()!.rxBytesPerSec)}
          </span>
          <Show when={stats()!.errorCount > 0}>
            <span class="text-error font-semibold">
              ERR: {stats()!.errorCount}
            </span>
          </Show>
        </Show>
      </div>

      {/* Right: detailed params */}
      <div class="ml-auto opacity-40">
        <Show when={isConnected() && deviceType() === "serial"}>
          <span>{activePortId()}</span>
        </Show>
      </div>
    </div>
  );
}
