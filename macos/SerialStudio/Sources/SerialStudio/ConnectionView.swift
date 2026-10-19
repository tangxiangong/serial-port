import SwiftUI

struct ConnectionView: View {
    @ObservedObject var serial: SerialStore
    private let baudRates = [1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600]

    var body: some View {
        VStack(spacing: 0) {
            Form {
                Section("串口设备") {
                    Picker("设备", selection: $serial.preferences.selectedPath) {
                        if serial.preferences.selectedPath.isEmpty { Text("未发现串口").tag("") }
                        if !serial.preferences.selectedPath.isEmpty && !serial.ports.contains(where: { $0.path == serial.preferences.selectedPath }) {
                            Text(serial.preferences.selectedPath + "（已移除）").tag(serial.preferences.selectedPath)
                        }
                        ForEach(serial.ports) { Text("\($0.displayName) · \($0.path)").tag($0.path) }
                    }.disabled(serial.locked || serial.ports.isEmpty)
                    if let port = serial.selectedPort {
                        LabeledContent("路径", value: port.path).textSelection(.enabled)
                        if let vid = port.vid, let pid = port.pid { LabeledContent("VID / PID", value: "\(vid) / \(pid)") }
                        if let manufacturer = port.manufacturer { LabeledContent("厂商", value: manufacturer) }
                    } else if serial.ports.isEmpty {
                        Text("接入 USB 串口设备后会自动出现在这里。").font(.caption).foregroundStyle(.secondary)
                    }
                    Button { Task { await serial.refreshPorts() } } label: { Label("刷新串口", systemImage: "arrow.clockwise") }
                        .disabled(serial.refreshing)
                }
                Section("通信参数") {
                    Picker("波特率", selection: $serial.preferences.config.baudRate) {
                        ForEach(baudRates, id: \.self) { Text(String($0)).tag($0) }
                    }
                    Picker("数据位", selection: $serial.preferences.config.dataBits) {
                        ForEach(5...8, id: \.self) { Text(String($0)).tag($0) }
                    }
                    Picker("停止位", selection: $serial.preferences.config.stopBits) {
                        Text("1").tag(1); Text("2").tag(2)
                    }
                    Picker("校验位", selection: $serial.preferences.config.parity) {
                        Text("无").tag("none"); Text("偶校验").tag("even"); Text("奇校验").tag("odd")
                    }
                    Picker("流控制", selection: $serial.preferences.config.flowControl) {
                        Text("无").tag("none"); Text("XON / XOFF").tag("software"); Text("RTS / CTS").tag("hardware")
                    }
                }.disabled(serial.locked)
            }.formStyle(.grouped)
            Divider()
            Button { Task { await serial.toggleConnection() } } label: {
                Label(serial.busy ? "正在处理…" : serial.status.connected ? "断开连接" : "连接串口", systemImage: serial.status.connected ? "eject" : "cable.connector")
                    .frame(maxWidth: .infinity)
            }
            .buttonStyle(.borderedProminent).controlSize(.large)
            .disabled(serial.busy || (!serial.status.connected && serial.preferences.selectedPath.isEmpty))
            .padding(16)
        }
        .navigationTitle("设备与配置")
    }
}
