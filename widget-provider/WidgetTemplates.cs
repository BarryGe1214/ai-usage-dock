namespace AiUsageDock.Widgets;

internal static class WidgetTemplates
{
    private const string SmallTemplate = """
        {
          "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
          "type": "AdaptiveCard",
          "version": "1.5",
          "body": [
            {
              "type": "TextBlock",
              "text": "${codexLabel}",
              "size": "Small",
              "isSubtle": true,
              "spacing": "None",
              "wrap": true
            },
            {
              "type": "TextBlock",
              "text": "${codexRemaining}",
              "size": "ExtraLarge",
              "weight": "Bolder",
              "spacing": "None",
              "wrap": true
            },
            {
              "type": "TextBlock",
              "text": "${funcloudLabel} ${funcloudToday}",
              "size": "Small",
              "isSubtle": true,
              "spacing": "Small",
              "wrap": true
            }
          ]
        }
        """;

    private const string MediumTemplate = """
        {
          "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
          "type": "AdaptiveCard",
          "version": "1.5",
          "body": [
            {
              "type": "ColumnSet",
              "columns": [
                {
                  "type": "Column",
                  "width": "stretch",
                  "items": [
                    {
                      "type": "TextBlock",
                      "text": "${codexLabel}",
                      "size": "Small",
                      "isSubtle": true,
                      "spacing": "None",
                      "wrap": true
                    },
                    {
                      "type": "TextBlock",
                      "text": "${codexRemaining}",
                      "size": "ExtraLarge",
                      "weight": "Bolder",
                      "spacing": "None",
                      "wrap": true
                    }
                  ]
                },
                {
                  "type": "Column",
                  "width": "auto",
                  "items": [
                    {
                      "type": "TextBlock",
                      "text": "${funcloudLabel}",
                      "size": "Small",
                      "isSubtle": true,
                      "spacing": "None",
                      "horizontalAlignment": "Right",
                      "wrap": true
                    },
                    {
                      "type": "TextBlock",
                      "text": "${funcloudToday}",
                      "size": "Medium",
                      "weight": "Bolder",
                      "spacing": "None",
                      "horizontalAlignment": "Right",
                      "wrap": true
                    },
                    {
                      "type": "TextBlock",
                      "text": "${funcloudTotal}",
                      "size": "Small",
                      "isSubtle": true,
                      "spacing": "None",
                      "horizontalAlignment": "Right",
                      "wrap": true
                    }
                  ]
                }
              ]
            },
            {
              "type": "TextBlock",
              "text": "${status} · ${updatedAt}",
              "size": "Small",
              "isSubtle": true,
              "color": "${statusColor}",
              "spacing": "Medium",
              "wrap": true
            }
          ],
          "actions": [
            {
              "type": "Action.Execute",
              "title": "${refreshLabel}",
              "verb": "refresh"
            }
          ]
        }
        """;

    public static string ForSize(string size) =>
        string.Equals(size, "small", StringComparison.OrdinalIgnoreCase)
            ? SmallTemplate
            : MediumTemplate;
}
