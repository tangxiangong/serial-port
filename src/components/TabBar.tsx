import { For } from "solid-js";
import { activeTab, setActiveTab, type TabId } from "../stores/ui";
import {
  MonitorIcon,
  SendIcon,
  ZapIcon,
  LinkIcon,
  CpuIcon,
} from "../assets/icons";
import type { Component } from "solid-js";

const tabs: { id: TabId; label: string; icon: Component<{ class?: string }> }[] = [
  { id: "monitor", label: "监控", icon: MonitorIcon },
  { id: "send", label: "发送", icon: SendIcon },
  { id: "quick", label: "快捷指令", icon: ZapIcon },
  { id: "bridge", label: "桥接", icon: LinkIcon },
  { id: "modbus", label: "Modbus", icon: CpuIcon },
];

export default function TabBar() {
  return (
    <div class="flex items-end bg-base-200 px-3 border-b border-base-300">
      <For each={tabs}>
        {(tab) => (
          <button
            class="flex items-center gap-1.5 px-4 py-2 text-xs font-medium transition-colors border-b-2 -mb-px"
            classList={{
              "border-primary text-primary": activeTab() === tab.id,
              "border-transparent text-base-content/50 hover:text-base-content/70": activeTab() !== tab.id,
            }}
            onClick={() => setActiveTab(tab.id)}
          >
            <tab.icon class="w-3.5 h-3.5" />
            {tab.label}
          </button>
        )}
      </For>
    </div>
  );
}
