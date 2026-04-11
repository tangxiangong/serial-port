import { describe, expect, test } from "bun:test";
import {
  getBridgeDirectionLabel,
  partitionBridgeEntries,
} from "./bridge-utils";

describe("bridge-utils", () => {
  test("normalizes backend bridge directions for display", () => {
    expect(getBridgeDirectionLabel("a_to_b")).toBe("A→B");
    expect(getBridgeDirectionLabel("b_to_a")).toBe("B→A");
  });

  test("preserves existing display labels", () => {
    expect(getBridgeDirectionLabel("A→B")).toBe("A→B");
    expect(getBridgeDirectionLabel("B→A")).toBe("B→A");
  });

  test("partitions entries by normalized direction", () => {
    const entries = [
      { direction: "a_to_b", data: "01 02", timestamp: "t1" },
      { direction: "B→A", data: "03 04", timestamp: "t2" },
      { direction: "b_to_a", data: "05 06", timestamp: "t3" },
    ];

    const result = partitionBridgeEntries(entries);

    expect(result.aToB).toHaveLength(1);
    expect(result.bToA).toHaveLength(2);
    expect(result.bToA.map((entry) => entry.data)).toEqual(["03 04", "05 06"]);
  });
});
