using PezMax.Core.Json;

namespace PezMax.Core.Models;

public sealed class Bookmark
{
    public long Id { get; init; }
    public long BookmarkId { get; init; }
    public long UserId { get; init; }
    public string Url { get; init; } = "";
    public string Title { get; init; } = "";
    public string Description { get; init; } = "";
    public string CoverImage { get; init; } = "";
    public string Subject { get; init; } = "";
    public string ResourceType { get; init; } = "";
    public string Collection { get; init; } = "";
    public long Status { get; init; }
    public string Remark { get; init; } = "";
    public string CreateBy { get; init; } = "";
    public string CreateTime { get; init; } = "";
    public string UpdateTime { get; init; } = "";

    public bool IsApproved => Status == 1;

    public static Bookmark From(JsonValue? value) => new()
    {
        Id = JsonModel.Long(value, "id"),
        BookmarkId = JsonModel.Long(value, "bookmarkId"),
        UserId = JsonModel.Long(value, "userId"),
        Url = JsonModel.Str(value, "url"),
        Title = JsonModel.Str(value, "title"),
        Description = JsonModel.Str(value, "description"),
        CoverImage = JsonModel.Str(value, "coverImage"),
        Subject = JsonModel.Str(value, "subject"),
        ResourceType = JsonModel.Str(value, "resourceType"),
        Collection = JsonModel.Str(value, "collection"),
        Status = JsonModel.Long(value, "status"),
        Remark = JsonModel.Str(value, "remark"),
        CreateBy = JsonModel.Str(value, "createBy"),
        CreateTime = JsonModel.Str(value, "createTime"),
        UpdateTime = JsonModel.Str(value, "updateTime"),
    };
}

public sealed class BookmarkFavoriteRelation
{
    public long BookmarkId { get; init; }
    public long UserId { get; init; }

    public static BookmarkFavoriteRelation From(JsonValue? value) => new()
    {
        BookmarkId = JsonModel.Long(value, "bookmarkId"),
        UserId = JsonModel.Long(value, "userId"),
    };
}
