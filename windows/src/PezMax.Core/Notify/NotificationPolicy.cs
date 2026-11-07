using System;
using System.Collections.Generic;
using System.Globalization;
using PezMax.Core.Models;

namespace PezMax.Core.Notify;

public enum NotifyChannel
{
    Popup,
    Scroll,
}

/// <summary>「已弹过 / 已读」的键：按用户隔离，见 <c>spec/fixtures/notification/</c>。</summary>
public readonly struct NotifyKey : IEquatable<NotifyKey>
{
    public NotifyKey(long userId, long notifyId)
    {
        UserId = userId;
        NotifyId = notifyId;
    }

    public long UserId { get; }
    public long NotifyId { get; }

    public bool Equals(NotifyKey other) => UserId == other.UserId && NotifyId == other.NotifyId;
    public override bool Equals(object? obj) => obj is NotifyKey other && Equals(other);
    public override int GetHashCode() => unchecked((int)(UserId * 397) ^ (int)NotifyId);
    public override string ToString() => UserId.ToString(CultureInfo.InvariantCulture) + ":" + NotifyId.ToString(CultureInfo.InvariantCulture);
}

/// <summary>本地已读集合 / 「已弹过」集合，两者形状相同：一组 <see cref="NotifyKey"/>。</summary>
public sealed class NoticeKeySet
{
    private readonly HashSet<NotifyKey> _keys = new();

    public int Count => _keys.Count;

    /// <summary>序列化与统计用；调用方不要直接改。</summary>
    public IReadOnlyCollection<NotifyKey> Items => _keys;

    public bool Contains(long userId, long notifyId) => _keys.Contains(new NotifyKey(userId, notifyId));

    public void Mark(long userId, long notifyId) => _keys.Add(new NotifyKey(userId, notifyId));

    public void Clear() => _keys.Clear();
}

/// <summary>
/// 通知的客户端侧判定，逐条对齐 <c>spec/fixtures/notification/</c>。
/// 可见窗口与通道**由服务端筛**（`status='0'` + 发布窗口 + `display_mode`），这里不再判时间；
/// 只做四件事：通道兜底、弹窗队列与一次性语义、关闭权、已读集合。
/// </summary>
public static class NotificationPolicy
{
    public const string TypeVersionUpdate = "1";
    public const string TypeFault = "2";
    public const string TypeMaintenance = "3";
    public const string TypeTakedown = "4";
    public const string TypeNotice = "5";

    /// <summary>
    /// 这条通知要不要展示。
    /// <para>
    /// 资料下架（type 4）只属于材料上传者本人：`popup` 通道服务端已经定向过
    /// （`notification_repository.go:44`），但 `scroll` 通道**没有**这层过滤（`:52-56`），
    /// 所以客户端两个通道都要自己挡一次——否则等于把「某人的资料被下架」推给所有人。
    /// </para>
    /// </summary>
    public static bool ShouldShow(NotificationItem? item, long userId, NotifyChannel channel)
    {
        if (item == null) return false;
        if (!string.Equals(item.NotifyType, TypeTakedown, StringComparison.Ordinal)) return true;
        if (userId <= 0) return false;
        return item.UploadUserId == userId;
    }

    /// <summary>
    /// 弹窗队列：版本更新（type 1）与下架（type 4）确认过一次就不再弹；
    /// 故障（2）/ 维护（3）/ 公告（5）每次都弹——故障没好就该再提醒。
    /// 顺序沿用服务端返回顺序，同一 <c>notifyId</c> 只弹一次。
    /// </summary>
    public static IReadOnlyList<NotificationItem> PlanPopup(
        IEnumerable<NotificationItem>? rows,
        NoticeKeySet? seen,
        long userId)
    {
        var planned = new List<NotificationItem>();
        if (rows == null) return planned;

        var emitted = new HashSet<long>();
        foreach (var item in rows)
        {
            if (item == null) continue;
            if (!ShouldShow(item, userId, NotifyChannel.Popup)) continue;
            if (!emitted.Add(item.NotifyId)) continue;

            if (IsOneShot(item.NotifyType) && seen != null && seen.Contains(userId, item.NotifyId)) continue;
            planned.Add(item);
        }

        return planned;
    }

    /// <summary>版本更新与资料下架是一次性通知（确认过就不再来）；其余每次都要提醒。</summary>
    public static bool IsOneShot(string? notifyType) =>
        string.Equals(notifyType, TypeVersionUpdate, StringComparison.Ordinal) ||
        string.Equals(notifyType, TypeTakedown, StringComparison.Ordinal);

    /// <summary>
    /// 用户能不能关掉这条。维护通知（type 3）从 `maintenanceStartTime` 起不可关闭。
    /// 版本更新的「强制更新」不由通知字段决定——响应里没有 `forceUpdate`，
    /// 要做只能靠客户端比版本（见 `spec/api/endpoints.md` 通知一节）。
    /// </summary>
    public static bool IsDismissible(NotificationItem? item, DateTimeOffset now)
    {
        if (item == null) return true;
        if (!string.Equals(item.NotifyType, TypeMaintenance, StringComparison.Ordinal)) return true;
        if (!TryParseServerTime(item.MaintenanceStartTime, out var start)) return true;
        return now < start;
    }

    /// <summary>服务端时间串是 `yyyy-MM-dd HH:mm:ss`（服务器本地时间，不带时区）。</summary>
    public static bool TryParseServerTime(string? value, out DateTimeOffset result)
    {
        result = default;
        if (string.IsNullOrWhiteSpace(value)) return false;
        return DateTimeOffset.TryParse(
            value,
            CultureInfo.InvariantCulture,
            DateTimeStyles.AssumeLocal | DateTimeStyles.AllowWhiteSpaces,
            out result);
    }
}
