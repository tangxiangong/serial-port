import SwiftUI

struct TerminalView: View {
    @ObservedObject var serial: SerialStore

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("数据终端").font(.headline)
                Text("\(serial.frames.count) 条").font(.caption).foregroundStyle(.secondary)
                Spacer()
                Picker("显示编码", selection: $serial.preferences.displayFormat) {
                    ForEach(DataFormat.allCases) { Text($0.title).tag($0) }
                }.pickerStyle(.segmented).frame(maxWidth: 280)
                Button(action: serial.clear) { Image(systemName: "trash") }
                    .disabled(serial.frames.isEmpty).help("清空终端（⌘L）").accessibilityLabel("清空终端")
            }.padding(16)
            HStack {
                Text("↓ RX 接收    ↑ TX 发送").font(.caption).foregroundStyle(.secondary)
                Spacer()
                Toggle("自动滚动", isOn: $serial.preferences.autoScroll).toggleStyle(.switch).controlSize(.small)
            }.padding(.horizontal, 16).padding(.bottom, 12)
            Divider()
            if serial.frames.isEmpty {
                ContentUnavailableView(serial.status.connected ? "等待设备数据" : "连接串口后开始", systemImage: "cable.connector", description: Text("收发数据会实时显示在这里"))
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(alignment: .leading, spacing: 0) {
                            ForEach(serial.frames) { frame in
                                HStack(alignment: .top, spacing: 12) {
                                    VStack(alignment: .leading, spacing: 4) {
                                        Text(frame.isReceive ? "RX" : "TX").fontWeight(.semibold)
                                            .foregroundStyle(frame.isReceive ? .secondary : Color.accentColor)
                                        Text(frame.time).foregroundStyle(.secondary)
                                        Text("\(frame.bytes.count) B").foregroundStyle(.secondary)
                                    }.font(.caption.monospaced()).frame(width: 100, alignment: .leading)
                                    Text(frame.display(serial.preferences.displayFormat)).font(.system(.body, design: .monospaced))
                                        .textSelection(.enabled).frame(maxWidth: .infinity, alignment: .leading)
                                }.padding(.horizontal, 16).padding(.vertical, 10).id(frame.id)
                                Divider().padding(.leading, 128)
                            }
                        }
                        Color.clear.frame(height: 1).id("bottom")
                    }
                    .onChange(of: serial.frames.last?.id) { _, _ in if serial.preferences.autoScroll { proxy.scrollTo("bottom", anchor: .bottom) } }
                    .onChange(of: serial.preferences.autoScroll) { _, enabled in if enabled { proxy.scrollTo("bottom", anchor: .bottom) } }
                }
            }
        }
    }
}
