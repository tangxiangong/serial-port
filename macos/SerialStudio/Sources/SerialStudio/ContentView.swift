import SwiftUI

struct ContentView: View {
    @ObservedObject var serial: SerialStore
    @SceneStorage("deviceSidebarVisible") private var showsDeviceSidebar = true
    @SceneStorage("connectionInspectorVisible") private var showsConnectionInspector = true
    @State private var showsInspectorPopover = false

    var body: some View {
        GeometryReader { geometry in
            let inspectorBinding = inspectorVisibility(width: geometry.size.width)
            panes(width: geometry.size.width, topInset: geometry.safeAreaInsets.top)
                .frame(width: geometry.size.width)
                .clipped()
                .navigationTitle("Serial Studio")
                .navigationSubtitle(serial.status.connected
                    ? "\(serial.status.path ?? "") · \((serial.status.config ?? serial.preferences.config).summary)"
                    : serial.selectedPort?.displayName ?? "串口终端")
                .toolbar {
                    ToolbarItem(placement: .navigation) {
                        Button {
                            showsDeviceSidebar.toggle()
                        } label: {
                            Label("设备侧栏", systemImage: "sidebar.leading")
                        }
                        .help(showsDeviceSidebar ? "隐藏设备侧栏（⌥⌘S）" : "显示设备侧栏（⌥⌘S）")
                        .accessibilityValue(showsDeviceSidebar ? "已显示" : "已隐藏")
                    }
                    ToolbarItem(placement: .primaryAction) {
                        Button {
                            Task { await serial.toggleConnection() }
                        } label: {
                            Label(serial.busy ? "正在处理…" : serial.status.connected ? "断开连接" : "连接",
                                  systemImage: serial.status.connected ? "stop.circle" : "play.fill")
                        }
                        .labelStyle(.titleAndIcon)
                        .disabled(serial.busy || (!serial.status.connected && serial.preferences.selectedPath.isEmpty))
                        .help(serial.status.connected ? "断开连接（⌘K）" : "连接所选串口（⌘K）")
                    }
                    ToolbarItem(placement: .primaryAction) {
                        Button {
                            inspectorBinding.wrappedValue.toggle()
                        } label: {
                            Label("连接设置", systemImage: "sidebar.trailing")
                        }
                        .help(inspectorBinding.wrappedValue ? "隐藏连接设置（⌥⌘I）" : "显示连接设置（⌥⌘I）")
                        .accessibilityValue(inspectorBinding.wrappedValue ? "已显示" : "已隐藏")
                        .popover(isPresented: $showsInspectorPopover) {
                            ConnectionInspectorView(serial: serial)
                                .frame(width: 300, height: min(560, geometry.size.height))
                        }
                    }
                }
                .focusedSceneValue(\.serialConnectionInspector, inspectorBinding)
                .onChange(of: geometry.size.width >= 1100) { _, wide in
                    if wide { showsInspectorPopover = false }
                }
        }
        .frame(minWidth: 840, minHeight: 600)
        .focusedSceneValue(\.serialDeviceSidebar, $showsDeviceSidebar)
    }

    private func inspectorVisibility(width: CGFloat) -> Binding<Bool> {
        width >= 1100 ? $showsConnectionInspector : $showsInspectorPopover
    }

    // HSplitView avoids the NavigationSplitView/inspector constraint cycle.
    // Forward the title-bar inset to each AppKit-hosted column.
    private func panes(width: CGFloat, topInset: CGFloat) -> some View {
        HSplitView {
            if showsDeviceSidebar {
                ConnectionView(serial: serial)
                    .padding(.top, topInset)
                    .frame(minWidth: 190, idealWidth: 220, maxWidth: 260, maxHeight: .infinity)
            }
            workspace.padding(.top, topInset)
                .frame(minWidth: 420, maxWidth: .infinity, maxHeight: .infinity)
                .layoutPriority(1)
            if showsConnectionInspector && width >= 1100 {
                ConnectionInspectorView(serial: serial)
                    .padding(.top, topInset)
                    .frame(minWidth: 260, idealWidth: 280, maxWidth: 300, maxHeight: .infinity)
            }
        }
    }

    private var workspace: some View {
        VStack(spacing: 0) {
            if let error = serial.error {
                errorBanner(error)
                Divider()
            }
            TerminalView(serial: serial)
            Divider()
            ComposerView(serial: serial)
            Divider()
            statusBar
        }
    }

    private var statusBar: some View {
        HStack(spacing: 16) {
            Label(serial.status.connected ? "已连接" : "未连接",
                  systemImage: serial.status.connected ? "circle.inset.filled" : "circle")
                .foregroundStyle(serial.status.connected ? .green : .secondary)
            Spacer(minLength: 8)
            Text("RX \(serial.status.rxBytes.formatted()) B")
            Text("TX \(serial.status.txBytes.formatted()) B")
        }
        .font(.system(size: 11).monospacedDigit())
        .foregroundStyle(.secondary)
        .padding(.horizontal, 16)
        .padding(.vertical, 7)
        .accessibilityElement(children: .combine)
    }

    private func errorBanner(_ message: String) -> some View {
        HStack(alignment: .top, spacing: 10) {
            Image(systemName: "exclamationmark.triangle.fill")
                .foregroundStyle(.orange)
                .accessibilityHidden(true)
            Text(message).textSelection(.enabled).frame(maxWidth: .infinity, alignment: .leading)
            Button { serial.error = nil } label: {
                Label("关闭错误提示", systemImage: "xmark").labelStyle(.iconOnly)
            }
            .buttonStyle(.borderless)
            .help("关闭错误提示")
        }
        .font(.callout)
        .padding(12)
        .background(.orange.opacity(0.08))
    }
}

private struct SerialDeviceSidebarKey: FocusedValueKey {
    typealias Value = Binding<Bool>
}

private struct SerialConnectionInspectorKey: FocusedValueKey {
    typealias Value = Binding<Bool>
}

private struct SerialSearchRequestKey: FocusedValueKey {
    typealias Value = Binding<Bool>
}

extension FocusedValues {
    var serialDeviceSidebar: Binding<Bool>? {
        get { self[SerialDeviceSidebarKey.self] }
        set { self[SerialDeviceSidebarKey.self] = newValue }
    }

    var serialConnectionInspector: Binding<Bool>? {
        get { self[SerialConnectionInspectorKey.self] }
        set { self[SerialConnectionInspectorKey.self] = newValue }
    }

    var serialSearchRequested: Binding<Bool>? {
        get { self[SerialSearchRequestKey.self] }
        set { self[SerialSearchRequestKey.self] = newValue }
    }
}
