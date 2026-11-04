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
            !Uri.TryCreate(normalizedBase, UriKind.Absolute, out var baseUri))
        {
            return text;
        }

        if (!IsInternalHost(parsed.Host)) return text;

        var builder = new UriBuilder(baseUri)
        {
            Path = parsed.AbsolutePath,
            Query = parsed.Query.TrimStart('?'),
        };
        return builder.Uri.ToString();
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
