export interface BridgeEntryLike {
  direction: string;
  data: string;
  timestamp: string;
}

export function getBridgeDirectionLabel(direction: string): "A→B" | "B→A" {
  return direction === "b_to_a" || direction === "B→A" ? "B→A" : "A→B";
}

export function partitionBridgeEntries(entries: BridgeEntryLike[]) {
  return entries.reduce(
    (groups, entry) => {
      const label = getBridgeDirectionLabel(entry.direction);
      const normalized = { ...entry, direction: label };

      if (label === "A→B") {
        groups.aToB.push(normalized);
      } else {
        groups.bToA.push(normalized);
      }

      return groups;
    },
    { aToB: [] as BridgeEntryLike[], bToA: [] as BridgeEntryLike[] },
  );
}
