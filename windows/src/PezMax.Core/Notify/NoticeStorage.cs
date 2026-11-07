using System;
using System.Collections.Generic;
using System.Text;
using PezMax.Core.Json;

namespace PezMax.Core.Notify;

/// <summary>
/// 本地的通知状态：哪些读过（铃铛里点开的）、哪些弹过（type 1/4 的一次性语义）。
/// 形状见 <c>spec/fixtures/notification/README.md</c>「本地存储」。
/// </summary>
public sealed class NoticeLocalState
{
    public NoticeKeySet Read { get; } = new();
    public NoticeKeySet Popped { get; } = new();

    public bool IsRead(long userId, long notifyId) => Read.Contains(userId, notifyId);
    public bool WasPopped(long userId, long notifyId) => Popped.Contains(userId, notifyId);

    public void MarkRead(long userId, long notifyId) => Read.Mark(userId, notifyId);
    public void MarkPopped(long userId, long notifyId) => Popped.Mark(userId, notifyId);
}

/// <summary>
/// <c>notice_reads.json</c> 的读写：`{"reads":[{userId,notifyId}],"popups":[…]}`。
/// 读不出来（文件丢了 / 手工改坏）就当空集合——通知状态丢了顶多再弹一次，不值得因此挡住界面。
/// </summary>
public static class NoticeStorage
{
    public const string FileName = "notice_reads.json";

    public static NoticeLocalState Parse(string? json)
    {
        var state = new NoticeLocalState();
        if (string.IsNullOrWhiteSpace(json)) return state;

        JsonValue root;
        try
        {
            root = JsonValue.Parse(json!);
        }
        catch (JsonParseException)
        {
            return state;
        }

        ReadInto(root, "reads", state.Read);
        ReadInto(root, "popups", state.Popped);
        return state;
    }

    public static string ToJson(NoticeLocalState? state)
    {
        var reads = Keys(state?.Read);
        var popups = Keys(state?.Popped);
        return new JsonObjectBuilder()
            .Raw("reads", reads)
            .Raw("popups", popups)
            .ToString();
    }

    private static void ReadInto(JsonValue root, string name, NoticeKeySet target)
    {
        if (root.GetArray(name) is not { } items) return;
        foreach (var item in items)
        {
            if (!item.IsObject) continue;
            var userId = item.GetLong("userId");
            var notifyId = item.GetLong("notifyId");
            if (userId > 0 && notifyId > 0) target.Mark(userId, notifyId);
        }
    }

    private static string Keys(NoticeKeySet? set)
    {
        if (set == null || set.Count == 0) return "[]";

        // NoticeKeySet 只暴露 Contains/Mark，序列化时按 (userId, notifyId) 重建。
        var builder = new StringBuilder();
        foreach (var key in set.Items)
        {
            if (builder.Length > 0) builder.Append(',');
            builder.Append(new JsonObjectBuilder()
                .Number("userId", key.UserId)
                .Number("notifyId", key.NotifyId)
                .ToString());
        }
        return "[" + builder + "]";
    }

}
