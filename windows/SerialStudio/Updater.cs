using System.Globalization;
using System.Net;
using System.Reflection;
using System.Runtime.InteropServices;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Velopack;

namespace SerialStudio;

internal static class Updater
{
    internal static string CurrentVersion => typeof(Updater).Assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion.Split('+')[0] ?? "0.1.0";
    private static readonly Lazy<System.Security.Cryptography.ECDsa?> PublicKey = new(() => UpdateFeed.ImportPublicKey(typeof(Updater).Assembly.GetCustomAttributes<AssemblyMetadataAttribute>().FirstOrDefault(x => x.Key == "UpdatePublicKey")?.Value));
    private static string Architecture => RuntimeInformation.ProcessArchitecture == System.Runtime.InteropServices.Architecture.Arm64 ? "arm64" : "x64";
    private static bool busy;
    private static long revision;
    private static CancellationTokenSource? downloading;
    private static string? announced;
    internal static event Action? Changed;
    internal static string? UnavailableReason { get; private set; }
    internal static bool CanCheck => !busy && !UpdateActivity.IsBusy && UnavailableReason is null;
    internal static string PendingVersion => Preferences.Current.PendingUpdate;
    internal static string Channel
    {
        get => UpdatePolicy.Channel(Preferences.Current.UpdateChannel, CurrentVersion);
        set { var next = UpdatePolicy.Channel(value, CurrentVersion); if (Channel == next) return; Preferences.Current.UpdateChannel = next; PreferencesChanged(); }
    }
    internal static bool AutomaticChecks
    {
        get => Preferences.Current.AutomaticChecks;
        set { if (AutomaticChecks == value) return; Preferences.Current.AutomaticChecks = value; PreferencesChanged(); }
    }
    internal static bool AutomaticDownloads
    {
        get => Preferences.Current.AutomaticDownloads;
        set { if (AutomaticDownloads == value) return; Preferences.Current.AutomaticDownloads = value; PreferencesChanged(); }
    }
    private static void PreferencesChanged()
    {
        revision++; downloading?.Cancel(); announced = null;
        Preferences.Current.PendingUpdate = "";
        Preferences.Current.Save(); Changed?.Invoke();
    }
    private static UpdateManager? CreateManager()
    {
        if (PublicKey.Value is not { } key) { UnavailableReason = "此构建未配置更新验证公钥"; return null; }
        try
        {
            var manager = new UpdateManager(new UpdateSource(key), new UpdateOptions { ExplicitChannel = UpdatePolicy.FeedChannel(Architecture, Channel), AllowVersionDowngrade = false, MaximumDeltasBeforeFallback = -1 });
            UnavailableReason = manager.IsInstalled ? null : "请先安装发布版本，再检查更新";
            return manager.IsInstalled ? manager : null;
        }
        catch (Exception error) { System.Diagnostics.Trace.TraceError("{0}", error); UnavailableReason = "自动更新不可用"; return null; }
    }
    internal static void ApplyPending()
    {
        var requested = PendingVersion;
        if (string.IsNullOrEmpty(requested)) return;
        try
        {
            var manager = CreateManager();
            var pending = manager?.UpdatePendingRestart;
            if (pending is not null && manager!.CurrentVersion is { } current && pending.Version.CompareTo(current) > 0
                && UpdatePolicy.CanApplyPending(AutomaticDownloads, requested, pending.Version.ToString(), Channel) && !UpdateActivity.IsBusy)
            {
                Preferences.Current.PendingUpdate = ""; Preferences.Current.Save();
                manager.ApplyUpdatesAndRestart(pending, Environment.GetCommandLineArgs()[1..]);
            }
        }
        catch (Exception error) { System.Diagnostics.Trace.TraceError("Pending update: {0}", error); }
    }
    internal static void ApplyOnExit()
    {
        if (UpdateActivity.IsBusy || string.IsNullOrEmpty(PendingVersion)) return;
        try
        {
            var manager = CreateManager(); var pending = manager?.UpdatePendingRestart;
            if (pending is not null && manager!.CurrentVersion is { } current && pending.Version.CompareTo(current) > 0
                && UpdatePolicy.CanApplyPending(AutomaticDownloads, PendingVersion, pending.Version.ToString(), Channel))
                manager.WaitExitThenApplyUpdates(pending, silent: true, restart: false);
        }
        catch (Exception error) { System.Diagnostics.Trace.TraceError("Update on exit: {0}", error); }
    }
    internal static async Task Start(FrameworkElement owner, CancellationToken cancellation)
    {
        CreateManager(); Changed?.Invoke();
        try
        {
            await Task.Delay(TimeSpan.FromSeconds(8), cancellation);
            while (!cancellation.IsCancellationRequested)
            {
                var delay = UpdatePolicy.CheckDelay(DateTimeOffset.UtcNow, Preferences.Current.LastCheck);
                if (delay > TimeSpan.Zero) await Task.Delay(delay, cancellation);
                else if (!AutomaticChecks || !CanCheck) await Task.Delay(TimeSpan.FromMinutes(1), cancellation);
                else await Check(owner, false, cancellation);
            }
        }
        catch (OperationCanceledException) when (cancellation.IsCancellationRequested) { }
    }
    internal static async Task Check(FrameworkElement owner, bool manual = true, CancellationToken lifetime = default)
    {
        if (busy || UpdateActivity.IsBusy || (!manual && !AutomaticChecks)) return;
        var manager = CreateManager(); Changed?.Invoke();
        if (manager is null) return;
        busy = true; Changed?.Invoke();
        var startedRevision = revision;
        try
        {
            Preferences.Current.LastCheck = DateTimeOffset.UtcNow; Preferences.Current.Save();
            var update = await manager.CheckForUpdatesAsync();
            if (!owner.IsLoaded || lifetime.IsCancellationRequested || revision != startedRevision || (!manual && !AutomaticChecks)) return;
            if (update is null)
            {
                if (manual) await DialogHost.Show(new ContentDialog { XamlRoot = owner.XamlRoot, Title = "检查更新", Content = "当前已是最新版本", CloseButtonText = "好" });
                return;
            }
            var version = update.TargetFullRelease.Version.ToString();
            if (!manual && announced == version) return;
            if (!UpdatePolicy.AcceptsVersion(version, Channel)) throw new InvalidDataException("更新版本与通道不匹配");
            if (!manual && AutomaticDownloads)
            {
                using var cancellation = CancellationTokenSource.CreateLinkedTokenSource(lifetime);
                downloading = cancellation;
                try
                {
                    await manager.DownloadUpdatesAsync(update, cancelToken: cancellation.Token);
                    if (!cancellation.IsCancellationRequested && AutomaticChecks && UpdatePolicy.CanSchedule(startedRevision, revision, AutomaticDownloads))
                    { Preferences.Current.PendingUpdate = version; Preferences.Current.Save(); announced = version; Changed?.Invoke(); }
                }
                finally { downloading = null; }
                return;
            }
            var language = CultureInfo.CurrentUICulture.Name.StartsWith("zh", StringComparison.OrdinalIgnoreCase) ? "zh-Hans" : "en";
            var content = new StackPanel { Spacing = 12, MinWidth = 420 };
            content.Children.Add(new TextBlock { Text = $"Serial Studio {version} 现已推出，当前版本 {CurrentVersion}。", TextWrapping = TextWrapping.Wrap });
            content.Children.Add(new ScrollViewer { MaxHeight = 380, Content = UpdateMarkdown.Render(UpdatePolicy.Notes(update.TargetFullRelease.NotesMarkdown ?? "", language)) });
            var progress = new ProgressBar { Visibility = Visibility.Collapsed, Minimum = 0, Maximum = 100 }; content.Children.Add(progress);
            var status = new TextBlock { TextWrapping = TextWrapping.Wrap }; content.Children.Add(status);
            var errors = new InfoBar { Severity = InfoBarSeverity.Error, IsClosable = false }; content.Children.Add(errors);
            var dialog = new ContentDialog { XamlRoot = owner.XamlRoot, Title = "检查更新", Content = content, PrimaryButtonText = "下载并安装", CloseButtonText = "稍后", DefaultButton = ContentDialogButton.Primary };
            using var cancel = CancellationTokenSource.CreateLinkedTokenSource(lifetime);
            Task? installation = null;
            async Task Install()
            {
                dialog.IsPrimaryButtonEnabled = false; progress.Visibility = Visibility.Visible; errors.IsOpen = false; status.Text = "正在下载…";
                try
                {
                    await manager.DownloadUpdatesAsync(update, value => owner.DispatcherQueue.TryEnqueue(() => progress.Value = value), cancelToken: cancel.Token);
                    if (UpdateActivity.IsBusy) status.Text = "请断开串口，随后将安装并重新启动…";
                    while (UpdateActivity.IsBusy) await Task.Delay(500, cancel.Token);
                    cancel.Token.ThrowIfCancellationRequested();
                    if (revision != startedRevision) throw new InvalidOperationException("更新设置已更改，请重新检查更新");
                    Preferences.Current.PendingUpdate = ""; Preferences.Current.Save();
                    manager.ApplyUpdatesAndRestart(update);
                }
                catch (OperationCanceledException) when (cancel.IsCancellationRequested) { }
                catch (Exception error) { errors.Message = Describe(error); errors.IsOpen = true; }
                finally { dialog.IsPrimaryButtonEnabled = true; progress.Visibility = Visibility.Collapsed; }
            }
            dialog.PrimaryButtonClick += (_, args) => { args.Cancel = true; if (installation is null || installation.IsCompleted) installation = Install(); };
            dialog.CloseButtonClick += (_, _) => cancel.Cancel();
            try { announced = version; await DialogHost.Show(dialog); }
            finally { cancel.Cancel(); if (installation is not null) await installation; }
        }
        catch (OperationCanceledException) { }
        catch (Exception error)
        {
            if (manual && owner.IsLoaded && !lifetime.IsCancellationRequested)
                await DialogHost.Show(new ContentDialog { XamlRoot = owner.XamlRoot, Title = "检查更新失败", Content = Describe(error), CloseButtonText = "好" });
            else System.Diagnostics.Trace.TraceWarning("Update check: {0}", error);
        }
        finally { busy = false; Changed?.Invoke(); }
    }
    private static string Describe(Exception error) => error switch
    {
        HttpRequestException { StatusCode: HttpStatusCode.NotFound } => "更新源尚未发布，请稍后重试。",
        HttpRequestException or TaskCanceledException => "无法连接到更新服务器，请检查网络。",
        InvalidDataException or System.Security.Cryptography.CryptographicException => "更新文件验证失败，请稍后重试。",
        _ => error.Message
    };
}

internal static class DialogHost
{
    private static readonly SemaphoreSlim Gate = new(1, 1);
    internal static async Task<ContentDialogResult> Show(ContentDialog dialog)
    {
        await Gate.WaitAsync();
        try { return await dialog.ShowAsync(); }
        finally { Gate.Release(); }
    }
}
