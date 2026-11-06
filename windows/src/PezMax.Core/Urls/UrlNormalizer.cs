using System;

namespace PezMax.Core.Urls;

/// <summary>
/// 文件 URL 过渡归一：相对路径补 base；内网 host 改写到当前 base。
/// 上游修库后按 spec/fixtures/url 逐条撤除。
/// </summary>
public static class UrlNormalizer
{
    private static readonly string[] InternalHosts = { "localhost", "127.0.0.1", "minio", "server", "172." };

    public static string Normalize(string? url, string baseUrl)
    {
        var text = (url ?? "").Trim();
        if (text.Length == 0) return "";
        if (text.StartsWith("data:", StringComparison.OrdinalIgnoreCase) ||
            text.StartsWith("blob:", StringComparison.OrdinalIgnoreCase))
        {
            return text;
        }

        var normalizedBase = Net.DatumHttp.NormalizeBaseUrl(baseUrl);

        if (text.StartsWith("minio://", StringComparison.OrdinalIgnoreCase))
        {
            var tail = text.Substring("minio://".Length).TrimStart('/');
            return normalizedBase + "/" + tail;
        }

        if (!text.StartsWith("http://", StringComparison.OrdinalIgnoreCase) &&
            !text.StartsWith("https://", StringComparison.OrdinalIgnoreCase))
        {
            return normalizedBase + (text.StartsWith("/", StringComparison.Ordinal) ? text : "/" + text);
        }

        if (!Uri.TryCreate(text, UriKind.Absolute, out var parsed) ||
            !Uri.TryCreate(normalizedBase, UriKind.Absolute, out _))
        {
            return text;
        }

        if (!IsInternalHost(parsed.Host)) return text;

        // 只换 origin：path 与 query 原样搬过去。
        // 用 UriBuilder 会把 `%E5%9B%BE…` 解码成中文再重新拼，双重转义/丢失转义都踩过，所以直接拼字符串。
        var origin = normalizedBase;
        var pathAndQuery = parsed.PathAndQuery;
        if (pathAndQuery.Length <= 1) return origin;
        return origin + pathAndQuery;
    }

    private static bool IsInternalHost(string host)
    {
        foreach (var candidate in InternalHosts)
        {
            if (host.Equals(candidate, StringComparison.OrdinalIgnoreCase)) return true;
            if (host.StartsWith(candidate, StringComparison.OrdinalIgnoreCase)) return true;
        }
        return false;
    }
}
