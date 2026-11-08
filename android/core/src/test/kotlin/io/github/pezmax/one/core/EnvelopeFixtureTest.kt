package io.github.pezmax.one.core

import io.github.pezmax.one.core.SpecFixtures.expect
import io.github.pezmax.one.core.SpecFixtures.input
import io.github.pezmax.one.core.json.booleanOrNull
import io.github.pezmax.one.core.json.intOrNull
import io.github.pezmax.one.core.json.stringOrNull
import io.github.pezmax.one.core.net.ApiResult
import io.github.pezmax.one.core.net.Envelope
import io.github.pezmax.one.core.net.MessageClass
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** spec/fixtures/envelope/cases.json —— 信封解析的判定顺序。 */
class EnvelopeFixtureTest {

    @Test
    fun `所有信封用例与夹具逐条一致`() {
        val cases = SpecFixtures.cases("fixtures/envelope/cases.json")
        assertTrue(cases.size >= 13)
        cases.forEach { case ->
            val input = case.input()
            val expect = case.expect()
            val body = input.requireStringOrNull("body")
            val bytes = input.stringOrNull("bodyBase64")?.let { java.util.Base64.getDecoder().decode(it) }
            val result = Envelope.parse(
                status = input.intOrNull("status") ?: 0,
                contentType = input.stringOrNull("contentType") ?: "",
                body = body,
                bytes = bytes,
                expectStream = input.stringOrNull("requestKind") == "stream",
            )
            assertCase(case.str("id"), result, expect)
        }
    }

    private fun assertCase(id: String, result: ApiResult, expect: JsonObject) {
        val expectedKind = expect.str("kind")
        val actualKind = when (result) {
            is ApiResult.Success -> "success"
            is ApiResult.ListSuccess -> "list"
            is ApiResult.BusinessError -> "error"
            is ApiResult.TransportError -> "transport"
            is ApiResult.SessionExpired -> "session-expired"
            is ApiResult.State -> "state"
            is ApiResult.Stream -> "binary"
        }
        assertEquals("$id kind", expectedKind, actualKind)

        when (result) {
            is ApiResult.Success -> {
                assertEquals("$id code", expect.intOrNull("code") ?: 0, 0)
                assertEquals("$id data", normalize(expect["data"]), result.data ?: JsonNull)
            }

            is ApiResult.ListSuccess -> {
                assertEquals("$id rows", expect["rows"], JsonArray(result.rows))
                assertEquals("$id total", expect.intOrNull("total") ?: 0, result.total)
            }

            is ApiResult.BusinessError -> {
                assertEquals("$id code", expect.intOrNull("code") ?: 0, result.code)
                assertEquals("$id message", expect.stringOrNull("message").orEmpty(), result.message)
                assertEquals("$id retryable", expect.booleanOrNull("retryable") ?: false, result.retryable)
            }

            is ApiResult.TransportError -> {
                assertEquals("$id status", expect.intOrNull("status") ?: 0, result.status)
                val expectedClass = expect.stringOrNull("messageClass")
                val actualClass = if (result.status >= 500) MessageClass.ServiceUnavailable else MessageClass.BadResponse
                assertEquals("$id messageClass", expectedClass, actualClass.wire)
                assertEquals("$id retryable", expect.booleanOrNull("retryable") ?: false, result.retryable)
            }

            is ApiResult.SessionExpired -> {
                assertEquals("$id code", 401, expect.intOrNull("code") ?: 0)
                assertEquals("$id message", expect.stringOrNull("message").orEmpty(), result.message)
                assertEquals("$id session", "clear", expect.stringOrNull("session"))
                assertTrue("$id rememberRoute", expect.booleanOrNull("rememberRoute") == true)
            }

            is ApiResult.State -> {
                assertEquals("$id unchanged", expect.booleanOrNull("unchanged") ?: false, result.unchanged)
                assertEquals("$id hash", expect.stringOrNull("hash").orEmpty(), result.hash)
                assertEquals("$id data", normalize(expect["data"]), result.data ?: JsonNull)
                assertEquals("$id keepLocal", expect.booleanOrNull("keepLocalPayload") ?: false, result.unchanged)
            }

            is ApiResult.Stream -> {
                assertEquals("$id contentType", expect.stringOrNull("contentType"), result.contentType)
                assertTrue("$id notJson", expect.booleanOrNull("notJson") == true)
            }
        }

        // session 断言对所有 kind 都必须成立；"clear" 只可能来自 401。
        val expectedSession = expect.stringOrNull("session") ?: "keep"
        assertEquals("$id session", expectedSession, if (result is ApiResult.SessionExpired) "clear" else "keep")
    }

    private fun normalize(element: kotlinx.serialization.json.JsonElement?): kotlinx.serialization.json.JsonElement =
        element ?: JsonNull

    private fun JsonObject.requireStringOrNull(key: String): String? = stringOrNull(key)

    @Test
    fun `非 JSON 的 200 是传输错误而不是成功`() {
        val result = Envelope.parse(200, "text/plain; charset=utf-8", "hello")
        assertTrue(result is ApiResult.TransportError)
        assertNull((result as ApiResult.TransportError).message.ifEmpty { null })
    }

    @Test
    fun `code 为非 0 的 200 不得被当成成功`() {
        val result = Envelope.parse(200, "application/json", "{\"code\":7,\"msg\":\"x\",\"data\":null}")
        assertTrue(result is ApiResult.BusinessError)
        assertEquals(7, (result as ApiResult.BusinessError).code)
    }

    @Test
    fun `数字字符串的 code 也能读`() {
        val result = Envelope.parse(200, "application/json", "{\"code\":\"0\",\"data\":true}")
        assertTrue(result is ApiResult.Success)
        assertEquals(JsonPrimitive(true), (result as ApiResult.Success).data)
    }
}
