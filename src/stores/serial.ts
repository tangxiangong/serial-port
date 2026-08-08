import { createSignal } from "solid-js";
import { isTauri } from "@tauri-apps/api/core";
import {
  connectSerial,
  disconnectSerial,
  getSerialStatus,
  listSerialPorts,
  sendSerial,
  type LineEnding,
  type PortInfo,
  type SendFormat,
  type SerialConfig,
  type SerialStatus,
} from "../lib/commands";
import {
  onPortsChanged,
  onSerialFrame,
  onSerialStatus,
  type SerialFrameEvent,
} from "../lib/events";

const DEFAULT_CONFIG: SerialConfig = {
  baudRate: 115200,
  dataBits: 8,
  stopBits: 1,
  parity: "none",
  flowControl: "none",
};

const EMPTY_STATUS: SerialStatus = {
  connected: false,
  path: null,
  config: null,
  rxBytes: 0,
  txBytes: 0,
  openedAt: null,
  lastError: null,
};

export interface TerminalFrame extends SerialFrameEvent {
  id: number;
}

export const [ports, setPorts] = createSignal<PortInfo[]>([]);
export const [selectedPath, setSelectedPath] = createSignal("");
export const [serialConfig, setSerialConfig] =
  createSignal<SerialConfig>(loadConfig());
export const [serialStatus, setSerialStatus] =
  createSignal<SerialStatus>(EMPTY_STATUS);
export const [frames, setFrames] = createSignal<TerminalFrame[]>([]);
export const [isWorking, setIsWorking] = createSignal(false);
export const [isRefreshing, setIsRefreshing] = createSignal(false);
export const [appError, setAppError] = createSignal<string | null>(null);

let nextFrameId = 1;
let disposeListeners: Array<() => void> = [];

export async function initializeSerial() {
  if (!isTauri()) return;

  const listeners = await Promise.all([
    onSerialFrame(appendFrame),
    onSerialStatus((status) => {
      setSerialStatus(status);
      if (status.lastError) setAppError(status.lastError);
    }),
    onPortsChanged(() => void refreshPorts()),
  ]).catch(() => []);
  disposeListeners = listeners;

  await Promise.all([
    refreshPorts(),
    getSerialStatus()
      .then(setSerialStatus)
      .catch(() => undefined),
  ]);
}

export function disposeSerial() {
  disposeListeners.forEach((dispose) => dispose());
  disposeListeners = [];
}

export async function refreshPorts() {
  setIsRefreshing(true);
  try {
    const nextPorts = await listSerialPorts();
    setPorts(nextPorts);
    if (
      !serialStatus().connected &&
      !nextPorts.some((port) => port.path === selectedPath())
    ) {
      setSelectedPath(nextPorts[0]?.path ?? "");
    }
  } catch (error) {
    setPorts([]);
    setAppError(readError(error));
  } finally {
    setIsRefreshing(false);
  }
}

export function updateConfig(patch: Partial<SerialConfig>) {
  const next = { ...serialConfig(), ...patch };
  setSerialConfig(next);
  localStorage.setItem("serial-studio.config", JSON.stringify(next));
}

export async function toggleConnection() {
  if (isWorking()) return;
  setIsWorking(true);
  setAppError(null);
  try {
    const status = serialStatus().connected
      ? await disconnectSerial()
      : await connectSerial(selectedPath(), serialConfig());
    setSerialStatus(status);
  } catch (error) {
    setAppError(readError(error));
  } finally {
    setIsWorking(false);
  }
}

export async function writeSerial(
  input: string,
  format: SendFormat,
  lineEnding: LineEnding,
) {
  if (!serialStatus().connected) {
    setAppError("请先连接串口");
    return false;
  }
  setAppError(null);
  try {
    await sendSerial(input, format, lineEnding);
    return true;
  } catch (error) {
    setAppError(readError(error));
    return false;
  }
}

export function clearFrames() {
  setFrames([]);
}

export function dismissError() {
  setAppError(null);
}

function appendFrame(frame: SerialFrameEvent) {
  setFrames((current) => {
    const next = [...current, { ...frame, id: nextFrameId++ }];
    return next.length > 2000 ? next.slice(-2000) : next;
  });
  setSerialStatus((status) => ({
    ...status,
    rxBytes:
      status.rxBytes + (frame.direction === "receive" ? frame.bytes.length : 0),
    txBytes:
      status.txBytes + (frame.direction === "send" ? frame.bytes.length : 0),
  }));
}

function loadConfig(): SerialConfig {
  try {
    const saved = localStorage.getItem("serial-studio.config");
    return saved ? { ...DEFAULT_CONFIG, ...JSON.parse(saved) } : DEFAULT_CONFIG;
  } catch {
    return DEFAULT_CONFIG;
  }
}

function readError(error: unknown) {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "操作失败，请重试";
}
