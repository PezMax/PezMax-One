using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Models;
using PezMax.Core.Net;

namespace PezMax.Core.Api;

public sealed class NotificationFeed
{
    public NotificationFeed(IReadOnlyList<NotificationItem> items, string? hash, bool unchanged)
    {
        Items = items;
        Hash = hash;
        Unchanged = unchanged;
    }

    public IReadOnlyList<NotificationItem> Items { get; }
    public string? Hash { get; }
    public bool Unchanged { get; }
}

public sealed class NotificationsApi
{
    private readonly DatumHttp _http;

    public NotificationsApi(DatumHttp http) => _http = http;

    public async Task<NotificationFeed> GetPopupAsync(
        long? userId = null,
        string? hash = null,
        CancellationToken cancellationToken = default)
    {
        var query = new[]
        {
            new KeyValuePair<string, string?>("userId", userId?.ToString()),
            new KeyValuePair<string, string?>("hash", hash),
        };
        var response = await _http.GetAsync("/system/notification/user/popup", query, cancellationToken).ConfigureAwait(false);
        ApiHelpers.EnsureSuccess(response);
        return new NotificationFeed(
            ApiHelpers.MapArray(response.Data, NotificationItem.From), response.Hash, response.Unchanged);
    }

    public async Task<NotificationFeed> GetScrollAsync(string? hash = null, CancellationToken cancellationToken = default)
    {
        var query = hash == null
            ? null
            : new[] { new KeyValuePair<string, string?>("hash", hash) };
        var response = await _http.GetAsync("/system/notification/user/scroll", query, cancellationToken).ConfigureAwait(false);
        ApiHelpers.EnsureSuccess(response);
        return new NotificationFeed(
            ApiHelpers.MapArray(response.Data, NotificationItem.From), response.Hash, response.Unchanged);
    }

    public async Task<PageResult<NotificationItem>> GetListAsync(
        int pageNum = 1,
        int pageSize = 20,
        CancellationToken cancellationToken = default)
    {
        var query = new[]
        {
            new KeyValuePair<string, string?>("pageNum", pageNum.ToString()),
            new KeyValuePair<string, string?>("pageSize", pageSize.ToString()),
        };
        var response = await _http.GetAsync("/datum/notification/list", query, cancellationToken).ConfigureAwait(false);
        return ApiHelpers.EnsurePage(response, NotificationItem.From);
    }

    public async Task<NotificationItem> GetDetailAsync(long notifyId, CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureData(
            await _http.GetAsync($"/datum/notification/{notifyId}", cancellationToken: cancellationToken).ConfigureAwait(false),
            NotificationItem.From);
}
