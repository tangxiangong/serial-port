import type { FormattedRecord, ModbusFrame } from "./commands";

export type DisplayFormat = "hex" | "ascii" | "dec";
export type RecordKind = "read" | "write" | "system";

export interface RecordPayload {
  hex: string;
  ascii: string;
  dec: string;
  raw: number[];
  timestamp: string;
}

export interface RecordItem extends RecordPayload {
  recordType: RecordKind;
  display: DisplayFormat;
  modbus?: ModbusFrame | null;
}

export function createRecordItem(
  payload: RecordPayload,
  recordType: RecordKind,
  display: DisplayFormat,
  modbus?: ModbusFrame | null,
): RecordItem {
  return {
    ...payload,
    recordType,
    display,
    modbus,
  };
}

export function recordItemFromFormattedRecord(
  record: FormattedRecord,
  display: DisplayFormat,
): RecordItem {
  return createRecordItem(
    {
      hex: record.hex,
      ascii: record.ascii,
      dec: record.dec,
      raw: [],
      timestamp: record.timestamp,
    },
    record.recordType,
    display,
  );
}

export function getRecordDisplayData(record: RecordItem) {
  switch (record.display) {
    case "hex":
      return record.hex;
    case "ascii":
      return record.ascii;
    case "dec":
      return record.dec;
  }
}
