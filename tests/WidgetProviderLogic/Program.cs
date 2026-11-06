using System.Reflection;
using AiUsageDock.Widgets;
using Microsoft.Windows.Widgets;

var provider = new UsageWidgetProvider();
var manager = WidgetManager.GetDefault();
var first = new WidgetContext { Id = "first", Size = WidgetSize.Small };
var second = new WidgetContext { Id = "second", Size = WidgetSize.Medium };
void Assert(bool condition, string message)
{
    if (!condition) throw new Exception(message);
    Console.WriteLine("PASS: " + message);
}
void Tick() => typeof(UsageWidgetProvider)
    .GetMethod("OnTimer", BindingFlags.Static | BindingFlags.NonPublic)!
    .Invoke(null, new object?[] { null });
try
{
    provider.CreateWidget(first);
    provider.CreateWidget(second);
    Assert(manager.Delivered.Count == 2, "both pinned widgets receive initial data");
    WidgetFeedStore.Current = new WidgetFeed { UpdatedAt = "second-feed" };
    provider.OnActionInvoked(new WidgetActionInvokedArgs { Verb = "refresh", WidgetContext = first });
    Tick();
    Assert(manager.Delivered[second.Id] == "second-feed",
        "refreshing one card does not suppress another card's periodic update");
    WidgetFeedStore.Current = new WidgetFeed { UpdatedAt = "third-feed" };
    manager.FailNextId = first.Id;
    Tick();
    Assert(manager.Delivered[first.Id] == "second-feed" && manager.Delivered[second.Id] == "third-feed",
        "a host failure on one card does not prevent delivery to another");
    Tick();
    Assert(manager.Delivered[first.Id] == "third-feed",
        "a failed card is retried even when the feed timestamp has not changed");
}
finally
{
    provider.DeleteWidget(first.Id, "");
    provider.DeleteWidget(second.Id, "");
}
