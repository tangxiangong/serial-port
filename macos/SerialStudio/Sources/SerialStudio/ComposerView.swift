import SwiftUI

struct ComposerView: View {
    @ObservedObject var serial: SerialStore

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 12) {
                Text("发送数据").font(.headline).lineLimit(1)
                Spacer(minLength: 8)
                Picker("发送编码", selection: $serial.preferences.sendFormat) {
                    ForEach(DataFormat.allCases) { Text($0.title).tag($0) }
                }
                .labelsHidden().frame(width: 90).help("发送编码")
                Picker("行尾", selection: $serial.preferences.lineEnding) {
                    Text("无行尾").tag("none")
                    Text("LF").tag("lf")
                    Text("CR").tag("cr")
                    Text("CRLF").tag("crlf")
                }
                .labelsHidden().frame(width: 90).disabled(serial.preferences.sendFormat == .hex)
                .help("发送行尾")
            }
            TextEditor(text: $serial.input)
                .font(.system(size: 13, design: .monospaced))
                .frame(maxWidth: .infinity).frame(height: 74).padding(6)
                .background(.background, in: RoundedRectangle(cornerRadius: 6))
                .overlay(RoundedRectangle(cornerRadius: 6).strokeBorder(.quaternary))
                .overlay(alignment: .topLeading) {
                    if serial.input.isEmpty {
                        Text(serial.preferences.sendFormat == .hex ? "十六进制数据，例如 AA 01 FF" : "输入文本或设备指令…")
                            .font(.system(size: 13, design: .monospaced)).foregroundStyle(.tertiary)
                            .lineLimit(2)
                            .padding(.horizontal, 11).padding(.vertical, 8)
                            .allowsHitTesting(false).accessibilityHidden(true)
                    }
                }
                .accessibilityLabel("要发送的数据")
            HStack {
                Text(serial.preferences.sendFormat == .hex ? "HEX 按原始字节发送，不追加行尾。" : "⌘Return 发送 · Return 换行")
                    .font(.caption).foregroundStyle(.secondary)
                    .lineLimit(2).fixedSize(horizontal: false, vertical: true)
                Spacer()
                Button { Task { await serial.send() } } label: {
                    Label(serial.sending ? "发送中…" : "发送", systemImage: "arrow.up").frame(minWidth: 54)
                }
                .buttonStyle(.borderedProminent).disabled(!serial.canSend)
                .fixedSize()
                .help("发送数据（⌘Return）")
            }
        }
        .padding(16)
    }
}
