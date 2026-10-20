import AppKit
import SwiftUI

@main
struct SerialStudioApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var delegate
    @StateObject private var serial = SerialStore()
    @StateObject private var updater = Updater()

    var body: some Scene {
        Window("Serial Studio", id: "main") {
            ContentView(serial: serial)
                .preferredColorScheme(serial.colorScheme)
                .task { delegate.serial = serial; await serial.start() }
        }
        .defaultSize(width: 1180, height: 780)
        .windowResizability(.contentMinSize)
        .commands {
            SerialViewCommands()
            CommandGroup(after: .appInfo) {
                Button("检查更新…", action: updater.check).disabled(!updater.canCheck)
            }
            CommandMenu("串口") {
                Button(serial.status.connected ? "断开连接" : "连接串口") { Task { await serial.toggleConnection() } }
                    .keyboardShortcut("k").disabled(serial.busy || (!serial.status.connected && serial.preferences.selectedPath.isEmpty))
                Button("刷新串口") { Task { await serial.refreshPorts() } }.keyboardShortcut("r")
                Divider()
                Button("发送数据") { Task { await serial.send() } }.keyboardShortcut(.return).disabled(!serial.canSend)
                Button("清空终端", action: serial.clear).keyboardShortcut("l")
            }
        }
        Settings { SettingsView(serial: serial, updater: updater).preferredColorScheme(serial.colorScheme) }
    }
}

private struct SerialViewCommands: Commands {
    @FocusedBinding(\.serialSearchRequested) private var searchRequested

    var body: some Commands {
        CommandGroup(after: .textEditing) {
            Button("查找数据") { searchRequested = true }
                .keyboardShortcut("f")
                .disabled(searchRequested == nil)
        }
    }
}

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    weak var serial: SerialStore?
    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.regular)
        NSApp.activate(ignoringOtherApps: true)
    }
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { true }
    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        guard let serial else { return .terminateNow }
        Task { await serial.shutdown(); sender.reply(toApplicationShouldTerminate: true) }
        return .terminateLater
    }
}
