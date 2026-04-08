import { createSignal, Show } from "solid-js";
import {
  sendData,
  formatHexInput,
  type SendFormat,
  type LineEnding,
  type CheckAlgorithm,
} from "../lib/commands";
import { activePortId, deviceType } from "../stores/ui";

const CHECK_ALGORITHM_OPTIONS: { label: string; value: CheckAlgorithm }[] = [
  { label: "无", value: "none" },
  { label: "校验和", value: "checksum" },
  { label: "异或", value: "xor" },
  { label: "Modbus CRC16", value: "modbus_crc16" },
];

export default function SendPanel() {
  const [input, setInput] = createSignal("");
  const [sendFormat, setSendFormat] = createSignal<SendFormat>("hex");
  const [lineEnding, setLineEnding] = createSignal("None");
  const [checkAlgorithm, setCheckAlgorithm] =
    createSignal<CheckAlgorithm>("none");
  const [autoSend, setAutoSend] = createSignal(false);
  const [autoInterval, setAutoInterval] = createSignal(1000);
  const [sending, setSending] = createSignal(false);
  let autoTimer: ReturnType<typeof setInterval> | undefined;

  const getLineEnding = (): LineEnding => {
    const val = lineEnding();
    if (val === "None") return "None";
    if (val === "LF") return "LF";
    if (val === "CR") return "CR";
    if (val === "CRLF") return "CRLF";
    return "None";
  };

  const handleSend = async () => {
    if (!input().trim()) return;
    setSending(true);
    try {
      await sendData(
        activePortId(),
        input(),
        sendFormat(),
        getLineEnding(),
        checkAlgorithm(),
        deviceType(),
      );
    } catch (e) {
      console.error("Failed to send data:", e);
    } finally {
      setSending(false);
    }
  };

  const handleReformat = async () => {
    try {
      const formatted = await formatHexInput(input());
      setInput(formatted);
    } catch (e) {
      console.error("Failed to format hex input:", e);
    }
  };

  const toggleAutoSend = () => {
    if (autoSend()) {
      setAutoSend(false);
      if (autoTimer) {
        clearInterval(autoTimer);
        autoTimer = undefined;
      }
    } else {
      setAutoSend(true);
      handleSend();
      autoTimer = setInterval(() => {
        if (autoSend()) {
          handleSend();
        }
      }, autoInterval());
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.shiftKey && e.key === "Enter") {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div class="flex flex-col h-full">
      <textarea
        class="textarea textarea-ghost flex-1 w-full text-sm resize-none rounded-none focus:outline-none"
        value={input()}
        onInput={(e) => setInput(e.currentTarget.value)}
        onKeyDown={handleKeyDown}
        placeholder="输入要发送的数据..."
      />

      <div class="px-3 py-2 flex items-center justify-end gap-2 flex-wrap">
        {/* Format selector */}
        <select
          class="select select-xs select-bordered"
          value={sendFormat()}
          onChange={(e) => setSendFormat(e.currentTarget.value as SendFormat)}
        >
          <option value="hex">HEX</option>
          <option value="ascii">ASCII</option>
          <option value="dec">DEC</option>
        </select>

        {/* Line ending selector */}
        <select
          class="select select-xs select-bordered"
          value={lineEnding()}
          onChange={(e) => setLineEnding(e.currentTarget.value)}
        >
          <option value="None">无换行</option>
          <option value="LF">LF</option>
          <option value="CR">CR</option>
          <option value="CRLF">CRLF</option>
        </select>

        {/* Check algorithm - only for hex mode */}
        <Show when={sendFormat() === "hex"}>
          <select
            class="select select-xs select-bordered"
            value={checkAlgorithm()}
            onChange={(e) =>
              setCheckAlgorithm(e.currentTarget.value as CheckAlgorithm)
            }
          >
            {CHECK_ALGORITHM_OPTIONS.map((opt) => (
              <option value={opt.value}>{opt.label}</option>
            ))}
          </select>

          {/* Reformat hex */}
          <button
            class="btn btn-ghost btn-xs btn-square"
            onClick={handleReformat}
            title="整理格式"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="16"
              height="16"
              viewBox="0 0 24 24"
            >
              <path
                fill="currentColor"
                d="M11 22q-.825 0-1.412-.587T9 20v-4H6q-.825 0-1.412-.587T4 14V7q0-1.65 1.175-2.825T8 3h12v11q0 .825-.587 1.413T18 16h-3v4q0 .825-.587 1.413T13 22zM6 10h12V5h-1v4h-2V5h-1v2h-2V5H8q-.825 0-1.412.588T6 7zm0 4h12v-2H6zm0 0v-2z"
              />
            </svg>
          </button>
        </Show>

        {/* Auto send toggle */}
        <button
          class="btn btn-xs"
          classList={{
            "btn-error": autoSend(),
            "btn-outline": !autoSend(),
          }}
          onClick={toggleAutoSend}
        >
          {autoSend() ? "停止" : "自动"}
        </button>

        <Show when={!autoSend()}>
          <input
            type="number"
            class="input input-xs input-bordered w-16 text-center disable-spin-button"
            value={autoInterval()}
            onInput={(e) =>
              setAutoInterval(parseInt(e.currentTarget.value) || 1000)
            }
            min="100"
            step="100"
          />
          <span class="text-xs opacity-60">ms</span>
        </Show>

        {/* Send button */}
        <button
          class="btn btn-primary btn-sm"
          disabled={!input().trim() || sending() || autoSend()}
          onClick={handleSend}
          title="Shift + Enter"
        >
          发送
        </button>
      </div>
    </div>
  );
}
