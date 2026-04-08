import { createSignal } from "solid-js";
import SerialSetting from "./SerialSetting";
import BleSetting from "./BleSetting";
import { BluetoothIcon, SerialIcon } from "../../assets/icons";

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
          <SerialIcon class="w-4 h-4" />
          串口
        </button>
        <button
          role="tab"
          class="tab gap-1"
          classList={{ "tab-active": tab() === "ble" }}
          onClick={() => setTab("ble")}
        >
          <BluetoothIcon class="w-4 h-4" />
          蓝牙
        </button>
      </div>
      <div class="mt-2">
        {tab() === "serial" ? <SerialSetting /> : <BleSetting />}
      </div>
    </div>
  );
}
