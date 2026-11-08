package io.github.pezmax.one.core.reading

import kotlin.math.ceil
import kotlin.math.floor
import kotlin.math.max
import kotlin.math.min
import kotlin.math.roundToInt

/**
 * PDF 阅读器的视图几何与渲染调度（spec/fixtures/reading/）。
 *
 * 只管「算」不管「画」：像素尺寸、纹理、动画曲线在各端渲染层。
 * 平台差异只有一个——渲染并发数（pdfium 1、`Windows.Data.Pdf` 3、`PdfRenderer` 由调用方定），
 * 所以这里不写死并发。
 */
data class PageSize(val width: Float, val height: Float) {
    val safeWidth: Float get() = if (width > 0f) width else A4_WIDTH
    val safeHeight: Float get() = if (height > 0f) height else A4_HEIGHT

    companion object {
        const val A4_WIDTH = 595f
        const val A4_HEIGHT = 842f
    }
}

data class IntRange2(val first: Int, val last: Int) {
    val isEmpty: Boolean get() = first > last

    companion object {
        val Empty = IntRange2(0, -1)
    }
}

object PdfGeometry {

    const val OVERSCAN = 2
    const val MIN_ZOOM = 0.5f
    const val MAX_ZOOM = 4.0f
    const val ZOOM_STEP = 0.25f
    const val GRID_MIN_TILE_WIDTH = 180f
    const val GRID_GAP = 12f
    const val THUMBNAIL_WIDTH = 150f
    const val OVERVIEW_THUMBNAIL_WIDTH = 120f
    const val OVERVIEW_PANEL_WIDTH = 150f

    fun pageHeight(page: PageSize, displayWidth: Float): Float =
        page.safeHeight / page.safeWidth * displayWidth

    /** 页顶偏移：前面所有页高 + 页间距；逐页算，不拿第一页高度乘页数。 */
    fun tops(pages: List<PageSize>, displayWidth: Float, gap: Float): List<Float> {
        val result = ArrayList<Float>(pages.size)
        var y = 0f
        pages.forEachIndexed { index, page ->
            y += if (index == 0) 0f else gap
            result += y
            y += pageHeight(page, displayWidth)
        }
        return result
    }

    fun contentHeight(pages: List<PageSize>, displayWidth: Float, gap: Float): Float {
        if (pages.isEmpty()) return 0f
        val heights = pages.sumOf { pageHeight(it, displayWidth).toDouble() }.toFloat()
        return heights + gap * (pages.size - 1)
    }

    /**
     * 可见范围（闭区间）。取 [scroll, scroll+viewport) 与 [top, top+h) 相交；
     * 页底正好等于滚动位置算不可见。滚过底/负偏移都返回最后一页/第一页，不出空范围。
     */
    fun visible(
        pages: List<PageSize>,
        displayWidth: Float,
        gap: Float,
        scrollOffset: Float,
        viewportHeight: Float,
    ): IntRange2 {
        if (pages.isEmpty()) return IntRange2.Empty
        val tops = tops(pages, displayWidth, gap)
        val content = contentHeight(pages, displayWidth, gap)
        val scroll = scrollOffset.coerceAtLeast(0f)
        if (scroll >= content) return IntRange2(pages.lastIndex, pages.lastIndex)
        if (viewportHeight >= content) return IntRange2(0, pages.lastIndex)
        val end = scroll + viewportHeight
        var first = -1
        var last = -1
        tops.forEachIndexed { index, top ->
            val height = pageHeight(pages[index], displayWidth)
            val intersects = top < end && top + height > scroll
            if (intersects) {
                if (first < 0) first = index
                last = index
            }
        }
        return if (first < 0) IntRange2.Empty else IntRange2(first, last)
    }

    /** 点落在哪一页：空隙里取前一页，内容之前取 0，之后取最后一页，空文档 -1。 */
    fun pageAt(pages: List<PageSize>, displayWidth: Float, gap: Float, y: Float): Int {
        if (pages.isEmpty()) return -1
        if (y < 0f) return 0
        val tops = tops(pages, displayWidth, gap)
        var candidate = 0
        for (index in tops.indices) {
            if (tops[index] <= y) candidate = index else break
        }
        return candidate
    }

    /**
     * 工具栏显示的当前页：取**视口中心**，有效视口高 = min(视口高, 内容高 − 滚动位置)。
     * 否则一屏容下整本时中心点落到内容外，页码会一路显示最后一页。
     */
    fun currentPage(
        pages: List<PageSize>,
        displayWidth: Float,
        gap: Float,
        scrollOffset: Float,
        viewportHeight: Float,
    ): Int {
        if (pages.isEmpty()) return -1
        val content = contentHeight(pages, displayWidth, gap)
        val scroll = scrollOffset.coerceAtLeast(0f)
        val effective = min(viewportHeight, (content - scroll).coerceAtLeast(0f))
        return pageAt(pages, displayWidth, gap, scroll + effective / 2f)
    }

    /** 渲染窗口 = 可见范围两侧各扩 overscan，再夹到 [0, pageCount-1]；夹取不整体平移窗口。 */
    fun renderWindow(visible: IntRange2, pageCount: Int, overscan: Int = OVERSCAN): IntRange2 {
        if (pageCount <= 0 || visible.isEmpty) return IntRange2.Empty
        val first = max(0, visible.first - overscan)
        val last = min(pageCount - 1, visible.last + overscan)
        return IntRange2(first, last)
    }

    /**
     * 渲染顺序：先可视区（索引升序），其余按离可视区边缘的距离由近及远、同距离索引升序；
     * 已缓存的跳过。
     */
    fun renderOrder(visible: IntRange2, render: IntRange2, cached: Set<Int>): List<Int> {
        if (render.isEmpty) return emptyList()
        val visiblePages = (visible.first..visible.last)
            .filter { it in render.first..render.last && it !in cached }
        val remaining = (render.first..render.last)
            .filter { it !in visible.first..visible.last && it !in cached }
            .sortedWith(compareBy({ distanceTo(it, visible) }, { it }))
        return visiblePages + remaining
    }

    private fun distanceTo(page: Int, visible: IntRange2): Int = when {
        page < visible.first -> visible.first - page
        page > visible.last -> page - visible.last
        else -> 0
    }

    fun clampZoom(zoom: Float): Float = zoom.coerceIn(MIN_ZOOM, MAX_ZOOM)

    /** 放大取不小于当前值的下一个格点；缩小取不大于当前值的上一个格点；已在上限/下限不动。 */
    fun stepZoom(zoom: Float, direction: Int): Float {
        if (direction == 0) return clampZoom(zoom)
        val epsilon = 1e-4f
        val grid = if (direction > 0) {
            ceil((zoom + epsilon) / ZOOM_STEP) * ZOOM_STEP
        } else {
            floor((zoom - epsilon) / ZOOM_STEP) * ZOOM_STEP
        }
        return clampZoom(grid)
    }

    /** 适应宽度：可用宽度 / 页宽，只夹上下限、不吸附步进；页宽非法按 1 倍。 */
    fun fitZoom(availableWidth: Float, pageWidth: Float): Float {
        if (pageWidth <= 0f || availableWidth <= 0f) return if (pageWidth <= 0f) 1f else clampZoom(availableWidth)
        return clampZoom(availableWidth / pageWidth)
    }

    /** 缓存文件主名 `p{页码:0000}_s{缩放×100 四舍五入:0000}`。 */
    fun cacheKey(pageIndex: Int, scale: Float): String {
        val percent = (scale * 100f).roundToInt()
        return "p%04d_s%04d".format(pageIndex, percent)
    }

    fun gridColumns(availableWidth: Float): Int {
        val raw = floor((availableWidth + GRID_GAP) / (GRID_MIN_TILE_WIDTH + GRID_GAP)).toInt()
        return max(1, raw)
    }

    fun gridTileWidth(availableWidth: Float, columns: Int): Float {
        val safeColumns = max(1, columns)
        return (availableWidth - GRID_GAP * (safeColumns - 1)) / safeColumns
    }

    fun gridRows(pageCount: Int, columns: Int): Int {
        if (pageCount <= 0) return 0
        return ceil(pageCount.toFloat() / max(1, columns)).toInt()
    }

    /** 窗口外的缓存按离窗口距离由远及近释放，同距离索引升序。 */
    fun evictOrder(cached: Set<Int>, keep: IntRange2): List<Int> {
        if (keep.isEmpty) return cached.sorted()
        return cached
            .filter { it < keep.first || it > keep.last }
            .sortedWith(compareByDescending<Int> { distanceTo(it, keep) }.thenBy { it })
    }
}
