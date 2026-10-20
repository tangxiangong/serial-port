import AppKit
import SwiftUI

struct TerminalView: View {
    @ObservedObject var serial: SerialStore
    @State private var searchText = ""
    @State private var direction = TerminalDirection.all
    @State private var searchRequested = false

    private var visibleFrames: [SerialFrame] {
        serial.frames.filter { frame in
            (direction == .all || frame.isReceive == (direction == .receive))
                && (searchText.isEmpty || frame.display(serial.preferences.displayFormat).localizedStandardContains(searchText))
        }
    }

    var body: some View {
        let frames = visibleFrames
        VStack(spacing: 0) {
            HStack(spacing: 12) {
                Text("数据终端").font(.headline)
                Text(searchText.isEmpty && direction == .all
                     ? "\(serial.frames.count) 条" : "\(frames.count) / \(serial.frames.count) 条")
                    .font(.caption.monospacedDigit()).foregroundStyle(.secondary)
                Spacer(minLength: 8)
                Picker("显示编码", selection: $serial.preferences.displayFormat) {
                    ForEach(DataFormat.allCases) { Text($0.title).tag($0) }
                }
                .labelsHidden().frame(width: 90)
            }
            .padding(.horizontal, 16).padding(.vertical, 12)
            HStack(spacing: 12) {
                Picker("数据方向", selection: $direction) {
                    ForEach(TerminalDirection.allCases) { Text($0.rawValue).tag($0) }
                }
                .labelsHidden().frame(width: 106)
                Spacer(minLength: 8)
                TerminalSearchField(text: $searchText, focusRequested: $searchRequested)
                    .frame(minWidth: 140, idealWidth: 180, maxWidth: 220)
                    .frame(height: 24)
                    .help("搜索数据（⌘F）")
            }
            .padding(.horizontal, 16).padding(.bottom, 12)
            Divider()
            columnHeadings
            Divider()
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 0) {
                        ForEach(Array(frames.enumerated()), id: \.element.id) { index, frame in
                            TerminalRow(frame: frame, format: serial.preferences.displayFormat)
                                .background(index.isMultiple(of: 2) ? Color.primary.opacity(0.025) : .clear)
                                .id(frame.id)
                        }
                        Color.clear.frame(height: 1).id("terminalBottom")
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                .background(.background)
                .overlay {
                    if serial.frames.isEmpty {
                        ContentUnavailableView {
                            Label(serial.status.connected ? "等待设备数据" : "准备连接",
                                  systemImage: serial.status.connected ? "waveform" : "terminal")
                        } description: {
                            Text(serial.status.connected ? "收到的数据会实时出现在这里，也可以在下方发送指令。" : "选择串口，确认连接设置，然后点击工具栏中的“连接”。")
                        }
                    } else if frames.isEmpty {
                        ContentUnavailableView {
                            Label("没有匹配的数据", systemImage: "magnifyingglass")
                        } description: {
                            Text("尝试其他搜索内容，或显示全部方向。")
                        } actions: {
                            Button("清除筛选") { searchText = ""; direction = .all }
                        }
                    }
                }
                .onChange(of: frames.last?.id) { _, _ in
                    if serial.preferences.autoScroll { proxy.scrollTo("terminalBottom", anchor: .bottom) }
                }
                .onChange(of: serial.preferences.autoScroll) { _, enabled in
                    if enabled { proxy.scrollTo("terminalBottom", anchor: .bottom) }
                }
                .clipped()
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .focusedSceneValue(\.serialSearchRequested, $searchRequested)
        .toolbar {
            ToolbarItemGroup {
                Toggle(isOn: $serial.preferences.autoScroll) {
                    Label("自动滚动", systemImage: "arrow.down.to.line")
                }
                .toggleStyle(.button)
                .help(serial.preferences.autoScroll ? "关闭自动滚动" : "滚动到最新数据")
                Button(action: serial.clear) {
                    Label("清空终端", systemImage: "trash")
                }
                .disabled(serial.frames.isEmpty)
                .help("清空终端（⌘L）")
            }
        }
    }

    private var columnHeadings: some View {
        HStack(spacing: 12) {
            Text("时间").frame(width: 94, alignment: .leading)
            Text("方向").frame(width: 36, alignment: .leading)
            Text("字节").frame(width: 44, alignment: .trailing)
            Text("数据").frame(maxWidth: .infinity, alignment: .leading)
        }
        .font(.system(size: 11)).foregroundStyle(.secondary)
        .padding(.horizontal, 16).padding(.vertical, 7)
        .accessibilityHidden(true)
    }
}

private enum TerminalDirection: String, CaseIterable, Identifiable {
    case all = "全部方向"
    case receive = "接收 RX"
    case send = "发送 TX"
    var id: Self { self }
}

private struct TerminalRow: View {
    let frame: SerialFrame
    let format: DataFormat

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            Text(frame.time).foregroundStyle(.secondary).frame(width: 94, alignment: .leading)
            Text(frame.isReceive ? "↓ RX" : "↑ TX")
                .foregroundStyle(frame.isReceive ? .secondary : Color.accentColor)
                .frame(width: 36, alignment: .leading)
            Text("\(frame.bytes.count)").foregroundStyle(.secondary).frame(width: 44, alignment: .trailing)
            Text(frame.display(format)).textSelection(.enabled)
                .fixedSize(horizontal: false, vertical: true)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .font(.system(size: 12, design: .monospaced))
        .padding(.horizontal, 16).padding(.vertical, 7)
        .accessibilityElement(children: .combine)
    }
}

private struct TerminalSearchField: NSViewRepresentable {
    @Binding var text: String
    @Binding var focusRequested: Bool

    func makeNSView(context: Context) -> NSSearchField {
        let field = NSSearchField()
        field.placeholderString = "搜索数据"
        field.sendsSearchStringImmediately = true
        field.sendsWholeSearchString = false
        field.delegate = context.coordinator
        field.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        field.setAccessibilityLabel("搜索数据")
        return field
    }

    func updateNSView(_ field: NSSearchField, context: Context) {
        context.coordinator.parent = self
        if field.stringValue != text { field.stringValue = text }
        if focusRequested, let window = field.window {
            window.makeFirstResponder(field)
            field.currentEditor()?.selectAll(nil)
            let request = $focusRequested
            DispatchQueue.main.async { request.wrappedValue = false }
        }
    }

    func sizeThatFits(_ proposal: ProposedViewSize, nsView: NSSearchField, context: Context) -> CGSize? {
        CGSize(width: proposal.width ?? 180, height: nsView.intrinsicContentSize.height)
    }

    func makeCoordinator() -> Coordinator { Coordinator(self) }

    final class Coordinator: NSObject, NSSearchFieldDelegate {
        var parent: TerminalSearchField
        init(_ parent: TerminalSearchField) { self.parent = parent }
        func controlTextDidChange(_ notification: Notification) {
            if let field = notification.object as? NSSearchField { parent.text = field.stringValue }
        }
    }
}
