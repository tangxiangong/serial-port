import SwiftUI

struct SettingsView: View {
    @ObservedObject var serial: SerialStore
    @ObservedObject var updater: Updater

    var body: some View {
        TabView {
            Form {
                Section("外观") {
                    Picker("主题", selection: $serial.preferences.appearance) {
                        Text("跟随系统").tag("system")
                        Text("浅色").tag("light")
                        Text("深色").tag("dark")
                    }
                }
                Section("终端") {
                    Picker("默认显示编码", selection: $serial.preferences.displayFormat) {
                        ForEach(DataFormat.allCases) { Text($0.title).tag($0) }
                    }
                    Toggle("自动滚动到最新数据", isOn: $serial.preferences.autoScroll)
                }
            }
            .formStyle(.grouped)
            .tabItem { Label("通用", systemImage: "gearshape") }

            Form {
                Section {
                    LabeledContent("当前版本", value: serial.version)
                    Picker("更新通道", selection: $updater.channel) {
                        Text("稳定版").tag("stable")
                        Text("Beta").tag("beta")
                        Text("Alpha").tag("alpha")
                    }
                }
                Section {
                    Toggle("自动检查更新", isOn: $updater.automaticChecks)
                    Toggle("自动下载更新", isOn: $updater.automaticDownloads)
                }
                Section {
                    if let reason = updater.unavailableReason {
                        Text(reason).font(.callout).foregroundStyle(.secondary)
                    }
                    if serial.status.connected {
                        Label("断开串口后可检查和安装更新。", systemImage: "info.circle")
                            .font(.callout).foregroundStyle(.secondary)
                    }
                    HStack {
                        Link("发布公告", destination: URL(string: "https://github.com/tangxiangong/serial-port/releases")!)
                        Spacer()
                        Button("检查更新…", action: updater.check).disabled(!updater.canCheck)
                    }
                }
            }
            .formStyle(.grouped)
            .tabItem { Label("软件更新", systemImage: "arrow.triangle.2.circlepath") }
        }
        .frame(width: 500, height: 370)
        .padding(12)
    }
}
