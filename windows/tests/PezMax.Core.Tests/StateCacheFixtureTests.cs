using System.Text.Json;
using PezMax.Core.State;
using Xunit;

namespace PezMax.Core.Tests;

/// <summary>哈希缓存的本地状态机，逐条对齐 `spec/fixtures/state-cache/cases.json`。</summary>
public class StateCacheFixtureTests
{
    [Fact]
    public void FeedCache_MatchesStateCacheFixtures()
    {
        using var document = Spec.LoadJson("fixtures", "state-cache", "cases.json");
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
                case "key":
                {
                    var current = FeedCache.Key(input.GetProperty("serverUrl").GetString(), input.GetProperty("resource").GetString()!);
                    var hit = false;
                    foreach (var entry in input.GetProperty("entries").EnumerateArray())
                    {
                        var key = FeedCache.Key(entry.GetProperty("serverUrl").GetString(), entry.GetProperty("resource").GetString()!);
                        if (string.Equals(key, current, StringComparison.Ordinal)) hit = true;
                    }
                    Assert.True(expect.GetProperty("hit").GetBoolean() == hit, $"{id}: hit = {hit}");
                    break;
                }

                case "load":
                {
                    var usable = ReadUsable(input.GetProperty("entry"));
                    Assert.True(expect.GetProperty("usable").GetBoolean() == usable, $"{id}: usable = {usable}");
                    break;
                }

                case "apply":
                {
                    var previous = input.GetProperty("previous");
                    var response = input.GetProperty("response");
                    var state = new FeedCacheState(
                        previous.GetProperty("hash").GetString(),
                        previous.GetProperty("usable").GetBoolean());

                    var succeeded = response.GetProperty("kind").GetString() == "success";
                    var reply = succeeded
                        ? new FeedResponse(
                            true,
                            response.GetProperty("unchanged").GetBoolean(),
                            response.TryGetProperty("hash", out var h) ? h.GetString() : null,
                            response.TryGetProperty("data", out var data) && data.ValueKind != JsonValueKind.Null)
                        : FeedResponse.Failure();

                    var decision = FeedCache.Decide(state, reply);
                    Assert.True(expect.GetProperty("action").GetString() == ActionName(decision.Action),
                        $"{id}: action = {decision.Action}");
                    if (expect.TryGetProperty("hash", out var expectedHash))
                        Assert.True(expectedHash.GetString() == decision.Hash, $"{id}: hash = {decision.Hash}");
                    break;
                }

                case "stale":
                {
                    var apply = FeedCache.Accepts(
                        input.GetProperty("appliedSeq").GetInt64(),
                        input.GetProperty("incomingSeq").GetInt64());
                    Assert.True(expect.GetProperty("apply").GetBoolean() == apply, $"{id}: apply = {apply}");
                    break;
                }

                default:
                    throw new Xunit.Sdk.XunitException($"{id}: 未知 kind {kind}");
            }

            checkedCases++;
        }

        Assert.True(checkedCases >= 17, $"哈希缓存夹具跑到的用例太少：{checkedCases}");
    }

    [Fact]
    public void Key_KeepsPathPrefixButIgnoresTrailingSlashAndHostCase()
    {
        var a = FeedCache.Key("https://PZ.example.com/", "tree");
        var b = FeedCache.Key("https://pz.example.com/pez/", "tree");
        var c = FeedCache.Key("https://pz.example.com", "tree");
        Assert.Equal(c, a);
        Assert.NotEqual(c, b);
    }

    private static bool ReadUsable(JsonElement entry)
    {
        if (entry.ValueKind != JsonValueKind.Object) return false;

        // 原始文本损坏 → 解析失败，按没有缓存处理。
        if (entry.TryGetProperty("raw", out _)) return false;

        var hash = entry.TryGetProperty("hash", out var h) && h.ValueKind == JsonValueKind.String ? h.GetString() : null;
        var hasPayload = entry.TryGetProperty("payload", out var payload) && payload.ValueKind != JsonValueKind.Null;
        return FeedCache.IsUsable(hash, hasPayload);
    }

    private static string ActionName(FeedAction action) => action switch
    {
        FeedAction.Store => "store",
        FeedAction.Keep => "keep",
        FeedAction.Refetch => "refetch",
        FeedAction.KeepAndWarn => "keep-and-warn",
        _ => action.ToString().ToLowerInvariant(),
    };
}
