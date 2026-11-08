package io.github.pezmax.one.core

import io.github.pezmax.one.core.SpecFixtures.expect
import io.github.pezmax.one.core.SpecFixtures.input
import io.github.pezmax.one.core.json.arrayOrNull
import io.github.pezmax.one.core.json.intOrNull
import io.github.pezmax.one.core.json.objectOrNull
import io.github.pezmax.one.core.json.stringOrNull
import io.github.pezmax.one.core.reading.IntRange2
import io.github.pezmax.one.core.reading.PageSize
import io.github.pezmax.one.core.reading.PdfGeometry
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Test
import kotlin.math.abs

/** spec/fixtures/reading/cases.json —— 连续视图几何、渲染窗口与顺序、缩放、缓存键、网格、淘汰。 */
class ReadingFixtureTest {

    private val cases = SpecFixtures.cases("fixtures/reading/cases.json")

    @Test
    fun `连续视图几何`() {
        cases.filter { it.input().str("kind") == "continuous" }.forEach { case ->
            val input = case.input()
            val pages = input.arr("pages").map { row ->
                val obj = row as JsonObject
                PageSize(obj.float("width"), obj.float("height"))
            }
            val displayWidth = input.float("displayWidth")
            val gap = input.float("gap")
            val tops = PdfGeometry.tops(pages, displayWidth, gap)
            val contentHeight = PdfGeometry.contentHeight(pages, displayWidth, gap)

            input.arr("ops").forEach { opElement ->
                val op = opElement as JsonObject
                val expect = case.expect()
                when (op.str("op")) {
                    "tops" -> assertFloats(case.str("id") + " tops", expect.arr("tops").map { (it as JsonPrimitive).content.toFloat() }, tops)
                    "contentHeight" -> assertEquals(case.str("id") + " contentHeight", expect.float("contentHeight"), contentHeight, 1e-3f)
                    "visible" -> {
                        val visible = PdfGeometry.visible(
                            pages, displayWidth, gap,
                            scrollOffset = op.float("scrollOffset"),
                            viewportHeight = op.float("viewportHeight"),
                        )
                        assertRange(case.str("id") + " visible", expect.arr("visible"), visible)
                    }
                    "pageAt" -> assertEquals(
                        case.str("id") + " pageAt",
                        expect.intOrNull("pageAt"),
                        PdfGeometry.pageAt(pages, displayWidth, gap, op.float("y")),
                    )
                    "offsetOf" -> {
                        val page = op.intOrNull("page") ?: 0
                        assertEquals(case.str("id") + " offsetOf", expect.float("offsetOf"), tops[page], 1e-3f)
                    }
                    "current" -> assertEquals(
                        case.str("id") + " current",
                        expect.intOrNull("current"),
                        PdfGeometry.currentPage(
                            pages, displayWidth, gap,
                            scrollOffset = op.float("scrollOffset"),
                            viewportHeight = op.float("viewportHeight"),
                        ),
                    )
                }
            }
        }
    }

    @Test
    fun `overscan 用例`() {
        cases.filter { it.input().str("kind") == "overscan" }.forEach { case ->
            val input = case.input()
            val visible = input.rangeOrEmpty("visible")
            val result = PdfGeometry.renderWindow(
                visible = visible,
                pageCount = input.intOrNull("pageCount") ?: 0,
                overscan = input.intOrNull("overscan") ?: PdfGeometry.OVERSCAN,
            )
            assertRange(case.str("id"), case.expect().arr("render"), result)
        }
    }

    @Test
    fun `renderOrder 用例`() {
        cases.filter { it.input().str("kind") == "renderOrder" }.forEach { case ->
            val input = case.input()
            val order = PdfGeometry.renderOrder(
                visible = input.range("visible"),
                render = input.range("render"),
                cached = input.intArray("cached").toSet(),
            )
            assertEquals(
                case.str("id"),
                case.expect().arr("order").map { (it as JsonPrimitive).content.toInt() },
                order,
            )
        }
    }

    @Test
    fun `zoom 用例`() {
        cases.filter { it.input().str("kind") == "zoom" }.forEach { case ->
            val input = case.input()
            val zoom = when (input.str("op")) {
                "clamp" -> PdfGeometry.clampZoom(input.float("zoom"))
                "step" -> PdfGeometry.stepZoom(input.float("zoom"), input.intOrNull("direction") ?: 0)
                "fit" -> PdfGeometry.fitZoom(input.float("availableWidth"), input.float("pageWidth"))
                else -> error("未知 zoom op")
            }
            assertEquals(case.str("id"), case.expect().float("zoom"), zoom, 1e-4f)
        }
    }

    @Test
    fun `cacheKey 用例`() {
        cases.filter { it.input().str("kind") == "cacheKey" }.forEach { case ->
            val key = PdfGeometry.cacheKey(
                pageIndex = case.input().intOrNull("pageIndex") ?: 0,
                scale = case.input().float("scale"),
            )
            assertEquals(case.str("id"), case.expect().str("key"), key)
        }
    }

    @Test
    fun `grid 用例`() {
        cases.filter { it.input().str("kind") == "grid" }.forEach { case ->
            val input = case.input()
            val expect = case.expect()
            when (input.str("op")) {
                "columns" -> assertEquals(
                    case.str("id"),
                    expect.intOrNull("columns"),
                    PdfGeometry.gridColumns(input.float("availableWidth")),
                )
                "tileWidth" -> assertEquals(
                    case.str("id"),
                    expect.float("tileWidth"),
                    PdfGeometry.gridTileWidth(input.float("availableWidth"), input.intOrNull("columns") ?: 1),
                    1e-3f,
                )
                "rows" -> assertEquals(
                    case.str("id"),
                    expect.intOrNull("rows"),
                    PdfGeometry.gridRows(input.intOrNull("pageCount") ?: 0, input.intOrNull("columns") ?: 1),
                )
            }
        }
    }

    @Test
    fun `evict 用例`() {
        cases.filter { it.input().str("kind") == "evict" }.forEach { case ->
            val input = case.input()
            val released = PdfGeometry.evictOrder(
                cached = input.intArray("cached").toSet(),
                keep = input.range("keep"),
            )
            assertEquals(
                case.str("id"),
                case.expect().arr("release").map { (it as JsonPrimitive).content.toInt() },
                released,
            )
        }
    }

    @Test
    fun `尺寸缺失按 A4 兜底`() {
        assertEquals(842f, PdfGeometry.pageHeight(PageSize(0f, 0f), 595f), 1e-3f)
        assertEquals(421f, PdfGeometry.pageHeight(PageSize(595f, 0f), 297.5f), 1e-3f)
    }

    private fun assertRange(id: String, expected: JsonArray, actual: IntRange2) {
        val values = expected.map { (it as JsonPrimitive).content.toInt() }
        val actualValues = if (actual.isEmpty) emptyList() else listOf(actual.first, actual.last)
        assertEquals(id, values, actualValues)
    }

    private fun assertFloats(id: String, expected: List<Float>, actual: List<Float>) {
        assertEquals("$id size", expected.size, actual.size)
        expected.forEachIndexed { index, value -> assertEquals("$id #$index", value, actual[index], 1e-3f) }
    }

    private fun JsonObject.str(key: String): String = stringOrNull(key).orEmpty()
    private fun JsonObject.float(key: String): Float = (this[key] as? JsonPrimitive)?.content?.toFloat() ?: 0f
    private fun JsonObject.arr(key: String): JsonArray = this[key] as JsonArray
    private fun JsonObject.range(key: String): IntRange2 {
        val array = arr(key)
        if (array.isEmpty()) return IntRange2.Empty
        return IntRange2(
            (array[0] as JsonPrimitive).content.toInt(),
            (array[1] as JsonPrimitive).content.toInt(),
        )
    }

    private fun JsonObject.rangeOrEmpty(key: String): IntRange2 {
        val element = this[key] ?: return IntRange2.Empty
        if (element is JsonNull) return IntRange2.Empty
        val array = element as JsonArray
        if (array.isEmpty()) return IntRange2.Empty
        return IntRange2(
            (array[0] as JsonPrimitive).content.toInt(),
            (array[1] as JsonPrimitive).content.toInt(),
        )
    }

    private fun JsonObject.intArray(key: String): List<Int> {
        val element = this[key] ?: return emptyList()
        if (element is JsonNull) return emptyList()
        return (element as JsonArray).map { (it as JsonPrimitive).content.toInt() }
    }

    private fun JsonObject.intOrNull(key: String): Int? = (this[key] as? JsonPrimitive)?.content?.toIntOrNull()
}
