using System.Text.Json;
using PezMax.Core.Transfer;
using Xunit;

namespace PezMax.Core.Tests;

/// <summary>
/// 断点续传决策表：逐条对齐 `spec/fixtures/download/cases.json`。
/// 夹具里的源是 PezMax-Go 的 `ServeContent` 行为；旧 Electron 端只借用了判断顺序。
/// </summary>
public class DownloadFixtureTests
{
    [Fact]
    public void Planner_MatchesDownloadFixtures()
    {
        using var document = Spec.LoadJson("fixtures", "download", "cases.json");
        var cases = document.RootElement;
        Assert.Equal(JsonValueKind.Array, cases.ValueKind);

        var checkedCases = 0;
        foreach (var item in cases.EnumerateArray())
        {
            var id = item.GetProperty("id").GetString()!;
            var input = item.GetProperty("input");
            var kind = input.GetProperty("kind").GetString();
            var expect = item.GetProperty("expect");

            var decision = kind switch
            {
                "before" => DownloadPlanner.BeforeRequest(ReadPart(input)),
                "response" => DownloadPlanner.Plan(ReadPart(input), ReadResponse(input)),
                "after" => DownloadPlanner.AfterWrite(
                    input.GetProperty("written").GetInt64(),
                    input.GetProperty("total").GetInt64()),
                _ => throw new Xunit.Sdk.XunitException($"{id}: 未知 kind {kind}"),
            };

            var expectedAction = expect.GetProperty("action").GetString();
            Assert.True(
                string.Equals(expectedAction, ActionName(decision.Action), StringComparison.Ordinal),
                $"{id}: 期望动作 {expectedAction}，实际 {ActionName(decision.Action)}");

            if (expect.TryGetProperty("offset", out var offset))
                Assert.True(offset.GetInt64() == decision.Offset, $"{id}: offset = {decision.Offset}");

            if (expect.TryGetProperty("total", out var total))
                Assert.True(total.GetInt64() == decision.Total, $"{id}: total = {decision.Total}");

            if (expect.TryGetProperty("discardPart", out var discard))
                Assert.True(discard.GetBoolean() == decision.DiscardPart, $"{id}: discardPart = {decision.DiscardPart}");

            if (expect.TryGetProperty("range", out var range))
                Assert.True(range.GetString() == decision.RangeHeader, $"{id}: Range = {decision.RangeHeader}");

            if (expect.TryGetProperty("ifRange", out var ifRange))
                Assert.True(ifRange.GetString() == decision.IfRangeHeader, $"{id}: If-Range = {decision.IfRangeHeader}");

            checkedCases++;
        }

        Assert.True(checkedCases >= 20, $"续传夹具跑到的用例太少：{checkedCases}");
    }

    [Fact]
    public void Planner_KeepsPartOnTransientFailures()
    {
        // 5xx 是可以重试的失败：把半成品删了等于让用户白下一遍。
        var decision = DownloadPlanner.Plan(
            new DownloadPartState { Size = 2048, ETag = "\"datum-12-4096\"" },
            new DownloadResponseInfo { Status = 503 });
        Assert.Equal(DownloadAction.Fail, decision.Action);
        Assert.False(decision.DiscardPart);
    }

    private static DownloadPartState? ReadPart(JsonElement input)
    {
        if (!input.TryGetProperty("part", out var part)) return null;
        return new DownloadPartState
        {
            Size = part.TryGetProperty("size", out var size) ? size.GetInt64() : 0,
            ETag = part.TryGetProperty("etag", out var etag) ? etag.GetString() : null,
        };
    }

    private static DownloadResponseInfo? ReadResponse(JsonElement input)
    {
        if (!input.TryGetProperty("response", out var response)) return null;
        return new DownloadResponseInfo
        {
            Status = response.GetProperty("status").GetInt32(),
            ETag = response.TryGetProperty("etag", out var etag) ? etag.GetString() : null,
            ContentLength = response.TryGetProperty("contentLength", out var length) ? length.GetInt64() : 0,
            ContentRange = response.TryGetProperty("contentRange", out var range) ? range.GetString() : null,
        };
    }

    private static string ActionName(DownloadAction action) => action switch
    {
        DownloadAction.Start => "start",
        DownloadAction.Resume => "resume",
        DownloadAction.Restart => "restart",
        DownloadAction.Complete => "complete",
        DownloadAction.Continue => "continue",
        DownloadAction.Fail => "fail",
        DownloadAction.SessionExpired => "session-expired",
        _ => action.ToString().ToLowerInvariant(),
    };
}
