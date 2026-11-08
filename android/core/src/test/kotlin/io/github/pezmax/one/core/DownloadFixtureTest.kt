package io.github.pezmax.one.core

import io.github.pezmax.one.core.SpecFixtures.expect
import io.github.pezmax.one.core.SpecFixtures.input
import io.github.pezmax.one.core.download.DownloadPlanner
import io.github.pezmax.one.core.download.LocalDownloadRecord
import io.github.pezmax.one.core.download.LocalDownloadStatus
import io.github.pezmax.one.core.download.PartState
import io.github.pezmax.one.core.download.ResponseHeaders
import io.github.pezmax.one.core.download.ServerDownload
import io.github.pezmax.one.core.download.mergeDownloads
import io.github.pezmax.one.core.json.booleanOrNull
import io.github.pezmax.one.core.json.intOrNull
import io.github.pezmax.one.core.json.longOrNull
import io.github.pezmax.one.core.json.stringOrNull
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** spec/fixtures/download/cases.json —— 续传决策与「我的下载」合并。 */
class DownloadFixtureTest {

    @Test
    fun `before 决策`() {
        casesOfKind("before").forEach { case ->
            val part = case.input()["part"] as? JsonObject
            val decision = DownloadPlanner.before(
                PartState(
                    size = part?.longOrNull("size") ?: 0L,
                    etag = part?.stringOrNull("etag"),
                ),
            )
            val expect = case.expect()
            assertEquals(case.str("id") + " action", expect.stringOrNull("action"), decision.action.wire)
            assertEquals(case.str("id") + " offset", expect.longOrNull("offset") ?: 0L, decision.offset)
            assertEquals(case.str("id") + " discard", expect.booleanOrNull("discardPart") ?: false, decision.discardPart)
            if (expect.containsKey("range")) {
                assertEquals(case.str("id") + " range", expect.stringOrNull("range"), decision.range)
            }
            if (expect.containsKey("ifRange")) {
                assertEquals(case.str("id") + " ifRange", expect.stringOrNull("ifRange"), decision.ifRange)
            }
        }
    }

    @Test
    fun `response 决策`() {
        casesOfKind("response").forEach { case ->
            val part = case.input()["part"] as? JsonObject
            val response = case.input().obj("response")
            val decision = DownloadPlanner.onResponse(
                part = PartState(
                    size = part?.longOrNull("size") ?: 0L,
                    etag = part?.stringOrNull("etag"),
                ),
                headers = ResponseHeaders(
                    status = response.intOrNull("status") ?: 0,
                    etag = response.stringOrNull("etag"),
                    contentLength = response.longOrNull("contentLength") ?: 0L,
                    contentRange = response.stringOrNull("contentRange"),
                ),
            )
            val expect = case.expect()
            assertEquals(case.str("id") + " action", expect.stringOrNull("action"), decision.action.wire)
            assertEquals(case.str("id") + " offset", expect.longOrNull("offset") ?: 0L, decision.offset)
            assertEquals(case.str("id") + " total", expect.longOrNull("total") ?: 0L, decision.total)
            assertEquals(case.str("id") + " discard", expect.booleanOrNull("discardPart") ?: false, decision.discardPart)
        }
    }

    @Test
    fun `after 决策`() {
        casesOfKind("after").forEach { case ->
            val decision = DownloadPlanner.after(
                written = case.input().longOrNull("written") ?: 0L,
                total = case.input().longOrNull("total") ?: 0L,
            )
            val expect = case.expect()
            assertEquals(case.str("id") + " action", expect.stringOrNull("action"), decision.action.wire)
            assertEquals(case.str("id") + " total", expect.longOrNull("total") ?: 0L, decision.total)
            assertEquals(case.str("id") + " discard", expect.booleanOrNull("discardPart") ?: false, decision.discardPart)
        }
    }

    @Test
    fun `我的下载合并`() {
        casesOfKind("merge").forEach { case ->
            val server = (case.input()["server"] as JsonArray).map { it as JsonObject }.map {
                ServerDownload(
                    fileId = it.longOrNull("fileId") ?: 0L,
                    fileName = it.stringOrNull("fileName").orEmpty(),
                    firstDownloadTime = it.stringOrNull("firstDownloadTime").orEmpty(),
                )
            }
            val records = (case.input()["records"] as JsonArray).map { it as JsonObject }.map {
                LocalDownloadRecord(
                    kind = it.stringOrNull("kind") ?: "file",
                    id = it.longOrNull("id") ?: 0L,
                    fileName = it.stringOrNull("fileName").orEmpty(),
                    localPath = it.stringOrNull("localPath").orEmpty(),
                    size = it.longOrNull("size") ?: 0L,
                    bytesReceived = it.longOrNull("bytesReceived") ?: 0L,
                    status = LocalDownloadStatus.fromWire(it.stringOrNull("status")) ?: LocalDownloadStatus.Part,
                    updatedAt = it.stringOrNull("updatedAt").orEmpty(),
                )
            }
            val merged = mergeDownloads(server, records)
            val expect = case.expect()
            assertEquals(
                case.str("id") + " order",
                (expect["order"] as JsonArray).map { (it as JsonPrimitive).content.toLong() },
                merged.map { it.fileId },
            )
            val states = expect["states"] as JsonArray
            states.forEachIndexed { index, element ->
                val state = element as JsonObject
                val actual = merged[index]
                assertEquals(case.str("id") + " fileId #$index", state.longOrNull("fileId"), actual.fileId)
                assertEquals(case.str("id") + " statusText #$index", state.stringOrNull("statusText"), actual.displayStatus.text)
                assertEquals(
                    case.str("id") + " hasLocalPath #$index",
                    state.booleanOrNull("hasLocalPath") ?: false,
                    !actual.localPath.isNullOrBlank(),
                )
            }
            expect.stringOrNull("localPathOf")?.let { expectedPath ->
                assertEquals(case.str("id") + " localPath", expectedPath, merged[0].localPath)
            }
        }
    }

    @Test
    fun `Content-Range 解析与非法值`() {
        assertEquals(Triple(1024L, 4095L, 4096L), DownloadPlanner.parseContentRange("bytes 1024-4095/4096"))
        assertNull(DownloadPlanner.parseContentRange("bytes 0-1/*"))
        assertNull(DownloadPlanner.parseContentRange(null))
        assertTrue(DownloadPlanner.isStrongEtag("\"datum-12-4096\""))
        assertTrue(!DownloadPlanner.isStrongEtag("W/\"datum-12-4096\""))
    }

    private fun casesOfKind(kind: String): List<JsonObject> =
        SpecFixtures.cases("fixtures/download/cases.json").filter { it.input().str("kind") == kind }

    private fun JsonObject.obj(key: String): JsonObject = this[key] as JsonObject
}
