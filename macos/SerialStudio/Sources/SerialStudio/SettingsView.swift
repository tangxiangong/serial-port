import SwiftUI

struct SettingsView: View {
    @ObservedObject var serial: SerialStore
    @ObservedObject var updater: Updater
    var body: some View {
        Form {
            Section("外观") {
                Picker("主题", selection: $serial.preferences.appearance) {
                    Text("跟随系统").tag("system"); Text("浅色").tag("light"); Text("深色").tag("dark")
                }
            }
            Section("软件更新") {
                LabeledContent("当前版本", value: serial.version)
                Picker("更新通道", selection: $updater.channel) {
                    Text("稳定版").tag("stable"); Text("Beta").tag("beta"); Text("Alpha").tag("alpha")
                }
                Toggle("自动检查更新", isOn: $updater.automaticChecks)
                Toggle("自动下载更新", isOn: $updater.automaticDownloads)
                if let reason = updater.unavailableReason { Text(reason).foregroundStyle(.secondary) }
                if serial.status.connected { Text("断开串口后可检查和安装更新。").foregroundStyle(.secondary) }
                Button("检查更新…", action: updater.check).disabled(!updater.canCheck)
            }
            Section {
                Link("GitHub 与发布公告", destination: URL(string: "https://github.com/tangxiangong/serial-port/releases")!)
            }
        }.formStyle(.grouped).frame(width: 480).padding(8)
    }
}
