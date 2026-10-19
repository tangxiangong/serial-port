import SwiftUI

struct ContentView: View {
    @ObservedObject var serial: SerialStore

    var body: some View {
        NavigationSplitView {
            ConnectionView(serial: serial).navigationSplitViewColumnWidth(min: 240, ideal: 270, max: 320)
        } detail: {
            VStack(spacing: 0) {
                if let error = serial.error {
                    HStack(alignment: .top) {
                        Label(error, systemImage: "exclamationmark.triangle").textSelection(.enabled)
                        Spacer(minLength: 8)
                        Button { serial.error = nil } label: { Image(systemName: "xmark") }
                            .buttonStyle(.plain).help("关闭错误提示").accessibilityLabel("关闭错误提示")
                    }.padding(12).background(.orange.opacity(0.12))
                }
                TerminalView(serial: serial)
                Divider()
                ComposerView(serial: serial)
                Divider()
                HStack(spacing: 16) {
                    Label(serial.status.connected ? "已连接" : "未连接", systemImage: serial.status.connected ? "checkmark.circle.fill" : "circle")
                        .foregroundStyle(serial.status.connected ? .green : .secondary)
                    Text(serial.status.path ?? "选择串口设备").lineLimit(1).truncationMode(.middle)
                    Spacer(minLength: 8)
                    Text("RX \(serial.status.rxBytes.formatted()) B")
                    Text("TX \(serial.status.txBytes.formatted()) B")
                }.font(.caption.monospacedDigit()).padding(.horizontal, 16).padding(.vertical, 8)
            }
            .navigationTitle("Serial Studio")
            .navigationSubtitle(serial.status.connected ? serial.preferences.config.summary : "串口工作台")
            .toolbar {
                ToolbarItem {
                    SettingsLink { Image(systemName: "gearshape") }.help("设置与更新").accessibilityLabel("设置与更新")
                }
            }
        }
        .frame(minWidth: 900, minHeight: 620)
    }
}
