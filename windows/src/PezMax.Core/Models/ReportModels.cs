using PezMax.Core.Json;

namespace PezMax.Core.Models;

public sealed class ReportItem
{
    public long ReportId { get; init; }
    public long FileId { get; init; }
    public long BookmarkId { get; init; }
    public long UserId { get; init; }
    public string Reason { get; init; } = "";
    public string Result { get; init; } = "";
    public string Remark { get; init; } = "";
    public string FileName { get; init; } = "";
    public string FileSubject { get; init; } = "";
    public string FileSchool { get; init; } = "";
    public string BookmarkTitle { get; init; } = "";
    public string EbookName { get; init; } = "";
    public string BookmarkUrl { get; init; } = "";
    public string CreateTime { get; init; } = "";
    public string UpdateTime { get; init; } = "";

    public string Title => !string.IsNullOrWhiteSpace(FileName) ? FileName
        : !string.IsNullOrWhiteSpace(BookmarkTitle) ? BookmarkTitle
        : !string.IsNullOrWhiteSpace(EbookName) ? EbookName
        : $"举报 #{ReportId}";
    public string KindText => BookmarkId > 0 ? "书签" : FileId > 0 ? "资料" : "电子书";
    public string ResultName => Result switch
    {
        "0" => "待处理",
        "1" => "属实，已下架",
        "2" => "不属实",
        _ => Result,
    };

    public static ReportItem From(JsonValue? value) => new()
    {
        ReportId = JsonModel.Long(value, "reportId"),
        FileId = JsonModel.Long(value, "fileId"),
        BookmarkId = JsonModel.Long(value, "bookmarkId"),
        UserId = JsonModel.Long(value, "userId"),
        Reason = JsonModel.Str(value, "reason"),
        Result = JsonModel.Str(value, "result"),
        Remark = JsonModel.Str(value, "remark"),
        FileName = JsonModel.Str(value, "fileName"),
        FileSubject = JsonModel.Str(value, "fileSubject"),
        FileSchool = JsonModel.Str(value, "fileSchool"),
        BookmarkTitle = JsonModel.Str(value, "bookmarkTitle"),
        EbookName = JsonModel.Str(value, "ebookName"),
        BookmarkUrl = JsonModel.Str(value, "bookmarkUrl"),
        CreateTime = JsonModel.Str(value, "createTime"),
        UpdateTime = JsonModel.Str(value, "updateTime"),
    };
}
