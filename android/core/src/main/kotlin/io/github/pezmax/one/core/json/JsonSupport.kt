package io.github.pezmax.one.core.json

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.longOrNull

/**
 * 零反射的 JSON 读法。三端核心都按「显式读键」解析信封与 payload，
 * 不做代码生成 / 反射映射：契约里的字段名就是唯一真相，缺字段按缺省处理，
 * 不因为多一个字段就整体解析失败（上游随时会加字段）。
 */
@OptIn(kotlinx.serialization.ExperimentalSerializationApi::class)
val PezJson: Json = Json {
    ignoreUnknownKeys = true
    isLenient = false
    explicitNulls = false
}

fun parseJsonOrNull(text: String): JsonElement? =
    try {
        PezJson.parseToJsonElement(text)
    } catch (_: Exception) {
        null
    }

fun JsonElement?.asObjectOrNull(): JsonObject? = this as? JsonObject
fun JsonElement?.asArrayOrNull(): JsonArray? = this as? JsonArray

fun JsonObject.stringOrNull(key: String): String? {
    val element = this[key] ?: return null
    if (element is JsonNull) return null
    return (element as? JsonPrimitive)?.contentOrNull
}

fun JsonObject.requireString(key: String, fallback: String = ""): String = stringOrNull(key) ?: fallback

fun JsonObject.intOrNull(key: String): Int? = (this[key] as? JsonPrimitive)?.intOrNull
fun JsonObject.longOrNull(key: String): Long? = (this[key] as? JsonPrimitive)?.longOrNull
fun JsonObject.booleanOrNull(key: String): Boolean? = (this[key] as? JsonPrimitive)?.booleanOrNull

fun JsonObject.intOr(key: String, fallback: Int = 0): Int = intOrNull(key) ?: fallback
fun JsonObject.longOr(key: String, fallback: Long = 0L): Long = longOrNull(key) ?: fallback
fun JsonObject.boolOr(key: String, fallback: Boolean = false): Boolean = booleanOrNull(key) ?: fallback

fun JsonObject.objectOrNull(key: String): JsonObject? = (this[key] as? JsonObject)
fun JsonObject.arrayOrNull(key: String): JsonArray? = (this[key] as? JsonArray)

fun JsonObject.objects(key: String): List<JsonObject> =
    arrayOrNull(key)?.mapNotNull { it as? JsonObject } ?: emptyList()

fun JsonObject.strings(key: String): List<String> =
    arrayOrNull(key)?.mapNotNull { (it as? JsonPrimitive)?.contentOrNull } ?: emptyList()

/** 数字或数字字符串都接受：书签的 id 两种形态都在上游出现过。 */
fun JsonObject.idLike(key: String): Long? {
    val primitive = this[key] as? JsonPrimitive ?: return null
    return primitive.longOrNull ?: primitive.contentOrNull?.trim()?.toLongOrNull()
}

fun primitive(text: String): JsonPrimitive = JsonPrimitive(text)
