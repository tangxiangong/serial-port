import { Show } from "solid-js";
import { serialConfig, serialStatus } from "../stores/serial";

function formatBytes(value: number) {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / 1024 / 1024).toFixed(1)} MB`;
}

export default function StatusBar() {
  return (
    <footer class="status-bar">
      <div class="status-left">
        <span
          classList={{
            "status-indicator": true,
            connected: serialStatus().connected,
          }}
        />
        <span>{serialStatus().connected ? serialStatus().path : "未连接"}</span>
        <Show when={serialStatus().connected}>
          <span class="status-separator" />
          <span>{serialConfig().baudRate} baud</span>
          <span>
            {serialConfig().dataBits}
            {serialConfig().parity === "none"
              ? "N"
              : serialConfig().parity === "even"
                ? "E"
                : "O"}
            {serialConfig().stopBits}
          </span>
        </Show>
      </div>
      <div class="status-traffic">
        <span class="rx-stat">
          <b>↓</b> {formatBytes(serialStatus().rxBytes)}
        </span>
        <span class="tx-stat">
          <b>↑</b> {formatBytes(serialStatus().txBytes)}
        </span>
      </div>
    </footer>
  );
}
