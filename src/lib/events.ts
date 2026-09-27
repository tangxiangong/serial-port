import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { SerialStatus } from "./commands";

export interface SerialFrameEvent {
  direction: "receive" | "send";
  bytes: number[];
  hex: string;
  text: Record<"ascii" | "gbk" | "utf-8", string>;
  timestamp: string;
}

export const onSerialFrame = (handler: (event: SerialFrameEvent) => void) =>
  listen<SerialFrameEvent>("serial:frame", (event) => handler(event.payload));

export const onSerialStatus = (handler: (event: SerialStatus) => void) =>
  listen<SerialStatus>("serial:status", (event) => handler(event.payload));

export const onPortsChanged = (handler: () => void): Promise<UnlistenFn> =>
  listen("serial:ports-changed", handler);
