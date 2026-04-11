import { createSignal, For } from "solid-js";
import {
  sendData,
  type SendFormat,
  type CheckAlgorithm,
} from "../lib/commands";
import {
  activePortId,
  bleConnected,
  deviceType,
  serialConnected,
} from "../stores/ui";
import {
  CheckSquareIcon,
  PlayIcon,
  PlusIcon,
  SquareIcon,
  StopIcon,
  TrashIcon,
} from "../assets/icons";

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
  {
    id: nextId++,
    enabled: true,
    content: "AT+RST",
    remark: "重启模块",
    delay: 1000,
    isHex: false,
  },
  {
    id: nextId++,
    enabled: true,
    content: "AT+GMR",
    remark: "查询版本信息",
    delay: 1000,
    isHex: false,
  },
  {
    id: nextId++,
    enabled: true,
    content: "AT+CWLAP",
    remark: "扫描WiFi热点",
    delay: 1000,
    isHex: false,
  },
  {
    id: nextId++,
    enabled: false,
    content: "01 02 03 04",
    remark: "HEX测试数据",
    delay: 1000,
    isHex: true,
  },
];

export default function QuickCommandTab() {
  const [items, setItems] = createSignal<QuickInputItem[]>(defaultItems);
  const [loopInterval, setLoopInterval] = createSignal(5000);
  const [isLoopSending, setIsLoopSending] = createSignal(false);
  const [currentLoopId, setCurrentLoopId] = createSignal<number | undefined>(
    undefined,
  );
  let loopTimer: ReturnType<typeof setInterval> | undefined;

  const isConnected = () => serialConnected() || bleConnected();

  const addRow = () => {
    setItems((prev) => [
      ...prev,
      {
        id: nextId++,
        enabled: false,
        content: "",
        remark: "",
        delay: 1000,
        isHex: false,
      },
    ]);
  };

  const deleteRow = (id: number) => {
    setItems((prev) => prev.filter((item) => item.id !== id));
  };

  const updateItem = (id: number, field: keyof QuickInputItem, value: any) => {
    setItems((prev) =>
      prev.map((item) => (item.id === id ? { ...item, [field]: value } : item)),
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
      await sendData(
        activePortId(),
        item.content,
        format,
        "None",
        "none" as CheckAlgorithm,
        deviceType(),
      );
    } catch (e) {
      console.error("Failed to send:", e);
    }
  };

  const sendAllEnabled = async () => {
    const enabled = items().filter((item) => item.enabled);
    for (const item of enabled) {
      if (!isLoopSending()) break;
      setCurrentLoopId(item.id);
      await sendItem(item);
      if (item.delay > 0) {
        await new Promise((r) => setTimeout(r, item.delay));
      }
    }
    setCurrentLoopId(undefined);
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
    setCurrentLoopId(undefined);
    if (loopTimer) {
      clearInterval(loopTimer);
      loopTimer = undefined;
    }
  };

  return (
    <div class="flex flex-col h-full">
      {/* Toolbar */}
      <div class="flex items-center px-3 py-1.5 gap-2 border-b border-base-300 bg-base-200/50 text-xs">
        <button
          class="btn btn-ghost btn-xs gap-1"
          onClick={addRow}
          title="新建命令"
        >
          <PlusIcon class="w-3.5 h-3.5" />
          新建
        </button>
        <span class="opacity-30">|</span>
        <button
          class="btn btn-ghost btn-xs"
          onClick={enableAll}
          title="全部启用"
        >
          <CheckSquareIcon class="w-3.5 h-3.5" />
        </button>
        <button
          class="btn btn-ghost btn-xs"
          onClick={disableAll}
          title="全部禁用"
        >
          <SquareIcon class="w-3.5 h-3.5" />
        </button>
        <button
          class="btn btn-ghost btn-xs text-error"
          onClick={clearAll}
          title="清空所有"
        >
          <TrashIcon class="w-3.5 h-3.5" />
        </button>
        {/* Spacer */}
        <div class="flex-1" />
        {/* Loop controls */}
        <input
          type="number"
          class="input input-xs input-bordered w-16 text-center disable-spin-button"
          value={loopInterval()}
          onInput={(e) =>
            setLoopInterval(parseInt(e.currentTarget.value) || 5000)
          }
          min="100"
          step="100"
          disabled={isLoopSending()}
          title="循环间隔"
        />
        <span class="opacity-60">ms</span>
        <button
          class="btn btn-ghost btn-xs"
          classList={{
            "text-error": isLoopSending(),
          }}
          disabled={!isConnected()}
          onClick={() =>
            isLoopSending() ? stopLoopSending() : startLoopSending()
          }
          title={isLoopSending() ? "停止循环" : "开始循环"}
        >
          {isLoopSending() ? (
            <StopIcon class="w-3.5 h-3.5" />
          ) : (
            <PlayIcon class="w-3.5 h-3.5" />
          )}
        </button>
      </div>

      {/* Table */}
      <div class="flex-1 overflow-auto">
        <table class="table table-xs w-full">
          <thead class="sticky top-0 bg-base-200 z-10">
            <tr>
              <th class="w-10 text-center">启用</th>
              <th>内容</th>
              <th class="w-12 text-center">HEX</th>
              <th class="w-20 text-center">延时(ms)</th>
              <th class="w-24 text-center">操作</th>
            </tr>
          </thead>
          <tbody>
            <For
              each={items()}
              fallback={
                <tr>
                  <td colspan="5" class="py-8 text-center opacity-50 text-sm">
                    暂无快捷命令，点击 "新建" 按钮添加
                  </td>
                </tr>
              }
            >
              {(item) => (
                <tr
                  class="hover"
                  classList={{
                    "border-l-2 border-primary bg-primary/5":
                      currentLoopId() === item.id,
                  }}
                >
                  <td class="text-center">
                    <input
                      type="checkbox"
                      class="checkbox checkbox-xs checkbox-primary"
                      checked={item.enabled}
                      onChange={(e) =>
                        updateItem(item.id, "enabled", e.currentTarget.checked)
                      }
                    />
                  </td>
                  <td>
                    <input
                      class="input input-xs input-bordered w-full"
                      value={item.content}
                      onInput={(e) =>
                        updateItem(item.id, "content", e.currentTarget.value)
                      }
                      placeholder={item.remark || "请输入内容"}
                    />
                  </td>
                  <td class="text-center">
                    <input
                      type="checkbox"
                      class="checkbox checkbox-xs checkbox-primary"
                      checked={item.isHex}
                      onChange={(e) =>
                        updateItem(item.id, "isHex", e.currentTarget.checked)
                      }
                    />
                  </td>
                  <td>
                    <input
                      type="number"
                      class="input input-xs input-bordered w-full text-center disable-spin-button"
                      value={item.delay}
                      onInput={(e) =>
                        updateItem(
                          item.id,
                          "delay",
                          parseInt(e.currentTarget.value) || 0,
                        )
                      }
                      min="0"
                      step="100"
                    />
                  </td>
                  <td>
                    <div class="flex justify-center gap-1">
                      <button
                        class="btn btn-xs btn-outline"
                        disabled={!isConnected() || !item.content}
                        onClick={() => sendItem(item)}
                        title="发送"
                      >
                        发送
                      </button>
                      <button
                        class="btn btn-xs btn-ghost text-error"
                        onClick={() => deleteRow(item.id)}
                        title="删除"
                      >
                        <TrashIcon class="w-3 h-3" />
                      </button>
                    </div>
                  </td>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </div>

      {/* Bottom bar */}
      <div class="flex items-center px-3 py-2 gap-2 border-t border-base-300">
        <button
          class="btn btn-primary btn-sm"
          disabled={!isConnected() || isLoopSending()}
          onClick={() => {
            setIsLoopSending(true);
            sendAllEnabled().then(() => setIsLoopSending(false));
          }}
        >
          发送选中
        </button>
        <button
          class="btn btn-sm"
          classList={{
            "btn-error": isLoopSending(),
            "btn-outline": !isLoopSending(),
          }}
          disabled={!isConnected()}
          onClick={() =>
            isLoopSending() ? stopLoopSending() : startLoopSending()
          }
        >
          {isLoopSending() ? "停止循环" : "循环发送"}
        </button>
      </div>
    </div>
  );
}
