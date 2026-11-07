using System;

namespace PezMax.Core.State;

public enum FeedAction
{
    /// <summary>用响应里的 payload 与 hash 整体替换本地。</summary>
    Store,

    /// <summary>沿用本地 payload（服务端说未变，或请求失败但有旧数据）。</summary>
    Keep,

    /// <summary>服务端说未变、但本地没有可用 payload：立刻<strong>不带 hash</strong> 重取。</summary>
    Refetch,

    /// <summary>响应自相矛盾（未变却给了新 hash、变了却没给 data）：保留本地并报错，别清空。</summary>
    KeepAndWarn,
}

public sealed class FeedDecision
{
    public FeedDecision(FeedAction action, string? hash)
    {
        Action = action;
        Hash = hash;
    }

    public FeedAction Action { get; }
    public string? Hash { get; }
}

/// <summary>本地缓存状态：上次的 hash + 有没有可用的 payload。</summary>
public sealed class FeedCacheState
{
    public static readonly FeedCacheState Empty = new();

    public FeedCacheState(string? hash = null, bool hasPayload = false)
    {
        Hash = hash;
        HasPayload = hasPayload;
    }

    public string? Hash { get; }
    public bool HasPayload { get; }
}

/// <summary>一次哈希缓存的响应；失败时只有 <see cref="Succeeded"/> 为 false。</summary>
public sealed class FeedResponse
{
    public FeedResponse(bool succeeded, bool unchanged = false, string? hash = null, bool hasData = false)
    {
        Succeeded = succeeded;
        Unchanged = unchanged;
        Hash = hash;
        HasData = hasData;
    }

    public bool Succeeded { get; }
    public bool Unchanged { get; }
    public string? Hash { get; }
    public bool HasData { get; }

    public static FeedResponse Failure() => new(false);
}

/// <summary>
/// 哈希缓存的本地状态机，逐条对齐 <c>spec/fixtures/state-cache/</c>。
/// 树 / 排行（随机 hash + 写时失效）与通知 feed（内容哈希）共用这一套；
/// 信封层面的规则见 <c>spec/api/envelope.md</c> §5。
/// </summary>
public static class FeedCache
{
    public static FeedDecision Decide(FeedCacheState? local, FeedResponse? response)
    {
        var state = local ?? FeedCacheState.Empty;
        var reply = response ?? FeedResponse.Failure();

        // 失败：保留旧数据继续显示，绝不用错误覆盖缓存。
        if (!reply.Succeeded) return new FeedDecision(FeedAction.Keep, state.Hash);

        if (reply.Unchanged)
        {
            // 本地没有可用 payload（缓存被清 / 首次启动）：立刻不带 hash 重取，
            // 而不是把 data:null 当空列表。
            if (!state.HasPayload) return new FeedDecision(FeedAction.Refetch, null);
            return new FeedDecision(FeedAction.Keep, Coalesce(reply.Hash, state.Hash));
        }

        if (!reply.HasData) return new FeedDecision(FeedAction.KeepAndWarn, state.Hash);
        if (string.IsNullOrEmpty(reply.Hash)) return new FeedDecision(FeedAction.KeepAndWarn, state.Hash);

        return new FeedDecision(FeedAction.Store, reply.Hash);
    }

    /// <summary>缓存读出来的那一刻：文件在、能解析、hash 非空、payload 非 null，才算可用。</summary>
    public static bool IsUsable(string? hash, bool hasPayload, bool parseFailed = false) =>
        !parseFailed && !string.IsNullOrEmpty(hash) && hasPayload;

    /// <summary>并发响应：只接受序号不小于已应用序号的；先发后到的旧响应丢弃。</summary>
    public static bool Accepts(long appliedSequence, long incomingSequence) => incomingSequence >= appliedSequence;

    /// <summary>
    /// 缓存键：<c>(服务器地址, 资源)</c>。换服务器必须未命中——否则渲染的是另一个站点的数据；
    /// 树与排行也不能互相顶替。
    /// </summary>
    public static string Key(string? serverUrl, string resource)
    {
        var server = NormalizeServer(serverUrl);
        var name = NormalizeResource(resource);
        return server + "|" + name;
    }

    /// <summary>服务器地址归一：去空白、去末尾斜杠、scheme 与 host 小写（路径前缀保持原样）。</summary>
    public static string NormalizeServer(string? serverUrl)
    {
        var text = (serverUrl ?? "").Trim();
        if (text.Length == 0) return "";

        if (Uri.TryCreate(text, UriKind.Absolute, out var parsed))
        {
            var path = parsed.AbsolutePath;
            if (path.Length > 1) path = path.TrimEnd('/');
            if (path == "/") path = "";
            return parsed.Scheme.ToLowerInvariant() + "://" + parsed.Host.ToLowerInvariant() +
                   (parsed.IsDefaultPort ? "" : ":" + parsed.Port) + path;
        }

        return text.TrimEnd('/');
    }

    private static string NormalizeResource(string? resource) => (resource ?? "").Trim().ToLowerInvariant();

    private static string? Coalesce(string? preferred, string? fallback) =>
        string.IsNullOrEmpty(preferred) ? fallback : preferred;
}
