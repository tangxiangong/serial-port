namespace SerialStudio;

/// Tracks queued and running service operations across the workbench and helper.
internal static class UpdateActivity
{
    private static int active;
    private static int connected;
    internal static void SetConnected(bool value) { var next = value ? 1 : 0; if (Interlocked.Exchange(ref connected, next) != next) Changed?.Invoke(); }
    internal static bool IsBusy => Volatile.Read(ref active) != 0 || Volatile.Read(ref connected) != 0;
    internal static event Action? Changed;

    internal static Task Run(Action operation) => Run(() => { operation(); return true; });
    internal static async Task<T> Run<T>(Func<T> operation)
    {
        Interlocked.Increment(ref active);
        Changed?.Invoke();
        try { return await Task.Run(operation).ConfigureAwait(false); }
        finally { Interlocked.Decrement(ref active); Changed?.Invoke(); }
    }
}
