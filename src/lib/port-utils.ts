export function isPortConnected(openPortIds: string[], portId: string) {
  return portId.length > 0 && openPortIds.includes(portId);
}

export function getNextActivePortId(
  currentActivePortId: string,
  openPortIds: string[],
  closingPortId: string,
) {
  if (currentActivePortId !== closingPortId) {
    return currentActivePortId;
  }

  const next = openPortIds.find((portId) => portId !== closingPortId);
  return next ?? "";
}

export function getSerialConnectionSummary(openPortIds: string[]) {
  const count = openPortIds.length;

  if (count === 0) return "未连接";
  if (count === 1) return "1 个串口已连接";
  return `${count} 个串口已连接`;
}
