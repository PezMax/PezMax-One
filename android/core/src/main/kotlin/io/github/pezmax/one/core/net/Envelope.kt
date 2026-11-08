package io.github.pezmax.one.core.net

import io.github.pezmax.one.core.json.PezJson
import io.github.pezmax.one.core.json.arrayOrNull
import io.github.pezmax.one.core.json.booleanOrNull
import io.github.pezmax.one.core.json.intOr
import io.github.pezmax.one.core.json.intOrNull
import io.github.pezmax.one.core.json.objectOrNull
import io.github.pezmax.one.core.json.parseJsonOrNull
import io.github.pezmax.one.core.json.stringOrNull
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject

/**
 * 信封解析的唯一入口。规则见 spec/api/envelope.md §2，判定顺序不能改：
 * 401 先于内容类型判定，rows 先于 code，非 JSON 一律传输错误。
 *
 * 这里只做「形状 → 结论」，不碰 HTTP 库的细节；OkHttp 适配在 [PezMaxClient]。
 */
sealed interface ApiResult {
    data class Success(val data: JsonElement?) : ApiResult

    data class ListSuccess(val rows: List<JsonElement>, val total: Int) : ApiResult

    data class BusinessError(
        val code: Int,
        val message: String,
        val retryable: Boolean,
    ) : ApiResult

    data class TransportError(
        /** HTTP 状态；网络层失败（连不上、超时）用 0。 */
        val status: Int,
        val message: String,
        val retryable: Boolean,
    ) : ApiResult

    data class SessionExpired(val message: String) : ApiResult

    data class State(
        val unchanged: Boolean,
        val hash: String,
        val data: JsonElement?,
    ) : ApiResult

    data class Stream(
        val contentType: String,
        val bytes: ByteArray,
    ) : ApiResult {
        override fun equals(other: Any?): Boolean =
            this === other || (other is Stream && contentType == other.contentType && bytes.contentEquals(other.bytes))

        override fun hashCode(): Int = 31 * contentType.hashCode() + bytes.contentHashCode()
    }
}

enum class MessageClass(val wire: String) {
    ServiceUnavailable("service-unavailable"),
    BadResponse("bad-response"),
}

object Envelope {

    /** 5xx 与网络失败可重试；4xx 不自动重试（429 也交给用户）。 */
    fun isRetryable(status: Int): Boolean = status >= 500 || status == 0

    /**
     * @param status HTTP 状态码
     * @param contentType 原样传响应头，可带 charset
     * @param body UTF-8 解码后的正文；二进制流用 [bytes]
     * @param expectStream true 表示该请求本来就期待文件流（下载 / 预览）
     */
    fun parse(
        status: Int,
        contentType: String,
        body: String?,
        bytes: ByteArray? = null,
        expectStream: Boolean = false,
    ): ApiResult {
        // 1. 401 不看正文：代理层回 HTML 或空响应都算会话失效。
        if (status == 401) {
            val message = body?.let(::parseFailureMessage).orEmpty()
            return ApiResult.SessionExpired(message)
        }

        val payload = bytes ?: body?.toByteArray(Charsets.UTF_8) ?: ByteArray(0)

        // 2. 文件流请求：不解析 JSON；2xx 交给流式处理，其余仍按错误。
        if (expectStream) {
            if (status in 200..299) {
                return ApiResult.Stream(contentType.substringBefore(';').trim(), payload)
            }
            return errorForStatus(status, contentType, body)
        }

        if (!isJson(contentType)) {
            // 3. 非 JSON：传输错误，保留会话。
            return transportForStatus(status, contentType)
        }

        val element = body?.let(::parseJsonOrNull)
        val obj = element as? JsonObject
            ?: return ApiResult.TransportError(
                status = status,
                message = "",
                retryable = isRetryable(status),
            )

        // 4. 哈希缓存信封：先于 code==0 的对象成功，否则 unchanged 的 null data 会被当成功。
        val hash = obj.stringOrNull("hash")
        val unchanged = obj.booleanOrNull("unchanged")
        if (hash != null || unchanged != null) {
            return ApiResult.State(
                unchanged = unchanged ?: false,
                hash = hash.orEmpty(),
                data = obj["data"]?.takeUnless { it is JsonNull },
            )
        }

        // 5. 列表：顶层 rows 数组，缺 total 时按本页条数兜底。
        val rows = obj.arrayOrNull("rows")
        if (rows != null) {
            return ApiResult.ListSuccess(
                rows = rows.toList(),
                total = obj.intOrNull("total") ?: rows.size,
            )
        }

        val code = obj.intOr("code", -1)
        if (status in 200..299 && code == 0) {
            return ApiResult.Success(obj["data"]?.takeUnless { it is JsonNull })
        }

        // 6. 业务错误：HTTP 状态即 code；上游不会在 2xx 里回非 0，但防御性当错误。
        if (status !in 200..299 || code != 0) {
            val message = failureMessage(obj)
            return ApiResult.BusinessError(
                code = if (status in 200..299) code else status,
                message = message,
                retryable = isRetryable(status),
            )
        }

        return ApiResult.TransportError(status, "", false)
    }

    fun isJson(contentType: String): Boolean {
        val media = contentType.substringBefore(';').trim().lowercase()
        return media == "application/json" || media.endsWith("+json")
    }

    private fun failureMessage(obj: JsonObject): String =
        obj.stringOrNull("msg")?.takeIf { it.isNotBlank() }
            ?: obj.stringOrNull("message")?.takeIf { it.isNotBlank() }
            ?: ""

    private fun parseFailureMessage(body: String): String {
        val obj = (parseJsonOrNull(body) as? JsonObject) ?: return ""
        return failureMessage(obj)
    }

    private fun transportForStatus(status: Int, contentType: String): ApiResult = ApiResult.TransportError(
        status = status,
        message = "",
        retryable = isRetryable(status),
    )

    private fun errorForStatus(status: Int, contentType: String, body: String?): ApiResult {
        val obj = body?.let(::parseJsonOrNull) as? JsonObject
        val message = obj?.let(::failureMessage).orEmpty()
        return ApiResult.BusinessError(status, message, isRetryable(status))
    }
}

/** 便于测试与 UI 判断：数组响应用 [rowsOrNull]，对象响应用 [dataObjectOrNull]。 */
fun ApiResult.rowsOrNull(): List<JsonElement>? = (this as? ApiResult.ListSuccess)?.rows
fun ApiResult.dataObjectOrNull(): JsonObject? = (this as? ApiResult.Success)?.data as? JsonObject
fun ApiResult.dataArrayOrNull(): JsonArray? = (this as? ApiResult.Success)?.data as? JsonArray

/** 判断题：这条结果要不要清会话。401 是唯一条件。 */
val ApiResult.clearsSession: Boolean
    get() = this is ApiResult.SessionExpired
