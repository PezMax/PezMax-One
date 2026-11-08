package io.github.pezmax.one.data

import android.content.Context
import io.github.pezmax.one.core.json.PezJson
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import java.io.File

/**
 * 本地 JSON 存储：缓存、下载记录、通知已读都写在这里。
 *
 * 目录分工（ARCHITECTURE.md 核心原则 5）：
 * - `filesDir/`：不可清除的记录（下载记录、通知已读）
 * - `cacheDir/`：可清除的缓存（树 / 排行 / 封面 / PDF 页）
 *
 * 所有读都容错：文件损坏当空，不让坏缓存挡住界面（state-cache / notification 夹具的 load 语义）。
 */
class LocalStore(private val context: Context) {

    val downloadsFile: File get() = File(context.filesDir, "downloads.json")
    val noticeFile: File get() = File(context.filesDir, "notice_reads.json")
    val treeCacheFile: File get() = File(cacheDir(), "tree.json")
    val rankCacheFile: File get() = File(cacheDir(), "rank.json")

    fun cacheDir(): File = File(context.cacheDir, "pezmax").apply { mkdirs() }

    fun pdfCacheDir(fileId: Long): File = File(cacheDir(), "pdf/$fileId").apply { mkdirs() }

    fun downloadDir(): File = File(context.filesDir, "Downloads").apply { mkdirs() }

    fun readJson(file: File): JsonObject? = runCatching {
        if (!file.isFile) return null
        PezJson.parseToJsonElement(file.readText()) as? JsonObject
    }.getOrNull()

    fun writeJson(file: File, element: JsonElement) {
        runCatching {
            file.parentFile?.mkdirs()
            val tmp = File(file.parentFile, file.name + ".tmp")
            tmp.writeText(element.toString())
            if (!tmp.renameTo(file)) {
                file.writeText(element.toString())
                tmp.delete()
            }
        }
    }

    fun clearCache() {
        cacheDir().listFiles()?.forEach { it.deleteRecursively() }
    }

    fun cacheSizeBytes(): Long = cacheDir().walkTopDown().filter { it.isFile }.sumOf { it.length() }
}
