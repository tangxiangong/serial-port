import { describe, expect, test } from "bun:test";
import {
  getNextActivePortId,
  getSerialConnectionSummary,
  isPortConnected,
} from "./port-utils";

describe("port-utils", () => {
  test("detects whether the selected port is already connected", () => {
    expect(isPortConnected(["COM3", "COM7"], "COM7")).toBe(true);
    expect(isPortConnected(["COM3", "COM7"], "COM5")).toBe(false);
  });

  test("falls back to the next connected port when the active one is closed", () => {
    expect(getNextActivePortId("COM7", ["COM3", "COM7"], "COM7")).toBe("COM3");
    expect(getNextActivePortId("COM7", ["COM7"], "COM7")).toBe("");
  });

  test("builds a compact serial connection summary for the UI", () => {
    expect(getSerialConnectionSummary([])).toBe("未连接");
    expect(getSerialConnectionSummary(["COM3"])).toBe("1 个串口已连接");
    expect(getSerialConnectionSummary(["COM3", "COM7"])).toBe("2 个串口已连接");
  });
});
