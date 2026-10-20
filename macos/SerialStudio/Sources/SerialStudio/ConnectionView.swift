import SwiftUI

struct ConnectionView: View {
    @ObservedObject var serial: SerialStore

    private var selection: Binding<String?> {
        Binding(
            get: { serial.preferences.selectedPath.isEmpty ? nil : serial.preferences.selectedPath },
            set: { if let path = $0, !serial.locked { serial.preferences.selectedPath = path } }
        )
    }

    var body: some View {
        List(selection: selection) {
            Section {
                ForEach(serial.ports) { port in
                    Label {
                        VStack(alignment: .leading, spacing: 3) {
                            Text(port.displayName).lineLimit(1).truncationMode(.middle)
                            if port.displayName != port.path.replacingOccurrences(of: "/dev/", with: "") {
                                Text(port.path.replacingOccurrences(of: "/dev/", with: ""))
                                    .font(.caption).foregroundStyle(.secondary)
                                    .lineLimit(1).truncationMode(.middle)
                            }
                        }
                    } icon: {
                        Image(systemName: "cable.connector").foregroundStyle(.secondary)
                    }
                    .tag(port.path)
                    .help(port.path)
                }
                if serial.status.connected, serial.selectedPort == nil, let path = serial.status.path {
                    Label {
                        VStack(alignment: .leading, spacing: 3) {
                            Text(path).lineLimit(1).truncationMode(.middle)
                            Text("设备已移除").font(.caption).foregroundStyle(.secondary)
                        }
                    } icon: {
                        Image(systemName: "exclamationmark.triangle")
                    }
                    .tag(path)
                }
            } header: {
                HStack {
                    Text("串口设备")
                    Spacer()
                    Text("\(serial.ports.count)").monospacedDigit()
                }
            }
        }
        .listStyle(.sidebar)
        .disabled(serial.locked)
        .overlay {
            if serial.ports.isEmpty && !serial.status.connected {
                ContentUnavailableView {
                    Label("未发现设备", systemImage: "cable.connector")
                } description: {
                    Text("接入串口设备后，\n会自动显示在这里。")
                }
            }
        }
        .safeAreaInset(edge: .bottom, spacing: 0) {
            VStack(spacing: 0) {
                Divider()
                HStack {
                    Button { Task { await serial.refreshPorts() } } label: {
                        Label("刷新", systemImage: "arrow.clockwise")
                    }
                    .disabled(serial.refreshing)
                    .help("刷新串口（⌘R）")
                    Spacer()
                    SettingsLink {
                        Label("设置", systemImage: "gearshape").labelStyle(.iconOnly)
                    }
                    .help("设置（⌘,）")
                }
                .buttonStyle(.borderless)
                .padding(12)
            }
        }
    }
}

struct ConnectionInspectorView: View {
    @ObservedObject var serial: SerialStore
    private let baudRates = [1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600]

    var body: some View {
        Form {
            Section("通信参数") {
                Picker("波特率", selection: $serial.preferences.config.baudRate) {
                    ForEach(baudRates, id: \.self) { Text(String($0)).tag($0) }
                }
                Picker("数据位", selection: $serial.preferences.config.dataBits) {
                    ForEach(5...8, id: \.self) { Text(String($0)).tag($0) }
                }
                Picker("停止位", selection: $serial.preferences.config.stopBits) {
                    Text("1").tag(1)
                    Text("2").tag(2)
                }
                Picker("校验位", selection: $serial.preferences.config.parity) {
                    Text("无").tag("none")
                    Text("偶校验").tag("even")
                    Text("奇校验").tag("odd")
                }
                Picker("流控制", selection: $serial.preferences.config.flowControl) {
                    Text("无").tag("none")
                    Text("XON / XOFF").tag("software")
                    Text("RTS / CTS").tag("hardware")
                }
            }
            .disabled(serial.locked)
            Section("设备信息") {
                if let port = serial.selectedPort {
                    detail("名称", value: port.displayName)
                    detail("路径", value: port.path)
                    if let manufacturer = port.manufacturer, !manufacturer.isEmpty {
                        detail("厂商", value: manufacturer)
                    }
                    if let vid = port.vid, let pid = port.pid {
                        detail("VID / PID", value: "\(vid) / \(pid)")
                    }
                    if let number = port.serialNumber, !number.isEmpty {
                        detail("序列号", value: number)
                    }
                } else {
                    Text(serial.status.connected ? "设备已移除，请断开后重新连接。" : "从左侧选择一个串口设备。")
                        .foregroundStyle(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
            if serial.locked {
                Section {
                    Label(serial.busy ? "正在更改连接状态…" : "断开连接后可修改通信参数", systemImage: "lock")
                        .font(.caption).foregroundStyle(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
        }
        .formStyle(.grouped)
        .controlSize(.small)
    }

    private func detail(_ title: String, value: String) -> some View {
        LabeledContent(title) {
            Text(value)
                .fixedSize(horizontal: false, vertical: true)
                .multilineTextAlignment(.trailing)
                .textSelection(.enabled)
        }
    }
}
