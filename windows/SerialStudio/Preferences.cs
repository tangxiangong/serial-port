using System.Text.Json;
namespace SerialStudio;

internal sealed class Preferences
{
    private static readonly string DirectoryPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "SerialStudio");
    private static readonly string FilePath = Path.Combine(DirectoryPath, "preferences.json");
    internal static Preferences Current { get; } = Load();
    public uint BaudRate { get; set; } = 115200;
    public byte DataBits { get; set; } = 8;
    public byte StopBits { get; set; } = 1;
    public int Parity { get; set; }
    public int FlowControl { get; set; }
    public string SelectedPath { get; set; } = "";
    public string DisplayFormat { get; set; } = "utf-8";
    public string SendFormat { get; set; } = "utf-8";
    public string LineEnding { get; set; } = "none";
    public bool AutoScroll { get; set; } = true;
    public string Appearance { get; set; } = "system";
    public string UpdateChannel { get; set; } = "";
    public bool AutomaticChecks { get; set; } = true;
    public bool AutomaticDownloads { get; set; }
    public string PendingUpdate { get; set; } = "";
    public DateTimeOffset? LastCheck { get; set; }
    private static Preferences Load()
    {
        try { return JsonSerializer.Deserialize<Preferences>(File.ReadAllText(FilePath)) ?? new(); }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException or JsonException) { return new(); }
    }
    internal void Save()
    {
        Directory.CreateDirectory(DirectoryPath);
        var temporary = Path.Combine(DirectoryPath, $"preferences.{Guid.NewGuid():N}.tmp");
        try { File.WriteAllText(temporary, JsonSerializer.Serialize(this)); File.Move(temporary, FilePath, true); }
        finally { if (File.Exists(temporary)) File.Delete(temporary); }
    }
}
