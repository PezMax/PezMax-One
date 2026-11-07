using System;
using System.Collections.Generic;
using System.Globalization;

namespace PezMax.Core.Reading;

/// <summary>
/// PDF 阅读器的渲染调度与缩放规则，逐条对齐 <c>spec/fixtures/reading/</c>。
/// 只管算：像素尺寸、纹理上传、动画曲线都属于各端渲染层。
/// </summary>
public static class PdfLayout
{
    /// <summary>可见范围两侧各多渲染几页。</summary>
    public const int OverscanPages = 2;

    /// <summary>同时在渲的页数上限（Windows 端取值；pdfium 不是线程安全的，Linux 端只能是 1）。</summary>
    public const int MaxConcurrentRenders = 3;

    public const double MinZoom = 0.5;

    public const double MaxZoom = 4.0;

    public const double ZoomStep = 0.25;

    /// <summary>网格视图里一个缩略图格子的最小宽度（DIP）。</summary>
    public const double GridMinTileWidth = 180;

    /// <summary>网格视图的行列间距（DIP）。</summary>
    public const double GridGap = 12;

    /// <summary>网格缩略图的目标宽度（DIP），沿用 egui 版的 <c>THUMB_WIDTH</c>。</summary>
    public const double ThumbnailWidth = 150;

    /// <summary>左侧概览面板的缩略图宽度（DIP）。</summary>
    public const double OverviewThumbnailWidth = 120;

    /// <summary>左侧概览面板宽度（DIP）。</summary>
    public const double OverviewPanelWidth = 150;

    /// <summary>
    /// 渲染窗口 = 可见范围两侧各扩 <paramref name="overscan"/> 页，再夹到 <c>[0, pageCount-1]</c>。
    /// 夹下界时窗口不平移（预取只在有页的方向上发生）。
    /// </summary>
    public static PageSpan WithOverscan(PageSpan visible, int pageCount, int overscan = OverscanPages)
    {
        if (pageCount <= 0 || visible.IsEmpty) return PageSpan.Empty;
        if (overscan < 0) overscan = 0;

        var lastPage = pageCount - 1;
        var first = visible.First - overscan;
        var last = visible.Last + overscan;
        if (first < 0) first = 0;
        if (last > lastPage) last = lastPage;
        if (first > lastPage) first = lastPage;
        if (last < first) return PageSpan.Empty;
        return new PageSpan(first, last);
    }

    /// <summary>
    /// 渲染顺序：先可视区（索引升序），再按离可视区边缘的距离由近及远，同距离索引升序；
    /// 已缓存的跳过。读者最先看到视口顶部那一页，所以先渲中间页是错的。
    /// </summary>
    public static int[] RenderOrder(PageSpan visible, PageSpan render, IEnumerable<int>? cached = null)
    {
        if (render.IsEmpty) return Array.Empty<int>();

        var have = cached == null ? new HashSet<int>() : new HashSet<int>(cached);
        var order = new List<int>(render.Count);

        if (!visible.IsEmpty)
        {
            for (var i = visible.First; i <= visible.Last; i++)
            {
                if (render.Contains(i) && !have.Contains(i)) order.Add(i);
            }

            var reach = Math.Max(visible.First - render.First, render.Last - visible.Last);
            for (var distance = 1; distance <= reach; distance++)
            {
                var before = visible.First - distance;
                if (before >= render.First && !have.Contains(before)) order.Add(before);

                var after = visible.Last + distance;
                if (after <= render.Last && !have.Contains(after)) order.Add(after);
            }

            return order.ToArray();
        }

        for (var i = render.First; i <= render.Last; i++)
        {
            if (!have.Contains(i)) order.Add(i);
        }

        return order.ToArray();
    }

    /// <summary>
    /// 该从内存里丢掉的页：<paramref name="keep"/> 之外的已缓存页，离窗口最远的排前面。
    /// 内存吃紧时可以只丢前几个，把窗口附近的留到最后。
    /// </summary>
    public static int[] PagesToRelease(IEnumerable<int>? cached, PageSpan keep)
    {
        if (cached == null) return Array.Empty<int>();

        var outside = new List<int>();
        foreach (var page in cached)
        {
            if (!keep.Contains(page)) outside.Add(page);
        }

        outside.Sort((left, right) =>
        {
            var leftDistance = DistanceTo(left, keep);
            var rightDistance = DistanceTo(right, keep);
            return leftDistance != rightDistance
                ? rightDistance.CompareTo(leftDistance)
                : left.CompareTo(right);
        });

        return outside.ToArray();
    }

    /// <summary>夹到 <c>[MinZoom, MaxZoom]</c>，不吸附步进（读存档时不能擅自改写用户的缩放）。</summary>
    public static double ClampZoom(double zoom)
    {
        if (double.IsNaN(zoom)) return 1.0;
        if (zoom < MinZoom) return MinZoom;
        if (zoom > MaxZoom) return MaxZoom;
        return zoom;
    }

    /// <summary>
    /// 走一档缩放：放大取不小于当前值的下一个 <see cref="ZoomStep"/> 倍数，缩小取不大于的下一个；
    /// 当前值正好在格点上时走一步。
    /// </summary>
    public static double StepZoom(double zoom, int direction)
    {
        var current = ClampZoom(zoom);
        if (direction == 0) return current;

        if (direction > 0)
        {
            var next = Math.Ceiling(current / ZoomStep) * ZoomStep;
            if (next <= current) next = current + ZoomStep;
            return ClampZoom(next);
        }

        var previous = Math.Floor(current / ZoomStep) * ZoomStep;
        if (previous >= current) previous = current - ZoomStep;
        return ClampZoom(previous);
    }

    /// <summary>
    /// 适应宽度：可用宽度 / 页宽，只夹上下限、不吸附步进——吸附会让页面比视口宽一丁点，
    /// 于是出现横向滚动条，而这正是「适应宽度」要消灭的东西。
    /// </summary>
    public static double FitZoom(double pageWidth, double availableWidth)
    {
        if (!IsUsable(pageWidth) || pageWidth <= 0) return 1.0;
        if (!IsUsable(availableWidth) || availableWidth <= 0) return MinZoom;
        return ClampZoom(availableWidth / pageWidth);
    }

    /// <summary>
    /// 页缓存的文件主名：<c>p{页码:0000}_s{缩放×100:0000}</c>。
    /// 缩放取四舍五入的整数百分比，浮点噪声因此落在同一个键上。
    /// 注意这里的缩放是**渲染缩放**（DPI 因子 × 逻辑缩放），可以超过 <see cref="MaxZoom"/>，所以不夹上限。
    /// </summary>
    public static string PageCacheKey(int pageIndex, double renderScale)
    {
        var index = pageIndex < 0 ? 0 : pageIndex;
        var scale = IsUsable(renderScale) && renderScale > 0 ? renderScale : MinZoom;
        var hundredths = (int)Math.Round(scale * 100, MidpointRounding.AwayFromZero);
        return "p" + index.ToString("D4", CultureInfo.InvariantCulture) +
               "_s" + hundredths.ToString("D4", CultureInfo.InvariantCulture);
    }

    /// <summary>网格列数；窄到放不下一格时给 1 列，0 列就是布局崩溃。</summary>
    public static int GridColumns(double availableWidth, double minTileWidth = GridMinTileWidth, double gap = GridGap)
    {
        var width = IsUsable(availableWidth) && availableWidth > 0 ? availableWidth : 0;
        var min = IsUsable(minTileWidth) && minTileWidth > 0 ? minTileWidth : GridMinTileWidth;
        var spacing = IsUsable(gap) && gap > 0 ? gap : 0;

        var columns = (int)Math.Floor((width + spacing) / (min + spacing));
        return columns < 1 ? 1 : columns;
    }

    /// <summary>网格格宽 =（可用宽度 − 间距×(列数−1)）/ 列数。</summary>
    public static double GridTileWidth(double availableWidth, int columns, double gap = GridGap)
    {
        var count = columns < 1 ? 1 : columns;
        var width = IsUsable(availableWidth) && availableWidth > 0 ? availableWidth : 0;
        var spacing = IsUsable(gap) && gap > 0 ? gap : 0;

        var tile = (width - (spacing * (count - 1))) / count;
        return tile > 0 ? tile : 0;
    }

    /// <summary>网格行数；没有页时是 0，不是 1。</summary>
    public static int GridRows(int pageCount, int columns)
    {
        if (pageCount <= 0) return 0;
        var count = columns < 1 ? 1 : columns;
        return (pageCount + count - 1) / count;
    }

    private static int DistanceTo(int page, PageSpan span)
    {
        if (span.IsEmpty) return 0;
        if (page < span.First) return span.First - page;
        if (page > span.Last) return page - span.Last;
        return 0;
    }

    private static bool IsUsable(double value) => !double.IsNaN(value) && !double.IsInfinity(value);
}
