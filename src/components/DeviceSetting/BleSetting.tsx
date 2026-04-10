import { createSignal, For, Show } from "solid-js";
import {
  bleConnect,
  type BleDeviceInfo,
  bleDisconnect,
  bleScan,
  type BleServiceConfig,
} from "../../lib/commands";
import {
  activePortId,
  bleConnected,
  connectedSerialPorts,
  setActivePortId,
  setBleConnected,
  setDeviceType,
} from "../../stores/ui";

export default function BleSetting() {
  const [serviceUuid, setServiceUuid] = createSignal("");
  const [notifyUuid, setNotifyUuid] = createSignal("");
  const [writeUuid, setWriteUuid] = createSignal("");
  const [devices, setDevices] = createSignal<BleDeviceInfo[]>([]);
  const [selectedDevice, setSelectedDevice] = createSignal<string | null>(null);
  const [scanning, setScanning] = createSignal(false);
  const [connecting, setConnecting] = createSignal(false);
  const [disconnecting, setDisconnecting] = createSignal(false);

  const handleScan = async () => {
    setScanning(true);
    try {
      const results = await bleScan(serviceUuid(), 5);
      setDevices(results);
    } catch (e) {
      console.error("BLE scan failed:", e);
    } finally {
      setScanning(false);
    }
  };

  const handleConnect = async () => {
    const deviceId = selectedDevice();
    if (!deviceId) return;
    setConnecting(true);
    try {
      const config: BleServiceConfig = {
        serviceUuid: serviceUuid(),
        notifyCharacteristicUuid: notifyUuid(),
        writeCharacteristicUuid: writeUuid(),
      };
      await bleConnect(deviceId, config);
      setBleConnected(true);
      setActivePortId("ble");
      setDeviceType("ble");
    } catch (e) {
      console.error("BLE connect failed:", e);
    } finally {
      setConnecting(false);
    }
  };

  const handleDisconnect = async () => {
    setDisconnecting(true);
    try {
      await bleDisconnect();
      setBleConnected(false);
      if (activePortId() === "ble") {
        const nextSerialPort = connectedSerialPorts()[0] ?? "";
        setActivePortId(nextSerialPort);
        setDeviceType(nextSerialPort ? "serial" : "ble");
      }
    } catch (e) {
      console.error("BLE disconnect failed:", e);
    } finally {
      setDisconnecting(false);
    }
  };

  return (
    <div class="flex flex-col p-4">
      <div class="flex flex-col gap-y-1.5 pb-4">
        <h3 class="font-semibold leading-none tracking-tight">蓝牙设置</h3>
        <p class="text-sm opacity-60">请配置蓝牙连接相关参数</p>
      </div>

      <div class="flex flex-col space-y-3 pb-4">
        <label class="text-sm font-medium">Service UUID</label>
        <input
          class="input input-sm input-bordered w-full"
          value={serviceUuid()}
          onInput={(e) => setServiceUuid(e.currentTarget.value)}
          placeholder="输入 Service UUID"
        />

        <label class="text-sm font-medium">Notify Characteristic UUID</label>
        <input
          class="input input-sm input-bordered w-full"
          value={notifyUuid()}
          onInput={(e) => setNotifyUuid(e.currentTarget.value)}
          placeholder="输入 Notify UUID"
        />

        <label class="text-sm font-medium">Write Characteristic UUID</label>
        <input
          class="input input-sm input-bordered w-full"
          value={writeUuid()}
          onInput={(e) => setWriteUuid(e.currentTarget.value)}
          placeholder="输入 Write UUID"
        />

        {/* Scan button */}
        <button
          class="btn btn-secondary btn-sm w-full"
          disabled={scanning()}
          onClick={handleScan}
        >
          {scanning()
            ? <span class="loading loading-spinner loading-xs" />
            : null}
          {scanning() ? "扫描中..." : "扫描设备"}
        </button>

        {/* Device list */}
        <Show when={devices().length > 0}>
          <label class="text-sm font-medium">发现的设备</label>
          <div class="border border-base-300 rounded-lg max-h-40 overflow-y-auto custom-scrollbar">
            <For each={devices()}>
              {(device) => (
                <div
                  class="px-3 py-2 text-sm cursor-pointer hover:bg-base-200 border-b border-base-300 last:border-b-0"
                  classList={{
                    "bg-base-200": selectedDevice() === device.id,
                  }}
                  onClick={() => setSelectedDevice(device.id)}
                >
                  <div class="font-medium">{device.name ?? "未知设备"}</div>
                  <div class="text-xs opacity-60">
                    {device.id}
                    {device.rssi != null && ` (${device.rssi} dBm)`}
                  </div>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>

      {/* Connect / Disconnect buttons */}
      <Show when={!bleConnected()}>
        <button
          class="btn btn-primary btn-sm w-full mb-3"
          disabled={connecting() || !selectedDevice()}
          onClick={handleConnect}
        >
          {connecting()
            ? <span class="loading loading-spinner loading-xs" />
            : null}
          {connecting() ? "连接中..." : "连 接"}
        </button>
      </Show>

      <Show when={bleConnected()}>
        <button
          class="btn btn-error btn-sm w-full mb-3"
          disabled={disconnecting()}
          onClick={handleDisconnect}
        >
          {disconnecting()
            ? <span class="loading loading-spinner loading-xs" />
            : null}
          {disconnecting() ? "断开中..." : "断 开"}
        </button>
      </Show>
    </div>
  );
}
