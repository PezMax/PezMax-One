using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Models;
using PezMax.Core.Net;

namespace PezMax.Core.Api;

public sealed class CommunityApi
{
    private readonly DatumHttp _http;

    public CommunityApi(DatumHttp http) => _http = http;

    public async Task<IReadOnlyList<RankEntry>> GetRankAsync(
        string? hash = null,
        CancellationToken cancellationToken = default)
    {
        var query = hash == null
            ? null
            : new[] { new KeyValuePair<string, string?>("hash", hash) };
        var response = ApiHelpers.EnsureSuccess(
            await _http.GetAsync("/datum/user/rank", query, cancellationToken).ConfigureAwait(false));
        return ApiHelpers.MapArray(response.Data, RankEntry.From);
    }

    public async Task<UserInfo> GetUserAsync(long userId, CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureData(
            await _http.GetAsync($"/datum/user/{userId}", cancellationToken: cancellationToken).ConfigureAwait(false),
            UserInfo.From);
}
