using System;
using System.Collections.Generic;
using System.IO;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Json;
using PezMax.Core.Models;
using PezMax.Core.Net;

namespace PezMax.Core.Api;

public sealed class EbookQuery
{
    public int PageNum { get; set; } = 1;
    public int PageSize { get; set; } = 20;
    public string? Keyword { get; set; }
    public string? Subject { get; set; }
    public long? EbookType { get; set; }
}

public sealed class EbookUploadResult
{
    public long EbookId { get; init; }
    public string EbookUrl { get; init; } = "";
    public string EbookName { get; init; } = "";

    public static EbookUploadResult From(JsonValue? value) => new()
    {
        EbookId = JsonModel.Long(value, "ebookId"),
        EbookUrl = JsonModel.Str(value, "ebookUrl"),
        EbookName = JsonModel.Str(value, "ebookName"),
    };
}

public sealed class EbooksApi
{
    private readonly DatumHttp _http;

    public EbooksApi(DatumHttp http) => _http = http;

    public async Task<PageResult<EbookItem>> ListAsync(EbookQuery query, CancellationToken cancellationToken = default)
    {
        var values = new List<KeyValuePair<string, string?>>
        {
            new("pageNum", query.PageNum.ToString()),
            new("pageSize", query.PageSize.ToString()),
            new("keyword", query.Keyword),
            new("subject", query.Subject),
            new("ebookType", query.EbookType?.ToString()),
        };
        var response = await _http.GetAsync("/datum/ebook/list", values, cancellationToken).ConfigureAwait(false);
        return ApiHelpers.EnsurePage(response, EbookItem.From);
    }

    public async Task<IReadOnlyList<SubjectCount>> GetSubjectsAsync(CancellationToken cancellationToken = default)
    {
        var response = ApiHelpers.EnsureSuccess(
            await _http.GetAsync("/datum/ebook/subjects", cancellationToken: cancellationToken).ConfigureAwait(false));
        return ApiHelpers.MapArray(response.Data, SubjectCount.From);
    }

    public Task<HttpResponseMessage> OpenContentAsync(long ebookId, CancellationToken cancellationToken = default)
    {
        var query = new[] { new KeyValuePair<string, string?>("ebookId", ebookId.ToString()) };
        return _http.GetStreamAsync("/datum/ebook/content", query, cancellationToken);
    }

    public async Task AddFavoriteAsync(long ebookId, CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder().Number("ebookId", ebookId).ToString();
        ApiHelpers.EnsureTrue(
            await _http.PostJsonAsync("/datum/ebook/favorite", body, cancellationToken).ConfigureAwait(false));
    }

    public async Task<bool> IsFavoritedAsync(long ebookId, CancellationToken cancellationToken = default)
    {
        var query = new[] { new KeyValuePair<string, string?>("ebookId", ebookId.ToString()) };
        var response = ApiHelpers.EnsureSuccess(
            await _http.GetAsync("/datum/ebook/favorite", query, cancellationToken).ConfigureAwait(false));
        return response.Data?.GetBool("favorited") ?? false;
    }

    public async Task<IReadOnlyList<long>> GetFavoriteIdsAsync(CancellationToken cancellationToken = default)
    {
        var response = ApiHelpers.EnsureSuccess(
            await _http.GetAsync("/datum/ebook/favorite/list", cancellationToken: cancellationToken).ConfigureAwait(false));
        var result = new List<long>();
        if (response.Data?.IsArray == true)
        {
            foreach (var item in response.Data.Items) result.Add(item.LongValue);
        }
        return result;
    }

    public async Task RemoveFavoriteAsync(long ebookId, CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureTrue(
            await _http.DeleteAsync($"/datum/ebook/favorite/{ebookId}", cancellationToken).ConfigureAwait(false));

    public async Task<long> ReportAsync(
        long ebookId,
        string reason,
        string? remark = null,
        CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder()
            .Number("ebookId", ebookId)
            .String("reason", reason)
            .String("remark", remark ?? "")
            .ToString();
        var response = ApiHelpers.EnsureSuccess(
            await _http.PostJsonAsync("/datum/ebook/report", body, cancellationToken).ConfigureAwait(false));
        return response.Data?.GetLong("reportId") ?? 0;
    }

    public async Task<PageResult<ReportItem>> GetMyReportsAsync(
        long userId,
        int pageNum = 1,
        int pageSize = 20,
        CancellationToken cancellationToken = default)
    {
        var query = new[]
        {
            new KeyValuePair<string, string?>("userId", userId.ToString()),
            new KeyValuePair<string, string?>("pageNum", pageNum.ToString()),
            new KeyValuePair<string, string?>("pageSize", pageSize.ToString()),
        };
        var response = await _http.GetAsync("/datum/ebook/report/list", query, cancellationToken).ConfigureAwait(false);
        return ApiHelpers.EnsurePage(response, ReportItem.From);
    }

    public Task<EbookUploadResult> UploadAsync(
        string fileName,
        Stream stream,
        string contentType,
        string ebookName,
        string author,
        string publisher,
        string subject,
        long ebookType,
        CancellationToken cancellationToken = default,
        IProgress<long>? bytesSent = null)
    {
        var fields = new List<KeyValuePair<string, string?>>
        {
            new("ebookName", ebookName),
            new("author", author),
            new("publisher", publisher),
            new("ebookSubject", subject),
            new("ebookType", ebookType.ToString()),
        };
        return UploadCoreAsync(fileName, stream, contentType, fields, cancellationToken, bytesSent);
    }

    private async Task<EbookUploadResult> UploadCoreAsync(
        string fileName,
        Stream stream,
        string contentType,
        IEnumerable<KeyValuePair<string, string?>> fields,
        CancellationToken cancellationToken,
        IProgress<long>? bytesSent = null)
    {
        var response = await _http.UploadAsync(
            "/datum/ebook/upload", fields, fileName, stream, contentType, cancellationToken, bytesSent).ConfigureAwait(false);
        return ApiHelpers.EnsureData(response, EbookUploadResult.From);
    }

    public async Task DeleteAsync(long ebookId, CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureTrue(
            await _http.DeleteAsync($"/datum/ebook/{ebookId}", cancellationToken).ConfigureAwait(false));
}
