using System.Text;

namespace AiUsageDock.Widgets;

internal static class Log
{
    private static readonly object Gate = new();

    private static readonly string LogPath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "com.barryge1214.aiusagedock",
        "widget-provider.log");

    public static void Error(string message, Exception? exception = null)
    {
        try
        {
            var line = $"{DateTimeOffset.Now:O} {message}";
            if (exception is not null)
            {
                line += $": {exception}";
            }

            lock (Gate)
            {
                Directory.CreateDirectory(Path.GetDirectoryName(LogPath)!);
                File.AppendAllText(LogPath, line + Environment.NewLine, Encoding.UTF8);
            }
        }
        catch
        {
            // Diagnostics must never take down the provider.
        }
    }
}
