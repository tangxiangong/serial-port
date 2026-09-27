import { createSignal } from "solid-js";
import { SendIcon } from "../assets/icons";
import type { LineEnding, SendFormat } from "../lib/commands";
import { serialStatus, writeSerial } from "../stores/serial";

export default function Composer() {
  const [input, setInput] = createSignal("");
  const [format, setFormat] = createSignal<SendFormat>("text");
  const [lineEnding, setLineEnding] = createSignal<LineEnding>("none");
  const [sending, setSending] = createSignal(false);

  const submit = async () => {
    if (sending() || !input()) return;
    setSending(true);
    await writeSerial(input(), format(), lineEnding());
    setSending(false);
  };

  return (
    <section class="composer-panel" aria-label="发送数据">
      <div class="composer-options">
        <span class="panel-title">发送</span>
        <div
          class="segmented-control compact"
          role="group"
          aria-label="发送格式"
        >
          <button
            classList={{ active: format() === "text" }}
            aria-pressed={format() === "text"}
            onClick={() => setFormat("text")}
          >
            文本
          </button>
          <button
            classList={{ active: format() === "hex" }}
            aria-pressed={format() === "hex"}
            onClick={() => setFormat("hex")}
          >
            HEX
          </button>
        </div>

        <label class="ending-select">
          <span>行尾</span>
          <select
            value={lineEnding()}
            disabled={format() === "hex"}
            onChange={(event) =>
              setLineEnding(event.currentTarget.value as LineEnding)
            }
          >
            <option value="none">无</option>
            <option value="lf">LF</option>
            <option value="cr">CR</option>
            <option value="crlf">CRLF</option>
          </select>
        </label>
        <span class="composer-hint">⌘ ↵ 发送</span>
      </div>

      <div class="composer-input-row">
        <textarea
          aria-label="要发送的数据"
          value={input()}
          placeholder={
            format() === "hex"
              ? "输入十六进制字节，例如 AA 01 FF"
              : "输入要发送到设备的内容…"
          }
          spellcheck={false}
          onInput={(event) => setInput(event.currentTarget.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && event.metaKey) {
              event.preventDefault();
              void submit();
            }
          }}
        />
        <button
          class="send-button"
          disabled={!serialStatus().connected || !input() || sending()}
          title="发送（⌘ Enter）"
          aria-label={sending() ? "正在发送" : "发送数据"}
          onClick={submit}
        >
          <SendIcon />
          <span>{sending() ? "发送中" : "发送"}</span>
        </button>
      </div>
    </section>
  );
}
