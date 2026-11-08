package io.github.pezmax.one.core.net

/**
 * 分页参数归一（spec/fixtures/pagination/README.md）：
 * `pageNum`/`pageSize` 优先，`page`/`size` 是别名；默认 1/20；非法值回落默认；上限 100。
 *
 * 别名只在首选参数**整个缺失**时才看——首选参数写了非法值就按缺省处理，
 * 不再去猜别名的值，否则客户端与服务端会各归一一次、结果不同。
 */
data class PageQuery(val page: Int, val pageSize: Int) {
    companion object {
        const val DEFAULT_PAGE = 1
        const val DEFAULT_PAGE_SIZE = 20
        const val MAX_PAGE_SIZE = 100
    }
}

fun normalizePageQuery(query: Map<String, String?>): PageQuery {
    val pageRaw = query["pageNum"]?.takeIf { it.isNotBlank() } ?: query["page"]?.takeIf { it.isNotBlank() }
    val sizeRaw = query["pageSize"]?.takeIf { it.isNotBlank() } ?: query["size"]?.takeIf { it.isNotBlank() }
    val page = pageRaw?.toIntOrNull()?.takeIf { it > 0 } ?: PageQuery.DEFAULT_PAGE
    val size = sizeRaw?.toIntOrNull()?.takeIf { it > 0 } ?: PageQuery.DEFAULT_PAGE_SIZE
    return PageQuery(page = page, pageSize = size.coerceAtMost(PageQuery.MAX_PAGE_SIZE))
}

/** 一页数据；[total] 缺失时调用方按本页条数兜底。 */
data class PageSlice<T>(val rows: List<T>, val total: Int? = null)

enum class StopReason(val wire: String) {
    ReachedTotal("reached-total"),
    EmptyPage("empty-page"),
    MaxPages("max-pages"),
}

data class FetchAllResult<T>(
    val rows: List<T>,
    val total: Int,
    val requestedPages: List<Int>,
    val stopReason: StopReason,
)

/**
 * 按 total 拉全量。算法见 spec/fixtures/pagination/README.md「全量拉取算法」：
 * 首页失败向上抛（不返回半份冒充成功），中途空页立即停，maxPages 是死循环保险。
 */
suspend fun <T> fetchAllPages(
    pageSize: Int = PageQuery.MAX_PAGE_SIZE,
    maxPages: Int = 100,
    fetch: suspend (PageQuery) -> PageSlice<T>,
): FetchAllResult<T> {
    require(pageSize in 1..PageQuery.MAX_PAGE_SIZE) { "pageSize 必须落在 1..100" }
    require(maxPages >= 1) { "maxPages 至少 1" }

    val requested = mutableListOf(1)
    val first = fetch(PageQuery(1, pageSize))
    val rows = first.rows.toMutableList()
    val total = first.total ?: rows.size

    var stopReason = StopReason.ReachedTotal
    var page = 2
    while (rows.size < total) {
        if (page > maxPages) {
            stopReason = StopReason.MaxPages
            break
        }
        requested += page
        val part = fetch(PageQuery(page, pageSize))
        if (part.rows.isEmpty()) {
            stopReason = StopReason.EmptyPage
            break
        }
        rows += part.rows
        page++
    }
    return FetchAllResult(rows = rows, total = total, requestedPages = requested, stopReason = stopReason)
}
