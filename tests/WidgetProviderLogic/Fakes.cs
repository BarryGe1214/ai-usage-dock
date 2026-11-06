// Test doubles for the Windows host boundary. The real provider source is linked
// by the project; these tests exercise its scheduling, not Windows COM or MSIX.
namespace Microsoft.Windows.Widgets
{
    public enum WidgetSize { Small, Medium }
    public sealed class WidgetContext
    {
        public string Id { get; init; } = "";
        public WidgetSize Size { get; init; } = WidgetSize.Medium;
    }
    public sealed class WidgetInfo { public WidgetContext WidgetContext { get; init; } = new(); }
    public sealed class WidgetActionInvokedArgs
    {
        public string Verb { get; init; } = "";
        public WidgetContext WidgetContext { get; init; } = new();
    }
    public sealed class WidgetContextChangedArgs { public WidgetContext WidgetContext { get; init; } = new(); }
    public sealed class WidgetUpdateRequestOptions(string id)
    {
        public string Id { get; } = id;
        public string Template { get; init; } = "";
        public string Data { get; init; } = "";
        public string CustomState { get; init; } = "";
    }
    public sealed class WidgetManager
    {
        private static readonly WidgetManager Instance = new();
        public static WidgetManager GetDefault() => Instance;
        public readonly Dictionary<string, string> Delivered = new();
        public string? FailNextId;
        public WidgetInfo[] GetWidgetInfos() => [];
        public void UpdateWidget(WidgetUpdateRequestOptions options)
        {
            if (FailNextId == options.Id)
            {
                FailNextId = null;
                throw new InvalidOperationException("Simulated host failure");
            }
            Delivered[options.Id] = options.Data;
        }
    }
}
namespace Microsoft.Windows.Widgets.Providers
{
    public interface IWidgetProvider { }
}
namespace AiUsageDock.Widgets
{
    internal sealed class WidgetFeed { public string UpdatedAt { get; set; } = "first"; }
    internal static class WidgetFeedStore
    {
        public static WidgetFeed Current = new();
        public static WidgetFeed Load() => Current;
    }
    internal static class WidgetData { public static string Build(WidgetFeed feed) => feed.UpdatedAt; }
    internal static class WidgetTemplates { public static string ForSize(string size) => size; }
    internal static class Log { public static void Error(string message, Exception exception) { } }
}
