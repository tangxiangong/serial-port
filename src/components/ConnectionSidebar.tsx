import { createMemo, For, Show } from "solid-js";
import { RefreshIcon, UnplugIcon, ZapIcon } from "../assets/icons";
import {
  isRefreshing,
  isWorking,
  ports,
  refreshPorts,
  selectedPath,
  serialConfig,
  serialStatus,
  setSelectedPath,
  toggleConnection,
  updateConfig,
} from "../stores/serial";

const baudRates = [
  1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600,
];

export default function ConnectionSidebar() {
  const selectedPort = createMemo(() =>
    ports().find((port) => port.path === selectedPath()),
  );
  const locked = () => serialStatus().connected || isWorking();

  return (
    <aside class="connection-sidebar" aria-label="串口连接设置">
      <div class="sidebar-scroll">
        <div class="sidebar-heading">
          <div>
            <p class="sidebar-caption">设备与配置</p>
            <h2>串口连接</h2>
          </div>
          <button
            classList={{ "icon-button": true, spinning: isRefreshing() }}
            title="刷新串口"
            aria-label="刷新串口"
            onClick={refreshPorts}
            disabled={isRefreshing()}
          >
            <RefreshIcon />
          </button>
        </div>

        <section class="sidebar-section port-section">
          <label class="field-label" for="serial-port">
            设备
          </label>
          <div class="select-shell">
            <select
              id="serial-port"
              value={selectedPath()}
              disabled={locked() || ports().length === 0}
              onChange={(event) => setSelectedPath(event.currentTarget.value)}
            >
              <Show
                when={ports().length > 0}
                fallback={<option value="">未发现串口</option>}
              >
                <For each={ports()}>
                  {(port) => (
                    <option value={port.path}>
                      {port.displayName} · {port.path}
                    </option>
                  )}
                </For>
              </Show>
            </select>
          </div>

          <Show when={ports().length === 0}>
            <p class="port-help">接入串口设备后，点击右上角刷新。</p>
          </Show>
          <Show when={selectedPort()}>
            {(port) => (
              <div class="port-card">
                <span class="port-card-icon">
                  <ZapIcon />
                </span>
                <div>
                  <strong>{port().displayName}</strong>
                  <span>{port().path}</span>
                  <Show when={port().vid && port().pid}>
                    <small>
                      VID {port().vid} · PID {port().pid}
                    </small>
                  </Show>
                </div>
              </div>
            )}
          </Show>
        </section>

        <section class="sidebar-section settings-section">
          <div class="section-title-row">
            <span>通信参数</span>
            <span class="config-summary">
              {serialConfig().dataBits}
              {serialConfig().parity === "none"
                ? "N"
                : serialConfig().parity === "even"
                  ? "E"
                  : "O"}
              {serialConfig().stopBits}
            </span>
          </div>

          <label class="field-label" for="baud-rate">
            波特率
          </label>
          <div class="select-shell">
            <select
              id="baud-rate"
              value={serialConfig().baudRate}
              disabled={locked()}
              onChange={(event) =>
                updateConfig({ baudRate: Number(event.currentTarget.value) })
              }
            >
              <For each={baudRates}>
                {(baudRate) => <option value={baudRate}>{baudRate}</option>}
              </For>
            </select>
          </div>

          <div class="field-grid">
            <div>
              <label class="field-label" for="data-bits">
                数据位
              </label>
              <div class="select-shell">
                <select
                  id="data-bits"
                  value={serialConfig().dataBits}
                  disabled={locked()}
                  onChange={(event) =>
                    updateConfig({
                      dataBits: Number(event.currentTarget.value),
                    })
                  }
                >
                  <For each={[5, 6, 7, 8]}>
                    {(value) => <option value={value}>{value}</option>}
                  </For>
                </select>
              </div>
            </div>
            <div>
              <label class="field-label" for="stop-bits">
                停止位
              </label>
              <div class="select-shell">
                <select
                  id="stop-bits"
                  value={serialConfig().stopBits}
                  disabled={locked()}
                  onChange={(event) =>
                    updateConfig({
                      stopBits: Number(event.currentTarget.value),
                    })
                  }
                >
                  <option value={1}>1</option>
                  <option value={2}>2</option>
                </select>
              </div>
            </div>
          </div>

          <span class="field-label" id="parity-label">
            校验位
          </span>
          <div
            class="segmented-control full-width"
            role="group"
            aria-labelledby="parity-label"
          >
            {[
              ["none", "无"],
              ["even", "偶"],
              ["odd", "奇"],
            ].map(([value, label]) => (
              <button
                classList={{ active: serialConfig().parity === value }}
                aria-pressed={serialConfig().parity === value}
                disabled={locked()}
                onClick={() =>
                  updateConfig({ parity: value as "none" | "even" | "odd" })
                }
              >
                {label}
              </button>
            ))}
          </div>

          <label class="field-label" for="flow-control">
            流控制
          </label>
          <div class="select-shell">
            <select
              id="flow-control"
              value={serialConfig().flowControl}
              disabled={locked()}
              onChange={(event) =>
                updateConfig({
                  flowControl: event.currentTarget.value as
                    | "none"
                    | "software"
                    | "hardware",
                })
              }
            >
              <option value="none">无</option>
              <option value="software">软件（XON / XOFF）</option>
              <option value="hardware">硬件（RTS / CTS）</option>
            </select>
          </div>
        </section>
      </div>

      <button
        classList={{
          "connection-button": true,
          disconnect: serialStatus().connected,
        }}
        disabled={isWorking() || (!serialStatus().connected && !selectedPath())}
        onClick={toggleConnection}
      >
        <Show when={serialStatus().connected} fallback={<ZapIcon />}>
          <UnplugIcon />
        </Show>
        {isWorking()
          ? "正在处理…"
          : serialStatus().connected
            ? "断开连接"
            : "连接串口"}
      </button>
    </aside>
  );
}
