using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Json;
using PezMax.Core.Models;
using PezMax.Core.Net;

namespace PezMax.Core.Api;

public sealed class FavoritesApi
{
    private readonly DatumHttp _http;

    public FavoritesApi(DatumHttp http) => _http = http;

    public async Task AddFileAsync(long fileId, CancellationToken cancellationToken = default)
    {
        var body = new JsonObjectBuilder().Number("fileId", fileId).ToString();
        ApiHelpers.EnsureTrue(
            await _http.PostJsonAsync("/datum/favorite", body, cancellationToken).ConfigureAwait(false));
    }

    public async Task<bool> IsFileFavoritedAsync(long fileId, CancellationToken cancellationToken = default)
    {
        var response = ApiHelpers.EnsureSuccess(
            await _http.GetAsync($"/datum/favorite/{fileId}", cancellationToken: cancellationToken).ConfigureAwait(false));
        return response.Data?.GetBool("favorited") ?? false;
    }

    public async Task<PageResult<FileItem>> GetMyFilesAsync(
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
            $"/datum/desktop/favorite/list/{userId}", query, cancellationToken).ConfigureAwait(false);
        return ApiHelpers.EnsurePage(response, FileItem.From);
    }

    public async Task RemoveFileAsync(long userId, long fileId, CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureTrue(
            await _http.DeleteAsync($"/datum/desktop/favorite/{userId}/{fileId}", cancellationToken).ConfigureAwait(false));

    public async Task<PageResult<EbookItem>> GetMyEbooksAsync(
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
            $"/datum/desktop/ebook/favorite/list/{userId}", query, cancellationToken).ConfigureAwait(false);
        return ApiHelpers.EnsurePage(response, EbookItem.From);
    }

    public async Task RemoveEbookAsync(long userId, long ebookId, CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureTrue(
            await _http.DeleteAsync($"/datum/desktop/ebook/favorite/{userId}/{ebookId}", cancellationToken).ConfigureAwait(false));

    public async Task<PageResult<Bookmark>> GetMyBookmarksAsync(
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
            $"/datum/desktop/bookmark/favorite/list/{userId}", query, cancellationToken).ConfigureAwait(false);
        return ApiHelpers.EnsurePage(response, Bookmark.From);
    }

    public async Task RemoveBookmarkAsync(long userId, long bookmarkId, CancellationToken cancellationToken = default) =>
        ApiHelpers.EnsureTrue(
            await _http.DeleteAsync($"/datum/desktop/bookmark/favorite/{userId}/{bookmarkId}", cancellationToken).ConfigureAwait(false));
}
