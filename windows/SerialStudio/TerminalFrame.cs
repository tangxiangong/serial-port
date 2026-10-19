using System.ComponentModel;
using SerialStudio.Core;
namespace SerialStudio;

public sealed class TerminalFrame : INotifyPropertyChanged
{
    private readonly SerialFrameEvent frame;
    private string format;
    internal TerminalFrame(SerialFrameEvent frame, string format) { this.frame = frame; this.format = format; }
    public string Direction => frame.direction.IsReceive ? "RX" : "TX";
    public string Time => DateTimeOffset.TryParse(frame.timestamp, System.Globalization.CultureInfo.InvariantCulture, System.Globalization.DateTimeStyles.None, out var value)
        ? value.ToLocalTime().ToString("HH:mm:ss.fff", System.Globalization.CultureInfo.InvariantCulture) : frame.timestamp;
    public string Size => $"{frame.bytes.Count} B";
    internal int MemoryCost => frame.bytes.Count * 16 + 1024;
    public string DisplayText => format == "hex" ? frame.hex : (format switch { "ascii" => frame.text.ascii, "gbk" => frame.text.gbk, _ => frame.text.utf8 })
        .Replace("\0", "␀").Replace("\r", "␍").Replace("\n", "␊\n").Replace("\t", "⇥");
    internal void SetFormat(string value) { format = value; PropertyChanged?.Invoke(this, new(nameof(DisplayText))); }
    public event PropertyChangedEventHandler? PropertyChanged;
}
