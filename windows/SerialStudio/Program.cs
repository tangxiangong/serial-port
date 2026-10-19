using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Velopack;
using System.Runtime.CompilerServices;

namespace SerialStudio;

internal static class Program
{
    [STAThread]
    private static void Main()
    {
        // Velopack fast-exit hooks run before WinUI or the Rust service starts.
        VelopackApp.Build().SetAutoApplyOnStartup(false).Run();
        StartApplication();
    }

    [MethodImpl(MethodImplOptions.NoInlining)]
    private static void StartApplication()
    {
        Updater.ApplyPending();
        WinRT.ComWrappersSupport.InitializeComWrappers();
        Application.Start(parameters =>
        {
            SynchronizationContext.SetSynchronizationContext(new DispatcherQueueSynchronizationContext(DispatcherQueue.GetForCurrentThread()));
            _ = new App();
        });
    }
}
