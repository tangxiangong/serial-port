import { onMount, Show } from "solid-js";
import "./App.css";
import { initTheme } from "./stores/ui";
import { initStore } from "./stores/persistent";
import {
  showSettingPanel,
  showSendPanel,
  showQuickInputPanel,
  showFullScreen,
  showBridgePanel,
  setShowBridgePanel,
} from "./stores/ui";
import ActivityBar from "./components/ActivityBar";
import TopBar from "./components/TopBar";
import StatusBar from "./components/StatusBar";
import DeviceSetting from "./components/DeviceSetting";
import RecordPanel from "./components/RecordPanel";
import ControlPanel from "./components/ControlPanel";
import SendPanel from "./components/SendPanel";
import QuickInputPanel from "./components/QuickInputPanel";
import BridgePanel from "./components/BridgePanel";

function App() {
  onMount(async () => {
    initTheme();
    await initStore();
  });

  return (
    <div class="flex justify-center items-center">
      <div
        class="h-screen aspect-video flex flex-row relative"
        classList={{
          "w-full": showFullScreen(),
          "container 2xl:mx-56 py-[40px] lg:py-[100px]": !showFullScreen(),
        }}
      >
        {/* ActivityBar */}
        <div class="rounded-l-lg rounded-r-none bg-base-200 border border-base-300 flex flex-col h-full">
          <ActivityBar />
        </div>

        {/* Main content */}
        <div class="w-full h-full flex flex-col">
          {/* TopBar */}
          <div
            class="bg-base-200 flex items-center justify-end px-3 py-1 gap-2"
            classList={{
              "border-b border-base-300": showFullScreen(),
              "border-t border-base-300": !showFullScreen(),
            }}
          >
            <TopBar />
          </div>

          {/* Panels area */}
          <div
            class="flex-1 flex overflow-hidden"
            classList={{
              "border-t border-r border-b border-base-300": !showFullScreen(),
            }}
          >
            {/* Setting panel (left) */}
            <Show when={showSettingPanel()}>
              <div class="h-full bg-base-200 w-[250px] min-w-[200px] overflow-y-auto border-r border-base-300 custom-scrollbar">
                <DeviceSetting />
              </div>
            </Show>

            {/* Main area (center) */}
            <div class="flex-1 flex flex-col overflow-hidden">
              {/* Record panel */}
              <div class="flex-7 overflow-y-auto bg-base-100 custom-scrollbar">
                <RecordPanel />
              </div>

              {/* Control + Send panel */}
              <Show when={showSendPanel()}>
                <div class="flex-3 flex flex-col bg-base-100 border-t border-base-300">
                  <div class="border-b border-base-300">
                    <ControlPanel />
                  </div>
                  <div class="flex-1">
                    <SendPanel />
                  </div>
                </div>
              </Show>
            </div>
          </div>

          {/* StatusBar */}
          <StatusBar />
        </div>

        {/* Quick Input Panel (right) */}
        <Show when={showQuickInputPanel()}>
          <div
            class="h-full bg-base-200 max-w-[600px] w-full border-l border-base-300"
            classList={{
              "rounded-r-lg border border-base-300": !showFullScreen(),
            }}
          >
            <QuickInputPanel />
          </div>
        </Show>

        {/* Bridge Panel modal overlay */}
        <Show when={showBridgePanel()}>
          <div
            class="absolute inset-0 z-40 bg-black/50 backdrop-blur-sm"
            onClick={() => setShowBridgePanel(false)}
          />
          <div class="absolute inset-x-8 inset-y-12 z-50 flex flex-col bg-base-100 border border-base-300 rounded-lg shadow-2xl overflow-hidden">
            <div class="flex items-center justify-between px-4 py-2 border-b border-base-300 bg-base-200">
              <span class="text-sm font-semibold">串口桥接</span>
              <button
                class="btn btn-ghost btn-xs"
                onClick={() => setShowBridgePanel(false)}
              >
                ✕
              </button>
            </div>
            <div class="flex-1 min-h-0">
              <BridgePanel />
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
}

export default App;
