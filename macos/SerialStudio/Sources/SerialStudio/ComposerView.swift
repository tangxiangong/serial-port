import SwiftUI

struct ComposerView: View {
    @ObservedObject var serial: SerialStore
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("发送").font(.headline)
                Picker("发送编码", selection: $serial.preferences.sendFormat) {
                    ForEach(DataFormat.allCases) { Text($0.title).tag($0) }
                }.pickerStyle(.segmented).frame(maxWidth: 280)
                Spacer()
                Picker("行尾", selection: $serial.preferences.lineEnding) {
                    Text("无").tag("none"); Text("LF").tag("lf"); Text("CR").tag("cr"); Text("CRLF").tag("crlf")
                }.frame(width: 125).disabled(serial.preferences.sendFormat == .hex)
            }
            HStack(alignment: .bottom, spacing: 12) {
                TextEditor(text: $serial.input).font(.system(.body, design: .monospaced))
                    .frame(height: 80).padding(4).overlay(RoundedRectangle(cornerRadius: 6).stroke(.quaternary))
                    .accessibilityLabel("要发送的数据")
                Button { Task { await serial.send() } } label: { Label(serial.sending ? "发送中" : "发送", systemImage: "paperplane") }
                    .buttonStyle(.borderedProminent).disabled(!serial.canSend).help("发送数据（⌘↵）")
            }
        }.padding(16)
    }
}
