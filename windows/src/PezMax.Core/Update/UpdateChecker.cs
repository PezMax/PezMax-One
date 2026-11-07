using System;
using System.Collections.Generic;
using System.Globalization;
using System.Text;

namespace PezMax.Core.Update;

/// <summary>
/// 版本号：最多 4 段、缺段补 0、允许前缀 <c>v</c>。
/// 归一后至少 3 段、去掉尾部多余的 0（`v1.2` → `1.2.0`，`1.2.0.0` → `1.2.0`）。
/// </summary>
public readonly struct ClientVersion : IComparable<ClientVersion>, IEquatable<ClientVersion>
{
    private readonly int[] _parts;

    private ClientVersion(int[] parts) => _parts = parts;

    public bool IsValid => _parts != null && _parts.Length > 0;

    public static bool TryParse(string? text, out ClientVersion version)
    {
        version = default;
        if (string.IsNullOrWhiteSpace(text)) return false;

        var value = text!.Trim();
        if (value.StartsWith("v", StringComparison.OrdinalIgnoreCase)) value = value.Substring(1);
        if (value.Length == 0) return false;

        var segments = value.Split('.');
        if (segments.Length == 0 || segments.Length > 4) return false;

        var parts = new int[4];
        for (var i = 0; i < segments.Length; i++)
        {
            if (segments[i].Length == 0) return false;
            if (!int.TryParse(segments[i], NumberStyles.None, CultureInfo.InvariantCulture, out var number)) return false;
            parts[i] = number;
        }

        version = new ClientVersion(parts);
        return true;
    }

    public int CompareTo(ClientVersion other)
    {
        if (!IsValid) return other.IsValid ? -1 : 0;
        if (!other.IsValid) return 1;
        for (var i = 0; i < 4; i++)
        {
            var delta = _parts[i].CompareTo(other._parts[i]);
            if (delta != 0) return delta;
        }
        return 0;
    }

    public bool Equals(ClientVersion other) => CompareTo(other) == 0;
    public override bool Equals(object? obj) => obj is ClientVersion other && Equals(other);

    public override int GetHashCode()
    {
        if (!IsValid) return 0;
        var hash = 17;
        for (var i = 0; i < 4; i++) hash = unchecked(hash * 31 + _parts[i]);
        return hash;
    }

    public static bool operator >(ClientVersion left, ClientVersion right) => left.CompareTo(right) > 0;
    public static bool operator <(ClientVersion left, ClientVersion right) => left.CompareTo(right) < 0;

    /// <summary>最多 4 段、去尾部 0、至少 3 段。</summary>
    public override string ToString()
    {
        if (!IsValid) return "";
        var length = 4;
        while (length > 3 && _parts[length - 1] == 0) length--;

        var builder = new StringBuilder();
        for (var i = 0; i < length; i++)
        {
            if (i > 0) builder.Append('.');
            builder.Append(_parts[i].ToString(CultureInfo.InvariantCulture));
        }
        return builder.ToString();
    }
}

public sealed class UpdateAsset
{
    public string Name { get; init; } = "";
    public string Url { get; init; } = "";
    public long Size { get; init; }
}

public sealed class UpdateRelease
{
    public string TagName { get; init; } = "";
    public string HtmlUrl { get; init; } = "";
    public bool Draft { get; init; }
    public bool Prerelease { get; init; }
    public IReadOnlyList<UpdateAsset> Assets { get; init; } = Array.Empty<UpdateAsset>();
}

/// <summary>命中的更新：版本、所在班次、以及按扩展名索引的下载地址（缺项为 null）。</summary>
public sealed class UpdateChoice
{
    public UpdateChoice(ClientVersion version, string releaseTag, string releasePageUrl, IReadOnlyDictionary<string, string?> assets)
    {
        Version = version;
        ReleaseTag = releaseTag;
        ReleasePageUrl = releasePageUrl;
        Assets = assets;
    }

    public ClientVersion Version { get; }
    public string ReleaseTag { get; }
    public string ReleasePageUrl { get; }
    public IReadOnlyDictionary<string, string?> Assets { get; }
}

/// <summary>
/// 应用内更新检查，逐条对齐 <c>spec/fixtures/update/</c> 与 <c>docs/RELEASES.md</c>。
/// 只看资产名：不依赖 Latest 徽章、不依赖 release 顺序；draft 与 prerelease 整条忽略；
/// 只认严格高于当前版本的版本（原地换包靠升版本传递）。
/// </summary>
public static class UpdateChecker
{
    public const string AssetPrefix = "pezmax-one-";

    public static readonly IReadOnlyList<string> WindowsExtensions = new[] { "appinstaller", "msix", "cer" };
    public static readonly IReadOnlyList<string> LinuxExtensions = new[] { "deb", "pkg.tar.zst" };
    public static readonly IReadOnlyList<string> AndroidExtensions = new[] { "apk" };

    public static IReadOnlyList<string> ExtensionsFor(string? platform) => platform switch
    {
        "windows" => WindowsExtensions,
        "linux" => LinuxExtensions,
        "android" => AndroidExtensions,
        _ => Array.Empty<string>(),
    };

    public static UpdateChoice? Find(
        string? currentVersion,
        string? platform,
        string? arch,
        IEnumerable<UpdateRelease>? releases,
        string? mirrorBase = null)
    {
        if (!ClientVersion.TryParse(currentVersion, out var current)) return null;
        if (string.IsNullOrEmpty(platform) || string.IsNullOrEmpty(arch)) return null;
        if (releases == null) return null;

        var extensions = ExtensionsFor(platform);
        if (extensions.Count == 0) return null;

        UpdateChoice? best = null;

        foreach (var release in releases)
        {
            if (release == null || release.Draft || release.Prerelease) continue;

            // 班次内取最高：先在该 release 的资产里挑出最高版本。
            ClientVersion? highest = null;
            foreach (var asset in release.Assets ?? Array.Empty<UpdateAsset>())
            {
                if (!TryParseAsset(asset.Name, platform!, arch!, extensions, out var version)) continue;
                if (!(version > current)) continue;
                if (highest == null || version > highest.Value) highest = version;
            }

            if (highest == null) continue;
            if (best != null && !(highest.Value > best.Version)) continue;

            // 三件套必须来自**同一个最高版本**；缺项留 null，不借旧版本的资产补齐。
            var map = new Dictionary<string, string?>(StringComparer.Ordinal);
            foreach (var extension in extensions) map[extension] = null;
            foreach (var asset in release.Assets ?? Array.Empty<UpdateAsset>())
            {
                if (!TryParseAsset(asset.Name, platform!, arch!, extensions, out var version)) continue;
                if (!version.Equals(highest.Value)) continue;
                var extension = ExtensionOf(asset.Name, extensions);
                if (extension == null) continue;
                map[extension] = RewriteForMirror(asset.Url, mirrorBase);
            }

            best = new UpdateChoice(highest.Value, release.TagName, release.HtmlUrl, map);
        }

        return best;
    }

    /// <summary>
    /// 解析资产名：<c>pezmax-one-[平台]-[架构]-[版本].[扩展名]</c>。
    /// 扩展名按**最长后缀**匹配（`pkg.tar.zst` 不会被当成 `zst`），平台与架构段逐字符相等。
    /// </summary>
    public static bool TryParseAsset(
        string? name,
        string platform,
        string arch,
        IReadOnlyList<string> extensions,
        out ClientVersion version)
    {
        version = default;
        if (string.IsNullOrEmpty(name)) return false;
        var text = name!;
        if (!text.StartsWith(AssetPrefix, StringComparison.Ordinal)) return false;

        var extension = ExtensionOf(text, extensions);
        if (extension == null) return false;

        // 去掉 ".<扩展名>"，剩下的形如 "<platform>-<arch>-<version>"。
        // 版本段里只有数字和点，不会出现连字符，所以按 '-' 正好切三段。
        var stem = text.Substring(0, text.Length - extension.Length - 1);
        var pieces = stem.Substring(AssetPrefix.Length).Split('-');
        if (pieces.Length != 3) return false;
        if (!string.Equals(pieces[0], platform, StringComparison.Ordinal)) return false;
        if (!string.Equals(pieces[1], arch, StringComparison.Ordinal)) return false;

        return ClientVersion.TryParse(pieces[2], out version);
    }

    /// <summary>命中的扩展名（最长匹配）；不是可安装扩展名时返回 null。</summary>
    public static string? ExtensionOf(string? name, IReadOnlyList<string> extensions)
    {
        if (string.IsNullOrEmpty(name)) return null;
        var text = name!;
        string? match = null;
        foreach (var extension in extensions)
        {
            if (!text.EndsWith("." + extension, StringComparison.OrdinalIgnoreCase)) continue;
            if (match == null || extension.Length > match.Length) match = extension;
        }
        return match;
    }

    /// <summary>镜像只改「从哪儿下」：<c>mirrorBase + 原 URL 的 path 与 query</c>。</summary>
    public static string RewriteForMirror(string? url, string? mirrorBase)
    {
        var original = url ?? "";
        var mirror = (mirrorBase ?? "").Trim();
        if (mirror.Length == 0 || original.Length == 0) return original;
        if (!Uri.TryCreate(original, UriKind.Absolute, out var parsed)) return original;

        return mirror.TrimEnd('/') + parsed.PathAndQuery;
    }
}
