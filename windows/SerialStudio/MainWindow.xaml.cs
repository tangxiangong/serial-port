using System.Collections.ObjectModel;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using SerialStudio.Core;

namespace SerialStudio;

public sealed partial class MainWindow : Window
{
    private readonly ObservableCollection<TerminalFrame> frames = [];
    private readonly ObservableCollection<TerminalFrame> visibleFrames = [];
    private readonly CancellationTokenSource lifetime = new();
    private readonly DispatcherQueueTimer timer;
    private SerialStatus status = new() { connected = false, path = null, config = null, rx_bytes = 0, tx_bytes = 0, opened_at = null, last_error = null };
    private bool busy, sending, polling, refreshing, updating = true, started, closed;
    private bool? wideLayout;
    private int ticks, retainedBytes, connectionRevision;
    private static readonly string[] Formats = ["hex", "ascii", "gbk", "utf-8"];
    private static readonly string[] Endings = ["none", "lf", "cr", "crlf"];
    private readonly Preferences preferences = Preferences.Current;

    public MainWindow()
    {
        InitializeComponent();
        ExtendsContentIntoTitleBar = true;
        SetTitleBar(AppTitleBar);
        AppWindow.Resize(new Windows.Graphics.SizeInt32(1360, 860));
        if (AppWindow.Presenter is OverlappedPresenter presenter)
        {
            presenter.PreferredMinimumWidth = 920;
            presenter.PreferredMinimumHeight = 640;
        }
        Terminal.ItemsSource = visibleFrames;
        BaudPicker.ItemsSource = new uint[] { 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600 };
        DataBitsPicker.ItemsSource = new byte[] { 5, 6, 7, 8 }; StopBitsPicker.ItemsSource = new byte[] { 1, 2 };
        ParityPicker.ItemsSource = new[] { "无", "奇校验", "偶校验" }; FlowPicker.ItemsSource = new[] { "无", "XON / XOFF", "RTS / CTS" };
        BaudPicker.SelectedItem = preferences.BaudRate; if (BaudPicker.SelectedIndex < 0) BaudPicker.SelectedItem = (uint)115200;
        DataBitsPicker.SelectedItem = preferences.DataBits; if (DataBitsPicker.SelectedIndex < 0) DataBitsPicker.SelectedItem = (byte)8;
        StopBitsPicker.SelectedItem = preferences.StopBits; if (StopBitsPicker.SelectedIndex < 0) StopBitsPicker.SelectedItem = (byte)1;
        ParityPicker.SelectedIndex = Math.Clamp(preferences.Parity, 0, 2); FlowPicker.SelectedIndex = Math.Clamp(preferences.FlowControl, 0, 2);
        DisplayPicker.ItemsSource = SendFormatPicker.ItemsSource = new[] { "HEX", "ASCII", "GBK", "UTF-8" };
        DisplayPicker.SelectedIndex = IndexOrDefault(Formats, preferences.DisplayFormat, 3);
        SendFormatPicker.SelectedIndex = IndexOrDefault(Formats, preferences.SendFormat, 3);
        EndingPicker.ItemsSource = new[] { "无", "LF", "CR", "CRLF" }; EndingPicker.SelectedIndex = IndexOrDefault(Endings, preferences.LineEnding, 0);
        AutoScrollToggle.IsChecked = preferences.AutoScroll; ApplyAppearance();
        timer = DispatcherQueue.CreateTimer(); timer.Interval = TimeSpan.FromMilliseconds(100); timer.Tick += OnTick;
        Updater.Changed += OnUpdaterChanged;
        UpdateActivity.Changed += OnUpdaterChanged;
        Closed += OnClosed;
        updating = false; RefreshState();
    }

    private static int IndexOrDefault(string[] values, string value, int fallback) => Array.IndexOf(values, value) is var index && index >= 0 ? index : fallback;
    private async void OnLoaded(object sender, RoutedEventArgs args)
    {
        if (started) return; started = true;
        if (AppWindow.Presenter is OverlappedPresenter presenter)
        {
            var scale = Root.XamlRoot.RasterizationScale;
            presenter.PreferredMinimumWidth = (int)Math.Ceiling(920 * scale);
            presenter.PreferredMinimumHeight = (int)Math.Ceiling(640 * scale);
        }
        await RefreshPorts(); timer.Start();
        await Updater.Start(Root, lifetime.Token);
    }
    private async void OnTick(DispatcherQueueTimer sender, object args)
    {
        if (closed || polling || busy) return;
        polling = true; var revision = connectionRevision;
        try
        {
            var batch = await RustCore.Poll();
            if (closed) return;
            if (!busy && revision == connectionRevision)
            {
                if (batch.status.last_error is { } error && error != status.last_error) ShowError(error);
                status = batch.status; UpdateActivity.SetConnected(status.connected);
            }
            foreach (var frame in batch.frames)
            {
                var item = new TerminalFrame(frame, preferences.DisplayFormat); frames.Add(item); retainedBytes += item.MemoryCost;
            }
            while (frames.Count > 2000 || retainedBytes > 8 * 1024 * 1024) { retainedBytes -= frames[0].MemoryCost; frames.RemoveAt(0); }
            if (batch.dropped_frames > 0) ShowError($"显示缓冲已丢弃 {batch.dropped_frames} 条旧记录；字节统计保持完整。");
            if (batch.frames.Count > 0) RefreshTerminal();
            RefreshState();
            if (++ticks % 10 == 0) await RefreshPorts(silent: true);
        }
        catch (Exception error) { if (!closed) ShowError(error.Message); }
        finally { polling = false; }
    }
    private async Task RefreshPorts(bool silent = false)
    {
        if (refreshing || closed) return; refreshing = true;
        try
        {
            var ports = await RustCore.Ports(); if (closed) return;
            var choices = ports.Select(p => new PortChoice(p.path, p.display_name, p)).ToList();
            var selected = status.connected ? status.path : preferences.SelectedPath;
            if (status.connected && !choices.Any(p => p.Path == selected) && selected is not null) choices.Add(new(selected, selected + "（已移除）", null));
            // Preserve native list selection and keyboard focus between hotplug scans.
            if (PortPicker.ItemsSource is List<PortChoice> current && current.Count == choices.Count && current.Zip(choices).All(pair => SamePort(pair.First, pair.Second))) return;
            updating = true;
            PortPicker.ItemsSource = choices;
            PortPicker.SelectedItem = choices.FirstOrDefault(p => p.Path == selected) ?? choices.FirstOrDefault();
            if (!status.connected) preferences.SelectedPath = (PortPicker.SelectedItem as PortChoice)?.Path ?? "";
            ShowPortDetails();
        }
        catch (Exception error) { if (!silent && !closed) ShowError(error.Message); }
        finally { updating = false; refreshing = false; if (!closed) RefreshState(); }
    }
    private async void OnConnect(object sender, RoutedEventArgs args) => await ToggleConnection();
    private async Task ToggleConnection()
    {
        if (busy || closed || (!status.connected && PortPicker.SelectedItem is not PortChoice)) return;
        busy = true; connectionRevision++; RefreshState();
        try
        {
            SavePreferences(); ErrorBar.IsOpen = false;
            status = status.connected ? await RustCore.Disconnect() : await RustCore.Connect(preferences.SelectedPath, new SerialConfig
            {
                baud_rate = preferences.BaudRate, data_bits = preferences.DataBits, stop_bits = preferences.StopBits,
                parity = preferences.Parity switch { 1 => Parity.Odd, 2 => Parity.Even, _ => Parity.None },
                flow_control = preferences.FlowControl switch { 1 => FlowControl.Software, 2 => FlowControl.Hardware, _ => FlowControl.None }
            });
            UpdateActivity.SetConnected(status.connected);
        }
        catch (Exception error) { if (!closed) ShowError(error.Message); }
        finally { busy = false; if (!closed) RefreshState(); }
    }
    private async void OnSend(object sender, RoutedEventArgs args) => await Send();
    private async Task Send()
    {
        if (!status.connected || sending || busy || InputBox.Text.Length == 0) return;
        sending = true; RefreshState();
        var format = SendFormatPicker.SelectedIndex switch { 0 => Core.SendFormat.Hex, 1 => Core.SendFormat.Ascii, 2 => Core.SendFormat.Gbk, _ => Core.SendFormat.Utf8 };
        var ending = EndingPicker.SelectedIndex switch { 1 => Core.LineEnding.Lf, 2 => Core.LineEnding.Cr, 3 => Core.LineEnding.Crlf, _ => Core.LineEnding.None };
        try { await RustCore.Send(InputBox.Text, format, ending); }
        catch (Exception error) { if (!closed) ShowError(error.Message); }
        finally { sending = false; if (!closed) RefreshState(); }
    }
    private void RefreshState()
    {
        if (closed) return;
        PortPicker.IsEnabled = !busy && !status.connected && PortPicker.Items.Count > 0;
        foreach (var picker in new[] { BaudPicker, DataBitsPicker, StopBitsPicker, ParityPicker, FlowPicker })
            picker.IsEnabled = !busy && !status.connected;
        ConnectButton.IsEnabled = !busy && (status.connected || PortPicker.SelectedItem is PortChoice);
        ConnectButton.Label = busy ? "正在处理…" : status.connected ? "断开连接" : "连接";
        var connectionSymbol = status.connected ? Symbol.Stop : Symbol.Play;
        if (ConnectButton.Icon is not SymbolIcon icon || icon.Symbol != connectionSymbol)
            ConnectButton.Icon = new SymbolIcon(connectionSymbol);
        SendButton.IsEnabled = status.connected && !sending && !busy && InputBox.Text.Length > 0;
        SendButton.Content = sending ? "发送中…" : "发送";
        EndingPicker.IsEnabled = SendFormatPicker.SelectedIndex != 0;
        InputBox.PlaceholderText = SendFormatPicker.SelectedIndex == 0 ? "十六进制数据，例如 AA 01 FF" : "输入文本或设备指令…";
        SendHint.Text = SendFormatPicker.SelectedIndex == 0 ? "HEX 按原始字节发送，不追加行尾。" : "Ctrl+Enter 发送 · Enter 换行";
        EmptyState.Visibility = visibleFrames.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
        var filteredEmpty = frames.Count > 0 && visibleFrames.Count == 0;
        EmptyTitle.Text = filteredEmpty ? "没有匹配的数据" : status.connected ? "等待设备数据" : "准备连接";
        EmptyDescription.Text = filteredEmpty ? "尝试其他搜索内容，或显示全部方向。"
            : status.connected ? "收到的数据会实时出现在这里，也可以在下方发送指令。" : "选择串口，确认连接设置，然后点击“连接”。";
        ResetFiltersButton.Visibility = filteredEmpty ? Visibility.Visible : Visibility.Collapsed;
        FrameCount.Text = TerminalSearch.Text.Length == 0 && DirectionPicker.SelectedIndex == 0
            ? $"{frames.Count} 条记录" : $"{visibleFrames.Count} / {frames.Count} 条记录";
        ClearButton.IsEnabled = frames.Count > 0;
        var selected = PortPicker.SelectedItem as PortChoice;
        AppTitleBar.Subtitle = status.connected ? $"{status.path} · {preferences.BaudRate} baud" : selected?.DisplayName ?? "串口终端";
        ConnectionStatus.Text = status.connected ? $"已连接 · {status.path} · {preferences.BaudRate} baud" : "未连接";
        ByteStatus.Text = $"RX {status.rx_bytes:N0} B    TX {status.tx_bytes:N0} B";
        PortCount.Text = PortPicker.Items.Count.ToString();
        DeviceEmptyState.Visibility = PortPicker.Items.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
        RefreshButton.IsEnabled = !refreshing;
        ConfigHint.Text = busy ? "正在更改连接状态…" : status.connected ? "断开连接后可修改通信参数。" : "选择设备并确认通信参数。";
        CheckUpdateButton.IsEnabled = Updater.CanCheck;
        ApplyUpdateButton.Visibility = string.IsNullOrEmpty(Updater.PendingVersion) ? Visibility.Collapsed : Visibility.Visible;
        ApplyUpdateButton.IsEnabled = !UpdateActivity.IsBusy;
    }
    private void ShowPortDetails()
    {
        var port = (PortPicker.SelectedItem as PortChoice)?.Info;
        if (port is null)
        {
            PortDetails.Text = status.connected ? "设备已移除，请断开后重新连接。" : "从左侧选择一个串口设备。";
            return;
        }
        var details = new List<string> { port.display_name, port.path };
        if (!string.IsNullOrWhiteSpace(port.manufacturer)) details.Add($"厂商  {port.manufacturer}");
        if (!string.IsNullOrWhiteSpace(port.vid) && !string.IsNullOrWhiteSpace(port.pid)) details.Add($"VID / PID  {port.vid} / {port.pid}");
        if (!string.IsNullOrWhiteSpace(port.serial_number)) details.Add($"序列号  {port.serial_number}");
        PortDetails.Text = string.Join("\n\n", details);
    }
    private void SavePreferences()
    {
        preferences.BaudRate = BaudPicker.SelectedItem is uint baud ? baud : 115200;
        preferences.DataBits = DataBitsPicker.SelectedItem is byte bits ? bits : (byte)8;
        preferences.StopBits = StopBitsPicker.SelectedItem is byte stops ? stops : (byte)1;
        preferences.Parity = Math.Max(0, ParityPicker.SelectedIndex); preferences.FlowControl = Math.Max(0, FlowPicker.SelectedIndex);
        preferences.SelectedPath = (PortPicker.SelectedItem as PortChoice)?.Path ?? "";
        preferences.DisplayFormat = Formats[Math.Clamp(DisplayPicker.SelectedIndex, 0, 3)]; preferences.SendFormat = Formats[Math.Clamp(SendFormatPicker.SelectedIndex, 0, 3)];
        preferences.LineEnding = Endings[Math.Clamp(EndingPicker.SelectedIndex, 0, 3)]; preferences.AutoScroll = AutoScrollToggle.IsChecked == true;
        preferences.Save();
    }
    private void OnSelectionChanged(object sender, SelectionChangedEventArgs args)
    {
        if (updating) return;
        try { SavePreferences(); ShowPortDetails(); RefreshState(); } catch (Exception error) { ShowError(error.Message); }
    }
    private void OnDisplayChanged(object sender, SelectionChangedEventArgs args)
    {
        if (updating) return;
        OnSelectionChanged(sender, args);
        foreach (var frame in frames) frame.SetFormat(preferences.DisplayFormat);
        RefreshTerminal(); RefreshState();
    }
    private void OnAutoScroll(object sender, RoutedEventArgs args)
    {
        if (updating) return;
        try { SavePreferences(); ScrollToLatest(); } catch (Exception error) { ShowError(error.Message); }
    }
    private void OnInputChanged(object sender, TextChangedEventArgs args) { if (!updating) RefreshState(); }
    private void OnClear(object sender, RoutedEventArgs args) { frames.Clear(); visibleFrames.Clear(); retainedBytes = 0; RefreshState(); }
    private async void OnRefresh(object sender, RoutedEventArgs args) => await RefreshPorts();
    private void RefreshTerminal()
    {
        var query = TerminalSearch.Text;
        var direction = DirectionPicker.SelectedIndex;
        var matches = frames.Where(frame =>
            (direction <= 0 || frame.IsReceive == (direction == 1))
            && (query.Length == 0 || frame.DisplayText.Contains(query, StringComparison.CurrentCultureIgnoreCase))).ToList();
        var retained = matches.ToHashSet();
        // Keep existing rows alive so incoming data doesn't reset selection or scroll position.
        for (var index = visibleFrames.Count - 1; index >= 0; index--)
            if (!retained.Contains(visibleFrames[index])) visibleFrames.RemoveAt(index);
        for (var index = 0; index < matches.Count; index++)
            if (index >= visibleFrames.Count || !ReferenceEquals(visibleFrames[index], matches[index]))
                visibleFrames.Insert(index, matches[index]);
        ScrollToLatest();
    }
    private void ScrollToLatest()
    {
        if (preferences.AutoScroll && visibleFrames.Count > 0) Terminal.ScrollIntoView(visibleFrames[^1]);
    }
    private void OnSearchChanged(object sender, TextChangedEventArgs args)
    {
        if (updating) return;
        RefreshTerminal(); RefreshState();
    }
    private void OnFilterChanged(object sender, SelectionChangedEventArgs args)
    {
        if (updating) return;
        RefreshTerminal(); RefreshState();
    }
    private void OnResetFilters(object sender, RoutedEventArgs args)
    {
        updating = true;
        TerminalSearch.Text = ""; DirectionPicker.SelectedIndex = 0;
        updating = false;
        RefreshTerminal(); RefreshState();
    }
    private void OnInspector(object sender, RoutedEventArgs args)
        => ConnectionInspector.IsPaneOpen = InspectorToggle.IsChecked == true;
    private void OnInspectorPaneOpened(SplitView sender, object args)
        => InspectorToggle.IsChecked = sender.IsPaneOpen;
    private void OnInspectorPaneClosed(SplitView sender, object args)
        => InspectorToggle.IsChecked = sender.IsPaneOpen;
    private void OnRootSizeChanged(object sender, SizeChangedEventArgs args)
    {
        if (ConnectionInspector is null || InspectorToggle is null || args.NewSize.Width <= 0) return;
        var wide = args.NewSize.Width >= 1120;
        if (wideLayout == wide) return;
        wideLayout = wide;
        ConnectionInspector.DisplayMode = wide ? SplitViewDisplayMode.Inline : SplitViewDisplayMode.Overlay;
        if (!wide) { ConnectionInspector.IsPaneOpen = false; InspectorToggle.IsChecked = false; }
    }
    private void ShowError(string message) { ErrorBar.Message = message; ErrorBar.IsOpen = true; }
    private void OnUpdaterChanged() { DispatcherQueue.TryEnqueue(RefreshState); }
    private async void OnCheckUpdate(object sender, RoutedEventArgs args) => await Updater.Check(Root, lifetime: lifetime.Token);
    private void OnApplyUpdate(object sender, RoutedEventArgs args) => Updater.ApplyPending();
    private async void OnSettings(object sender, RoutedEventArgs args)
    {
        var appearance = new ComboBox { Header = "主题", HorizontalAlignment = HorizontalAlignment.Stretch, ItemsSource = new[] { "跟随系统", "浅色", "深色" }, SelectedIndex = preferences.Appearance == "dark" ? 2 : preferences.Appearance == "light" ? 1 : 0 };
        var channels = new[] { "stable", "beta", "alpha" };
        var channel = new ComboBox { Header = "更新通道", HorizontalAlignment = HorizontalAlignment.Stretch, ItemsSource = new[] { "稳定版", "Beta", "Alpha" }, SelectedIndex = IndexOrDefault(channels, Updater.Channel, 0) };
        var checks = new ToggleSwitch { Header = "自动检查更新", IsOn = Updater.AutomaticChecks };
        var downloads = new ToggleSwitch { Header = "自动下载更新", IsOn = Updater.AutomaticDownloads };
        var content = new StackPanel { Spacing = 16, MinWidth = 360 };
        content.Children.Add(new TextBlock { Text = "外观", Style = (Style)Application.Current.Resources["SubtitleTextBlockStyle"] });
        content.Children.Add(appearance);
        content.Children.Add(new TextBlock { Text = "软件更新", Style = (Style)Application.Current.Resources["SubtitleTextBlockStyle"], Margin = new Thickness(0, 12, 0, 0) });
        content.Children.Add(new TextBlock { Text = $"当前版本  {Updater.CurrentVersion}", FontSize = 12 });
        foreach (var element in new FrameworkElement[] { channel, checks, downloads }) content.Children.Add(element);
        content.Children.Add(new TextBlock { Text = Updater.UnavailableReason ?? "断开串口后可检查和安装更新。", TextWrapping = TextWrapping.Wrap, FontSize = 12 });
        var scroll = new ScrollViewer { Content = content, MaxHeight = 420, VerticalScrollBarVisibility = ScrollBarVisibility.Auto, HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled };
        var dialog = new ContentDialog { XamlRoot = Root.XamlRoot, Title = "设置", Content = scroll, PrimaryButtonText = "保存", CloseButtonText = "取消", DefaultButton = ContentDialogButton.Primary };
        if (await DialogHost.Show(dialog) != ContentDialogResult.Primary) return;
        try
        {
            preferences.Appearance = appearance.SelectedIndex == 2 ? "dark" : appearance.SelectedIndex == 1 ? "light" : "system";
            Updater.Channel = channels[Math.Clamp(channel.SelectedIndex, 0, 2)]; Updater.AutomaticChecks = checks.IsOn; Updater.AutomaticDownloads = downloads.IsOn;
            preferences.Save(); ApplyAppearance();
        }
        catch (Exception error) { ShowError(error.Message); }
    }
    private void ApplyAppearance() => Root.RequestedTheme = preferences.Appearance == "dark" ? ElementTheme.Dark : preferences.Appearance == "light" ? ElementTheme.Light : ElementTheme.Default;
    private async void OnConnectShortcut(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args) { args.Handled = true; await ToggleConnection(); }
    private async void OnSendShortcut(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args) { args.Handled = true; await Send(); }
    private async void OnRefreshShortcut(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args) { args.Handled = true; await RefreshPorts(); }
    private void OnClearShortcut(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args) { args.Handled = true; OnClear(sender, new()); }
    private void OnFindShortcut(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args)
    {
        args.Handled = true; TerminalSearch.Focus(FocusState.Programmatic); TerminalSearch.SelectAll();
    }
    private void OnInspectorShortcut(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args)
    {
        args.Handled = true;
        ConnectionInspector.IsPaneOpen = !ConnectionInspector.IsPaneOpen;
        InspectorToggle.IsChecked = ConnectionInspector.IsPaneOpen;
    }
    private void OnClosed(object sender, WindowEventArgs args)
    {
        closed = true; timer.Stop(); lifetime.Cancel();
        Updater.Changed -= OnUpdaterChanged; UpdateActivity.Changed -= OnUpdaterChanged;
        try { RustCore.Shutdown(); } catch (Exception error) { System.Diagnostics.Trace.TraceError("{0}", error); }
        UpdateActivity.SetConnected(false); Updater.ApplyOnExit(); lifetime.Dispose();
    }
    private sealed record PortChoice(string Path, string DisplayName, PortInfo? Info);
    private static bool SamePort(PortChoice left, PortChoice right) => left.Path == right.Path && left.DisplayName == right.DisplayName
        && left.Info?.manufacturer == right.Info?.manufacturer && left.Info?.serial_number == right.Info?.serial_number
        && left.Info?.vid == right.Info?.vid && left.Info?.pid == right.Info?.pid;
}
