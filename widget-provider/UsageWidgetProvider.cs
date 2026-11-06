using Microsoft.Windows.Widgets;
using Microsoft.Windows.Widgets.Providers;

namespace AiUsageDock.Widgets;

internal sealed class UsageWidgetProvider : IWidgetProvider
{
    private const string RefreshVerb = "refresh";
    private static readonly TimeSpan RefreshInterval = TimeSpan.FromSeconds(30);

    private sealed class WidgetState
    {
        public string Size { get; set; } = "medium";
        public string? LastUpdatedAt { get; set; }
    }

    private static readonly object Gate = new();
    private static readonly Dictionary<string, WidgetState> Widgets = new(StringComparer.OrdinalIgnoreCase);
    private static readonly ManualResetEvent EmptyListEvent = new(false);

    private static Timer? _timer;
    private static bool _everHadWidget;

    public UsageWidgetProvider()
    {
        RestoreWidgets();
        EnsureTimer();
    }

    public static void WaitForExit() => EmptyListEvent.WaitOne();

    public void CreateWidget(WidgetContext widgetContext)
    {
        lock (Gate)
        {
            Widgets[widgetContext.Id] = new WidgetState
            {
                Size = SizeName(widgetContext.Size),
            };
            _everHadWidget = true;
        }

        EnsureTimer();
        UpdateWidget(widgetContext.Id);
    }

    public void DeleteWidget(string widgetId, string customState)
    {
        bool empty;
        lock (Gate)
        {
            Widgets.Remove(widgetId);
            empty = Widgets.Count == 0;
        }

        if (!empty)
        {
            return;
        }

        StopTimer();

        // Only stop the COM server once the user has actually unpinned a widget.
        if (_everHadWidget)
        {
            EmptyListEvent.Set();
        }
    }

    public void OnActionInvoked(WidgetActionInvokedArgs actionInvokedArgs)
    {
        if (string.Equals(actionInvokedArgs.Verb, RefreshVerb, StringComparison.OrdinalIgnoreCase))
        {
            UpdateWidget(actionInvokedArgs.WidgetContext.Id);
        }
    }

    public void OnWidgetContextChanged(WidgetContextChangedArgs contextChangedArgs)
    {
        var widgetContext = contextChangedArgs.WidgetContext;
        lock (Gate)
        {
            if (Widgets.TryGetValue(widgetContext.Id, out var state))
            {
                state.Size = SizeName(widgetContext.Size);
            }
        }

        UpdateWidget(widgetContext.Id);
    }

    public void Activate(WidgetContext widgetContext)
    {
        UpdateWidget(widgetContext.Id);
    }

    public void Deactivate(string widgetId)
    {
        // The provider keeps serving updates while the host keeps the process alive.
    }

    private void RestoreWidgets()
    {
        try
        {
            foreach (var widgetInfo in WidgetManager.GetDefault().GetWidgetInfos())
            {
                var widgetContext = widgetInfo.WidgetContext;
                lock (Gate)
                {
                    if (!Widgets.ContainsKey(widgetContext.Id))
                    {
                        Widgets[widgetContext.Id] = new WidgetState
                        {
                            Size = SizeName(widgetContext.Size),
                        };
                    }
                }
            }
        }
        catch (Exception exception)
        {
            Log.Error("Failed to restore pinned widgets", exception);
        }
    }

    private static void UpdateWidget(string widgetId, bool skipUnchanged = false)
    {
        string size;
        lock (Gate)
        {
            if (!Widgets.TryGetValue(widgetId, out var state))
            {
                return;
            }

            size = state.Size;
        }

        try
        {
            var feed = WidgetFeedStore.Load();
            lock (Gate)
            {
                if (skipUnchanged && !string.IsNullOrEmpty(feed.UpdatedAt) &&
                    Widgets.TryGetValue(widgetId, out var current) &&
                    current.LastUpdatedAt == feed.UpdatedAt)
                {
                    return;
                }
            }
            var options = new WidgetUpdateRequestOptions(widgetId)
            {
                Template = WidgetTemplates.ForSize(size),
                Data = WidgetData.Build(feed),
                CustomState = string.Empty,
            };

            WidgetManager.GetDefault().UpdateWidget(options);

            lock (Gate)
            {
                if (Widgets.TryGetValue(widgetId, out var current))
                {
                    current.LastUpdatedAt = feed.UpdatedAt;
                }
            }
        }
        catch (Exception exception)
        {
            Log.Error($"Failed to update widget {widgetId}", exception);
        }
    }

    private static void EnsureTimer()
    {
        lock (Gate)
        {
            _timer ??= new Timer(OnTimer, null, RefreshInterval, RefreshInterval);
        }
    }

    private static void StopTimer()
    {
        lock (Gate)
        {
            _timer?.Dispose();
            _timer = null;
        }
    }

    private static void OnTimer(object? state)
    {
        try
        {
            string[] widgetIds;
            lock (Gate)
            {
                if (Widgets.Count == 0)
                {
                    return;
                }

                widgetIds = Widgets.Keys.ToArray();
            }

            foreach (var widgetId in widgetIds)
            {
                UpdateWidget(widgetId, skipUnchanged: true);
            }
        }
        catch (Exception exception)
        {
            Log.Error("Periodic widget refresh failed", exception);
        }
    }

    private static string SizeName(WidgetSize size) =>
        size.ToString().ToLowerInvariant();
}
