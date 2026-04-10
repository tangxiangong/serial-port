import { createEffect, createSignal, For, onCleanup, Show } from "solid-js";
import {
  closeSerialPort,
  type FlowControl,
  getOpenPorts,
  listSerialPorts,
  openSerialPort,
  type Parity,
  type PortInfo,
  type SerialConfig,
} from "../../lib/commands";
import {
  onSerialDisconnect,
  onSerialPortAdded,
  onSerialPortRemoved,
} from "../../lib/events";
import {
  activePortId,
  connectedSerialPorts,
  deviceType,
  setActivePortId,
  setConnectedSerialPorts,
  setDeviceType,
  serialConnected,
  setSerialConnected,
} from "../../stores/ui";
import { RefreshIcon } from "../../assets/icons";
import {
  getNextActivePortId,
  getSerialConnectionSummary,
  isPortConnected,
} from "../../lib/port-utils";

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
      await openSerialPort(path, path, config);
      setActivePortId(path);
      setDeviceType("serial");
      await refreshOpenPorts();
    } catch (e) {
      console.error("Failed to open serial port:", e);
    } finally {
      setConnecting(false);
    }
  };

  const handleDisconnect = async () => {
    const portId = selectedPort();
    if (!portId) return;
    setDisconnecting(true);
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
      setDisconnecting(false);
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

  return (
    <div class="flex flex-col p-4">
      <div class="flex flex-col gap-y-1.5 pb-4">
        <h3 class="font-semibold leading-none tracking-tight">
          {serialConnected()
            ? getSerialConnectionSummary(connectedSerialPorts())
            : "串口设置"}
        </h3>
        <p class="text-sm opacity-60">
          {serialConnected()
            ? `当前活动端口: ${activePortId() || "未选择"}`
            : "请选择串口连接相关参数"}
        </p>
      </div>

      <Show when={connectedSerialPorts().length > 0}>
        <div class="flex flex-wrap gap-2 pb-4">
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

      <div class="flex flex-col space-y-3 pb-4">
        <label class="text-sm font-medium">端口</label>
        <div class="flex gap-2">
          <select
            class="select select-sm select-bordered flex-1"
            value={selectedPort()}
            onChange={(e) => handleSelectPort(e.currentTarget.value)}
          >
            <option value="" disabled>选择串口</option>
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
            class="btn btn-sm btn-square btn-ghost"
            onClick={() => {
              refreshPorts();
              refreshOpenPorts();
            }}
            title="刷新端口列表"
          >
            <RefreshIcon class="w-3.5 h-3.5" />
          </button>
        </div>

        <label class="text-sm font-medium">波特率</label>
        <input
          type="text"
          inputmode="numeric"
          class="input input-sm input-bordered w-full"
          value={baudRate()}
          list="baud-rate-list"
          onInput={(e) => setBaudRate(e.currentTarget.value.replace(/\D/g, ""))}
        />
        <datalist id="baud-rate-list">
          <For each={BAUD_RATES}>
            {(rate) => <option value={rate} />}
          </For>
        </datalist>

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

      <div class="flex gap-2">
        <button
          class="btn btn-primary btn-sm flex-1"
          disabled={connecting() || disconnecting() || !selectedPort() ||
            selectedPortConnected()}
          onClick={handleConnect}
        >
          {connecting()
            ? <span class="loading loading-spinner loading-xs" />
            : null}
          {connecting() ? "连接中..." : "打开所选端口"}
        </button>
        <button
          class="btn btn-error btn-sm flex-1"
          disabled={disconnecting() || !selectedPortConnected()}
          onClick={handleDisconnect}
        >
          {disconnecting()
            ? <span class="loading loading-spinner loading-xs" />
            : null}
          {disconnecting() ? "断开中..." : "关闭所选端口"}
        </button>
      </div>
    </div>
  );
}
