import { describe, expect, test } from "bun:test";
import { createRecordItem } from "./record-utils";

describe("record-utils", () => {
  test("creates read records with the requested default display format", () => {
    const item = createRecordItem(
      {
        hex: "0x41, 0x54",
        ascii: "AT",
        dec: "65 84",
        raw: [0x41, 0x54],
        timestamp: "2026-04-10 12:00:00.000",
      },
      "read",
      "ascii",
    );

    expect(item.recordType).toBe("read");
    expect(item.display).toBe("ascii");
    expect(item.ascii).toBe("AT");
  });

  test("creates write records without mutating the payload", () => {
    const item = createRecordItem(
      {
        hex: "0x01, 0x02",
        ascii: "..",
        dec: "1 2",
        raw: [1, 2],
        timestamp: "2026-04-10 12:00:00.000",
      },
      "write",
      "hex",
    );

    expect(item.recordType).toBe("write");
    expect(item.raw).toEqual([1, 2]);
    expect(item.display).toBe("hex");
  });
});
