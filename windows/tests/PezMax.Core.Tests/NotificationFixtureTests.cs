using System.Text.Json;
using PezMax.Core.Models;
using PezMax.Core.Notify;
using PezMax.Core.State;
using Xunit;

namespace PezMax.Core.Tests;

/// <summary>通知的客户端侧判定，逐条对齐 `spec/fixtures/notification/cases.json`。</summary>
public class NotificationFixtureTests
{
    [Fact]
    public void Policy_MatchesNotificationFixtures()
    {
        using var document = Spec.LoadJson("fixtures", "notification", "cases.json");
        var cases = document.RootElement;
        Assert.Equal(JsonValueKind.Array, cases.ValueKind);

        var checkedCases = 0;
        foreach (var item in cases.EnumerateArray())
        {
            var id = item.GetProperty("id").GetString()!;
            var input = item.GetProperty("input");
            var kind = input.GetProperty("kind").GetString();
            var expect = item.GetProperty("expect");

            switch (kind)
            {
                case "channel":
                {
                    var userId = input.GetProperty("userId").GetInt64();
                    var channel = input.GetProperty("channel").GetString() == "scroll"
                        ? NotifyChannel.Scroll
                        : NotifyChannel.Popup;
                    var notification = ReadNotification(input.GetProperty("notification"));

                    var shown = NotificationPolicy.ShouldShow(notification, userId, channel);
                    Assert.True(expect.GetProperty("show").GetBoolean() == shown, $"{id}: show = {shown}");
                    break;
                }

                case "popup-plan":
                {
                    var userId = input.GetProperty("userId").GetInt64();
                    var seen = new NoticeKeySet();
                    foreach (var key in input.GetProperty("seen").EnumerateArray())
                        seen.Mark(key.GetProperty("userId").GetInt64(), key.GetProperty("notifyId").GetInt64());

                    var rows = input.GetProperty("rows").EnumerateArray().Select(ReadNotification).ToList();
                    var planned = NotificationPolicy.PlanPopup(rows, seen, userId).Select(n => n.NotifyId).ToList();
                    var expected = expect.GetProperty("notifyIds").EnumerateArray().Select(e => e.GetInt64()).ToList();

                    Assert.True(expected.SequenceEqual(planned),
                        $"{id}: 期望 [{string.Join(",", expected)}]，实际 [{string.Join(",", planned)}]");
                    break;
                }

                case "read-set":
                {
                    var userId = input.GetProperty("userId").GetInt64();
                    var notifyId = input.GetProperty("notifyId").GetInt64();
                    var reads = new NoticeKeySet();
                    foreach (var read in input.GetProperty("reads").EnumerateArray())
                        reads.Mark(read.GetProperty("userId").GetInt64(), read.GetProperty("notifyId").GetInt64());

                    var isRead = reads.Contains(userId, notifyId);
                    Assert.True(expect.GetProperty("read").GetBoolean() == isRead, $"{id}: read = {isRead}");
                    break;
                }

                case "feed-cache":
                {
                    var previous = input.GetProperty("previous");
                    var response = input.GetProperty("response");
                    var state = new FeedCacheState(
                        previous.GetProperty("hash").GetString(),
                        previous.GetProperty("hasPayload").GetBoolean());

                    var data = response.GetProperty("data");
                    var reply = new FeedResponse(
                        succeeded: true,
                        unchanged: response.GetProperty("unchanged").GetBoolean(),
                        hash: response.TryGetProperty("hash", out var h) ? h.GetString() : null,
                        hasData: data.ValueKind == JsonValueKind.Array || data.ValueKind == JsonValueKind.Object);

                    var decision = FeedCache.Decide(state, reply);
                    Assert.True(expect.GetProperty("action").GetString() == ActionName(decision.Action),
                        $"{id}: action = {decision.Action}");
                    if (expect.TryGetProperty("hash", out var expectedHash))
                        Assert.True(expectedHash.GetString() == decision.Hash, $"{id}: hash = {decision.Hash}");
                    break;
                }

                case "dismissible":
                {
                    var now = DateTimeOffset.Parse(input.GetProperty("now").GetString()!);
                    var notification = ReadNotification(input.GetProperty("notification"));
                    var dismissible = NotificationPolicy.IsDismissible(notification, now);
                    Assert.True(expect.GetProperty("dismissible").GetBoolean() == dismissible,
                        $"{id}: dismissible = {dismissible}");
                    break;
                }

                default:
                    throw new Xunit.Sdk.XunitException($"{id}: 未知 kind {kind}");
            }

            checkedCases++;
        }

        Assert.True(checkedCases >= 25, $"通知夹具跑到的用例太少：{checkedCases}");
    }

    [Fact]
    public void Policy_TreatsMissingMaintenanceStartAsDismissible()
    {
        var item = new NotificationItem { NotifyId = 5, NotifyType = NotificationPolicy.TypeMaintenance };
        Assert.True(NotificationPolicy.IsDismissible(item, DateTimeOffset.Now));
    }

    private static string ActionName(FeedAction action) => action switch
    {
        FeedAction.Store => "replace",
        FeedAction.Keep => "keep",
        FeedAction.Refetch => "refetch",
        FeedAction.KeepAndWarn => "keep-and-warn",
        _ => action.ToString().ToLowerInvariant(),
    };

    private static NotificationItem ReadNotification(JsonElement element) => new()
    {
        NotifyId = Long(element, "notifyId"),
        NotifyType = Str(element, "notifyType"),
        DisplayMode = Str(element, "displayMode"),
        UploadUserId = Long(element, "uploadUserId"),
        MaterialId = Long(element, "materialId"),
        MaintenanceStartTime = Str(element, "maintenanceStartTime"),
        MaintenanceEndTime = Str(element, "maintenanceEndTime"),
        Title = Str(element, "title"),
    };

    private static string Str(JsonElement element, string name) =>
        element.TryGetProperty(name, out var value) && value.ValueKind == JsonValueKind.String ? value.GetString()! : "";

    private static long Long(JsonElement element, string name) =>
        element.TryGetProperty(name, out var value) && value.ValueKind == JsonValueKind.Number ? value.GetInt64() : 0;
}
