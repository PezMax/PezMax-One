package io.github.pezmax.one.core.cache

import io.github.pezmax.one.core.json.PezJson
import io.github.pezmax.one.core.json.parseJsonOrNull
import io.github.pezmax.one.core.json.stringOrNull
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject

/**
 * 哈希缓存的客户端一侧（spec/fixtures/state-cache/）。协议本身见 envelope.md §5：
 * 这里只做「键怎么算、读出来算不算可用、两个响应抢着落地听谁的」。
 */
data class CacheEntry(
    val serverUrl: String,
    val resource: String,
    val hash: String,
    val payload: JsonElement?,
)

enum class CacheResource(val wire: String) {
    Tree("tree"),
    Rank("rank"),
    Notice("notice"),
}

/** 键是 (服务器, 资源)。服务器地址先归一，末尾斜杠不算换服务器。 */
fun normalizeServerKey(serverUrl: String): String = serverUrl.trim().trimEnd('/').lowercase()

fun cacheHit(serverUrl: String, resource: String, entries: List<CacheEntry>): Boolean {
    val server = normalizeServerKey(serverUrl)
    return entries.any { normalizeServerKey(it.serverUrl) == server && it.resource == resource }
}

fun findCache(serverUrl: String, resource: String, entries: List<CacheEntry>): CacheEntry? {
    val server = normalizeServerKey(serverUrl)
    return entries.lastOrNull { normalizeServerKey(it.serverUrl) == server && it.resource == resource }
}

/**
 * 缓存文件是「一段 JSON」，随时可能损坏。可用条件缺一不可：
 * 能解析、hash 非空、payload 非 null。坏数据当没有缓存，不当空列表。
 */
@Suppress("ReturnCount")
 fun loadCache(raw: String?): CacheEntry? {
    if (raw.isNullOrBlank()) return null
    val obj = (parseJsonOrNull(raw) as? JsonObject) ?: return null
    val hash = obj.stringOrNull("hash")?.takeIf { it.isNotBlank() } ?: return null
    val payload = obj["payload"]?.takeUnless { it is JsonNull } ?: return null
    return CacheEntry(
        serverUrl = obj.stringOrNull("serverUrl").orEmpty(),
        resource = obj.stringOrNull("resource").orEmpty(),
        hash = hash,
        payload = payload,
    )
}

fun encodeCache(entry: CacheEntry): String = buildString {
    append('{')
    append("\"serverUrl\":").append(PezJson.encodeToString(JsonElement.serializer(), kotlinx.serialization.json.JsonPrimitive(entry.serverUrl)))
    append(",\"resource\":").append(PezJson.encodeToString(JsonElement.serializer(), kotlinx.serialization.json.JsonPrimitive(entry.resource)))
    append(",\"hash\":").append(PezJson.encodeToString(JsonElement.serializer(), kotlinx.serialization.json.JsonPrimitive(entry.hash)))
    append(",\"payload\":").append(entry.payload?.toString() ?: "null")
    append('}')
}

/** 本地已知状态；usable=false 表示「有 hash 但 payload 不可用」。 */
data class LocalCacheState(val hash: String = "", val usable: Boolean = false)

data class StateResponse(
    val success: Boolean,
    val unchanged: Boolean = false,
    val hash: String = "",
    val data: JsonElement? = null,
)

sealed interface CacheAction {
    data class Store(val hash: String, val payload: JsonElement?) : CacheAction
    data class Keep(val hash: String) : CacheAction
    data object Refetch : CacheAction
}

/**
 * 哈希缓存的落地决策（state-cache 夹具的 apply）：
 * 失败保留旧值；unchanged 沿用本地；unchanged 但本地不可用则不带 hash 重取；否则整体替换。
 */
fun decideCacheAction(previous: LocalCacheState, response: StateResponse): CacheAction {
    if (!response.success) return CacheAction.Keep(previous.hash)
    if (response.unchanged) {
        return if (previous.usable) CacheAction.Keep(previous.hash.ifBlank { response.hash }) else CacheAction.Refetch
    }
    val hash = response.hash.ifBlank { previous.hash }
    if (hash.isBlank()) return CacheAction.Keep(previous.hash)
    return CacheAction.Store(hash, response.data)
}

/** 两个请求抢着落地：只接受序号不小于已应用序号的响应。 */
fun shouldApplyResponse(appliedSeq: Long, incomingSeq: Long): Boolean = incomingSeq >= appliedSeq
