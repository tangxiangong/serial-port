import SwiftUI

struct ComposerView: View {
    @ObservedObject var serial: SerialStore

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            TextEditor(text: $serial.input)
                .font(.system(size: 13, design: .monospaced))
                .frame(height: 80)
                .border(.quaternary)
                .overlay(alignment: .topLeading) {
                    if serial.input.isEmpty {
                        Text(serial.preferences.sendFormat == .hex ? "十六进制数据，例如 AA 01 FF" : "输入文本或设备指令…")
                            .font(.system(size: 13, design: .monospaced)).foregroundStyle(.tertiary)
                            .lineLimit(2)
                            .padding(.horizontal, 5).padding(.vertical, 1)
                            .allowsHitTesting(false).accessibilityHidden(true)
                    }
                }
                .accessibilityLabel("要发送的数据")
                .help(serial.preferences.sendFormat == .hex ? "HEX 按原始字节发送，不追加行尾。" : "⌘Return 发送 · Return 换行")
            HStack(spacing: 12) {
                Picker("编码", selection: $serial.preferences.sendFormat) {
                    ForEach(DataFormat.allCases) { Text($0.title).tag($0) }
                }
                .frame(width: 136)
                Picker("行尾", selection: $serial.preferences.lineEnding) {
                    Text("无").tag("none")
                    Text("LF").tag("lf")
                    Text("CR").tag("cr")
                    Text("CRLF").tag("crlf")
                }
                .frame(width: 120)
                .disabled(serial.preferences.sendFormat == .hex)
                Spacer(minLength: 0)
                Button { Task { await serial.send() } } label: {
                    Text(serial.sending ? "发送中…" : "发送")
                }
                .disabled(!serial.canSend)
                .fixedSize()
                .help("发送数据（⌘Return）")
            }
        }
        .controlSize(.small)
        .padding(12)
    }
}
