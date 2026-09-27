import { createEffect, For, Show } from "solid-js";
import { ENCODING_OPTIONS } from "../lib/commands";
import { SerialIcon, TrashIcon } from "../assets/icons";
import {
  clearFrames,
  frames,
  serialStatus,
  type TerminalFrame,
} from "../stores/serial";
import {
  autoScroll,
  chooseDisplayMode,
  displayMode,
  setAutoScroll,
} from "../stores/ui";

const timeFormatter = new Intl.DateTimeFormat("zh-CN", {
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
  fractionalSecondDigits: 3,
  hour12: false,
});

function readableText(text: string) {
  return text
    .replaceAll("\0", "␀")
    .replaceAll("\r", "␍")
    .replaceAll("\n", "␊\n")
    .replaceAll("\t", "⇥");
}

function frameTime(frame: TerminalFrame) {
  const date = new Date(frame.timestamp);
  return Number.isNaN(date.getTime())
    ? frame.timestamp
    : timeFormatter.format(date);
}

export default function Terminal() {
  let viewport!: HTMLDivElement;

  createEffect(() => {
    frames().length;
    if (autoScroll()) {
      queueMicrotask(() => viewport?.scrollTo({ top: viewport.scrollHeight }));
    }
  });

  return (
    <section class="terminal-panel" aria-label="串口终端">
      <div class="terminal-heading">
        <div>
          <span class="panel-title">数据终端</span>
          <span class="frame-count">{frames().length} 条记录</span>
        </div>
        <div class="terminal-actions">
          <div class="segmented-control" role="group" aria-label="数据显示格式">
            {ENCODING_OPTIONS.map((option) => (
              <button
                classList={{ active: displayMode() === option.value }}
                aria-pressed={displayMode() === option.value}
                onClick={() => chooseDisplayMode(option.value)}
              >
                {option.label}
              </button>
            ))}
          </div>
          <button
            class="icon-button"
            title="清空终端"
            aria-label="清空终端"
            disabled={frames().length === 0}
            onClick={clearFrames}
          >
            <TrashIcon />
          </button>
        </div>
      </div>
      <div class="terminal-subheading">
        <span class="terminal-legend">
          <span>↓ 接收</span>
          <span>↑ 发送</span>
        </span>
        <label class="switch-label">
          <span>自动滚动</span>
          <input
            type="checkbox"
            checked={autoScroll()}
            onChange={(event) => setAutoScroll(event.currentTarget.checked)}
          />
          <span class="switch-track" />
        </label>
      </div>

      <div class="terminal-viewport" ref={viewport}>
        <Show
          when={frames().length > 0}
          fallback={
            <div class="terminal-empty">
              <span class="empty-icon" aria-hidden="true">
                <SerialIcon />
              </span>
              <h3>
                {serialStatus().connected ? "等待设备数据" : "连接串口后开始"}
              </h3>
              <p>
                {serialStatus().connected
                  ? "收到的数据会实时显示在这里"
                  : "选择设备和通信参数，然后点击“连接串口”"}
              </p>
            </div>
          }
        >
          <div class="frame-list">
            <For each={frames()}>
              {(frame) => (
                <article
                  classList={{
                    "terminal-frame": true,
                    receive: frame.direction === "receive",
                    send: frame.direction === "send",
                  }}
                >
                  <div class="frame-meta">
                    <span class="direction-pill">
                      {frame.direction === "receive" ? "RX" : "TX"}
                    </span>
                    <time>{frameTime(frame)}</time>
                    <span>{frame.bytes.length} B</span>
                  </div>
                  <div class="frame-content">
                    <Show when={displayMode() !== "hex"}>
                      <pre class="text-value">
                        {readableText(
                          frame.text[
                            displayMode() as keyof typeof frame.text
                          ] ?? "",
                        ) || "∅"}
                      </pre>
                    </Show>
                    <Show when={displayMode() === "hex"}>
                      <pre class="hex-value">{frame.hex}</pre>
                    </Show>
                  </div>
                </article>
              )}
            </For>
          </div>
        </Show>
      </div>
    </section>
  );
}
