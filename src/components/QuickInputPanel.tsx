import { createSignal, For, Show } from "solid-js";
import { sendData, type SendFormat } from "../lib/commands";
import { activePortId, deviceType, serialConnected, bleConnected } from "../stores/ui";

interface QuickInputItem {
  id: number;
  enabled: boolean;
  content: string;
  remark: string;
  delay: number;
  isHex: boolean;
}

let nextId = 1;

const defaultItems: QuickInputItem[] = [
  { id: nextId++, enabled: true, content: "AT+RST", remark: "重启模块", delay: 1000, isHex: false },
  { id: nextId++, enabled: true, content: "AT+GMR", remark: "查询版本信息", delay: 1000, isHex: false },
  { id: nextId++, enabled: true, content: "AT+CWLAP", remark: "扫描WiFi热点", delay: 1000, isHex: false },
  { id: nextId++, enabled: false, content: "01 02 03 04", remark: "HEX测试数据", delay: 1000, isHex: true },
];

export default function QuickInputPanel() {
  const [items, setItems] = createSignal<QuickInputItem[]>(defaultItems);
  const [loopInterval, setLoopInterval] = createSignal(5000);
  const [isLoopSending, setIsLoopSending] = createSignal(false);
  let loopTimer: ReturnType<typeof setInterval> | undefined;

  const isConnected = () => serialConnected() || bleConnected();

  const addRow = () => {
    setItems((prev) => [
      ...prev,
      { id: nextId++, enabled: false, content: "", remark: "", delay: 1000, isHex: false },
    ]);
  };

  const deleteRow = (id: number) => {
    setItems((prev) => prev.filter((item) => item.id !== id));
  };

  const updateItem = (id: number, field: keyof QuickInputItem, value: any) => {
    setItems((prev) =>
      prev.map((item) => (item.id === id ? { ...item, [field]: value } : item))
    );
  };

  const enableAll = () => {
    setItems((prev) => prev.map((item) => ({ ...item, enabled: true })));
  };

  const disableAll = () => {
    setItems((prev) => prev.map((item) => ({ ...item, enabled: false })));
  };

  const clearAll = () => {
    setItems([]);
  };

  const sendItem = async (item: QuickInputItem) => {
    if (!item.content) return;
    try {
      const format: SendFormat = item.isHex ? "hex" : "ascii";
      await sendData(activePortId(), item.content, format, "None", "", deviceType());
    } catch (e) {
      console.error("Failed to send:", e);
    }
  };

  const sendAllEnabled = async () => {
    const enabled = items().filter((item) => item.enabled);
    for (const item of enabled) {
      if (!isLoopSending()) break;
      await sendItem(item);
      if (item.delay > 0) {
        await new Promise((r) => setTimeout(r, item.delay));
      }
    }
  };

  const startLoopSending = () => {
    if (isLoopSending()) return;
    const enabled = items().filter((item) => item.enabled);
    if (enabled.length === 0) return;
    setIsLoopSending(true);
    sendAllEnabled();
    loopTimer = setInterval(() => {
      if (isLoopSending()) {
        sendAllEnabled();
      } else {
        clearInterval(loopTimer);
        loopTimer = undefined;
      }
    }, loopInterval());
  };

  const stopLoopSending = () => {
    setIsLoopSending(false);
    if (loopTimer) {
      clearInterval(loopTimer);
      loopTimer = undefined;
    }
  };

  return (
    <div class="p-2 h-full flex flex-col">
      {/* Header */}
      <div class="flex justify-between items-center border-b border-base-300 pb-2 mb-3">
        <h3 class="font-semibold leading-none tracking-tight text-sm">快捷输入面板</h3>
        <div class="flex items-center gap-1">
          <button class="btn btn-ghost btn-xs btn-square" onClick={addRow} title="新增">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="12" x2="12" y1="5" y2="19" />
              <line x1="5" x2="19" y1="12" y2="12" />
            </svg>
          </button>
          <button class="btn btn-ghost btn-xs btn-square text-error" onClick={clearAll} title="清空">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 6h18" />
              <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
              <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
            </svg>
          </button>
          <button class="btn btn-ghost btn-xs btn-square" onClick={enableAll} title="全部启用">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="9 11 12 14 22 4" />
              <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
            </svg>
          </button>
          <button class="btn btn-ghost btn-xs btn-square" onClick={disableAll} title="全部禁用">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect width="18" height="18" x="3" y="3" rx="2" />
            </svg>
          </button>
        </div>
      </div>

      {/* Send controls */}
      <div class="grid grid-cols-3 gap-2 mb-3">
        <button
          class="btn btn-primary btn-xs"
          disabled={!isConnected() || isLoopSending()}
          onClick={() => { setIsLoopSending(true); sendAllEnabled().then(() => setIsLoopSending(false)); }}
        >
          发送选中
        </button>
        <button
          class="btn btn-xs"
          classList={{
            "btn-error": isLoopSending(),
            "btn-primary": !isLoopSending(),
          }}
          disabled={!isConnected()}
          onClick={() => (isLoopSending() ? stopLoopSending() : startLoopSending())}
        >
          {isLoopSending() ? "停止循环" : "循环发送"}
        </button>
        <div class="flex items-center justify-center gap-1">
          <span class="text-xs whitespace-nowrap">间隔:</span>
          <input
            type="number"
            class="input input-xs input-bordered w-16 text-center disable-spin-button"
            value={loopInterval()}
            onInput={(e) => setLoopInterval(parseInt(e.currentTarget.value) || 5000)}
            min="1000"
            step="1000"
            disabled={isLoopSending()}
          />
          <span class="text-xs">ms</span>
        </div>
      </div>

      {/* Table */}
      <div class="flex-1 overflow-auto border border-base-300 rounded-lg">
        <table class="table table-xs">
          <thead>
            <tr>
              <th class="w-10 text-center">启用</th>
              <th>内容</th>
              <th class="w-10 text-center">HEX</th>
              <th class="w-16 text-center">延时</th>
              <th class="w-24 text-center">操作</th>
            </tr>
          </thead>
          <tbody>
            <For each={items()} fallback={
              <tr>
                <td colspan="5" class="py-6 text-center opacity-50 text-sm">
                  暂无快捷输入项，点击 "新增" 按钮添加
                </td>
              </tr>
            }>
              {(item) => (
                <tr class="hover">
                  <td class="text-center">
                    <input
                      type="checkbox"
                      class="checkbox checkbox-xs checkbox-primary"
                      checked={item.enabled}
                      onChange={(e) => updateItem(item.id, "enabled", e.currentTarget.checked)}
                    />
                  </td>
                  <td>
                    <input
                      class="input input-xs input-bordered w-full"
                      value={item.content}
                      onInput={(e) => updateItem(item.id, "content", e.currentTarget.value)}
                      placeholder="请输入内容"
                    />
                    <Show when={item.remark}>
                      <div class="text-xs opacity-50 mt-0.5">{item.remark}</div>
                    </Show>
                  </td>
                  <td class="text-center">
                    <input
                      type="checkbox"
                      class="checkbox checkbox-xs checkbox-primary"
                      checked={item.isHex}
                      onChange={(e) => updateItem(item.id, "isHex", e.currentTarget.checked)}
                    />
                  </td>
                  <td>
                    <input
                      type="number"
                      class="input input-xs input-bordered w-full text-center disable-spin-button"
                      value={item.delay}
                      onInput={(e) => updateItem(item.id, "delay", parseInt(e.currentTarget.value) || 0)}
                      min="0"
                      step="100"
                    />
                  </td>
                  <td>
                    <div class="flex justify-center gap-1">
                      <button
                        class="btn btn-xs btn-outline"
                        disabled={!item.enabled}
                        onClick={() => sendItem(item)}
                      >
                        发送
                      </button>
                      <button
                        class="btn btn-xs btn-ghost text-error"
                        onClick={() => deleteRow(item.id)}
                      >
                        删除
                      </button>
                    </div>
                  </td>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </div>
    </div>
  );
}
