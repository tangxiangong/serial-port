import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface SerialFrameEvent {
  portId: string;
  raw: number[];
  hex: string;
  ascii: string;
  dec: string;
  timestamp: string;
  modbus?: unknown;
}

export interface BleFrameEvent {
  raw: number[];
  hex: string;
  ascii: string;
  dec: string;
  timestamp: string;
}

export interface StatsEvent {
  portId: string;
  rxBytesPerSec: number;
  txBytesPerSec: number;
  rxFramesPerSec: number;
  txFramesPerSec: number;
  errorCount: number;
}

export interface PortAddedEvent {
  path: string;
}

export interface PortRemovedEvent {
  path: string;
}

export const onSerialFrame = (
  handler: (e: SerialFrameEvent) => void,
): Promise<UnlistenFn> =>
  listen<SerialFrameEvent>("serial:frame", (event) => handler(event.payload));

export const onSerialDisconnect = (
  handler: (e: { portId: string; reason: string }) => void,
): Promise<UnlistenFn> =>
  listen("serial:disconnect", (event) => handler(event.payload as any));

export const onSerialPortAdded = (
  handler: (e: PortAddedEvent) => void,
): Promise<UnlistenFn> =>
  listen<PortAddedEvent>(
    "serial:port_added",
    (event) => handler(event.payload),
  );

export const onSerialPortRemoved = (
  handler: (e: PortRemovedEvent) => void,
): Promise<UnlistenFn> =>
  listen<PortRemovedEvent>(
    "serial:port_removed",
    (event) => handler(event.payload),
  );

export const onSerialStats = (
  handler: (e: StatsEvent) => void,
): Promise<UnlistenFn> =>
  listen<StatsEvent>("serial:stats", (event) => handler(event.payload));

export const onBleFrame = (
  handler: (e: BleFrameEvent) => void,
): Promise<UnlistenFn> =>
  listen<BleFrameEvent>("ble:frame", (event) => handler(event.payload));

export const onBleDisconnect = (
  handler: (e: { reason: string }) => void,
): Promise<UnlistenFn> =>
  listen("ble:disconnect", (event) => handler(event.payload as any));

export const onBridgeData = (
  handler: (e: { direction: string; data: string; timestamp: string }) => void,
): Promise<UnlistenFn> =>
  listen("bridge:data", (event) => handler(event.payload as any));
