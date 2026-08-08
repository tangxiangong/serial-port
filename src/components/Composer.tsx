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
    <section class="composer-panel">
      <div class="composer-options">
        <div class="segmented-control compact">
          <button
            classList={{ active: format() === "text" }}
            onClick={() => setFormat("text")}
          >
            文本
          </button>
          <button
            classList={{ active: format() === "hex" }}
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
          onClick={submit}
        >
          <SendIcon />
        </button>
      </div>
    </section>
  );
}
