using System.Globalization;
using System.Text.Json;

namespace AiUsageDock.Widgets;

internal sealed class WidgetFeed
{
    public int Version { get; set; }

    public string Language { get; set; } = "zh";

    public string UpdatedAt { get; set; } = "";

    public double? CodexRemainingPercent { get; set; }

    public double FuncloudTodayCost { get; set; }

    public double FuncloudTotalCost { get; set; }

    public int ErrorCount { get; set; }

    public int AccountCount { get; set; }
}

internal static class WidgetFeedStore
{
    private static readonly JsonSerializerOptions Options = new()
    {
        PropertyNameCaseInsensitive = true,
    };

    private static readonly string FeedPath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "com.barryge1214.aiusagedock",
        "widget.json");

    public static WidgetFeed Load()
    {
        try
        {
            if (!File.Exists(FeedPath))
            {
                return new WidgetFeed();
            }

            using var stream = File.OpenRead(FeedPath);
            return JsonSerializer.Deserialize<WidgetFeed>(stream, Options) ?? new WidgetFeed();
        }
        catch (Exception exception)
        {
            Log.Error("Failed to read widget feed", exception);
            return new WidgetFeed();
        }
    }
}

internal static class WidgetData
{
    public static string Build(WidgetFeed feed)
    {
        var english = string.Equals(feed.Language, "en", StringComparison.OrdinalIgnoreCase);

        var status = feed.AccountCount == 0
            ? (english ? "Open AI Usage Dock to sync" : "请启动 AI Usage Dock 同步数据")
            : feed.ErrorCount > 0
                ? (english
                    ? $"{feed.ErrorCount} account(s) need attention"
                    : $"{feed.ErrorCount} 个账号异常")
                : (english
                    ? $"Up to date · {feed.AccountCount} account(s)"
                    : $"已同步 · {feed.AccountCount} 个账号");

        var data = new Dictionary<string, string>
        {
            ["codexLabel"] = english ? "Codex remaining" : "Codex 剩余",
            ["codexRemaining"] = FormatPercent(feed.CodexRemainingPercent),
            ["funcloudLabel"] = english ? "Funcloud today" : "Funcloud 今日",
            ["funcloudToday"] = FormatMoney(feed.FuncloudTodayCost),
            ["funcloudTotal"] = (english ? "total " : "总计 ") + FormatMoney(feed.FuncloudTotalCost),
            ["status"] = status,
            ["statusColor"] = feed.ErrorCount > 0 ? "Attention" : "Good",
            ["updatedAt"] = FormatTime(feed.UpdatedAt),
            ["refreshLabel"] = english ? "Refresh" : "刷新",
        };

        return JsonSerializer.Serialize(data);
    }

    private static string FormatPercent(double? value) =>
        value is null ? "--" : $"{Math.Round(value.Value)}%";

    private static string FormatMoney(double value) =>
        "$" + value.ToString("0.00", CultureInfo.InvariantCulture);

    private static string FormatTime(string value)
    {
        return DateTimeOffset.TryParse(
            value,
            CultureInfo.InvariantCulture,
            DateTimeStyles.RoundtripKind,
            out var parsed)
            ? parsed.ToLocalTime().ToString("HH:mm", CultureInfo.InvariantCulture)
            : "";
    }
}
