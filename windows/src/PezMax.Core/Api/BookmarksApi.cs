using System;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Json;
using PezMax.Core.Models;
using PezMax.Core.Net;

namespace PezMax.Core.Api;

public sealed class BookmarkWrite
{
    public long Id { get; set; }
    public string? Url { get; set; }
    public string? Title { get; set; }
    public string? Description { get; set; }
    public string? CoverImage { get; set; }
    public string? Subject { get; set; }
    public string? ResourceType { get; set; }
    public string? Collection { get; set; }
    public string? Remark { get; set; }
}

public sealed class BookmarkQuery
{
    public int PageNum { get; set; } = 1;
    public int PageSize { get; set; } = 20;
    public string? Keyword { get; set; }
    public string? Subject { get; set; }
    public string? ResourceType { get; set; }
    public string? Collection { get; set; }
    public string? Url { get; set; }
    public long? UserId { get; set; }
}

public sealed class BookmarkCoverResult
{
    public long BookmarkId { get; init; }
    public string Url { get; init; } = "";

    public static BookmarkCoverResult From(JsonValue? value) => new()
    {
        BookmarkId = JsonModel.Long(value, "bookmarkId"),
        Url = JsonModel.Str(value, "url"),
    };
}

public sealed class BookmarksApi
{
    private readonly DatumHttp _http;

    public BookmarksApi(DatumHttp http) => _http = http;

    public async Task<PageResult<Bookmark>> ListAsync(BookmarkQuery query, CancellationToken cancellationToken = default)
    {
        var response = await _http.GetAsync("/datum/bookmark/list", BuildQuery(query), cancellationToken).ConfigureAwait(false);
        return ApiHelpers.EnsurePage(response, Bookmark.From);
    }

    public async Task<Bookmark> GetDetailAsync(long bookmarkId, CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureData(
            await _http.GetAsync($"/datum/bookmark/{bookmarkId}", cancellationToken: cancellationToken).ConfigureAwait(false),
            Bookmark.From);

    public async Task<long> CreateAsync(BookmarkWrite write, CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder()
            .String("url", write.Url ?? "")
            .String("title", write.Title ?? "")
            .String("description", write.Description ?? "")
            .String("coverImage", write.CoverImage ?? "")
            .String("subject", write.Subject ?? "")
            .String("resourceType", write.ResourceType ?? "other")
            .String("collection", write.Collection ?? "")
            .String("remark", write.Remark ?? "")
            .ToString();
        var response = ApiHelpers.EnsureSuccess(
            await _http.PostJsonAsync("/datum/bookmark", body, cancellationToken).ConfigureAwait(false));
        return response.Data?.GetLong("bookmarkId") ?? response.Data?.GetLong("id") ?? 0;
    }

    public async Task UpdateAsync(BookmarkWrite write, CancellationToken cancellationToken = default)
    {
        var builder = new JsonObjectBuilder().Number("id", write.Id);
        if (write.Url != null) builder.String("url", write.Url);
        if (write.Title != null) builder.String("title", write.Title);
        if (write.Description != null) builder.String("description", write.Description);
        if (write.CoverImage != null) builder.String("coverImage", write.CoverImage);
        if (write.Subject != null) builder.String("subject", write.Subject);
        if (write.ResourceType != null) builder.String("resourceType", write.ResourceType);
        if (write.Collection != null) builder.String("collection", write.Collection);
        if (write.Remark != null) builder.String("remark", write.Remark);

        ApiHelpers.EnsureTrue(
            await _http.PutJsonAsync("/datum/bookmark", builder.ToString(), cancellationToken).ConfigureAwait(false));
    }

    public async Task DeleteAsync(long bookmarkId, CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureTrue(
            await _http.DeleteAsync($"/datum/bookmark/{bookmarkId}", cancellationToken).ConfigureAwait(false));

    public async Task<BookmarkCoverResult> UploadCoverAsync(
        long bookmarkId,
        string fileName,
        Stream stream,
        string contentType,
        CancellationToken cancellationToken = default)
    {
        var fields = new List<KeyValuePair<string, string?>>
        {
            new("bookmarkId", bookmarkId.ToString()),
        };
        var response = await _http.UploadAsync(
            "/datum/bookmark/uploadCover", fields, fileName, stream, contentType, cancellationToken).ConfigureAwait(false);
        return ApiHelpers.EnsureData(response, BookmarkCoverResult.From);
    }

    public async Task AddFavoriteAsync(long bookmarkId, CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder().Number("bookmarkId", bookmarkId).ToString();
        ApiHelpers.EnsureTrue(
            await _http.PostJsonAsync("/datum/bookmark/favorite", body, cancellationToken).ConfigureAwait(false));
    }

    public async Task<PageResult<BookmarkFavoriteRelation>> GetFavoriteRelationsAsync(
        int pageNum = 1,
        int pageSize = 100,
        CancellationToken cancellationToken = default)
    {
        var query = new[]
        {
            new KeyValuePair<string, string?>("pageNum", pageNum.ToString()),
            new KeyValuePair<string, string?>("pageSize", pageSize.ToString()),
        };
        var response = await _http.GetAsync("/datum/bookmark/favorite/list", query, cancellationToken).ConfigureAwait(false);
        return ApiHelpers.EnsurePage(response, BookmarkFavoriteRelation.From);
    }

    private static IEnumerable<KeyValuePair<string, string?>> BuildQuery(BookmarkQuery query)
    {
        yield return new KeyValuePair<string, string?>("pageNum", query.PageNum.ToString());
        yield return new KeyValuePair<string, string?>("pageSize", query.PageSize.ToString());
        yield return new KeyValuePair<string, string?>("keyword", query.Keyword);
        yield return new KeyValuePair<string, string?>("subject", query.Subject);
        yield return new KeyValuePair<string, string?>("resourceType", query.ResourceType);
        yield return new KeyValuePair<string, string?>("collection", query.Collection);
        yield return new KeyValuePair<string, string?>("url", query.Url);
        yield return new KeyValuePair<string, string?>("userId", query.UserId?.ToString());
    }
}
