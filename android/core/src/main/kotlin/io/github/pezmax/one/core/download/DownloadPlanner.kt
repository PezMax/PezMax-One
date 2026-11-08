package io.github.pezmax.one.core.download

import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull

/**
 * 断点续传决策表（spec/fixtures/download/cases.json）。
 * 这里只「算」动作，不碰文件与网络：字节搬运由各端 I/O 层照动作执行。
 */
enum class DownloadAction(val wire: String) {
    Start("start"),
    Resume("resume"),
    Restart("restart"),
    Fail("fail"),
    Complete("complete"),
    Continue("continue"),
    SessionExpired("session-expired"),
}

/** 本地半成品：`<最终文件名>.part` + 同目录 `.part.json`（只存 etag）。 */
data class PartState(val size: Long = 0, val etag: String? = null) {
    val hasPart: Boolean get() = size > 0
}

data class BeforeDecision(
    val action: DownloadAction,
    val offset: Long,
    val discardPart: Boolean,
    val range: String? = null,
    val ifRange: String? = null,
)

data class ResponseHeaders(
    val status: Int,
    val etag: String? = null,
    val contentLength: Long = 0,
    val contentRange: String? = null,
)

data class ResponseDecision(
    val action: DownloadAction,
    val offset: Long = 0,
    val total: Long = 0,
    val discardPart: Boolean = false,
)

data class AfterDecision(
    val action: DownloadAction,
    val total: Long = 0,
    val discardPart: Boolean = false,
)

object DownloadPlanner {

    /** 强 ETag 才能进 If-Range；弱校验器服务端不接受。 */
    fun isStrongEtag(etag: String?): Boolean =
        !etag.isNullOrBlank() && !etag.startsWith("W/")

    fun before(part: PartState): BeforeDecision {
        if (!part.hasPart) return BeforeDecision(DownloadAction.Start, 0, discardPart = false)
        if (!isStrongEtag(part.etag)) return BeforeDecision(DownloadAction.Restart, 0, discardPart = true)
        return BeforeDecision(
            action = DownloadAction.Resume,
            offset = part.size,
            discardPart = false,
            range = "bytes=${part.size}-",
            ifRange = part.etag,
        )
    }

    fun onResponse(part: PartState, headers: ResponseHeaders): ResponseDecision = when (headers.status) {
        401 -> ResponseDecision(DownloadAction.SessionExpired)
        200 -> if (part.hasPart) {
            // If-Range 不匹配 / 被忽略：服务端回全量，必须截断重写。
            ResponseDecision(DownloadAction.Restart, 0, headers.contentLength, discardPart = true)
        } else {
            ResponseDecision(DownloadAction.Start, 0, headers.contentLength, discardPart = false)
        }
        206 -> onPartialContent(part, headers)
        416 -> if (part.hasPart) {
            // 半成品其实已经完整、只是没改名：删掉重下，别在 416 上死循环。
            ResponseDecision(DownloadAction.Restart, 0, 0, discardPart = true)
        } else {
            ResponseDecision(DownloadAction.Fail)
        }
        else -> ResponseDecision(DownloadAction.Fail)
    }

    private fun onPartialContent(part: PartState, headers: ResponseHeaders): ResponseDecision {
        if (!part.hasPart) return ResponseDecision(DownloadAction.Fail)
        if (!isStrongEtag(headers.etag) || headers.etag != part.etag) {
            return ResponseDecision(DownloadAction.Restart, 0, 0, discardPart = true)
        }
        val range = parseContentRange(headers.contentRange)
            ?: return ResponseDecision(DownloadAction.Restart, 0, 0, discardPart = true)
        val (start, end, total) = range
        val tailValid = start == part.size && end == total - 1
        if (!tailValid || total <= 0) {
            return ResponseDecision(DownloadAction.Restart, 0, 0, discardPart = true)
        }
        return ResponseDecision(DownloadAction.Resume, start, total, discardPart = false)
    }

    /** `Content-Range: bytes 1024-4095/4096` → (1024, 4095, 4096)；`*` 总长按缺失处理。 */
    fun parseContentRange(value: String?): Triple<Long, Long, Long>? {
        val text = value?.trim().orEmpty()
        if (!text.startsWith("bytes ")) return null
        val body = text.removePrefix("bytes ")
        val slash = body.indexOf('/')
        if (slash <= 0) return null
        val rangePart = body.substring(0, slash)
        val totalPart = body.substring(slash + 1)
        val total = totalPart.toLongOrNull() ?: return null
        val dash = rangePart.indexOf('-')
        if (dash <= 0) return null
        val start = rangePart.substring(0, dash).toLongOrNull() ?: return null
        val end = rangePart.substring(dash + 1).toLongOrNull() ?: return null
        if (start < 0 || end < start || total <= 0) return null
        return Triple(start, end, total)
    }

    fun after(written: Long, total: Long): AfterDecision = when {
        total > 0 && written > total -> AfterDecision(DownloadAction.Fail, total, discardPart = true)
        total > 0 && written == total -> AfterDecision(DownloadAction.Complete, total)
        else -> AfterDecision(DownloadAction.Continue, total)
    }
}

/** 本地下载记录状态；`failed` 保留半成品，用户可再试。 */
enum class LocalDownloadStatus(val wire: String) {
    Part("part"),
    Done("done"),
    Failed("failed"),
    ;

    companion object {
        fun fromWire(value: String?): LocalDownloadStatus? = entries.firstOrNull { it.wire == value }
    }
}

data class LocalDownloadRecord(
    val kind: String = "file",
    val id: Long = 0,
    val fileName: String = "",
    val localPath: String = "",
    val size: Long = 0,
    val bytesReceived: Long = 0,
    val etag: String? = null,
    val status: LocalDownloadStatus = LocalDownloadStatus.Part,
    val updatedAt: String = "",
)

data class ServerDownload(
    val fileId: Long,
    val fileName: String,
    val firstDownloadTime: String = "",
)

/** 展示状态。M1 只有 zh-CN，文案先留在这里；多语言时搬进 Strings。 */
enum class DownloadDisplayStatus(val text: String) {
    Cloud("云端记录"),
    Done("已下载"),
    Partial("未完成"),
    Failed("失败"),
}

data class MergedDownload(
    val fileId: Long,
    val fileName: String,
    val localPath: String?,
    val displayStatus: DownloadDisplayStatus,
    val isEbook: Boolean = false,
)

/**
 * 「我的下载」合并：服务端行做骨架（有题目 / 科目等元信息），本地记录按 id 挂上去；
 * 本地有而服务端没有的追加在后、按 updatedAt 倒序（否则刚下完的东西会凭空消失）。
 */
fun mergeDownloads(
    server: List<ServerDownload>,
    records: List<LocalDownloadRecord>,
    ebookRecords: List<LocalDownloadRecord> = emptyList(),
): List<MergedDownload> {
    val allRecords = (records + ebookRecords).sortedByDescending { it.updatedAt }
    fun newestFor(id: Long): LocalDownloadRecord? = allRecords.firstOrNull { it.id == id }
    fun statusOf(record: LocalDownloadRecord?): DownloadDisplayStatus = when (record?.status) {
        null -> DownloadDisplayStatus.Cloud
        LocalDownloadStatus.Done -> DownloadDisplayStatus.Done
        LocalDownloadStatus.Part -> DownloadDisplayStatus.Partial
        LocalDownloadStatus.Failed -> DownloadDisplayStatus.Failed
    }

    val serverIds = server.map { it.fileId }.toSet()
    val merged = server.map { row ->
        val record = newestFor(row.fileId)
        MergedDownload(
            fileId = row.fileId,
            fileName = row.fileName,
            localPath = record?.localPath?.takeIf { it.isNotBlank() },
            displayStatus = statusOf(record),
        )
    }.toMutableList()

    // 本地有、服务端没有：追加在后，按 updatedAt 倒序（allRecords 已排序）。
    allRecords.asSequence()
        .filter { it.id !in serverIds && it.kind == "file" }
        .distinctBy { it.id }
        .forEach { record ->
            merged += MergedDownload(
                fileId = record.id,
                fileName = record.fileName,
                localPath = record.localPath.takeIf { it.isNotBlank() },
                displayStatus = statusOf(record),
            )
        }
    return merged
}

/** downloads.json 的形状（ARCHITECTURE.md 核心原则 5）；写入整体重写，不做增量。 */
object DownloadRecordsCodec {
    fun decode(element: JsonElement?): List<LocalDownloadRecord> {
        val obj = element as? JsonObject ?: return emptyList()
        val rows = obj["records"] as? JsonArray ?: return emptyList()
        return rows.mapNotNull { row ->
            val record = row as? JsonObject ?: return@mapNotNull null
            val status = LocalDownloadStatus.fromWire(record.string("status")) ?: LocalDownloadStatus.Part
            LocalDownloadRecord(
                kind = record.string("kind") ?: "file",
                id = record.long("id"),
                fileName = record.string("fileName").orEmpty(),
                localPath = record.string("localPath").orEmpty(),
                size = record.long("size"),
                bytesReceived = record.long("bytesReceived"),
                etag = record.string("etag"),
                status = status,
                updatedAt = record.string("updatedAt").orEmpty(),
            )
        }
    }

    fun encode(records: List<LocalDownloadRecord>): JsonObject = JsonObject(
        mapOf(
            "records" to JsonArray(
                records.map { record ->
                    JsonObject(
                        buildMap {
                            put("kind", JsonPrimitive(record.kind))
                            put("id", JsonPrimitive(record.id))
                            put("fileName", JsonPrimitive(record.fileName))
                            put("localPath", JsonPrimitive(record.localPath))
                            put("size", JsonPrimitive(record.size))
                            put("bytesReceived", JsonPrimitive(record.bytesReceived))
                            record.etag?.let { put("etag", JsonPrimitive(it)) }
                            put("status", JsonPrimitive(record.status.wire))
                            put("updatedAt", JsonPrimitive(record.updatedAt))
                        },
                    )
                },
            ),
        ),
    )

    private fun JsonObject.string(key: String): String? =
        (this[key] as? JsonPrimitive)?.takeUnless { it is JsonNull }?.contentOrNull

    private fun JsonObject.long(key: String): Long =
        (this[key] as? JsonPrimitive)?.contentOrNull?.toLongOrNull() ?: 0L
}
