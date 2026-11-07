using System.Text.Json;
using PezMax.Core.Update;
using Xunit;

namespace PezMax.Core.Tests;

/// <summary>更新检查：资产解析、版本比较、班次挑选、镜像前缀，对齐 `spec/fixtures/update/cases.json`。</summary>
public class UpdateFixtureTests
{
    [Fact]
    public void UpdateChecker_MatchesUpdateFixtures()
    {
        using var document = Spec.LoadJson("fixtures", "update", "cases.json");
        var cases = document.RootElement;
        Assert.Equal(JsonValueKind.Array, cases.ValueKind);

        var checkedCases = 0;
        foreach (var item in cases.EnumerateArray())
        {
            var id = item.GetProperty("id").GetString()!;
            var input = item.GetProperty("input");
            var expect = item.GetProperty("expect");

            var releases = input.GetProperty("releases").EnumerateArray().Select(ReadRelease).ToList();
            var mirror = input.TryGetProperty("mirrorBase", out var m) ? m.GetString() : null;

            var choice = UpdateChecker.Find(
                input.GetProperty("currentVersion").GetString(),
                input.GetProperty("platform").GetString(),
                input.GetProperty("arch").GetString(),
                releases,
                mirror);

            if (expect.ValueKind == JsonValueKind.Null)
            {
                Assert.True(choice == null, $"{id}: 期望无更新，实际选中 {choice?.Version}");
                checkedCases++;
                continue;
            }

            Assert.True(choice != null, $"{id}: 期望有更新，实际没有");
            Assert.True(expect.GetProperty("version").GetString() == choice!.Version.ToString(),
                $"{id}: version = {choice.Version}");
            Assert.True(expect.GetProperty("releaseTag").GetString() == choice.ReleaseTag,
                $"{id}: tag = {choice.ReleaseTag}");
            Assert.True(expect.GetProperty("releasePageUrl").GetString() == choice.ReleasePageUrl,
                $"{id}: url = {choice.ReleasePageUrl}");

            foreach (var asset in expect.GetProperty("assets").EnumerateObject())
            {
                choice.Assets.TryGetValue(asset.Name, out var actual);
                var expected = asset.Value.ValueKind == JsonValueKind.Null ? null : asset.Value.GetString();
                Assert.True(expected == actual, $"{id}: assets[{asset.Name}] 期望 {expected ?? "null"}，实际 {actual ?? "null"}");
            }

            checkedCases++;
        }

        Assert.True(checkedCases >= 23, $"更新夹具跑到的用例数太少：{checkedCases}");
    }

    [Fact]
    public void Version_ParsesAndNormalizes()
    {
        Assert.True(ClientVersion.TryParse("v1.2", out var shortVersion));
        Assert.Equal("1.2.0", shortVersion.ToString());

        Assert.True(ClientVersion.TryParse("1.2.0.0", out var fourZeros));
        Assert.Equal("1.2.0", fourZeros.ToString());

        Assert.True(ClientVersion.TryParse("1.2.0.5", out var four));
        Assert.Equal("1.2.0.5", four.ToString());

        Assert.False(ClientVersion.TryParse("1.2.3.4.5", out _));
        Assert.False(ClientVersion.TryParse("abc", out _));
        Assert.False(ClientVersion.TryParse("", out _));
    }

    [Fact]
    public void Asset_WrongExtensionIsIgnored()
    {
        Assert.False(UpdateChecker.TryParseAsset(
            "pezmax-one-windows-x86_64-1.2.0.msix.asc", "windows", "x86_64",
            UpdateChecker.WindowsExtensions, out _));
    }

    private static UpdateRelease ReadRelease(JsonElement element) => new()
    {
        TagName = Str(element, "tag_name"),
        HtmlUrl = Str(element, "html_url"),
        Draft = element.TryGetProperty("draft", out var draft) && draft.GetBoolean(),
        Prerelease = element.TryGetProperty("prerelease", out var pre) && pre.GetBoolean(),
        Assets = element.TryGetProperty("assets", out var assets) && assets.ValueKind == JsonValueKind.Array
            ? assets.EnumerateArray().Select(a => new UpdateAsset
            {
                Name = Str(a, "name"),
                Url = Str(a, "browser_download_url"),
                Size = a.TryGetProperty("size", out var size) && size.ValueKind == JsonValueKind.Number ? size.GetInt64() : 0,
            }).ToList()
            : new List<UpdateAsset>(),
    };

    private static string Str(JsonElement element, string name) =>
        element.TryGetProperty(name, out var value) && value.ValueKind == JsonValueKind.String ? value.GetString()! : "";
}
