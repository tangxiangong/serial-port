import { createResource, createSignal, For, onCleanup, Show } from "solid-js";
import { getOpenPorts, startBridge, stopBridge } from "../lib/commands";
import { onBridgeData } from "../lib/events";

interface BridgeEntry {
  direction: string;
  data: string;
  timestamp: string;
}

export default function BridgePanel() {
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

  const aToB = () => entries().filter((e) => e.direction === "A→B");
  const bToA = () => entries().filter((e) => e.direction === "B→A");

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
          setEntries((prev) => [...prev.slice(-499), e]);
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
    <div class="flex flex-col h-full min-h-0">
      {/* Header / controls */}
      <div class="flex items-center gap-3 px-4 py-2 border-b border-base-300 bg-base-200/60 flex-wrap">
        <span class="text-sm font-medium">串口桥接</span>

        {/* Port A selector */}
        <div class="flex items-center gap-1.5">
          <span class="text-xs opacity-60">端口 A:</span>
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
        </div>

        <span class="opacity-50 text-sm">⇌</span>

        {/* Port B selector */}
        <div class="flex items-center gap-1.5">
          <span class="text-xs opacity-60">端口 B:</span>
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
        </div>

        {/* Refresh ports */}
        <button class="btn btn-ghost btn-xs" onClick={() => refetch()}>
          刷新
        </button>

        <div class="flex-1" />

        {/* Start / Stop */}
        <button
          class="btn btn-xs"
          classList={{
            "btn-error": bridgeActive(),
            "btn-success": !bridgeActive(),
            "btn-disabled": !portA() || !portB() || portA() === portB(),
          }}
          onClick={handleToggleBridge}
        >
          <span
            class="inline-block w-2 h-2 rounded-full"
            classList={{
              "bg-success-content": !bridgeActive(),
              "bg-error-content animate-pulse": bridgeActive(),
            }}
          />
          {bridgeActive() ? "停止桥接" : "开始桥接"}
        </button>

        {/* Clear log */}
        <button class="btn btn-ghost btn-xs" onClick={() => setEntries([])}>
          清空
        </button>
      </div>

      {/* Dual-column traffic display */}
      <div class="flex-1 flex min-h-0 overflow-hidden">
        {/* A → B */}
        <div class="flex-1 flex flex-col border-r border-base-300 min-w-0">
          <div class="px-3 py-1 text-xs font-semibold text-success border-b border-base-300 bg-base-200/30">
            A → B
            <span class="opacity-50 font-normal ml-2">({aToB().length})</span>
          </div>
          <div class="flex-1 overflow-y-auto p-2 custom-scrollbar">
            <Show
              when={aToB().length > 0}
              fallback={
                <div class="opacity-50 text-xs text-center mt-4">暂无数据</div>
              }
            >
              <For each={aToB()}>
                {(entry) => (
                  <div class="mb-1">
                    <span class="opacity-50 text-xs mr-2">
                      {entry.timestamp}
                    </span>
                    <span class="font-mono text-xs">{entry.data}</span>
                  </div>
                )}
              </For>
            </Show>
          </div>
        </div>

        {/* B → A */}
        <div class="flex-1 flex flex-col min-w-0">
          <div class="px-3 py-1 text-xs font-semibold text-info border-b border-base-300 bg-base-200/30">
            B → A
            <span class="opacity-50 font-normal ml-2">({bToA().length})</span>
          </div>
          <div class="flex-1 overflow-y-auto p-2 custom-scrollbar">
            <Show
              when={bToA().length > 0}
              fallback={
                <div class="opacity-50 text-xs text-center mt-4">暂无数据</div>
              }
            >
              <For each={bToA()}>
                {(entry) => (
                  <div class="mb-1">
                    <span class="opacity-50 text-xs mr-2">
                      {entry.timestamp}
                    </span>
                    <span class="font-mono text-xs">{entry.data}</span>
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
