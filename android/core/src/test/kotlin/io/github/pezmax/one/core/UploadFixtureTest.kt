package io.github.pezmax.one.core

import io.github.pezmax.one.core.SpecFixtures.expect
import io.github.pezmax.one.core.SpecFixtures.input
import io.github.pezmax.one.core.json.booleanOrNull
import io.github.pezmax.one.core.json.intOrNull
import io.github.pezmax.one.core.json.longOrNull
import io.github.pezmax.one.core.json.stringOrNull
import io.github.pezmax.one.core.upload.UploadValidation
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.Base64

/** spec/fixtures/upload/cases.json —— 资料 / 电子书 / 批次 / UTF-8 四套判定。 */
class UploadFixtureTest {

    private val cases = SpecFixtures.cases("fixtures/upload/cases.json")

    @Test
    fun `file 用例`() {
        assertEquals(38, cases.size)
        cases.filter { it.input().str("kind") == "file" }.forEach { case ->
            val input = case.input()
            val check = UploadValidation.checkFile(
                fileName = input.str("fileName"),
                sizeBytes = input.longOrNull("sizeBytes") ?: 0L,
                maxUploadMb = input.intOrNull("maxUploadMb") ?: 200,
            )
            val expect = case.expect()
            assertEquals(case.str("id") + " accept", expect.booleanOrNull("accept") ?: false, check.accept)
            expect.stringOrNull("reason")?.let { assertEquals(case.str("id") + " reason", it, check.reason) }
            if (expect.containsKey("convertsToPdf")) {
                assertEquals(case.str("id") + " converts", expect.booleanOrNull("convertsToPdf") ?: false, check.convertsToPdf)
            }
            if (expect.containsKey("needsUtf8Check")) {
                assertEquals(case.str("id") + " utf8", expect.booleanOrNull("needsUtf8Check") ?: false, check.needsUtf8Check)
            }
            if (expect.containsKey("warnsNotEbook")) {
                assertEquals(case.str("id") + " warns", expect.booleanOrNull("warnsNotEbook") ?: false, check.warnsNotEbook)
            }
        }
    }

    @Test
    fun `meta 用例`() {
        cases.filter { it.input().str("kind") == "meta" }.forEach { case ->
            val input = case.input()
            val fileName = buildName(input)
            val result = UploadValidation.validateFileMeta(
                fileName = fileName,
                fileSubject = input.str("fileSubject"),
                fileSchool = input.str("fileSchool"),
                fileYear = input.intOrNull("fileYear") ?: 0,
                fileType = input.intOrNull("fileType") ?: 0,
            )
            val expect = case.expect()
            assertEquals(case.str("id") + " accept", expect.booleanOrNull("accept") ?: false, result.accept)
            expect.stringOrNull("reason")?.let { assertEquals(case.str("id") + " reason", it, result.reason) }
            if (expect.containsKey("warnsNotEbook")) {
                assertEquals(case.str("id") + " warns", expect.booleanOrNull("warnsNotEbook") ?: false, result.warnsNotEbook)
            }
        }
    }

    @Test
    fun `ebook 用例`() {
        cases.filter { it.input().str("kind") == "ebook" }.forEach { case ->
            val input = case.input()
            val result = UploadValidation.validateEbookMeta(
                fileName = input.str("fileName"),
                ebookName = input.str("ebookName"),
                ebookSubject = input.str("ebookSubject"),
                ebookType = input.intOrNull("ebookType") ?: 0,
                author = input["authorRunes"]?.let { (it as JsonPrimitive).content.toInt() }
                    ?.let { n -> input.str("author").repeat(n) } ?: input.str("author"),
            )
            val expect = case.expect()
            assertEquals(case.str("id") + " accept", expect.booleanOrNull("accept") ?: false, result.accept)
            expect.stringOrNull("reason")?.let { assertEquals(case.str("id") + " reason", it, result.reason) }
        }
    }

    @Test
    fun `batch 逐个文件判定`() {
        cases.filter { it.input().str("kind") == "batch" }.forEach { case ->
            val input = case.input()
            val files = (input["files"] as JsonArray).map { it as JsonObject }.map {
                UploadValidation.BatchItem(it.str("fileName"), it.longOrNull("sizeBytes") ?: 0L)
            }
            val result = UploadValidation.validateBatch(files, input.intOrNull("maxUploadMb") ?: 200)
            val expected = (case.expect()["results"] as JsonArray).map { it as JsonObject }
            assertEquals(case.str("id"), expected.size, result.items.size)
            expected.forEachIndexed { index, row ->
                assertEquals(case.str("id") + " name #$index", row.str("name"), result.items[index].first)
                assertEquals(case.str("id") + " accept #$index", row.booleanOrNull("accept") ?: false, result.items[index].second.accept)
            }
        }
    }

    @Test
    fun `encoding 用例`() {
        cases.filter { it.input().str("kind") == "encoding" }.forEach { case ->
            val bytes = Base64.getDecoder().decode(case.input().str("bytesBase64"))
            assertEquals(case.str("id"), case.expect().booleanOrNull("utf8") ?: false, UploadValidation.isUtf8(bytes))
        }
    }

    @Test
    fun `字节与 rune 分别计数`() {
        assertEquals(64, "数".repeat(21).toByteArray(Charsets.UTF_8).size + 1)
        assertTrue(UploadValidation.validateFileMeta("题".repeat(251) + ".pdf", "数学", "", 2025, 1).accept)
        val tooLong = UploadValidation.validateFileMeta("题".repeat(252) + ".pdf", "数学", "", 2025, 1)
        assertEquals(UploadValidation.ERR_FILE_NAME, tooLong.reason)
    }

    private fun buildName(input: JsonObject): String {
        val runes = input["fileNameRunes"]?.let { (it as JsonPrimitive).content.toInt() } ?: return input.str("fileName")
        val ext = input.str("fileNameExt")
        val base = input.str("fileName").ifEmpty { "x" }
        val prefixLength = (runes - ext.length).coerceAtLeast(0)
        val prefix = base.repeat(prefixLength / base.length + 1).take(prefixLength)
        return prefix + ext
    }
}
