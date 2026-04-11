import { createSignal, Show } from "solid-js";
import {
  type CheckAlgorithm,
  formatHexInput,
  type LineEnding,
  sendData,
  type SendFormat,
} from "../lib/commands";
import {
  activePortId,
  deviceType,
  sendDisplayFormat,
  setSendDisplayFormat,
} from "../stores/ui";
import { FormatTextIcon } from "../assets/icons";

const CHECK_ALGORITHM_OPTIONS: { label: string; value: CheckAlgorithm }[] = [
  { label: "无", value: "none" },
  { label: "校验和", value: "checksum" },
  { label: "异或", value: "xor" },
  { label: "Modbus CRC16", value: "modbus_crc16" },
];

const FORMAT_OPTIONS: { label: string; value: SendFormat }[] = [
  { label: "HEX", value: "hex" },
  { label: "ASCII", value: "ascii" },
  { label: "DEC", value: "dec" },
];

const LINE_ENDING_OPTIONS: { label: string; value: LineEnding }[] = [
  { label: "无", value: "None" },
  { label: "CR", value: "CR" },
  { label: "LF", value: "LF" },
  { label: "CRLF", value: "CRLF" },
];

export default function SendTab() {
  const [input, setInput] = createSignal("");
  const [lineEnding, setLineEnding] = createSignal<LineEnding>("None");
  const [checkAlgorithm, setCheckAlgorithm] = createSignal<CheckAlgorithm>("none");
  const [autoSend, setAutoSend] = createSignal(false);
  const [autoInterval, setAutoInterval] = createSignal(1000);
  const [sending, setSending] = createSignal(false);
  let autoTimer: ReturnType<typeof setInterval> | undefined;

  const handleSend = async () => {
    if (!input().trim()) return;
    setSending(true);
    try {
      await sendData(
        activePortId(),
        input(),
        sendDisplayFormat(),
        lineEnding(),
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
      {/* Sub-toolbar */}
      <div class="flex items-center px-3 py-1.5 gap-2 border-b border-base-300 bg-base-200/50 text-xs flex-wrap">
        {/* Format toggle group */}
        <span class="opacity-50">格式</span>
        <div class="flex rounded overflow-hidden border border-base-300">
          {FORMAT_OPTIONS.map((opt) => (
            <button
              class="px-2 py-0.5 text-xs transition-colors"
              classList={{
                "bg-primary text-primary-content": sendDisplayFormat() === opt.value,
                "bg-base-100 hover:bg-base-200": sendDisplayFormat() !== opt.value,
              }}
              onClick={() => setSendDisplayFormat(opt.value)}
            >
              {opt.label}
            </button>
          ))}
        </div>

        <span class="opacity-30">|</span>

        {/* Line ending toggle group */}
        <span class="opacity-50">行尾</span>
        <div class="flex rounded overflow-hidden border border-base-300">
          {LINE_ENDING_OPTIONS.map((opt) => (
            <button
              class="px-2 py-0.5 text-xs transition-colors"
              classList={{
                "bg-primary text-primary-content": lineEnding() === opt.value,
                "bg-base-100 hover:bg-base-200": lineEnding() !== opt.value,
              }}
              onClick={() => setLineEnding(opt.value)}
            >
              {opt.label}
            </button>
          ))}
        </div>

        {/* Checksum — hex mode only */}
        <Show when={sendDisplayFormat() === "hex"}>
          <span class="opacity-30">|</span>
          <span class="opacity-50">校验</span>
          <select
            class="select select-xs select-bordered h-5 min-h-0 py-0 text-xs"
            value={checkAlgorithm()}
            onChange={(e) => setCheckAlgorithm(e.currentTarget.value as CheckAlgorithm)}
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
            <FormatTextIcon class="w-4 h-4" />
          </button>
        </Show>
      </div>

      {/* Input area */}
      <div class="flex-1 p-3 min-h-0">
        <textarea
          class="w-full h-full bg-base-200 border border-base-300 rounded-lg p-3 font-mono text-sm resize-none focus:outline-none focus:border-primary"
          value={input()}
          onInput={(e) => setInput(e.currentTarget.value)}
          onKeyDown={handleKeyDown}
          placeholder="输入要发送的数据..."
        />
      </div>

      {/* Action bar */}
      <div class="flex items-center px-3 py-2 gap-2 border-t border-base-300">
        {/* Timed send */}
        <label class="flex items-center gap-1.5 cursor-pointer select-none text-xs">
          <input
            type="checkbox"
            class="checkbox checkbox-xs"
            checked={autoSend()}
            onChange={toggleAutoSend}
          />
          定时发送
        </label>

        {/* Interval input */}
        <input
          type="number"
          class="input input-xs input-bordered w-16 text-center disable-spin-button"
          value={autoInterval()}
          onInput={(e) => setAutoInterval(parseInt(e.currentTarget.value) || 1000)}
          min="100"
          step="100"
          disabled={autoSend()}
        />
        <span class="text-xs opacity-60">ms</span>

        {/* Spacer */}
        <div class="flex-1" />

        {/* Clear button */}
        <button
          class="btn btn-ghost btn-sm"
          onClick={() => setInput("")}
          disabled={autoSend()}
        >
          清空
        </button>

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
