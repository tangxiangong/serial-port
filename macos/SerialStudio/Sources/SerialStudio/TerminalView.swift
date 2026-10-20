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
                Picker("数据方向", selection: $direction) {
                    ForEach(TerminalDirection.allCases) { Text($0.rawValue).tag($0) }
                }
                .pickerStyle(.segmented)
                .labelsHidden()
                .frame(width: 164)
                Text(searchText.isEmpty && direction == .all
                     ? "\(serial.frames.count) 条" : "\(frames.count) / \(serial.frames.count) 条")
                    .font(.caption.monospacedDigit()).foregroundStyle(.secondary).lineLimit(1)
                Spacer(minLength: 0)
                Picker("显示编码", selection: $serial.preferences.displayFormat) {
                    ForEach(DataFormat.allCases) { Text($0.title).tag($0) }
                }
                .labelsHidden().frame(width: 90)
            }
            .controlSize(.small)
            .padding(.horizontal, 12).padding(.vertical, 8)
            .background(.bar)
            Divider()
            TerminalTable(frames: frames, format: serial.preferences.displayFormat, autoScroll: serial.preferences.autoScroll)
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
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .focusedSceneValue(\.serialSearchRequested, $searchRequested)
        .toolbar {
            ToolbarItem {
                TerminalSearchField(text: $searchText, focusRequested: $searchRequested)
                    .frame(width: 170)
                    .help("搜索数据（⌘F）")
            }
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
}

private enum TerminalDirection: String, CaseIterable, Identifiable {
    case all = "全部"
    case receive = "接收"
    case send = "发送"
    var id: Self { self }
}

// AppKit supplies column resizing, multiline rows, selection, copying and precise scrolling.
private struct TerminalTable: NSViewRepresentable {
    let frames: [SerialFrame]
    let format: DataFormat
    let autoScroll: Bool

    func makeNSView(context: Context) -> NSScrollView {
        let table = TerminalNSTableView()
        table.delegate = context.coordinator
        table.dataSource = context.coordinator
        table.style = .fullWidth
        table.usesAlternatingRowBackgroundColors = true
        table.usesAutomaticRowHeights = true
        table.rowHeight = 24
        table.allowsMultipleSelection = true
        table.columnAutoresizingStyle = .lastColumnOnlyAutoresizingStyle
        table.autoresizingMask = [.width]
        table.setAccessibilityLabel("串口数据终端")
        for (name, title, width, minimum) in [
            ("time", "时间", 104.0, 88.0),
            ("direction", "方向", 52.0, 44.0),
            ("size", "字节", 52.0, 40.0),
            ("data", "数据", 260.0, 120.0)
        ] {
            let column = NSTableColumn(identifier: NSUserInterfaceItemIdentifier(name))
            column.title = title
            column.width = width
            column.minWidth = minimum
            table.addTableColumn(column)
        }
        table.copySelection = { [weak coordinator = context.coordinator, weak table] in
            guard let coordinator, let table else { return }
            let text = table.selectedRowIndexes.compactMap { index -> String? in
                guard coordinator.frames.indices.contains(index) else { return nil }
                return coordinator.frames[index].display(coordinator.format)
            }.joined(separator: "\n")
            guard !text.isEmpty else { return }
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(text, forType: .string)
        }
        let scroll = NSScrollView()
        scroll.hasVerticalScroller = true
        scroll.hasHorizontalScroller = true
        scroll.autohidesScrollers = true
        scroll.documentView = table
        return scroll
    }

    func updateNSView(_ scroll: NSScrollView, context: Context) {
        guard let table = scroll.documentView as? NSTableView else { return }
        let coordinator = context.coordinator
        let changed = coordinator.frames.map(\.id) != frames.map(\.id) || coordinator.format != format
        let follow = autoScroll && (changed || !coordinator.autoScroll)
        if changed {
            let selection = Set(table.selectedRowIndexes.compactMap { index in
                coordinator.frames.indices.contains(index) ? coordinator.frames[index].id : nil
            })
            let topRow = table.row(at: NSPoint(x: 0, y: scroll.contentView.bounds.minY))
            let anchor = coordinator.frames.indices.contains(topRow) ? coordinator.frames[topRow].id : nil
            let offset = topRow >= 0 ? scroll.contentView.bounds.minY - table.rect(ofRow: topRow).minY : 0
            coordinator.frames = frames
            coordinator.format = format
            table.reloadData()
            table.selectRowIndexes(IndexSet(frames.indices.filter { selection.contains(frames[$0].id) }), byExtendingSelection: false)
            if !autoScroll, let anchor, let index = frames.firstIndex(where: { $0.id == anchor }) {
                table.layoutSubtreeIfNeeded()
                scroll.contentView.scroll(to: NSPoint(x: scroll.contentView.bounds.minX, y: max(0, table.rect(ofRow: index).minY + offset)))
                scroll.reflectScrolledClipView(scroll.contentView)
            }
        }
        coordinator.autoScroll = autoScroll
        if follow, !frames.isEmpty { table.scrollRowToVisible(frames.count - 1) }
    }

    func makeCoordinator() -> Coordinator { Coordinator() }

    final class Coordinator: NSObject, NSTableViewDataSource, NSTableViewDelegate {
        var frames: [SerialFrame] = []
        var format = DataFormat.utf8
        var autoScroll = false

        func numberOfRows(in tableView: NSTableView) -> Int { frames.count }

        func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
            guard frames.indices.contains(row), let column = tableColumn else { return nil }
            let cell: NSTableCellView
            if let reused = tableView.makeView(withIdentifier: column.identifier, owner: nil) as? NSTableCellView {
                cell = reused
            } else {
                cell = NSTableCellView()
                cell.identifier = column.identifier
                let text = NSTextField(wrappingLabelWithString: "")
                text.font = .monospacedSystemFont(ofSize: NSFont.smallSystemFontSize, weight: .regular)
                text.isSelectable = true
                text.translatesAutoresizingMaskIntoConstraints = false
                if column.identifier.rawValue != "data" { text.maximumNumberOfLines = 1 }
                if column.identifier.rawValue == "size" { text.alignment = .right }
                cell.addSubview(text)
                cell.textField = text
                NSLayoutConstraint.activate([
                    text.leadingAnchor.constraint(equalTo: cell.leadingAnchor, constant: 4),
                    text.trailingAnchor.constraint(equalTo: cell.trailingAnchor, constant: -4),
                    text.topAnchor.constraint(equalTo: cell.topAnchor, constant: 3),
                    text.bottomAnchor.constraint(equalTo: cell.bottomAnchor, constant: -3)
                ])
            }
            let frame = frames[row]
            switch column.identifier.rawValue {
            case "time": cell.textField?.stringValue = frame.time
            case "direction": cell.textField?.stringValue = frame.isReceive ? "↓ RX" : "↑ TX"
            case "size": cell.textField?.stringValue = String(frame.bytes.count)
            default: cell.textField?.stringValue = frame.display(format)
            }
            return cell
        }

        func tableViewColumnDidResize(_ notification: Notification) {
            guard let table = notification.object as? NSTableView else { return }
            table.noteHeightOfRows(withIndexesChanged: IndexSet(integersIn: 0..<frames.count))
        }
    }
}

private final class TerminalNSTableView: NSTableView {
    var copySelection: (() -> Void)?

    @objc func copy(_ sender: Any?) { copySelection?() }

    override func menu(for event: NSEvent) -> NSMenu? {
        let row = row(at: convert(event.locationInWindow, from: nil))
        if row >= 0, !selectedRowIndexes.contains(row) {
            selectRowIndexes(IndexSet(integer: row), byExtendingSelection: false)
        }
        let menu = NSMenu()
        menu.autoenablesItems = false
        let copy = menu.addItem(withTitle: "拷贝数据", action: #selector(copy(_:)), keyEquivalent: "")
        copy.target = self
        copy.isEnabled = !selectedRowIndexes.isEmpty
        return menu
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
