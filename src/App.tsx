import { isTauri } from "@tauri-apps/api/core";
import { onCleanup, onMount, Show } from "solid-js";
import "./App.css";
import { XIcon } from "./assets/icons";
import AppToolbar from "./components/AppToolbar";
import Composer from "./components/Composer";
import ConnectionSidebar from "./components/ConnectionSidebar";
import StatusBar from "./components/StatusBar";
import Terminal from "./components/Terminal";
import {
  appError,
  dismissError,
  disposeSerial,
  initializeSerial,
} from "./stores/serial";
import { initAppearance } from "./stores/ui";

function App() {
  onMount(() => {
    if (isTauri() && /Mac/.test(navigator.platform)) {
      document.documentElement.dataset.nativeGlass = "true";
    }
    initAppearance();
    void initializeSerial();
  });
  onCleanup(disposeSerial);

  return (
    <div class="app-shell">
      <AppToolbar />
      <main class="workspace">
        <ConnectionSidebar />
        <div class="terminal-workspace">
          <Terminal />
          <Composer />
        </div>
      </main>
      <StatusBar />

      <Show when={appError()}>
        <div class="error-toast" role="alert">
          <span class="error-dot">!</span>
          <span>{appError()}</span>
          <button title="关闭" aria-label="关闭错误提示" onClick={dismissError}>
            <XIcon />
          </button>
        </div>
      </Show>
    </div>
  );
}

export default App;
