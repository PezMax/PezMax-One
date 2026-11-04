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

    public Task<HttpResponseMessage> OpenFileStreamAsync(long fileId, CancellationToken cancellationToken = default)
    {
        var query = new[] { new KeyValuePair<string, string?>("fileId", fileId.ToString()) };
        return _http.GetStreamAsync("/datum/download/file", query, cancellationToken);
    }

    public Task<HttpResponseMessage> OpenEbookStreamAsync(long ebookId, CancellationToken cancellationToken = default)
    {
        var query = new[] { new KeyValuePair<string, string?>("ebookId", ebookId.ToString()) };
        return _http.GetStreamAsync("/datum/download/ebook", query, cancellationToken);
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
