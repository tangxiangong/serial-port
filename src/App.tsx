import { onMount, Show, Switch, Match } from "solid-js";
import "./App.css";
import { initTheme, activeTab, showSettingsDrawer } from "./stores/ui";
import { initStore } from "./stores/persistent";
import TitleBar from "./components/TitleBar";
import TabBar from "./components/TabBar";
import MonitorTab from "./components/MonitorTab";
import SendTab from "./components/SendTab";
import QuickCommandTab from "./components/QuickCommandTab";
import BridgeTab from "./components/BridgeTab";
import ModbusTab from "./components/ModbusTab";
import SettingsDrawer from "./components/SettingsDrawer";
import StatusBar from "./components/StatusBar";

function App() {
  onMount(async () => {
    initTheme();
    await initStore();
  });

  return (
    <div class="h-screen w-full flex flex-col bg-base-100">
      <TitleBar />
      <TabBar />

      {/* Content area */}
      <div class="flex-1 overflow-hidden">
        <Switch>
          <Match when={activeTab() === "monitor"}>
            <MonitorTab />
          </Match>
          <Match when={activeTab() === "send"}>
            <SendTab />
          </Match>
          <Match when={activeTab() === "quick"}>
            <QuickCommandTab />
          </Match>
          <Match when={activeTab() === "bridge"}>
            <BridgeTab />
          </Match>
          <Match when={activeTab() === "modbus"}>
            <ModbusTab />
          </Match>
        </Switch>
      </div>

      <StatusBar />

      {/* Settings drawer overlay */}
      <Show when={showSettingsDrawer()}>
        <SettingsDrawer />
      </Show>
    </div>
  );
}

export default App;
