using System.Text.Json;
using PezMax.Core.Urls;
using Xunit;

namespace PezMax.Core.Tests;

/// <summary>
/// 文件 URL 归一的过渡规则，逐条对齐 `spec/fixtures/url/cases.json`。
/// 规则撤除条件写在同目录 README 里——上游 ETL 改完库就删第 3、5 条。
/// </summary>
public class UrlFixtureTests
{
    [Fact]
    public void Normalize_MatchesUrlFixtures()
    {
        using var document = Spec.LoadJson("fixtures", "url", "cases.json");
        var cases = document.RootElement;
        Assert.Equal(JsonValueKind.Array, cases.ValueKind);

        var checkedCases = 0;
        foreach (var item in cases.EnumerateArray())
        {
            var id = item.GetProperty("id").GetString()!;
            var input = item.GetProperty("input");
            var url = input.GetProperty("url").GetString();
            var baseUrl = input.GetProperty("baseUrl").GetString()!;
            var expected = item.GetProperty("expect").GetProperty("url").GetString();

            var actual = UrlNormalizer.Normalize(url, baseUrl);
            Assert.True(expected == actual, $"{id}: 期望「{expected}」，实际「{actual}」");
            checkedCases++;
        }

        Assert.True(checkedCases >= 20, $"URL 夹具跑到的用例太少：{checkedCases}");
    }
}
