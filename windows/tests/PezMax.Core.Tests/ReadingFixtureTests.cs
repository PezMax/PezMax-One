using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using PezMax.Core.Reading;
using Xunit;
using Xunit.Sdk;

namespace PezMax.Core.Tests;

/// <summary>阅读器的视图几何与渲染调度，逐条对齐 `spec/fixtures/reading/cases.json`。</summary>
public class ReadingFixtureTests
{
    private const double Tolerance = 1e-6;

    [Fact]
    public void PdfLayout_MatchesReadingFixtures()
    {
        using var document = Spec.LoadJson("fixtures", "reading", "cases.json");
        var cases = document.RootElement;
        Assert.Equal(JsonValueKind.Array, cases.ValueKind);

        var checkedCases = 0;
        foreach (var item in cases.EnumerateArray())
        {
            var id = item.GetProperty("id").GetString()!;
            var input = item.GetProperty("input");
            var expect = item.GetProperty("expect");

            switch (input.GetProperty("kind").GetString())
            {
                case "continuous":
                    RunContinuous(id, input, expect);
                    break;

                case "overscan":
                {
                    var overscan = input.TryGetProperty("overscan", out var custom)
                        ? custom.GetInt32()
                        : PdfLayout.OverscanPages;
                    var render = PdfLayout.WithOverscan(
                        ReadSpan(input.GetProperty("visible")),
                        input.GetProperty("pageCount").GetInt32(),
                        overscan);
                    AssertSpan(id + "/render", expect.GetProperty("render"), render);
                    break;
                }

                case "renderOrder":
                {
                    var cached = input.GetProperty("cached").EnumerateArray().Select(x => x.GetInt32()).ToArray();
                    var order = PdfLayout.RenderOrder(
                        ReadSpan(input.GetProperty("visible")),
                        ReadSpan(input.GetProperty("render")),
                        cached);
                    Assert.Equal(
                        expect.GetProperty("order").EnumerateArray().Select(x => x.GetInt32()).ToArray(),
                        order);
                    break;
                }

                case "zoom":
                {
                    var zoom = ApplyZoom(input);
                    AssertClose(id + "/zoom", expect.GetProperty("zoom").GetDouble(), zoom);
                    break;
                }

                case "cacheKey":
                {
                    var key = PdfLayout.PageCacheKey(
                        input.GetProperty("pageIndex").GetInt32(),
                        input.GetProperty("scale").GetDouble());
                    Assert.True(expect.GetProperty("key").GetString() == key, $"{id}: key = {key}");
                    break;
                }

                case "grid":
                    RunGrid(id, input, expect);
                    break;

                case "evict":
                {
                    var cached = input.GetProperty("cached").EnumerateArray().Select(x => x.GetInt32()).ToArray();
                    var release = PdfLayout.PagesToRelease(cached, ReadSpan(input.GetProperty("keep")));
                    Assert.Equal(
                        expect.GetProperty("release").EnumerateArray().Select(x => x.GetInt32()).ToArray(),
                        release);
                    break;
                }

                default:
                    throw new XunitException($"{id}: 未知 kind {input.GetProperty("kind").GetString()}");
            }

            checkedCases++;
        }

        Assert.True(checkedCases >= 48, $"阅读器夹具跑到的用例太少：{checkedCases}");
    }

    [Fact]
    public void PdfPageSize_FallsBackToA4WhenMissing()
    {
        var size = default(PdfPageSize).Normalized;
        Assert.Equal(PdfPageSize.DefaultWidth, size.Width, 6);
        Assert.Equal(PdfPageSize.DefaultHeight, size.Height, 6);

        // 整本都读不出尺寸时也要排得出来，而不是内容高为 0。
        var layout = PdfContinuousLayout.Create(new[] { default(PdfPageSize) }, PdfPageSize.DefaultWidth, 0);
        Assert.Equal(PdfPageSize.DefaultHeight, layout.ContentHeight, 6);
    }

    [Fact]
    public void Visible_WhenViewportFallsInsideGap_ShowsFollowingPage()
    {
        var layout = PdfContinuousLayout.Create(
            new[] { new PdfPageSize(600, 800), new PdfPageSize(600, 800) },
            300,
            10);

        // 页 0 是 [0,400)，页 1 是 [410,810)，视口 [402,407) 整段落在间隙里。
        var span = layout.Visible(402, 5);
        Assert.Equal(1, span.First);
        Assert.Equal(1, span.Last);
    }

    [Fact]
    public void Visible_ExtendsToTheLastPageWhenViewportCoversTheRest()
    {
        var layout = PdfContinuousLayout.Create(
            new[] { new PdfPageSize(600, 800), new PdfPageSize(600, 800), new PdfPageSize(600, 800) },
            300,
            10);

        var span = layout.Visible(395, 1000);
        Assert.Equal(0, span.First);
        Assert.Equal(2, span.Last);
    }

    private static void RunContinuous(string id, JsonElement input, JsonElement expect)
    {
        var sizes = new List<PdfPageSize>();
        foreach (var page in input.GetProperty("pages").EnumerateArray())
        {
            sizes.Add(new PdfPageSize(page.GetProperty("width").GetDouble(), page.GetProperty("height").GetDouble()));
        }

        var layout = PdfContinuousLayout.Create(
            sizes,
            input.GetProperty("displayWidth").GetDouble(),
            input.GetProperty("gap").GetDouble());

        foreach (var op in input.GetProperty("ops").EnumerateArray())
        {
            var name = op.GetProperty("op").GetString()!;
            switch (name)
            {
                case "tops":
                    AssertDoubles(id + "/tops", expect.GetProperty("tops"), layout.Tops);
                    break;

                case "contentHeight":
                    AssertClose(
                        id + "/contentHeight",
                        expect.GetProperty("contentHeight").GetDouble(),
                        layout.ContentHeight);
                    break;

                case "visible":
                    AssertSpan(
                        id + "/visible",
                        expect.GetProperty("visible"),
                        layout.Visible(op.GetProperty("scrollOffset").GetDouble(), op.GetProperty("viewportHeight").GetDouble()));
                    break;

                case "pageAt":
                    Assert.Equal(
                        expect.GetProperty("pageAt").GetInt32(),
                        layout.PageAt(op.GetProperty("y").GetDouble()));
                    break;

                case "offsetOf":
                    AssertClose(
                        id + "/offsetOf",
                        expect.GetProperty("offsetOf").GetDouble(),
                        layout.OffsetOf(op.GetProperty("page").GetInt32()));
                    break;

                case "current":
                    Assert.Equal(
                        expect.GetProperty("current").GetInt32(),
                        layout.CurrentPage(op.GetProperty("scrollOffset").GetDouble(), op.GetProperty("viewportHeight").GetDouble()));
                    break;

                default:
                    throw new XunitException($"{id}: 未知 op {name}");
            }
        }
    }

    private static void RunGrid(string id, JsonElement input, JsonElement expect)
    {
        var gap = input.TryGetProperty("gap", out var customGap) ? customGap.GetDouble() : PdfLayout.GridGap;
        var minTile = input.TryGetProperty("minTileWidth", out var customTile)
            ? customTile.GetDouble()
            : PdfLayout.GridMinTileWidth;

        switch (input.GetProperty("op").GetString())
        {
            case "columns":
                Assert.Equal(
                    expect.GetProperty("columns").GetInt32(),
                    PdfLayout.GridColumns(input.GetProperty("availableWidth").GetDouble(), minTile, gap));
                break;

            case "rows":
                Assert.Equal(
                    expect.GetProperty("rows").GetInt32(),
                    PdfLayout.GridRows(input.GetProperty("pageCount").GetInt32(), input.GetProperty("columns").GetInt32()));
                break;

            case "tileWidth":
                AssertClose(
                    id + "/tileWidth",
                    expect.GetProperty("tileWidth").GetDouble(),
                    PdfLayout.GridTileWidth(
                        input.GetProperty("availableWidth").GetDouble(),
                        input.GetProperty("columns").GetInt32(),
                        gap));
                break;

            default:
                throw new XunitException($"{id}: 未知 grid op {input.GetProperty("op").GetString()}");
        }
    }

    private static double ApplyZoom(JsonElement input) => input.GetProperty("op").GetString() switch
    {
        "clamp" => PdfLayout.ClampZoom(input.GetProperty("zoom").GetDouble()),
        "step" => PdfLayout.StepZoom(
            input.GetProperty("zoom").GetDouble(),
            input.GetProperty("direction").GetInt32()),
        "fit" => PdfLayout.FitZoom(
            input.GetProperty("pageWidth").GetDouble(),
            input.GetProperty("availableWidth").GetDouble()),
        var other => throw new XunitException($"未知 zoom op {other}"),
    };

    private static PageSpan ReadSpan(JsonElement element)
    {
        var items = element.EnumerateArray().Select(x => x.GetInt32()).ToArray();
        return items.Length == 0 ? PageSpan.Empty : new PageSpan(items[0], items[1]);
    }

    private static void AssertSpan(string label, JsonElement expect, PageSpan actual)
    {
        var items = expect.EnumerateArray().Select(x => x.GetInt32()).ToArray();
        if (items.Length == 0)
        {
            Assert.True(actual.IsEmpty, $"{label}: 期望空范围，实际 {actual}");
            return;
        }

        Assert.True(items.Length == 2, $"{label}: 期望值要写成 [first, last]");
        Assert.True(items[0] == actual.First && items[1] == actual.Last,
            $"{label}: 期望 [{items[0]}, {items[1]}]，实际 {actual}");
    }

    private static void AssertDoubles(string label, JsonElement expect, IReadOnlyList<double> actual)
    {
        var items = expect.EnumerateArray().Select(x => x.GetDouble()).ToArray();
        Assert.True(items.Length == actual.Count, $"{label}: 期望 {items.Length} 项，实际 {actual.Count} 项");
        for (var i = 0; i < items.Length; i++) AssertClose($"{label}[{i}]", items[i], actual[i]);
    }

    private static void AssertClose(string label, double expect, double actual) =>
        Assert.True(Math.Abs(expect - actual) <= Tolerance, $"{label}: 期望 {expect}，实际 {actual}");
}
