import { invoke } from "@tauri-apps/api/core";

// ── Types ──────────────────────────────────────────────────────
export interface PortInfo {
  path: string;
  vid: string | null;
  pid: string | null;
  deviceName: string | null;
}

export type Parity = "none" | "odd" | "even";
export type FlowControl = "none" | "software" | "hardware";

export interface SerialConfig {
  baudRate: number;
  dataBits: number;
  stopBits: number;
  parity: Parity;
  flowControl: FlowControl;
}

export interface BleDeviceInfo {
  id: string;
  name: string | null;
  rssi: number | null;
}

export interface BleServiceConfig {
  serviceUuid: string;
  notifyCharacteristicUuid: string;
  writeCharacteristicUuid: string;
}

export interface FormattedRecord {
  recordType: "read" | "write" | "system";
  hex: string;
  ascii: string;
  dec: string;
  timestamp: string;
}

export interface ModbusFrame {
  slaveId: number;
  functionCode: number;
  functionName: string;
  data: number[];
  crcValid: boolean;
  raw: number[];
}

export interface StatsSnapshot {
  portId: string;
  rxBytesPerSec: number;
  txBytesPerSec: number;
  rxFramesPerSec: number;
  txFramesPerSec: number;
  errorCount: number;
}

export type SendFormat = "hex" | "ascii" | "dec";
export type LineEnding = "None" | "LF" | "CR" | "CRLF" | { Custom: string };
export type CheckAlgorithm = "none" | "checksum" | "parity" | "xor" | "modbus_crc16";
export type DeviceType = "serial" | "ble";
export type RecordType = "read" | "write" | "system";
export type LogFormat = "raw" | "hex" | "csv";
export type ValidateFormat = "hex" | "ascii" | "dec";

// ── Serial ─────────────────────────────────────────────────────
export const listSerialPorts = () => invoke<PortInfo[]>("list_serial_ports");

export const openSerialPort = (portId: string, path: string, config: SerialConfig) =>
  invoke<void>("open_serial_port", { portId, path, config });

export const closeSerialPort = (portId: string) =>
  invoke<void>("close_serial_port", { portId });

export const closeAllSerialPorts = () =>
  invoke<void>("close_all_serial_ports");

export const getOpenPorts = () => invoke<string[]>("get_open_ports");

// ── BLE ────────────────────────────────────────────────────────
export const bleScan = (serviceUuid: string, duration: number) =>
  invoke<BleDeviceInfo[]>("ble_scan", { serviceUuid, duration });

export const bleConnect = (deviceId: string, serviceConfig: BleServiceConfig) =>
  invoke<void>("ble_connect", { deviceId, serviceConfig });

export const bleDisconnect = () => invoke<void>("ble_disconnect");

export const bleWrite = (data: number[]) => invoke<void>("ble_write", { data });

// ── Send ───────────────────────────────────────────────────────
export const sendData = (
  portId: string,
  input: string,
  format: SendFormat,
  lineEnding: LineEnding,
  checkAlgorithm: CheckAlgorithm,
  deviceType: DeviceType
) => invoke<void>("send_data", { portId, input, format, lineEnding, checkAlgorithm, deviceType });

export const formatHexInput = (input: string) =>
  invoke<string>("format_hex_input_cmd", { input });

export const validateInput = (input: string, format: ValidateFormat) =>
  invoke<boolean>("validate_input", { input, format });

// ── Records ────────────────────────────────────────────────────
export const getRecords = (portId: string) =>
  invoke<FormattedRecord[]>("get_records", { portId });

export const clearRecords = (portId: string, recordType?: RecordType) =>
  invoke<void>("clear_records", { portId, recordType });

export const exportRecords = (portId: string) =>
  invoke<string>("export_records", { portId });

export const getRecordCounts = (portId: string) =>
  invoke<{ rx: number; tx: number }>("get_record_counts", { portId });

// ── Modbus ─────────────────────────────────────────────────────
export const parseModbusRtu = (data: number[]) =>
  invoke<ModbusFrame>("parse_modbus_rtu", { data });

export const parseModbusAscii = (data: number[]) =>
  invoke<ModbusFrame>("parse_modbus_ascii", { data });

// ── Stats ──────────────────────────────────────────────────────
export const getCurrentStats = (portId: string) =>
  invoke<StatsSnapshot>("get_current_stats", { portId });

// ── File Logger ────────────────────────────────────────────────
export const startFileLogging = (portId: string, path: string, format: LogFormat) =>
  invoke<void>("start_file_logging", { portId, path, format });

export const stopFileLogging = (portId: string) =>
  invoke<void>("stop_file_logging", { portId });

// ── Bridge ─────────────────────────────────────────────────────
export const startBridge = (portAId: string, portBId: string) =>
  invoke<void>("start_bridge", { portAId, portBId });

export const stopBridge = () => invoke<void>("stop_bridge");
