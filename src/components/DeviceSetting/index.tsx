import { createSignal } from "solid-js";
import SerialSetting from "./SerialSetting";
import BleSetting from "./BleSetting";

export default function DeviceSetting() {
  const [tab, setTab] = createSignal<"serial" | "ble">("serial");

  return (
    <div class="w-full p-2">
      <div role="tablist" class="tabs tabs-box">
        <button
          role="tab"
          class="tab gap-1"
          classList={{ "tab-active": tab() === "serial" }}
          onClick={() => setTab("serial")}
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24">
            <path fill="currentColor" d="M7 3h10v2h2v3h-3v6H8V8H5V5h2zm10 6h2v5h-2zm-6 6h2v7h-2zM5 9h2v5H5z" />
          </svg>
          串口
        </button>
        <button
          role="tab"
          class="tab gap-1"
          classList={{ "tab-active": tab() === "ble" }}
          onClick={() => setTab("ble")}
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24">
            <path
              fill="currentColor"
              d="M14.88 16.29L13 18.17v-3.76m0-8.58l1.88 1.88L13 9.58m4.71-1.87L12 2h-1v7.58L6.41 5L5 6.41L10.59 12L5 17.58L6.41 19L11 14.41V22h1l5.71-5.71l-4.3-4.29z"
            />
          </svg>
          蓝牙
        </button>
      </div>
      <div class="mt-2">
        {tab() === "serial" ? <SerialSetting /> : <BleSetting />}
      </div>
    </div>
  );
}
