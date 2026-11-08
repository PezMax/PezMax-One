package io.github.pezmax.one.core

import io.github.pezmax.one.core.SpecFixtures.expect
import io.github.pezmax.one.core.SpecFixtures.input
import io.github.pezmax.one.core.json.arrayOrNull
import io.github.pezmax.one.core.json.intOrNull
import io.github.pezmax.one.core.json.objectOrNull
import io.github.pezmax.one.core.json.stringOrNull
import io.github.pezmax.one.core.net.ApiResult
import io.github.pezmax.one.core.net.Envelope
import io.github.pezmax.one.core.net.PageQuery
import io.github.pezmax.one.core.net.PageSlice
import io.github.pezmax.one.core.net.StopReason
import io.github.pezmax.one.core.net.fetchAllPages
import io.github.pezmax.one.core.net.normalizePageQuery
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** spec/fixtures/pagination/cases.json —— 参数归一、列表响应、按 total 拉全量。 */
class PaginationFixtureTest {

    @Test
    fun `参数归一与列表响应逐条一致`() {
        val cases = SpecFixtures.cases("fixtures/pagination/cases.json")
        cases.filter { it.input().containsKey("query") && !it.input().containsKey("response") && !it.input().containsKey("pages") }
            .forEach { case ->
                val query = case.input().obj("query").entries.associate { (k, v) -> k to (v as JsonPrimitive).content }
                val normalized = normalizePageQuery(query)
                assertEquals(case.str("id") + " page", case.expect().intOrNull("page"), normalized.page)
                assertEquals(case.str("id") + " pageSize", case.expect().intOrNull("pageSize"), normalized.pageSize)
            }
    }

    @Test
    fun `列表响应按 total 与 rows 判定`() {
        val cases = SpecFixtures.cases("fixtures/pagination/cases.json")
        cases.filter { it.input().containsKey("response") }.forEach { case ->
            val response = case.input().obj("response")
            val result = Envelope.parse(
                status = response.intOrNull("status") ?: 0,
                contentType = response.stringOrNull("contentType") ?: "",
                body = response.stringOrNull("body"),
            )
            assertTrue(case.str("id"), result is ApiResult.ListSuccess)
            val expect = case.expect()
            val list = result as ApiResult.ListSuccess
            assertEquals(case.str("id") + " rows", expect.arr("rows"), JsonArray(list.rows))
            assertEquals(case.str("id") + " total", expect.intOrNull("total"), list.total)
            assertFalse(case.str("id") + " error", expect["error"]?.let { (it as JsonPrimitive).content } == "true")
        }
    }

    @Test
    fun `全量拉取的页序与终止原因逐条一致`() = runBlocking {
        val cases = SpecFixtures.cases("fixtures/pagination/cases.json")
        cases.filter { it.input().containsKey("pages") }.forEach { case ->
            val input = case.input()
            val pages = input.arr("pages").map { it as JsonObject }
                .associateBy { it.intOrNull("pageNum") ?: 0 }
            val requested = mutableListOf<Int>()
            val pageSize = input.intOrNull("pageSize") ?: PageQuery.MAX_PAGE_SIZE
            val maxPages = input.intOrNull("maxPages") ?: 100

            val result = fetchAllPages<Int>(
                pageSize = pageSize,
                maxPages = maxPages,
                fetch = { query ->
                    requested += query.page
                    val page = pages[query.page]
                    if (page == null) {
                        PageSlice(emptyList(), null)
                    } else {
                        val count = page.intOrNull("rows") ?: 0
                        PageSlice(List(count) { it }, page.intOrNull("total"))
                    }
                },
            )

            val expect = case.expect()
            assertEquals(
                case.str("id") + " requestedPages",
                expect.arr("requestedPages").map { (it as JsonPrimitive).content.toInt() },
                requested,
            )
            assertEquals(case.str("id") + " rowCount", expect.intOrNull("rowCount"), result.rows.size)
            assertEquals(case.str("id") + " total", expect.intOrNull("total"), result.total)
            assertEquals(case.str("id") + " stopReason", expect.stringOrNull("stopReason"), result.stopReason.wire)
        }
    }

    @Test
    fun `pageSize 上限与默认值`() {
        assertEquals(PageQuery(1, 20), normalizePageQuery(emptyMap()))
        assertEquals(PageQuery(3, 50), normalizePageQuery(mapOf("page" to "3", "size" to "50")))
        assertEquals(PageQuery(2, 10), normalizePageQuery(mapOf("pageNum" to "2", "page" to "9", "pageSize" to "10", "size" to "90")))
        assertEquals(PageQuery(1, 100), normalizePageQuery(mapOf("size" to "101")))
        assertEquals(PageQuery(1, 20), normalizePageQuery(mapOf("pageNum" to "abc")))
    }

    @Test
    fun `空页不算失败、total 不被清空`() {
        val result = Envelope.parse(200, "application/json", "{\"code\":0,\"rows\":[],\"total\":42}")
        assertEquals(42, (result as ApiResult.ListSuccess).total)
        assertEquals(emptyList<JsonArray>(), result.rows)
    }

    @Test
    fun `StopReason 的 wire 值与夹具一致`() {
        assertEquals("reached-total", StopReason.ReachedTotal.wire)
        assertEquals("empty-page", StopReason.EmptyPage.wire)
        assertEquals("max-pages", StopReason.MaxPages.wire)
    }
}
