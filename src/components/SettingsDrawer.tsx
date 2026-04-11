import { createEffect, createSignal, For, onCleanup, Show } from "solid-js";
import {
  bleConnect,
  type BleDeviceInfo,
  bleDisconnect,
  bleScan,
  type BleServiceConfig,
  closeSerialPort,
  type FlowControl,
  getOpenPorts,
  listSerialPorts,
  openSerialPort,
  type Parity,
  type PortInfo,
  type SerialConfig,
} from "../lib/commands";
import {
  onSerialDisconnect,
  onSerialPortAdded,
  onSerialPortRemoved,
} from "../lib/events";
import {
  activePortId,
  bleConnected,
  connectedSerialPorts,
  deviceType,
  serialConnected,
  setActivePortId,
  setBleConnected,
  setConnectedSerialPorts,
  setDeviceType,
  setSerialConnected,
  setShowSettingsDrawer,
  showSettingsDrawer,
} from "../stores/ui";
import { BluetoothIcon, RefreshIcon, SerialIcon, XIcon } from "../assets/icons";
import {
  getNextActivePortId,
  getSerialConnectionSummary,
  isPortConnected,
} from "../lib/port-utils";

const BAUD_RATES = [
  "1200",
  "2400",
  "4800",
  "9600",
  "14400",
  "19200",
  "38400",
  "56000",
  "57600",
  "115200",
  "128000",
  "230400",
  "256000",
  "460800",
  "921600",
  "1000000",
];

const DATA_BITS: { label: string; value: string }[] = [
  { label: "5", value: "5" },
  { label: "6", value: "6" },
  { label: "7", value: "7" },
  { label: "8", value: "8" },
];

const STOP_BITS: { label: string; value: string }[] = [
  { label: "1", value: "1" },
  { label: "2", value: "2" },
];

const PARITY_OPTIONS: { label: string; value: Parity }[] = [
  { label: "无", value: "none" },
  { label: "奇", value: "odd" },
  { label: "偶", value: "even" },
];

const FLOW_CONTROL_OPTIONS: { label: string; value: FlowControl }[] = [
  { label: "无", value: "none" },
  { label: "HW", value: "hardware" },
  { label: "SW", value: "software" },
];

type SettingsTab = "serial" | "ble";

export default function SettingsDrawer() {
  const [settingsTab, setSettingsTab] = createSignal<SettingsTab>("serial");

  // --- Serial state ---
  const [ports, setPorts] = createSignal<PortInfo[]>([]);
  const [selectedPort, setSelectedPort] = createSignal<string>("");
  const [baudRate, setBaudRate] = createSignal("115200");
  const [dataBits, setDataBits] = createSignal("8");
  const [stopBits, setStopBits] = createSignal("1");
  const [parity, setParity] = createSignal<Parity>("none");
  const [flowControl, setFlowControl] = createSignal<FlowControl>("none");
  const [serialConnecting, setSerialConnecting] = createSignal(false);
  const [serialDisconnecting, setSerialDisconnecting] = createSignal(false);

  // --- BLE state ---
  const [serviceUuid, setServiceUuid] = createSignal("");
  const [notifyUuid, setNotifyUuid] = createSignal("");
  const [writeUuid, setWriteUuid] = createSignal("");
  const [devices, setDevices] = createSignal<BleDeviceInfo[]>([]);
  const [selectedDevice, setSelectedDevice] = createSignal<string | null>(null);
  const [scanning, setScanning] = createSignal(false);
  const [bleConnecting, setBleConnecting] = createSignal(false);
  const [bleDisconnecting, setBleDisconnecting] = createSignal(false);

  // --- Serial helpers ---
  const refreshOpenPorts = async () => {
    try {
      const openPorts = await getOpenPorts();
      setConnectedSerialPorts(openPorts);
      setSerialConnected(openPorts.length > 0);

      if (!selectedPort() && openPorts.length > 0) {
        setSelectedPort(openPorts[0]);
      }

      if (
        deviceType() === "serial" &&
        activePortId() &&
        !openPorts.includes(activePortId())
      ) {
        setActivePortId(openPorts[0] ?? "");
      }
    } catch (e) {
      console.error("Failed to get open ports:", e);
    }
  };

  const refreshPorts = async () => {
    try {
      const list = await listSerialPorts();
      setPorts(list);
      if (!selectedPort() && list.length > 0) {
        setSelectedPort(list[0].path);
      }
    } catch (e) {
      console.error("Failed to list serial ports:", e);
    }
  };

  createEffect(() => {
    refreshPorts();
    refreshOpenPorts();
  });

  const setupListeners = async () => {
    const unlistenAdd = await onSerialPortAdded(() => {
      refreshPorts();
      refreshOpenPorts();
    });
    const unlistenRemove = await onSerialPortRemoved(() => {
      refreshPorts();
      refreshOpenPorts();
    });
    const unlistenDisconnect = await onSerialDisconnect(() => {
      refreshOpenPorts();
    });
    onCleanup(() => {
      unlistenAdd();
      unlistenRemove();
      unlistenDisconnect();
    });
  };
  setupListeners();

  const handleSerialConnect = async () => {
    const path = selectedPort();
    if (!path) return;
    setSerialConnecting(true);
    try {
      const config: SerialConfig = {
        baudRate: parseInt(baudRate()),
        dataBits: parseInt(dataBits()),
        stopBits: parseInt(stopBits()),
        parity: parity(),
        flowControl: flowControl(),
      };
      await openSerialPort(path, path, config);
      setActivePortId(path);
      setDeviceType("serial");
      await refreshOpenPorts();
    } catch (e) {
      console.error("Failed to open serial port:", e);
    } finally {
      setSerialConnecting(false);
    }
  };

  const handleSerialDisconnect = async () => {
    const portId = selectedPort();
    if (!portId) return;
    setSerialDisconnecting(true);
    try {
      await closeSerialPort(portId);
      const nextActivePortId = getNextActivePortId(
        activePortId(),
        connectedSerialPorts(),
        portId,
      );
      setActivePortId(nextActivePortId);
      await refreshOpenPorts();
    } catch (e) {
      console.error("Failed to close serial port:", e);
    } finally {
      setSerialDisconnecting(false);
    }
  };

  const handleSelectPort = (path: string) => {
    setSelectedPort(path);
    if (isPortConnected(connectedSerialPorts(), path)) {
      setActivePortId(path);
      setDeviceType("serial");
    }
  };

  const selectedPortConnected = () =>
    isPortConnected(connectedSerialPorts(), selectedPort());

  // --- BLE helpers ---
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

  const handleBleConnect = async () => {
    const deviceId = selectedDevice();
    if (!deviceId) return;
    setBleConnecting(true);
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
      setBleConnecting(false);
    }
  };

  const handleBleDisconnect = async () => {
    setBleDisconnecting(true);
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
      setBleDisconnecting(false);
    }
  };

  return (
    <Show when={showSettingsDrawer()}>
      {/* Overlay root */}
      <div class="absolute inset-0 z-50 flex">
        {/* Semi-transparent backdrop */}
        <div
          class="flex-1 bg-black/30 backdrop-blur-sm"
          onClick={() => setShowSettingsDrawer(false)}
        />

        {/* Drawer panel */}
        <div class="w-[280px] bg-base-100 border-l border-base-300 flex flex-col h-full">
          {/* Header */}
          <div class="flex items-center justify-between px-4 py-2.5 border-b border-base-300 shrink-0">
            <span class="font-semibold text-sm">设置</span>
            <button
              class="btn btn-ghost btn-xs btn-square"
              onClick={() => setShowSettingsDrawer(false)}
              title="关闭"
            >
              <XIcon class="w-3.5 h-3.5" />
            </button>
          </div>

          {/* Sub-tabs */}
          <div class="flex border-b border-base-300 px-4 shrink-0">
            <button
              class="flex items-center gap-1.5 px-2 py-2 text-xs font-medium border-b-2 transition-colors"
              classList={{
                "border-primary text-primary": settingsTab() === "serial",
                "border-transparent text-base-content/60 hover:text-base-content":
                  settingsTab() !== "serial",
              }}
              onClick={() => setSettingsTab("serial")}
            >
              <SerialIcon class="w-3.5 h-3.5" />
              串口
            </button>
            <button
              class="flex items-center gap-1.5 px-2 py-2 text-xs font-medium border-b-2 transition-colors"
              classList={{
                "border-primary text-primary": settingsTab() === "ble",
                "border-transparent text-base-content/60 hover:text-base-content":
                  settingsTab() !== "ble",
              }}
              onClick={() => setSettingsTab("ble")}
            >
              <BluetoothIcon class="w-3.5 h-3.5" />
              BLE
            </button>
          </div>

          {/* Content */}
          <div class="flex-1 overflow-y-auto custom-scrollbar px-4 py-3">
            {/* Serial form */}
            <Show when={settingsTab() === "serial"}>
              <div class="flex flex-col gap-3">
                {/* Connected port badges */}
                <Show when={connectedSerialPorts().length > 0}>
                  <div class="flex flex-wrap gap-1.5">
                    <For each={connectedSerialPorts()}>
                      {(portId) => (
                        <button
                          class="badge badge-sm cursor-pointer"
                          classList={{
                            "badge-primary": activePortId() === portId,
                            "badge-outline": activePortId() !== portId,
                          }}
                          onClick={() => handleSelectPort(portId)}
                        >
                          {portId}
                        </button>
                      )}
                    </For>
                  </div>
                </Show>

                {/* Port select + refresh */}
                <div class="flex flex-col gap-1">
                  <label class="text-xs font-medium text-base-content/70">
                    端口
                  </label>
                  <div class="flex gap-1.5">
                    <select
                      class="select select-xs select-bordered flex-1 min-w-0"
                      value={selectedPort()}
                      onChange={(e) => handleSelectPort(e.currentTarget.value)}
                    >
                      <option value="" disabled>
                        选择串口
                      </option>
                      <For each={ports()}>
                        {(p) => (
                          <option value={p.path}>
                            {p.path}
                            {p.deviceName ? ` (${p.deviceName})` : ""}
                          </option>
                        )}
                      </For>
                    </select>
                    <button
                      class="btn btn-xs btn-square btn-ghost shrink-0"
                      onClick={() => {
                        refreshPorts();
                        refreshOpenPorts();
                      }}
                      title="刷新端口列表"
                    >
                      <RefreshIcon class="w-3 h-3" />
                    </button>
                  </div>
                </div>

                {/* Baud rate */}
                <div class="flex flex-col gap-1">
                  <label class="text-xs font-medium text-base-content/70">
                    波特率
                  </label>
                  <input
                    type="text"
                    inputmode="numeric"
                    class="input input-xs input-bordered w-full"
                    value={baudRate()}
                    list="settings-drawer-baud-rate-list"
                    onInput={(e) =>
                      setBaudRate(e.currentTarget.value.replace(/\D/g, ""))
                    }
                  />
                  <datalist id="settings-drawer-baud-rate-list">
                    <For each={BAUD_RATES}>
                      {(rate) => <option value={rate} />}
                    </For>
                  </datalist>
                </div>

                {/* Data bits */}
                <div class="flex flex-col gap-1">
                  <label class="text-xs font-medium text-base-content/70">
                    数据位
                  </label>
                  <div class="flex gap-0">
                    <For each={DATA_BITS}>
                      {(opt) => (
                        <button
                          class="btn btn-xs flex-1 rounded-none first:rounded-l last:rounded-r"
                          classList={{
                            "btn-primary": dataBits() === opt.value,
                            "btn-ghost border border-base-300":
                              dataBits() !== opt.value,
                          }}
                          onClick={() => setDataBits(opt.value)}
                        >
                          {opt.label}
                        </button>
                      )}
                    </For>
                  </div>
                </div>

                {/* Parity */}
                <div class="flex flex-col gap-1">
                  <label class="text-xs font-medium text-base-content/70">
                    校验位
                  </label>
                  <div class="flex gap-0">
                    <For each={PARITY_OPTIONS}>
                      {(opt) => (
                        <button
                          class="btn btn-xs flex-1 rounded-none first:rounded-l last:rounded-r"
                          classList={{
                            "btn-primary": parity() === opt.value,
                            "btn-ghost border border-base-300":
                              parity() !== opt.value,
                          }}
                          onClick={() => setParity(opt.value)}
                        >
                          {opt.label}
                        </button>
                      )}
                    </For>
                  </div>
                </div>

                {/* Stop bits */}
                <div class="flex flex-col gap-1">
                  <label class="text-xs font-medium text-base-content/70">
                    停止位
                  </label>
                  <div class="flex gap-0">
                    <For each={STOP_BITS}>
                      {(opt) => (
                        <button
                          class="btn btn-xs flex-1 rounded-none first:rounded-l last:rounded-r"
                          classList={{
                            "btn-primary": stopBits() === opt.value,
                            "btn-ghost border border-base-300":
                              stopBits() !== opt.value,
                          }}
                          onClick={() => setStopBits(opt.value)}
                        >
                          {opt.label}
                        </button>
                      )}
                    </For>
                  </div>
                </div>

                {/* Flow control */}
                <div class="flex flex-col gap-1">
                  <label class="text-xs font-medium text-base-content/70">
                    流控
                  </label>
                  <div class="flex gap-0">
                    <For each={FLOW_CONTROL_OPTIONS}>
                      {(opt) => (
                        <button
                          class="btn btn-xs flex-1 rounded-none first:rounded-l last:rounded-r"
                          classList={{
                            "btn-primary": flowControl() === opt.value,
                            "btn-ghost border border-base-300":
                              flowControl() !== opt.value,
                          }}
                          onClick={() => setFlowControl(opt.value)}
                        >
                          {opt.label}
                        </button>
                      )}
                    </For>
                  </div>
                </div>

                {/* Connect / Disconnect */}
                <div class="flex gap-2 pt-1">
                  <button
                    class="btn btn-primary btn-xs flex-1"
                    disabled={
                      serialConnecting() ||
                      serialDisconnecting() ||
                      !selectedPort() ||
                      selectedPortConnected()
                    }
                    onClick={handleSerialConnect}
                  >
                    <Show when={serialConnecting()}>
                      <span class="loading loading-spinner loading-xs" />
                    </Show>
                    {serialConnecting() ? "连接中..." : "打开所选端口"}
                  </button>
                  <button
                    class="btn btn-error btn-xs flex-1"
                    disabled={serialDisconnecting() || !selectedPortConnected()}
                    onClick={handleSerialDisconnect}
                  >
                    <Show when={serialDisconnecting()}>
                      <span class="loading loading-spinner loading-xs" />
                    </Show>
                    {serialDisconnecting() ? "断开中..." : "关闭所选端口"}
                  </button>
                </div>
              </div>
            </Show>

            {/* BLE form */}
            <Show when={settingsTab() === "ble"}>
              <div class="flex flex-col gap-3">
                {/* Service UUID */}
                <div class="flex flex-col gap-1">
                  <label class="text-xs font-medium text-base-content/70">
                    Service UUID
                  </label>
                  <input
                    class="input input-xs input-bordered w-full"
                    value={serviceUuid()}
                    onInput={(e) => setServiceUuid(e.currentTarget.value)}
                    placeholder="输入 Service UUID"
                  />
                </div>

                {/* Notify Characteristic UUID */}
                <div class="flex flex-col gap-1">
                  <label class="text-xs font-medium text-base-content/70">
                    Notify Characteristic UUID
                  </label>
                  <input
                    class="input input-xs input-bordered w-full"
                    value={notifyUuid()}
                    onInput={(e) => setNotifyUuid(e.currentTarget.value)}
                    placeholder="输入 Notify UUID"
                  />
                </div>

                {/* Write Characteristic UUID */}
                <div class="flex flex-col gap-1">
                  <label class="text-xs font-medium text-base-content/70">
                    Write Characteristic UUID
                  </label>
                  <input
                    class="input input-xs input-bordered w-full"
                    value={writeUuid()}
                    onInput={(e) => setWriteUuid(e.currentTarget.value)}
                    placeholder="输入 Write UUID"
                  />
                </div>

                {/* Scan button */}
                <button
                  class="btn btn-secondary btn-xs w-full"
                  disabled={scanning()}
                  onClick={handleScan}
                >
                  <Show when={scanning()}>
                    <span class="loading loading-spinner loading-xs" />
                  </Show>
                  {scanning() ? "扫描中..." : "扫描设备"}
                </button>

                {/* Device list */}
                <Show when={devices().length > 0}>
                  <div class="flex flex-col gap-1">
                    <label class="text-xs font-medium text-base-content/70">
                      发现的设备
                    </label>
                    <div class="border border-base-300 rounded-lg max-h-40 overflow-y-auto custom-scrollbar">
                      <For each={devices()}>
                        {(device) => (
                          <div
                            class="px-3 py-2 text-xs cursor-pointer hover:bg-base-200 border-b border-base-300 last:border-b-0"
                            classList={{
                              "bg-base-200": selectedDevice() === device.id,
                            }}
                            onClick={() => setSelectedDevice(device.id)}
                          >
                            <div class="font-medium">
                              {device.name ?? "未知设备"}
                            </div>
                            <div class="opacity-60">
                              {device.id}
                              {device.rssi != null && ` (${device.rssi} dBm)`}
                            </div>
                          </div>
                        )}
                      </For>
                    </div>
                  </div>
                </Show>

                {/* BLE Connect / Disconnect */}
                <Show when={!bleConnected()}>
                  <button
                    class="btn btn-primary btn-xs w-full"
                    disabled={bleConnecting() || !selectedDevice()}
                    onClick={handleBleConnect}
                  >
                    <Show when={bleConnecting()}>
                      <span class="loading loading-spinner loading-xs" />
                    </Show>
                    {bleConnecting() ? "连接中..." : "连 接"}
                  </button>
                </Show>

                <Show when={bleConnected()}>
                  <button
                    class="btn btn-error btn-xs w-full"
                    disabled={bleDisconnecting()}
                    onClick={handleBleDisconnect}
                  >
                    <Show when={bleDisconnecting()}>
                      <span class="loading loading-spinner loading-xs" />
                    </Show>
                    {bleDisconnecting() ? "断开中..." : "断 开"}
                  </button>
                </Show>
              </div>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  );
}
