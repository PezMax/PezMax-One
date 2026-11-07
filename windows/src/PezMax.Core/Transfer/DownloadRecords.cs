using System;
using System.Collections.Generic;
using System.Globalization;
using System.Text;
using PezMax.Core.Json;
using PezMax.Core.Models;

namespace PezMax.Core.Transfer;

public static class DownloadKinds
{
    public const string File = "file";
    public const string Ebook = "ebook";
}

public static class DownloadStatuses
{
    public const string Part = "part";
    public const string Done = "done";
    public const string Failed = "failed";
}

/// <summary>
/// 本地下载记录（`downloads.json` 的一条），形状见 `spec/fixtures/download/README.md`「本地记录与我的下载合并」。
/// 只描述本机状态：下载到哪儿、有没有落盘、续传用的 ETag。服务端没有这些字段。
/// </summary>
public sealed class DownloadRecord
{
    public string Kind { get; init; } = DownloadKinds.File;
    public long Id { get; init; }
    public string FileName { get; init; } = "";
    public string LocalPath { get; init; } = "";
    public long Size { get; init; }
    public long BytesReceived { get; init; }
    public string ETag { get; init; } = "";
    public string Status { get; init; } = DownloadStatuses.Part;
    public string UpdatedAt { get; init; } = "";

    public bool IsDone => string.Equals(Status, DownloadStatuses.Done, StringComparison.Ordinal);
    public bool IsPart => string.Equals(Status, DownloadStatuses.Part, StringComparison.Ordinal);

    public DownloadPartState ToPartState() => new() { Size = BytesReceived, ETag = ETag };

    public DownloadRecord WithProgress(long bytesReceived, string? etag, string status) => new()
    {
        Kind = Kind,
        Id = Id,
        FileName = FileName,
        LocalPath = LocalPath,
        Size = Size,
        BytesReceived = bytesReceived,
        ETag = etag ?? ETag,
        Status = status,
        UpdatedAt = DownloadRecords.Timestamp(DateTimeOffset.Now),
    };

    public DownloadRecord WithCompleted(string localPath, long bytesReceived) => new()
    {
        Kind = Kind,
        Id = Id,
        FileName = FileName,
        LocalPath = localPath,
        Size = Size > 0 ? Size : bytesReceived,
        BytesReceived = bytesReceived,
        ETag = ETag,
        Status = DownloadStatuses.Done,
        UpdatedAt = DownloadRecords.Timestamp(DateTimeOffset.Now),
    };

    public static DownloadRecord From(JsonValue? value) => new()
    {
        Kind = value?.GetString("kind") ?? DownloadKinds.File,
        Id = value?.GetLong("id") ?? 0,
        FileName = value?.GetString("fileName") ?? "",
        LocalPath = value?.GetString("localPath") ?? "",
        Size = value?.GetLong("size") ?? 0,
        BytesReceived = value?.GetLong("bytesReceived") ?? 0,
        ETag = value?.GetString("etag") ?? "",
        Status = value?.GetString("status") ?? DownloadStatuses.Part,
        UpdatedAt = value?.GetString("updatedAt") ?? "",
    };
}

/// <summary>「我的下载」的一行：服务端的元信息 + 本地状态（可能没有本地记录）。</summary>
public sealed class DownloadRow
{
    public DownloadRow(FileItem file, string firstDownloadTime, DownloadRecord? local)
    {
        File = file;
        FirstDownloadTime = firstDownloadTime;
        Local = local;
    }

    public FileItem File { get; }
    public string FirstDownloadTime { get; }
    public DownloadRecord? Local { get; }

    public long FileId => File.FileId;
    public bool HasLocalPath => Local != null && !string.IsNullOrEmpty(Local.LocalPath);
    public bool IsPartial => Local != null && Local.IsPart;
    public string StatusText => Local?.Status switch
    {
        DownloadStatuses.Done => "已下载",
        DownloadStatuses.Part => "未完成",
        DownloadStatuses.Failed => "下载失败",
        _ => "云端记录",
    };
}

/// <summary>
/// `downloads.json` 的读写与「我的下载」合并规则，逐条对齐 `spec/fixtures/download/`。
/// 合并的要点：服务端行是骨架（有元信息），本地记录只补「下到哪儿了」；
/// 本地独有的记录也要显示，否则用户刚下完的东西会因为服务端清过记录而消失。
/// </summary>
public static class DownloadRecords
{
    public const string FileName = "downloads.json";

    public static IReadOnlyList<DownloadRecord> Parse(string? json)
    {
        var records = new List<DownloadRecord>();
        if (string.IsNullOrWhiteSpace(json)) return records;

        JsonValue root;
        try
        {
            root = JsonValue.Parse(json!);
        }
        catch (JsonParseException)
        {
            return records;
        }

        if (root.GetArray("records") is not { } items) return records;
        foreach (var item in items)
        {
            if (!item.IsObject) continue;
            var record = DownloadRecord.From(item);
            if (record.Id > 0) records.Add(record);
        }
        return records;
    }

    /// <summary>按 (kind, id) 取最新的一条：同一文件多次下载只留最新的。</summary>
    public static IReadOnlyDictionary<string, DownloadRecord> Index(IEnumerable<DownloadRecord>? records)
    {
        var index = new Dictionary<string, DownloadRecord>(StringComparer.Ordinal);
        if (records == null) return index;

        foreach (var record in records)
        {
            if (record == null || record.Id <= 0) continue;
            var key = Key(record.Kind, record.Id);
            if (!index.TryGetValue(key, out var existing) || IsNewer(record, existing)) index[key] = record;
        }

        return index;
    }

    public static string Key(string? kind, long id) =>
        (string.IsNullOrEmpty(kind) ? DownloadKinds.File : kind) + ":" + id.ToString(CultureInfo.InvariantCulture);

    /// <summary>
    /// 服务端行 + 本地记录 → 展示行。服务端顺序保留；本地独有的追加在后（按 updatedAt 倒序）。
    /// </summary>
    public static IReadOnlyList<DownloadRow> Merge(
        IEnumerable<DownloadEntry>? serverEntries,
        IEnumerable<DownloadRecord>? localRecords)
    {
        var index = Index(localRecords);
        var rows = new List<DownloadRow>();
        var seen = new HashSet<string>(StringComparer.Ordinal);

        if (serverEntries != null)
        {
            foreach (var entry in serverEntries)
            {
                if (entry?.File == null) continue;
                var key = Key(DownloadKinds.File, entry.File.FileId);
                seen.Add(key);
                index.TryGetValue(key, out var local);
                rows.Add(new DownloadRow(entry.File, entry.FirstDownloadTime, local));
            }
        }

        if (localRecords != null)
        {
            var leftovers = new List<DownloadRecord>();
            foreach (var record in index.Values)
            {
                if (seen.Contains(Key(record.Kind, record.Id))) continue;
                if (!string.Equals(record.Kind, DownloadKinds.File, StringComparison.Ordinal)) continue;
                if (record.Id <= 0) continue;
                leftovers.Add(record);
            }

            leftovers.Sort((left, right) => string.CompareOrdinal(right.UpdatedAt, left.UpdatedAt));
            foreach (var record in leftovers)
            {
                var file = new FileItem { FileId = record.Id, FileName = record.FileName, Label = record.FileName };
                rows.Add(new DownloadRow(file, "", record));
            }
        }

        return rows;
    }

    public static string ToJson(IEnumerable<DownloadRecord>? records)
    {
        // JsonObjectBuilder 只写扁平对象，records 是自己拼出来的数组（每条也是同一套转义）。
        var items = new StringBuilder();
        foreach (var record in records ?? Array.Empty<DownloadRecord>())
        {
            if (items.Length > 0) items.Append(',');
            items.Append(new JsonObjectBuilder()
                .String("kind", record.Kind)
                .Number("id", record.Id)
                .String("fileName", record.FileName)
                .String("localPath", record.LocalPath)
                .Number("size", record.Size)
                .Number("bytesReceived", record.BytesReceived)
                .String("etag", record.ETag)
                .String("status", record.Status)
                .String("updatedAt", record.UpdatedAt)
                .ToString());
        }

        return new JsonObjectBuilder().Raw("records", "[" + items + "]").ToString();
    }

    public static string Timestamp(DateTimeOffset value) =>
        value.ToUniversalTime().ToString("yyyy-MM-dd'T'HH:mm:ss'Z'", CultureInfo.InvariantCulture);

    private static bool IsNewer(DownloadRecord candidate, DownloadRecord existing)
    {
        if (string.IsNullOrEmpty(existing.UpdatedAt)) return true;
        if (string.IsNullOrEmpty(candidate.UpdatedAt)) return false;
        return string.CompareOrdinal(candidate.UpdatedAt, existing.UpdatedAt) >= 0;
    }
}
