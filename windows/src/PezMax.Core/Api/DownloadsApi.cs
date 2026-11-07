using System.Collections.Generic;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Models;
using PezMax.Core.Net;

namespace PezMax.Core.Api;

public sealed class DownloadsApi
{
    private readonly DatumHttp _http;

    public DownloadsApi(DatumHttp http) => _http = http;

    public Task<HttpResponseMessage> OpenFileStreamAsync(long fileId, CancellationToken cancellationToken = default) =>
        OpenFileStreamAsync(fileId, null, null, cancellationToken);

    /// <summary>
    /// 打开资料下载流。续传时带上 <c>Range</c> 与 <c>If-Range</c>——两个头要么都给、要么都不给
    /// （见 `spec/fixtures/download/` 的 before 决策）。<c>Accept-Encoding: identity</c> 是必须的：
    /// 一旦被压缩，落盘字节数与 Content-Length 对不上，续传偏移就错位。
    /// </summary>
    public Task<HttpResponseMessage> OpenFileStreamAsync(
        long fileId,
        string? range,
        string? ifRange,
        CancellationToken cancellationToken = default)
    {
        var query = new[] { new KeyValuePair<string, string?>("fileId", fileId.ToString()) };
        return _http.GetStreamAsync("/datum/download/file", query, DownloadHeaders(range, ifRange), cancellationToken);
    }

    public Task<HttpResponseMessage> OpenEbookStreamAsync(long ebookId, CancellationToken cancellationToken = default) =>
        OpenEbookStreamAsync(ebookId, null, null, cancellationToken);

    public Task<HttpResponseMessage> OpenEbookStreamAsync(
        long ebookId,
        string? range,
        string? ifRange,
        CancellationToken cancellationToken = default)
    {
        var query = new[] { new KeyValuePair<string, string?>("ebookId", ebookId.ToString()) };
        return _http.GetStreamAsync("/datum/download/ebook", query, DownloadHeaders(range, ifRange), cancellationToken);
    }

    private static List<KeyValuePair<string, string>> DownloadHeaders(string? range, string? ifRange)
    {
        var headers = new List<KeyValuePair<string, string>> { new("Accept-Encoding", "identity") };
        if (!string.IsNullOrEmpty(range)) headers.Add(new KeyValuePair<string, string>("Range", range!));
        if (!string.IsNullOrEmpty(ifRange)) headers.Add(new KeyValuePair<string, string>("If-Range", ifRange!));
        return headers;
    }

    public async Task<PageResult<DownloadEntry>> GetMyDownloadsAsync(
        long userId,
        int pageNum = 1,
        int pageSize = 20,
        CancellationToken cancellationToken = default)
    {
        var query = new[]
        {
            new KeyValuePair<string, string?>("pageNum", pageNum.ToString()),
            new KeyValuePair<string, string?>("pageSize", pageSize.ToString()),
        };
        var response = await _http.GetAsync(
            $"/datum/desktop/download/list/{userId}", query, cancellationToken).ConfigureAwait(false);
        return ApiHelpers.EnsurePage(response, DownloadEntry.From);
    }

    public async Task RemoveAsync(long userId, long fileId, CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureTrue(
            await _http.DeleteAsync($"/datum/desktop/download/{userId}/{fileId}", cancellationToken).ConfigureAwait(false));
}
