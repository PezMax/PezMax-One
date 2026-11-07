using System;
using System.Collections.Generic;

namespace PezMax.Core.Reading;

/// <summary>
/// 连续视图的页面几何：每页的顶部偏移与高度、内容总高，以及「滚到哪一页、哪几页可见」。
/// 逐页按自己的宽高比算高度——电子书里夹一张横向扫描页是常事，按第一页的高度乘页数会把它裁掉。
/// </summary>
public sealed class PdfContinuousLayout
{
    private readonly double[] _tops;
    private readonly double[] _heights;

    private PdfContinuousLayout(double[] tops, double[] heights, double contentHeight)
    {
        _tops = tops;
        _heights = heights;
        ContentHeight = contentHeight;
    }

    /// <summary>空文档的布局：没有页、内容高为 0。</summary>
    public static PdfContinuousLayout Empty { get; } =
        new(Array.Empty<double>(), Array.Empty<double>(), 0);

    public int Count => _tops.Length;

    /// <summary>内容总高（含最后一页之后不加页间距）。</summary>
    public double ContentHeight { get; }

    /// <summary>每页的顶部偏移，长度等于页数、单调递增。</summary>
    public IReadOnlyList<double> Tops => _tops;

    /// <summary>每页的高度。</summary>
    public IReadOnlyList<double> Heights => _heights;

    /// <param name="sizes">每页尺寸；缺尺寸的页按 A4 兜底。</param>
    /// <param name="displayWidth">页面显示宽度（DIP）；渲染出来多宽与阅读器的缩放有关，这里只管排版。</param>
    /// <param name="gap">页间距（DIP）。</param>
    public static PdfContinuousLayout Create(IReadOnlyList<PdfPageSize>? sizes, double displayWidth, double gap)
    {
        var pages = sizes ?? Array.Empty<PdfPageSize>();
        if (pages.Count == 0) return Empty;

        var width = IsUsable(displayWidth) ? displayWidth : PdfPageSize.DefaultWidth;
        var spacing = IsUsable(gap) || gap == 0 ? gap : 0;
        if (spacing < 0) spacing = 0;

        var tops = new double[pages.Count];
        var heights = new double[pages.Count];
        var y = 0d;
        for (var i = 0; i < pages.Count; i++)
        {
            var height = pages[i].Normalized.Aspect * width;
            tops[i] = y;
            heights[i] = height;
            y += height + spacing;
        }

        // 最后一个间距不算进内容高度，否则底部会多出一条空白可滚区。
        return new PdfContinuousLayout(tops, heights, y - spacing);
    }

    /// <summary>
    /// 与 <c>[scrollOffset, scrollOffset + viewportHeight)</c> 相交的页范围。
    /// 负偏移按 0 算；滚到内容底部之后仍返回最后一页（返回空范围会让视口白屏）。
    /// </summary>
    public PageSpan Visible(double scrollOffset, double viewportHeight)
    {
        var count = Count;
        if (count == 0) return PageSpan.Empty;
        if (!IsUsable(viewportHeight) || viewportHeight <= 0) return PageSpan.Empty;

        var top = IsUsable(scrollOffset) && scrollOffset > 0 ? scrollOffset : 0;
        var bottom = top + viewportHeight;

        // 第一页：页底 > 视口顶（这样正好落在页间距里时会跳到下一页）。
        var first = count - 1;
        for (var i = 0; i < count; i++)
        {
            if (_tops[i] + _heights[i] > top)
            {
                first = i;
                break;
            }
        }

        // 最后一页：页顶 < 视口底；至少包含 first。
        var last = first;
        for (var i = first + 1; i < count; i++)
        {
            if (_tops[i] < bottom) last = i;
            else break;
        }

        return new PageSpan(first, last);
    }

    /// <summary>
    /// <paramref name="y"/> 落在哪一页。落在页间距里取前一页，内容之前取第 0 页，之后取最后一页；
    /// 空文档返回 -1。
    /// </summary>
    public int PageAt(double y)
    {
        var count = Count;
        if (count == 0) return -1;
        if (!IsUsable(y) || y <= 0) return 0;

        var lo = 0;
        var hi = count - 1;
        var found = 0;
        while (lo <= hi)
        {
            var mid = lo + ((hi - lo) / 2);
            if (_tops[mid] <= y)
            {
                found = mid;
                lo = mid + 1;
            }
            else
            {
                hi = mid - 1;
            }
        }

        return found;
    }

    /// <summary>工具栏该显示的页码：取视口中心的页，避免稍微滚一点页码就跳。</summary>
    public int CurrentPage(double scrollOffset, double viewportHeight)
    {
        if (Count == 0) return -1;
        var offset = IsUsable(scrollOffset) && scrollOffset > 0 ? scrollOffset : 0;
        var height = IsUsable(viewportHeight) && viewportHeight > 0 ? viewportHeight : 0;

        // 一屏容下整本（或已滚到底）时，超出内容的视口高度不算数——否则中心点落在内容之外，
        // 页码会一直显示最后一页，看起来像「跳到了文档末尾」。
        var remaining = ContentHeight - offset;
        if (height > remaining) height = remaining > 0 ? remaining : 0;

        return PageAt(offset + (height / 2));
    }

    /// <summary>把某页顶部对齐到视口顶部所需的滚动偏移。</summary>
    public double OffsetOf(int pageIndex)
    {
        var count = Count;
        if (count == 0) return 0;
        var index = pageIndex < 0 ? 0 : pageIndex >= count ? count - 1 : pageIndex;
        return _tops[index];
    }

    private static bool IsUsable(double value) => !double.IsNaN(value) && !double.IsInfinity(value);
}
