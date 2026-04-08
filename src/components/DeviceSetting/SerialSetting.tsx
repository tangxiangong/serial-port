import { createSignal, createEffect, onCleanup, For, Show } from "solid-js";
import {
  listSerialPorts,
  openSerialPort,
  closeSerialPort,
  type PortInfo,
  type SerialConfig,
  type Parity,
  type FlowControl,
} from "../../lib/commands";
import {
  onSerialPortAdded,
  onSerialPortRemoved,
  onSerialDisconnect,
} from "../../lib/events";
import {
  activePortId,
  serialConnected,
  setSerialConnected,
} from "../../stores/ui";

const BAUD_RATES = ["9600", "19200", "38400", "57600", "115200"];
const DATA_BITS = ["8", "7", "6", "5"];
const STOP_BITS = ["1", "2"];
const PARITY_OPTIONS: { label: string; value: Parity }[] = [
  { label: "None", value: "none" },
  { label: "Even", value: "even" },
  { label: "Odd", value: "odd" },
];
const FLOW_CONTROL_OPTIONS: { label: string; value: FlowControl }[] = [
  { label: "None", value: "none" },
  { label: "Software", value: "software" },
  { label: "Hardware", value: "hardware" },
];

export default function SerialSetting() {
  const [ports, setPorts] = createSignal<PortInfo[]>([]);
  const [selectedPort, setSelectedPort] = createSignal<string>("");
  const [baudRate, setBaudRate] = createSignal("115200");
  const [dataBits, setDataBits] = createSignal("8");
  const [stopBits, setStopBits] = createSignal("1");
  const [parity, setParity] = createSignal<Parity>("none");
  const [flowControl, setFlowControl] = createSignal<FlowControl>("none");
  const [connecting, setConnecting] = createSignal(false);
  const [disconnecting, setDisconnecting] = createSignal(false);

  const refreshPorts = async () => {
    try {
      const list = await listSerialPorts();
      setPorts(list);
    } catch (e) {
      console.error("Failed to list serial ports:", e);
    }
  };

  createEffect(() => {
    refreshPorts();
  });

  const setupListeners = async () => {
    const unlistenAdd = await onSerialPortAdded(() => refreshPorts());
    const unlistenRemove = await onSerialPortRemoved(() => refreshPorts());
    const unlistenDisconnect = await onSerialDisconnect(() => {
      setSerialConnected(false);
    });
    onCleanup(() => {
      unlistenAdd();
      unlistenRemove();
      unlistenDisconnect();
    });
  };
  setupListeners();

  const handleConnect = async () => {
    const path = selectedPort();
    if (!path) return;
    setConnecting(true);
    try {
      const config: SerialConfig = {
        baudRate: parseInt(baudRate()),
        dataBits: parseInt(dataBits()),
        stopBits: parseInt(stopBits()),
        parity: parity(),
        flowControl: flowControl(),
      };
      await openSerialPort(activePortId(), path, config);
      setSerialConnected(true);
    } catch (e) {
      console.error("Failed to open serial port:", e);
    } finally {
      setConnecting(false);
    }
  };

  const handleDisconnect = async () => {
    setDisconnecting(true);
    try {
      await closeSerialPort(activePortId());
      setSerialConnected(false);
    } catch (e) {
      console.error("Failed to close serial port:", e);
    } finally {
      setDisconnecting(false);
    }
  };

  return (
    <div class="flex flex-col p-4">
      <div class="flex flex-col gap-y-1.5 pb-4">
        <h3 class="font-semibold leading-none tracking-tight">
          {selectedPort() && serialConnected() ? selectedPort() : "串口设置"}
        </h3>
        <p class="text-sm opacity-60">请选择串口连接相关参数</p>
      </div>

      <div class="flex flex-col space-y-3 pb-4">
        {/* Port selection */}
        <label class="text-sm font-medium">端口</label>
        <div class="flex gap-2">
          <select
            class="select select-sm select-bordered flex-1"
            value={selectedPort()}
            onChange={(e) => setSelectedPort(e.currentTarget.value)}
          >
            <option value="" disabled>选择串口</option>
            <For each={ports()}>
              {(p) => (
                <option value={p.path}>
                  {p.path}{p.deviceName ? ` (${p.deviceName})` : ""}
                </option>
              )}
            </For>
          </select>
          <button
            class="btn btn-sm btn-square btn-ghost"
            onClick={refreshPorts}
            title="刷新端口列表"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24">
              <path
                fill="currentColor"
                d="M12 20q-3.35 0-5.675-2.325T4 12t2.325-5.675T12 4q1.725 0 3.3.712T18 6.75V4h2v7h-7V9h4.2q-.8-1.4-2.187-2.2T12 6Q9.5 6 7.75 7.75T6 12t1.75 4.25T12 18q1.925 0 3.475-1.1T17.65 14h2.1q-.7 2.65-2.85 4.325T12 20"
              />
            </svg>
          </button>
        </div>

        {/* Baud Rate */}
        <label class="text-sm font-medium">波特率</label>
        <select
          class="select select-sm select-bordered w-full"
          value={baudRate()}
          onChange={(e) => setBaudRate(e.currentTarget.value)}
        >
          <For each={BAUD_RATES}>
            {(rate) => <option value={rate}>{rate}</option>}
          </For>
        </select>

        {/* Data Bits */}
        <label class="text-sm font-medium">数据位</label>
        <select
          class="select select-sm select-bordered w-full"
          value={dataBits()}
          onChange={(e) => setDataBits(e.currentTarget.value)}
        >
          <For each={DATA_BITS}>
            {(bits) => <option value={bits}>{bits}</option>}
          </For>
        </select>

        {/* Parity */}
        <label class="text-sm font-medium">校验位</label>
        <select
          class="select select-sm select-bordered w-full"
          value={parity()}
          onChange={(e) => setParity(e.currentTarget.value as Parity)}
        >
          <For each={PARITY_OPTIONS}>
            {(opt) => <option value={opt.value}>{opt.label}</option>}
          </For>
        </select>

        {/* Stop Bits */}
        <label class="text-sm font-medium">停止位</label>
        <select
          class="select select-sm select-bordered w-full"
          value={stopBits()}
          onChange={(e) => setStopBits(e.currentTarget.value)}
        >
          <For each={STOP_BITS}>
            {(bits) => <option value={bits}>{bits}</option>}
          </For>
        </select>

        {/* Flow Control */}
        <label class="text-sm font-medium">流控</label>
        <select
          class="select select-sm select-bordered w-full"
          value={flowControl()}
          onChange={(e) => setFlowControl(e.currentTarget.value as FlowControl)}
        >
          <For each={FLOW_CONTROL_OPTIONS}>
            {(opt) => <option value={opt.value}>{opt.label}</option>}
          </For>
        </select>
      </div>

      {/* Connect / Disconnect buttons */}
      <Show when={!serialConnected()}>
        <button
          class="btn btn-primary btn-sm w-full mb-3"
          disabled={connecting() || disconnecting() || !selectedPort()}
          onClick={handleConnect}
        >
          {connecting() ? <span class="loading loading-spinner loading-xs" /> : null}
          {connecting() ? "连接中..." : "连 接"}
        </button>
      </Show>

      <Show when={serialConnected()}>
        <button
          class="btn btn-error btn-sm w-full mb-3"
          disabled={disconnecting()}
          onClick={handleDisconnect}
        >
          {disconnecting() ? <span class="loading loading-spinner loading-xs" /> : null}
          {disconnecting() ? "断开中..." : "断 开"}
        </button>
      </Show>
    </div>
  );
}
