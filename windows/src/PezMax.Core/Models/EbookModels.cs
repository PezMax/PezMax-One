using PezMax.Core.Json;

namespace PezMax.Core.Models;

public sealed class EbookItem
{
    public long Id { get; init; }
    public long EbookId { get; init; }
    public string Label { get; init; } = "";
    public string EbookName { get; init; } = "";
    public string Author { get; init; } = "";
    public string Publisher { get; init; } = "";
    public string CoverUrl { get; init; } = "";
    public string EbookUrl { get; init; } = "";
    public long EbookSize { get; init; }
    public string EbookFormat { get; init; } = "";
    public string EbookSubject { get; init; } = "";
    public long EbookType { get; init; }
    public long EbookStatus { get; init; }
    public long UserId { get; init; }
    public string Remark { get; init; } = "";

    public string DisplayName => string.IsNullOrWhiteSpace(EbookName) ? Label : EbookName;
    public string SizeText => FileItem.FormatSize(EbookSize);
    public string TypeName => EbookType switch { 1 => "教材", 2 => "教辅/参考书", 3 => "课外读物", 4 => "其他", _ => "其他" };

    public static EbookItem From(JsonValue? value) => new()
    {
        Id = JsonModel.Long(value, "id"),
        EbookId = JsonModel.Long(value, "ebookId"),
        Label = JsonModel.Str(value, "label"),
        EbookName = JsonModel.Str(value, "ebookName"),
        Author = JsonModel.Str(value, "author"),
        Publisher = JsonModel.Str(value, "publisher"),
        CoverUrl = JsonModel.Str(value, "coverUrl"),
        EbookUrl = JsonModel.Str(value, "ebookUrl"),
        EbookSize = JsonModel.Long(value, "ebookSize"),
        EbookFormat = JsonModel.Str(value, "ebookFormat"),
        EbookSubject = JsonModel.Str(value, "ebookSubject"),
        EbookType = JsonModel.Long(value, "ebookType"),
        EbookStatus = JsonModel.Long(value, "ebookStatus"),
        UserId = JsonModel.Long(value, "userId"),
        Remark = JsonModel.Str(value, "remark"),
    };
}
