import { invoke } from "@tauri-apps/api/core";

export type Parity = "none" | "odd" | "even";
export type FlowControl = "none" | "software" | "hardware";
export type SendFormat = "hex" | "ascii" | "gbk" | "utf-8";
export const ENCODING_OPTIONS: { value: SendFormat; label: string }[] = [
  { value: "hex", label: "HEX" },
  { value: "ascii", label: "ASCII" },
  { value: "gbk", label: "GBK" },
  { value: "utf-8", label: "UTF-8" },
];
export type LineEnding = "none" | "lf" | "cr" | "crlf";

export interface SerialConfig {
  baudRate: number;
  dataBits: number;
  stopBits: number;
  parity: Parity;
  flowControl: FlowControl;
}

export interface PortInfo {
  path: string;
  displayName: string;
  manufacturer: string | null;
  serialNumber: string | null;
  vid: string | null;
  pid: string | null;
}

export interface SerialStatus {
  connected: boolean;
  path: string | null;
  config: SerialConfig | null;
  rxBytes: number;
  txBytes: number;
  openedAt: string | null;
  lastError: string | null;
}

export const listSerialPorts = () => invoke<PortInfo[]>("list_serial_ports");

export const connectSerial = (path: string, config: SerialConfig) =>
  invoke<SerialStatus>("connect_serial", { path, config });

export const disconnectSerial = () => invoke<SerialStatus>("disconnect_serial");

export const sendSerial = (
  input: string,
  format: SendFormat,
  lineEnding: LineEnding,
) => invoke<void>("send_serial", { input, format, lineEnding });

export const getSerialStatus = () => invoke<SerialStatus>("get_serial_status");
