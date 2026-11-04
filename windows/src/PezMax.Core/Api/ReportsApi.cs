using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Json;
using PezMax.Core.Models;
using PezMax.Core.Net;

namespace PezMax.Core.Api;

public sealed class ReportsApi
{
    private readonly DatumHttp _http;

    public ReportsApi(DatumHttp http) => _http = http;

    public async Task<long> CreateFileReportAsync(
        long fileId,
        string reason,
        string? remark = null,
        CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder()
            .Number("fileId", fileId)
            .String("reason", reason)
            .String("remark", remark ?? "")
            .ToString();
        var response = ApiHelpers.EnsureSuccess(
            await _http.PostJsonAsync("/datum/report", body, cancellationToken).ConfigureAwait(false));
        return response.Data?.GetLong("reportId") ?? 0;
    }

    public async Task<PageResult<ReportItem>> GetMyFileReportsAsync(
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
        var response = await _http.GetAsync("/datum/report/list", query, cancellationToken).ConfigureAwait(false);
        return ApiHelpers.EnsurePage(response, ReportItem.From);
    }

    public async Task<ReportItem> GetFileReportAsync(long reportId, CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureData(
            await _http.GetAsync($"/datum/report/{reportId}", cancellationToken: cancellationToken).ConfigureAwait(false),
            ReportItem.From);

    public async Task UpdateFileReportAsync(
        long reportId,
        string reason,
        string? remark = null,
        CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder()
            .Number("reportId", reportId)
            .String("reason", reason)
            .String("remark", remark ?? "")
            .ToString();
        ApiHelpers.EnsureTrue(
            await _http.PutJsonAsync("/datum/report", body, cancellationToken).ConfigureAwait(false));
    }

    public async Task<int> DeleteFileReportsAsync(string ids, CancellationToken cancellationToken = default)
    {
        var response = ApiHelpers.EnsureSuccess(
            await _http.DeleteAsync($"/datum/report/{ids}", cancellationToken).ConfigureAwait(false));
        return response.Data?.GetInt("deleted") ?? 0;
    }

    public async Task<IReadOnlyList<TimelineEvent>> GetFileTimelineAsync(
        long reportId,
        CancellationToken cancellationToken = default)
    {
        var response = ApiHelpers.EnsureSuccess(
            await _http.GetAsync($"/datum/report/timeline/{reportId}", cancellationToken: cancellationToken).ConfigureAwait(false));
        return ApiHelpers.MapArray(response.Data, TimelineEvent.From);
    }

    public async Task<long> CreateBookmarkReportAsync(
        long bookmarkId,
        string reason,
        string? remark = null,
        CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder()
            .Number("bookmarkId", bookmarkId)
            .String("reason", reason)
            .String("remark", remark ?? "")
            .ToString();
        var response = ApiHelpers.EnsureSuccess(
            await _http.PostJsonAsync("/datum/bookmarkReport", body, cancellationToken).ConfigureAwait(false));
        return response.Data?.GetLong("reportId") ?? 0;
    }

    public async Task<PageResult<ReportItem>> GetMyBookmarkReportsAsync(
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
        var response = await _http.GetAsync("/datum/bookmarkReport/list", query, cancellationToken).ConfigureAwait(false);
        return ApiHelpers.EnsurePage(response, ReportItem.From);
    }

    public async Task<IReadOnlyList<TimelineEvent>> GetBookmarkTimelineAsync(
        long reportId,
        CancellationToken cancellationToken = default)
    {
        var response = ApiHelpers.EnsureSuccess(
            await _http.GetAsync($"/datum/bookmarkReport/timeline/{reportId}", cancellationToken: cancellationToken).ConfigureAwait(false));
        return ApiHelpers.MapArray(response.Data, TimelineEvent.From);
    }
}
