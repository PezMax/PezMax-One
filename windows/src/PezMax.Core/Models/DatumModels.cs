using System;
using System.Collections.Generic;
using PezMax.Core.Json;

namespace PezMax.Core.Models;

internal static class JsonModel
{
    public static string Str(JsonValue? obj, string key, string fallback = "") =>
        obj?.GetString(key, fallback) ?? fallback;

    public static long Long(JsonValue? obj, string key, long fallback = 0) =>
        obj?.GetLong(key, fallback) ?? fallback;

    public static int Int(JsonValue? obj, string key, int fallback = 0) =>
        obj?.GetInt(key, fallback) ?? fallback;

    public static bool Bool(JsonValue? obj, string key, bool fallback = false) =>
        obj?.GetBool(key, fallback) ?? fallback;
}

public sealed class PageResult<T>
{
    public PageResult(IReadOnlyList<T> rows, long total)
    {
        Rows = rows;
        Total = total;
    }

    public IReadOnlyList<T> Rows { get; }
    public long Total { get; }
}

public sealed class UserInfo
{
    public long UserId { get; init; }
    public string UserName { get; init; } = "";
    public string NickName { get; init; } = "";
    public string Avatar { get; init; } = "";
    public long Count { get; init; }
    public string Status { get; init; } = "";

    public string DisplayName => string.IsNullOrWhiteSpace(NickName) ? UserName : NickName;

    public static UserInfo From(JsonValue? value) => new()
    {
        UserId = JsonModel.Long(value, "userId"),
        UserName = JsonModel.Str(value, "userName"),
        NickName = JsonModel.Str(value, "nickName"),
        Avatar = JsonModel.Str(value, "avatar"),
        Count = JsonModel.Long(value, "count"),
        Status = JsonModel.Str(value, "status"),
    };
}

public sealed class GetInfoResult
{
    public UserInfo User { get; init; } = new();
    public IReadOnlyList<string> Roles { get; init; } = Array.Empty<string>();
    public IReadOnlyList<string> Permissions { get; init; } = Array.Empty<string>();

    public static GetInfoResult From(JsonValue? value)
    {
        var roles = new List<string>();
        var permissions = new List<string>();
        if (value?.GetArray("roles") is { } roleValues)
        {
            foreach (var role in roleValues) if (role.StringValue is { } text) roles.Add(text);
        }
        if (value?.GetArray("permissions") is { } permissionValues)
        {
            foreach (var permission in permissionValues) if (permission.StringValue is { } text) permissions.Add(text);
        }
        return new GetInfoResult
        {
            User = UserInfo.From(value?.GetObject("user")),
            Roles = roles,
            Permissions = permissions,
        };
    }
}

public sealed class CaptchaResult
{
    public bool CaptchaEnabled { get; init; }
    public string Uuid { get; init; } = "";
    public string ImageBase64 { get; init; } = "";

    public static CaptchaResult From(JsonValue? value) => new()
    {
        CaptchaEnabled = JsonModel.Bool(value, "captchaEnabled", true),
        Uuid = JsonModel.Str(value, "uuid"),
        ImageBase64 = JsonModel.Str(value, "img"),
    };
}

public sealed class LoginResult
{
    public string Token { get; init; } = "";
    public long ExpiresAtUnixMilliseconds { get; init; }

    public DateTimeOffset ExpiresAt =>
        ExpiresAtUnixMilliseconds <= 0
            ? DateTimeOffset.MinValue
            : DateTimeOffset.FromUnixTimeMilliseconds(ExpiresAtUnixMilliseconds);

    public static LoginResult From(JsonValue? value) => new()
    {
        Token = JsonModel.Str(value, "token"),
        ExpiresAtUnixMilliseconds = JsonModel.Long(value, "expiresAt"),
    };
}

public sealed class ProfileStats
{
    public long UploadCount { get; init; }
    public long DownloadCount { get; init; }
    public long FavoriteCount { get; init; }
    public long FileFavoriteCount { get; init; }
    public long BookmarkFavoriteCount { get; init; }

    public static ProfileStats From(JsonValue? value) => new()
    {
        UploadCount = JsonModel.Long(value, "uploadCount"),
        DownloadCount = JsonModel.Long(value, "downloadCount"),
        FavoriteCount = JsonModel.Long(value, "favoriteCount"),
        FileFavoriteCount = JsonModel.Long(value, "fileFavoriteCount"),
        BookmarkFavoriteCount = JsonModel.Long(value, "bookmarkFavoriteCount"),
    };
}

public sealed class SubjectCount
{
    public string Subject { get; init; } = "";
    public long Total { get; init; }

    public static SubjectCount From(JsonValue? value) => new()
    {
        Subject = value?.GetString("Subject") ?? JsonModel.Str(value, "subject"),
        Total = value?.Has("Total") == true ? value.GetLong("Total") : JsonModel.Long(value, "total"),
    };
}

public sealed class SchoolCheck
{
    public bool Exists { get; init; }
    public static SchoolCheck From(JsonValue? value) => new() { Exists = JsonModel.Bool(value, "exists") };
}

public sealed class RankEntry
{
    public long UserId { get; init; }
    public string UserName { get; init; } = "";
    public string Avatar { get; init; } = "";
    public long Count { get; init; }
    public long Uploads { get; init; }

    public string DisplayName => UserName;
    public string UserIdText => UserId.ToString();
    public string UploadsText => Uploads.ToString();

    public static RankEntry From(JsonValue? value) => new()
    {
        UserId = JsonModel.Long(value, "userId"),
        UserName = JsonModel.Str(value, "userName"),
        Avatar = JsonModel.Str(value, "avatar"),
        Count = JsonModel.Long(value, "count"),
        Uploads = JsonModel.Long(value, "uploads"),
    };
}

public sealed class TimelineEvent
{
    public string Step { get; init; } = "";
    public string Time { get; init; } = "";
    public string Result { get; init; } = "";
    public string Reason { get; init; } = "";
    public string Remark { get; init; } = "";
    public string Conclusion { get; init; } = "";

    public static TimelineEvent From(JsonValue? value) => new()
    {
        Step = JsonModel.Str(value, "step"),
        Time = JsonModel.Str(value, "time"),
        Result = JsonModel.Str(value, "result"),
        Reason = JsonModel.Str(value, "reason"),
        Remark = JsonModel.Str(value, "remark"),
        Conclusion = JsonModel.Str(value, "conclusion"),
    };
}
