import { createResource, createSignal, For, onCleanup, Show } from "solid-js";
import { getOpenPorts, startBridge, stopBridge } from "../lib/commands";
import { onBridgeData } from "../lib/events";
import {
  getBridgeDirectionLabel,
  partitionBridgeEntries,
} from "../lib/bridge-utils";
import {
  ArrowLeftRightIcon,
  RefreshIcon,
  PlayIcon,
  StopIcon,
  TrashIcon,
} from "../assets/icons";

interface BridgeEntry {
  direction: string;
  data: string;
  timestamp: string;
}

export default function BridgeTab() {
  const [portA, setPortA] = createSignal("");
  const [portB, setPortB] = createSignal("");
  const [bridgeActive, setBridgeActive] = createSignal(false);
  const [entries, setEntries] = createSignal<BridgeEntry[]>([]);
  let cleanupBridge: (() => void) | undefined;

  const [openPorts, { refetch }] = createResource<string[]>(async () => {
    try {
      return await getOpenPorts();
    } catch {
      return [];
    }
  });

  const aToB = () => partitionBridgeEntries(entries()).aToB;
  const bToA = () => partitionBridgeEntries(entries()).bToA;

  const handleToggleBridge = async () => {
    if (bridgeActive()) {
      try {
        await stopBridge();
      } catch (e) {
        console.error(e);
      }
      cleanupBridge?.();
      cleanupBridge = undefined;
      setBridgeActive(false);
    } else {
      if (!portA() || !portB() || portA() === portB()) return;
      try {
        await startBridge(portA(), portB());
        const unlisten = await onBridgeData((e) => {
          setEntries((prev) => [
            ...prev.slice(-499),
            { ...e, direction: getBridgeDirectionLabel(e.direction) },
          ]);
        });
        cleanupBridge = unlisten;
        setBridgeActive(true);
      } catch (e) {
        console.error("Failed to start bridge:", e);
      }
    }
  };

  onCleanup(() => {
    cleanupBridge?.();
  });

  return (
    <div class="flex flex-col h-full">
      {/* Sub-toolbar */}
      <div class="flex items-center px-3 py-1.5 gap-3 border-b border-base-300 bg-base-200/50 text-xs">
        <span class="opacity-60">端口 A</span>
        <select
          class="select select-xs select-bordered min-w-[140px]"
          value={portA()}
          onChange={(e) => setPortA(e.currentTarget.value)}
          disabled={bridgeActive()}
        >
          <option value="">-- 选择端口 --</option>
          <For each={openPorts() ?? []}>
            {(p) => <option value={p}>{p}</option>}
          </For>
        </select>

        <ArrowLeftRightIcon class="w-4 h-4 opacity-50" />

        <span class="opacity-60">端口 B</span>
        <select
          class="select select-xs select-bordered min-w-[140px]"
          value={portB()}
          onChange={(e) => setPortB(e.currentTarget.value)}
          disabled={bridgeActive()}
        >
          <option value="">-- 选择端口 --</option>
          <For each={openPorts() ?? []}>
            {(p) => <option value={p}>{p}</option>}
          </For>
        </select>

        <button
          class="btn btn-ghost btn-xs"
          onClick={() => refetch()}
          title="刷新端口列表"
        >
          <RefreshIcon class="w-3.5 h-3.5" />
        </button>

        <div class="flex-1" />

        <button
          class="btn btn-xs"
          classList={{
            "btn-success": !bridgeActive(),
            "btn-error": bridgeActive(),
            "btn-disabled": !bridgeActive() && (!portA() || !portB() || portA() === portB()),
          }}
          onClick={handleToggleBridge}
        >
          <Show
            when={bridgeActive()}
            fallback={<PlayIcon class="w-3.5 h-3.5" />}
          >
            <StopIcon class="w-3.5 h-3.5" />
          </Show>
          {bridgeActive() ? "停止桥接" : "开始桥接"}
        </button>

        <button
          class="btn btn-ghost btn-xs"
          onClick={() => setEntries([])}
          title="清空日志"
        >
          <TrashIcon class="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Dual-pane traffic display */}
      <div class="flex flex-1 min-h-0 overflow-hidden">
        {/* A → B pane */}
        <div class="flex-1 flex flex-col border-r border-base-300">
          <div class="px-3 py-1 text-xs font-semibold text-success border-b border-base-300 bg-base-200/30">
            A → B
            <span class="opacity-50 font-normal ml-2">({aToB().length})</span>
          </div>
          <div class="flex-1 overflow-y-auto p-2 custom-scrollbar font-mono text-xs">
            <Show
              when={aToB().length > 0}
              fallback={
                <div class="opacity-50 text-center mt-4">暂无数据</div>
              }
            >
              <For each={aToB()}>
                {(entry) => (
                  <div class="mb-1">
                    <span class="opacity-50 mr-2">{entry.timestamp}</span>
                    <span>{entry.data}</span>
                  </div>
                )}
              </For>
            </Show>
          </div>
        </div>

        {/* B → A pane */}
        <div class="flex-1 flex flex-col">
          <div class="px-3 py-1 text-xs font-semibold text-info border-b border-base-300 bg-base-200/30">
            B → A
            <span class="opacity-50 font-normal ml-2">({bToA().length})</span>
          </div>
          <div class="flex-1 overflow-y-auto p-2 custom-scrollbar font-mono text-xs">
            <Show
              when={bToA().length > 0}
              fallback={
                <div class="opacity-50 text-center mt-4">暂无数据</div>
              }
            >
              <For each={bToA()}>
                {(entry) => (
                  <div class="mb-1">
                    <span class="opacity-50 mr-2">{entry.timestamp}</span>
                    <span>{entry.data}</span>
                  </div>
                )}
              </For>
            </Show>
          </div>
        </div>
      </div>
    </div>
  );
}
