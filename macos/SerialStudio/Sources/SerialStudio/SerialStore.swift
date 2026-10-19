import Foundation
import SwiftUI

@MainActor
final class SerialStore: ObservableObject {
    @Published var preferences = Preferences.load() { didSet { preferences.save() } }
    @Published private(set) var ports: [PortInfo] = []
    @Published private(set) var status = SerialStatus() { didSet { UpdateActivity.setConnected(status.connected) } }
    @Published private(set) var frames: [SerialFrame] = []
    @Published private(set) var busy = false
    @Published private(set) var sending = false
    @Published private(set) var refreshing = false
    @Published private(set) var version = ""
    @Published var error: String?
    @Published var input = ""
    private let core = RustCore()
    private var started = false
    private var connectionRevision = 0
    private var retainedBytes = 0

    var locked: Bool { status.connected || busy }
    var canSend: Bool { status.connected && !sending && !input.isEmpty && !busy }
    var selectedPort: PortInfo? { ports.first { $0.path == preferences.selectedPath } }
    var colorScheme: ColorScheme? { preferences.appearance == "dark" ? .dark : preferences.appearance == "light" ? .light : nil }

    func start() async {
        guard !started else { return }
        started = true
        do {
            version = await core.version()
            await refreshPorts()
            var tick = 0
            while !Task.isCancelled {
                let revision = connectionRevision
                let changingConnection = busy
                let batch = try await core.poll()
                if !changingConnection && !busy && revision == connectionRevision && status != batch.status {
                    if let failure = batch.status.lastError, failure != status.lastError { error = failure }
                    status = batch.status
                }
                if !batch.frames.isEmpty { append(batch.frames) }
                if batch.droppedFrames > 0 { error = "接收速度超过显示缓冲，已丢弃 \(batch.droppedFrames) 条旧记录；字节统计保持完整。" }
                tick += 1
                if tick % 10 == 0 { await refreshPorts(silent: true) }
                try await Task.sleep(for: .milliseconds(100))
            }
        } catch is CancellationError {
            _ = try? await core.disconnect()
        } catch { self.error = error.localizedDescription }
    }

    func refreshPorts(silent: Bool = false) async {
        guard !refreshing else { return }
        refreshing = true
        defer { refreshing = false }
        do {
            let found = try await core.ports()
            if found != ports { ports = found }
            if !locked && !found.contains(where: { $0.path == preferences.selectedPath }) {
                let selected = found.first?.path ?? ""
                if preferences.selectedPath != selected { preferences.selectedPath = selected }
            }
        } catch { if !silent { self.error = error.localizedDescription } }
    }

    func toggleConnection() async {
        guard !busy else { return }
        busy = true
        connectionRevision += 1
        let revision = connectionRevision
        defer { if revision == connectionRevision { busy = false } }
        do {
            error = nil
            let result: SerialStatus = status.connected
                ? try await core.disconnect()
                : try await core.connect(path: preferences.selectedPath, config: preferences.config)
            if revision == connectionRevision { status = result }
        } catch { self.error = error.localizedDescription }
    }

    func send() async {
        guard canSend else { return }
        sending = true
        defer { sending = false }
        do { try await core.send(input: input, format: preferences.sendFormat, lineEnding: preferences.lineEnding) }
        catch { self.error = error.localizedDescription }
    }

    func clear() { frames.removeAll(keepingCapacity: true); retainedBytes = 0 }

    private func append(_ incoming: [SerialFrame]) {
        frames.append(contentsOf: incoming)
        retainedBytes += incoming.reduce(0) { $0 + $1.memoryCost }
        var removeCount = 0
        while frames.count - removeCount > 2000 || retainedBytes > 8 * 1024 * 1024 {
            retainedBytes -= frames[removeCount].memoryCost
            removeCount += 1
        }
        if removeCount > 0 { frames.removeFirst(removeCount) }
    }

    func shutdown() async {
        busy = true
        connectionRevision += 1
        if let result = try? await core.disconnect() { status = result }
    }
}
