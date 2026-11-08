package io.github.pezmax.one.core

import io.github.pezmax.one.core.json.PezJson
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import java.io.File

/**
 * 夹具加载器。路径由 Gradle 的 `pezmax.specDir` 注入（sysprop 或环境变量），
 * 缺省按「从 android/core 出发的 ../../spec」兜底，保证 IDE 里直接跑测试也能找到。
 * 规则：不复制夹具、不为让测试变绿改夹具（spec/README.md）。
 */
object SpecFixtures {
    val root: File by lazy {
        val configured = System.getProperty("pezmax.specDir")
            ?: System.getenv("PEZMAX_SPEC_DIR")
            ?: "../../spec"
        File(configured).canonicalFile
    }

    fun array(relativePath: String): JsonArray {
        val file = File(root, relativePath)
        require(file.isFile) { "缺少夹具：${file.absolutePath}" }
        return PezJson.parseToJsonElement(file.readText()) as JsonArray
    }

    fun cases(relativePath: String): List<JsonObject> =
        array(relativePath).map { it as JsonObject }

    fun JsonObject.input(): JsonObject = this["input"] as JsonObject
    fun JsonObject.expect(): JsonObject = this["expect"] as JsonObject
    fun JsonObject.id(): String = (this["id"] as JsonElement).toString().trim('"')
}

fun JsonObject.str(key: String): String = (this[key] as? kotlinx.serialization.json.JsonPrimitive)?.content ?: ""
fun JsonObject.int(key: String): Int = (this[key] as? kotlinx.serialization.json.JsonPrimitive)?.content?.toIntOrNull() ?: 0
fun JsonObject.obj(key: String): JsonObject = this[key] as JsonObject
fun JsonObject.arr(key: String): JsonArray = this[key] as JsonArray
