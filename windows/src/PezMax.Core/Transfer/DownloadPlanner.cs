using System;
using System.Globalization;
using System.Text.RegularExpressions;

namespace PezMax.Core.Transfer;

public enum DownloadAction
{
    /// <summary>从 0 开始写；没有半成品时的正常开局。</summary>
    Start,

    /// <summary>追加写到已有半成品，偏移 = 半成品字节数。</summary>
    Resume,

    /// <summary>删掉半成品与元数据，从 0 重下。</summary>
    Restart,

    /// <summary>字节数已等于总大小，改名收工。</summary>
    Complete,

    /// <summary>继续写。</summary>
    Continue,

    /// <summary>终止并报错；半成品保留，用户可再试。</summary>
    Fail,

    /// <summary>401：清会话回登录层；半成品保留。</summary>
    SessionExpired,
}

/// <summary>续传决策：动作 + 请求该带什么头 + 要不要先删半成品。</summary>
public sealed class DownloadDecision
{
    public DownloadAction Action { get; init; }
    public long Offset { get; init; }
    public long Total { get; init; }
    public bool DiscardPart { get; init; }
    public string? RangeHeader { get; init; }
    public string? IfRangeHeader { get; init; }
    public string? Message { get; init; }

    public bool AppendsToPart => Action == DownloadAction.Resume;
    public bool WritesFromZero => Action is DownloadAction.Start or DownloadAction.Restart;
}

/// <summary>本地半成品：`.part` 的字节数与 `.part.json` 里记的 ETag。</summary>
public sealed class DownloadPartState
{
    public static readonly DownloadPartState Empty = new();

    public long Size { get; init; }
    public string? ETag { get; init; }
}

/// <summary>下载响应里与本状态机有关的头与状态码。</summary>
public sealed class DownloadResponseInfo
{
    public int Status { get; init; }
    public string? ETag { get; init; }
    public long ContentLength { get; init; }
    public string? ContentRange { get; init; }
}

/// <summary>
/// 断点续传决策表，逐条对齐 `spec/fixtures/download/cases.json`。
/// 协议事实来自 PezMax-Go 的 <c>http.ServeContent</c>（`datum_activity.go:562-573`）：
/// 强 ETag、`Range`→206、`If-Range` 不匹配则回 200 全量、越界 416。
/// 这里只做决定，不做 I/O——字节搬运由各端自己写。
/// </summary>
public static class DownloadPlanner
{
    private static readonly Regex ContentRangePattern =
        new(@"bytes\s+(\d+)-(\d+)/(\d+|\*)", RegexOptions.IgnoreCase | RegexOptions.Compiled);

    /// <summary>发请求前：要不要续、要不要带 If-Range。</summary>
    public static DownloadDecision BeforeRequest(DownloadPartState? part)
    {
        var state = part ?? DownloadPartState.Empty;
        if (state.Size <= 0) return new DownloadDecision { Action = DownloadAction.Start };

        // 没有可信的强 ETag 就不续：If-Range 送不出去，服务端会当普通请求回 200，
        // 那时候再截断重写只是白下一遍，不如现在就重下。
        if (!IsStrongETag(state.ETag))
        {
            return new DownloadDecision
            {
                Action = DownloadAction.Restart,
                DiscardPart = true,
                Message = "半成品缺少可校验的 ETag，已丢弃重下",
            };
        }

        return new DownloadDecision
        {
            Action = DownloadAction.Resume,
            Offset = state.Size,
            RangeHeader = "bytes=" + state.Size.ToString(CultureInfo.InvariantCulture) + "-",
            IfRangeHeader = state.ETag,
        };
    }

    /// <summary>收到响应头之后：能不能接着写、要不要重下。</summary>
    public static DownloadDecision Plan(DownloadPartState? part, DownloadResponseInfo? response)
    {
        var state = part ?? DownloadPartState.Empty;
        var info = response ?? new DownloadResponseInfo();

        if (info.Status == 401) return new DownloadDecision { Action = DownloadAction.SessionExpired };

        if (info.Status == 416)
        {
            // 半成品已经等于服务端文件（只是没改名）才会走到这里；删掉重下，不要死循环。
            return state.Size > 0
                ? Restart("半成品与服务端文件不匹配，已丢弃重下")
                : Fail("服务器拒绝了分段请求");
        }

        if (info.Status == 200)
        {
            // 没发 Range 是正常开局；发了却被忽略（If-Range 不匹配）就必须截断重写。
            return state.Size > 0
                ? Restart("服务端返回了完整内容，已丢弃半成品重下", info.ContentLength)
                : new DownloadDecision { Action = DownloadAction.Start, Total = Math.Max(0, info.ContentLength) };
        }

        if (info.Status == 206) return PlanPartial(state, info);

        return Fail(MessageForStatus(info.Status));
    }

    /// <summary>写完一批字节之后：收工还是继续。</summary>
    public static DownloadDecision AfterWrite(long written, long total)
    {
        if (total <= 0) return new DownloadDecision { Action = DownloadAction.Continue };
        if (written > total) return new DownloadDecision { Action = DownloadAction.Fail, Total = total, DiscardPart = true, Message = "收到超出预期的字节数" };
        if (written == total) return new DownloadDecision { Action = DownloadAction.Complete, Total = total };
        return new DownloadDecision { Action = DownloadAction.Continue, Total = total };
    }

    private static DownloadDecision PlanPartial(DownloadPartState state, DownloadResponseInfo info)
    {
        // 没要过分段却收到 206：服务端行为异常，猜着写只会写坏文件。
        if (state.Size <= 0)
        {
            return Fail("服务器返回了意外的分段响应");
        }

        var match = info.ContentRange == null ? null : ContentRangePattern.Match(info.ContentRange);
        if (match == null || !match.Success)
        {
            return Restart("分段响应缺少 Content-Range，已丢弃重下");
        }

        if (!long.TryParse(match.Groups[3].Value, NumberStyles.Integer, CultureInfo.InvariantCulture, out var total) ||
            total <= 0)
        {
            return Restart("分段响应没有总大小，已丢弃重下");
        }

        if (!long.TryParse(match.Groups[2].Value, NumberStyles.Integer, CultureInfo.InvariantCulture, out var rangeEnd))
        {
            return Restart("分段响应终点不可解析，已丢弃重下");
        }

        // 起点必须正好接在半成品末尾、终点必须是文件末尾，ETag 还必须是同一个强校验器。
        if (rangeEnd != total - 1 || !IsStrongETag(info.ETag) || !string.Equals(info.ETag, state.ETag, StringComparison.Ordinal))
        {
            return Restart("分段响应与半成品对不上，已丢弃重下");
        }

        return new DownloadDecision
        {
            Action = DownloadAction.Resume,
            Offset = state.Size,
            Total = total,
        };
    }

    private static bool IsStrongETag(string? etag) =>
        !string.IsNullOrWhiteSpace(etag) &&
        !etag!.StartsWith("W/", StringComparison.OrdinalIgnoreCase) &&
        etag.Trim().Length > 0;

    private static DownloadDecision Restart(string message, long total = 0) => new()
    {
        Action = DownloadAction.Restart,
        Total = Math.Max(0, total),
        DiscardPart = true,
        Message = message,
    };

    private static DownloadDecision Fail(string message) => new()
    {
        Action = DownloadAction.Fail,
        Message = message,
    };

    private static string MessageForStatus(int status) => status switch
    {
        403 => "没有权限下载该文件",
        404 => "文件不存在或已删除",
        413 => "文件过大",
        429 => "操作过于频繁，请稍后再试",
        >= 500 => "服务暂不可用，请稍后重试",
        _ => "下载失败（HTTP " + status.ToString(CultureInfo.InvariantCulture) + "）",
    };
}
