using System;
using System.Collections.Generic;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Json;

namespace PezMax.Core.Update;

/// <summary>
/// GitHub Releases 的取数与解析。放在 Core 是为了三端共用同一套字段映射；
/// 挑哪个版本**不**在这里（那是 <see cref="UpdateChecker"/>，有夹具锁着）。
/// <para>
/// 未认证 60 次 / 小时 / IP（`docs/RELEASES.md`「应用内更新」第 1 条），启动后查一次足够。
/// GitHub 要求带 <c>User-Agent</c>，否则直接 403。
/// </para>
/// </summary>
public static class ReleaseFeed
{
    public const string DefaultRepo = "PezMax/PezMax-One";
    public const string UserAgent = "PezMax-One";

    /// <summary>班次是滚动的，最近 30 个 release 足够覆盖当前各平台的最新版。</summary>
    public static string UrlFor(string repo) =>
        "https://api.github.com/repos/" + repo + "/releases?per_page=30";

    public static async Task<IReadOnlyList<UpdateRelease>> FetchAsync(
        HttpClient client,
        string? repo = null,
        CancellationToken cancellationToken = default)
    {
        if (client == null) throw new ArgumentNullException(nameof(client));
        var target = string.IsNullOrWhiteSpace(repo) ? DefaultRepo : repo!;

        using var request = new HttpRequestMessage(HttpMethod.Get, UrlFor(target));
        request.Headers.TryAddWithoutValidation("User-Agent", UserAgent);
        request.Headers.TryAddWithoutValidation("Accept", "application/vnd.github+json");

        using var response = await client.SendAsync(request, cancellationToken).ConfigureAwait(false);
        if (!response.IsSuccessStatusCode) return Array.Empty<UpdateRelease>();

        var json = await response.Content.ReadAsStringAsync().ConfigureAwait(false);
        return Parse(json);
    }

    /// <summary>解析 GitHub 的 release 数组；解析不了就返回空（更新检查永远不该把应用弄崩）。</summary>
    public static IReadOnlyList<UpdateRelease> Parse(string? json)
    {
        var releases = new List<UpdateRelease>();
        if (string.IsNullOrWhiteSpace(json)) return releases;

        JsonValue root;
        try
        {
            root = JsonValue.Parse(json!);
        }
        catch (JsonParseException)
        {
            return releases;
        }

        if (!root.IsArray) return releases;
        foreach (var item in root.Items)
        {
            if (!item.IsObject) continue;

            var assets = new List<UpdateAsset>();
            if (item.GetArray("assets") is { } assetValues)
            {
                foreach (var asset in assetValues)
                {
                    if (!asset.IsObject) continue;
                    assets.Add(new UpdateAsset
                    {
                        Name = asset.GetString("name") ?? "",
                        Url = asset.GetString("browser_download_url") ?? "",
                        Size = asset.GetLong("size"),
                    });
                }
            }

            releases.Add(new UpdateRelease
            {
                TagName = item.GetString("tag_name") ?? "",
                HtmlUrl = item.GetString("html_url") ?? "",
                Draft = item.GetBool("draft"),
                Prerelease = item.GetBool("prerelease"),
                Assets = assets,
            });
        }

        return releases;
    }
}
