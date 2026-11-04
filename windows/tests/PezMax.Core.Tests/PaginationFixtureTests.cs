using System.Text.Json;
using PezMax.Core.Models;
using PezMax.Core.Net;
using PezMax.Core.State;
using Xunit;

namespace PezMax.Core.Tests;

public class PaginationFixtureTests
{
    [Fact]
    public void QueryNormalization_MatchesFixtures()
    {
        using var document = Spec.LoadJson("fixtures", "pagination", "cases.json");
        var checkedCases = 0;

        foreach (var item in document.RootElement.EnumerateArray())
        {
            var id = item.GetProperty("id").GetString()!;
            var input = item.GetProperty("input");
            if (!input.TryGetProperty("query", out var queryElement)) continue;
            if (input.TryGetProperty("response", out _)) continue;

            var query = queryElement.EnumerateObject()
                .ToDictionary(p => p.Name, p => (string?)p.Value.GetString());
            var (page, size) = PageFetcher.FromQuery(query);
            var expect = item.GetProperty("expect");

            Assert.True(expect.GetProperty("page").GetInt32() == page, $"{id}: page");
            Assert.True(expect.GetProperty("pageSize").GetInt32() == size, $"{id}: pageSize");
            checkedCases++;
        }

        Assert.True(checkedCases >= 6, $"鍒嗛〉鍙傛暟澶瑰叿璺戝埌鐨勭敤渚嬪お灏戯細{checkedCases}");
    }

    [Fact]
    public void ListResponse_MatchesFixtures()
    {
        using var document = Spec.LoadJson("fixtures", "pagination", "cases.json");

        foreach (var item in document.RootElement.EnumerateArray())
        {
            var id = item.GetProperty("id").GetString()!;
            var input = item.GetProperty("input");
            if (!input.TryGetProperty("response", out var responseSpec)) continue;

            var status = responseSpec.GetProperty("status").GetInt32();
            var contentType = responseSpec.GetProperty("contentType").GetString();
            var body = responseSpec.GetProperty("body").GetString();
            var response = DatumEnvelope.Parse(status, contentType, body);
            var expect = item.GetProperty("expect");

            Assert.True(response.Kind == DatumResultKind.List, $"{id}: kind = {response.Kind}");
            Assert.True(expect.GetProperty("total").GetInt64() == response.Total, $"{id}: total");
            Assert.True(expect.GetProperty("rows").GetArrayLength() == response.Rows!.Count, $"{id}: rows");
        }
    }

    [Fact]
    public void FetchAll_MatchesFixtures()
    {
        using var document = Spec.LoadJson("fixtures", "pagination", "cases.json");

        foreach (var item in document.RootElement.EnumerateArray())
        {
            var id = item.GetProperty("id").GetString()!;
            var input = item.GetProperty("input");
            if (!input.TryGetProperty("pages", out var pagesElement)) continue;

            var pageSize = input.GetProperty("pageSize").GetInt32();
            var maxPages = input.GetProperty("maxPages").GetInt32();
            var pages = pagesElement.EnumerateArray().Select(p => new
            {
                PageNum = p.GetProperty("pageNum").GetInt32(),
                Rows = p.GetProperty("rows").GetInt32(),
                Total = p.GetProperty("total").GetInt64(),
            }).ToList();

            var requested = new List<int>();
            var result = PageFetcher.FetchAllAsync<int>(
                (pageNum, size) =>
                {
                    requested.Add(pageNum);
                    var page = pages.FirstOrDefault(p => p.PageNum == pageNum);
                    if (page == null) return Task.FromResult(new PageResult<int>(Array.Empty<int>(), pages[0].Total));
                    var rows = Enumerable.Range(0, page.Rows).Select(i => pageNum * 1000 + i).ToList();
                    return Task.FromResult(new PageResult<int>(rows, page.Total));
                },
                pageSize,
                maxPages).GetAwaiter().GetResult();

            var expect = item.GetProperty("expect");
            var expectedPages = expect.GetProperty("requestedPages").EnumerateArray().Select(p => p.GetInt32()).ToArray();
            Assert.True(expectedPages.SequenceEqual(requested), $"{id}: requested pages");
            Assert.True(expect.GetProperty("rowCount").GetInt32() == result.Rows.Count, $"{id}: row count");
            Assert.True(expect.GetProperty("total").GetInt64() == result.Total, $"{id}: total");
        }
    }
}
