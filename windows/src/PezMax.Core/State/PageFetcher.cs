using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Models;

namespace PezMax.Core.State;

public static class PageFetcher
{
    public const int MaxPageSize = 100;
    public const int DefaultPageSize = 20;

    public static (int Page, int Size) Normalize(int page, int size)
    {
        if (page < 1) page = 1;
        if (size < 1) size = DefaultPageSize;
        if (size > MaxPageSize) size = MaxPageSize;
        return (page, size);
    }

    /// <summary>按服务端别名规则解析查询参数：pageNum 优先于 page，pageSize 优先于 size。</summary>
    public static (int Page, int Size) FromQuery(IReadOnlyDictionary<string, string?> query)
    {
        var page = 1;
        var size = DefaultPageSize;
        if (TryReadPage(query, "pageNum", "page", out var parsedPage)) page = parsedPage;
        if (TryReadPage(query, "pageSize", "size", out var parsedSize)) size = parsedSize;
        return Normalize(page, size);
    }

    private static bool TryReadPage(IReadOnlyDictionary<string, string?> query, string primary, string alias, out int value)
    {
        value = 0;
        var key = query.ContainsKey(primary) && !string.IsNullOrWhiteSpace(query[primary]) ? primary
            : query.ContainsKey(alias) && !string.IsNullOrWhiteSpace(query[alias]) ? alias
            : null;
        if (key == null) return false;
        return int.TryParse(query[key], out value) && value > 0;
    }

    public static async Task<PageResult<T>> FetchAllAsync<T>(
        Func<int, int, Task<PageResult<T>>> fetch,
        int pageSize = MaxPageSize,
        int maxPages = 100,
        CancellationToken cancellationToken = default)
    {
        if (fetch == null) throw new ArgumentNullException(nameof(fetch));
        if (pageSize < 1) pageSize = DefaultPageSize;
        if (pageSize > MaxPageSize) pageSize = MaxPageSize;
        if (maxPages < 1) maxPages = 1;

        var all = new List<T>();
        var first = await fetch(1, pageSize).ConfigureAwait(false);
        all.AddRange(first.Rows);
        var total = first.Total > 0 ? first.Total : first.Rows.Count;

        for (var page = 2; page <= maxPages && all.Count < total; page++)
        {
            cancellationToken.ThrowIfCancellationRequested();
            var part = await fetch(page, pageSize).ConfigureAwait(false);
            if (part.Rows.Count == 0) break;
            all.AddRange(part.Rows);
        }

        return new PageResult<T>(all, total);
    }
}
