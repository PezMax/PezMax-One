package io.github.pezmax.one.core

import io.github.pezmax.one.core.SpecFixtures.expect
import io.github.pezmax.one.core.SpecFixtures.input
import io.github.pezmax.one.core.json.arrayOrNull
import io.github.pezmax.one.core.json.longOrNull
import io.github.pezmax.one.core.json.objectOrNull
import io.github.pezmax.one.core.json.stringOrNull
import io.github.pezmax.one.core.tree.FileTree
import io.github.pezmax.one.core.tree.RowKind
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** spec/fixtures/file-tree/cases.json —— 行顺序、叶子模型、面包屑、兜底标签。 */
class FileTreeFixtureTest {

    private val cases = SpecFixtures.cases("fixtures/file-tree/cases.json")

    @Test
    fun `rows 用例`() {
        cases.filter { it.input().str("kind") == "rows" }.forEach { case ->
            val tree = FileTree.parse(case.input()["tree"] as JsonArray)
            val path = (case.input()["path"] as JsonArray).map { (it as JsonPrimitive).content }
            val result = FileTree.rowsAt(tree, path)
            val order = result.rows.map {
                (if (it.kind == RowKind.Folder) "folder:" else "file:") + it.label
            }
            val expected = (case.expect()["order"] as JsonArray).map { (it as JsonPrimitive).content }
            assertEquals(case.str("id"), expected, order)
            val notFound = case.expect()["notFound"]?.let { (it as JsonPrimitive).content.toBoolean() } ?: false
            assertEquals(case.str("id") + " notFound", notFound, result.notFound)
        }
    }

    @Test
    fun `leaf 用例`() {
        cases.filter { it.input().str("kind") == "leaf" }.forEach { case ->
            val node = FileTree.parse(JsonArray(listOf(case.input().obj("node")))).single()
            val row = FileTree.leafRow(node)
            val expect = case.expect()
            assertEquals(case.str("id") + " fileId", expect.longOrNull("fileId"), row.fileId)
            assertEquals(case.str("id") + " title", expect.stringOrNull("title"), row.label)
            assertEquals(case.str("id") + " kind", expect.stringOrNull("kindOfRow"), "file")
            expect.stringOrNull("fileFormat")?.let {
                assertEquals(case.str("id") + " format", it, node.payload?.stringOrNull("fileFormat"))
            }
            expect.longOrNull("fileYear")?.let {
                assertEquals(case.str("id") + " year", it, node.payload?.longOrNull("fileYear"))
            }
            expect.stringOrNull("fileSubject")?.let {
                assertEquals(case.str("id") + " subject", it, node.payload?.stringOrNull("fileSubject"))
            }
        }
    }

    @Test
    fun `breadcrumb 用例`() {
        cases.filter { it.input().str("kind") == "breadcrumb" }.forEach { case ->
            val path = (case.input()["path"] as JsonArray).map { (it as JsonPrimitive).content }
            val expected = (case.expect()["labels"] as JsonArray).map { (it as JsonPrimitive).content }
            assertEquals(case.str("id"), expected, FileTree.breadcrumb(path))
        }
    }

    @Test
    fun `fallback 用例`() {
        cases.filter { it.input().str("kind") == "fallback" }.forEach { case ->
            val node = FileTree.parse(JsonArray(listOf(case.input().obj("node")))).single()
            assertEquals(case.str("id"), case.expect().stringOrNull("label"), FileTree.displayLabel(node))
        }
    }

    @Test
    fun `文件夹 id 不可持久化、fileId 是唯一数字来源`() {
        val tree = FileTree.parse(
            JsonArray(
                listOf(
                    JsonObject(
                        mapOf(
                            "id" to JsonPrimitive("file-12"),
                            "fileId" to JsonPrimitive(12),
                            "label" to JsonPrimitive("期中.pdf"),
                            "type" to JsonPrimitive("file"),
                            "children" to JsonArray(emptyList()),
                        ),
                    ),
                ),
            ),
        )
        val row = FileTree.leafRow(tree.single())
        assertEquals(12L, row.fileId)
        assertTrue(row.node.id.startsWith("file-"))
    }

    private fun JsonObject.obj(key: String): JsonObject = objectOrNull(key) ?: error("missing $key")
}
