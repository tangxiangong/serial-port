import Foundation
import SerialStudioCore

enum NativeError: LocalizedError {
    case operation(String)
    var errorDescription: String? { if case .operation(let text) = self { return text }; return nil }
}

/// All transport calls use generated UniFFI records on a background actor.
actor RustCore {
    func version() -> String { SerialStudioCore.version() }
    func ports() throws -> [PortInfo] {
        try translated { try SerialStudioCore.ports() }.map {
            PortInfo(path: $0.path, displayName: $0.displayName, manufacturer: $0.manufacturer, serialNumber: $0.serialNumber, vid: $0.vid, pid: $0.pid)
        }
    }
    func poll() throws -> PollBatch {
        let batch = try translated { try SerialStudioCore.poll() }
        return PollBatch(status: status(batch.status), frames: batch.frames.map {
            SerialFrame(direction: $0.direction == .receive ? "receive" : "send", bytes: Array($0.bytes), hex: $0.hex,
                text: ["ascii": $0.text.ascii, "gbk": $0.text.gbk, "utf-8": $0.text.utf8], timestamp: $0.timestamp)
        }, droppedFrames: batch.droppedFrames)
    }
    func connect(path: String, config: SerialConfig) throws -> SerialStatus {
        guard let baudRate = UInt32(exactly: config.baudRate), let dataBits = UInt8(exactly: config.dataBits), let stopBits = UInt8(exactly: config.stopBits) else {
            throw NativeError.operation("串口参数超出范围")
        }
        return try connection { try SerialStudioCore.connect(path: path, config: SerialStudioCore.SerialConfig(
            baudRate: baudRate, dataBits: dataBits, stopBits: stopBits,
            parity: config.parity == "even" ? .even : config.parity == "odd" ? .odd : .none,
            flowControl: config.flowControl == "hardware" ? .hardware : config.flowControl == "software" ? .software : .none)) }
    }
    func disconnect() throws -> SerialStatus { try connection { try SerialStudioCore.disconnect() } }
    func send(input: String, format: DataFormat, lineEnding: String) throws {
        let encoding: SendFormat = switch format { case .hex: .hex; case .ascii: .ascii; case .gbk: .gbk; case .utf8: .utf8 }
        let ending: LineEnding = switch lineEnding { case "lf": .lf; case "cr": .cr; case "crlf": .crlf; default: .none }
        try translated { try SerialStudioCore.send(input: input, format: encoding, lineEnding: ending) }
    }
    private func translated<T>(_ operation: () throws -> T) throws -> T {
        do { return try UpdateActivity.perform(operation) }
        catch let error as CoreError { switch error { case .Operation(let message): throw NativeError.operation(message) } }
    }
    private func connection(_ operation: () throws -> SerialStudioCore.SerialStatus) throws -> SerialStatus {
        let result = try translated {
            let value = try operation()
            // Mark the live connection before the operation's busy counter is released.
            UpdateActivity.setConnected(value.connected)
            return value
        }
        return status(result)
    }
    private func status(_ value: SerialStudioCore.SerialStatus) -> SerialStatus {
        SerialStatus(connected: value.connected, path: value.path, rxBytes: value.rxBytes, txBytes: value.txBytes, openedAt: value.openedAt, lastError: value.lastError)
    }
}
