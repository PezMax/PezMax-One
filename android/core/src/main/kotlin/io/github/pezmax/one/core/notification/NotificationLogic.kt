package io.github.pezmax.one.core.notification

import io.github.pezmax.one.core.json.longOr
import io.github.pezmax.one.core.json.parseJsonOrNull
import io.github.pezmax.one.core.json.stringOrNull
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive

/**
 * 通知的客户端判定（spec/fixtures/notification/cases.json）。
 * 时间窗口与通道筛选服务端已经做了；客户端只判「这条该不该出现在这个通道 / 弹不弹 / 已读」。
 * 注意：滚动通道没有 type 4 定向，客户端必须自己挡掉别人的下架通知（上游补上后可删）。
 */
data class NoticeRef(val userId: Long, val notifyId: Long)

data class NoticeState(
    val reads: Set<NoticeRef> = emptySet(),
    val popups: Set<NoticeRef> = emptySet(),
) {
    fun isRead(userId: Long, notifyId: Long): Boolean = NoticeRef(userId, notifyId) in reads
    fun hasPopped(userId: Long, notifyId: Long): Boolean = NoticeRef(userId, notifyId) in popups

    fun markRead(userId: Long, notifyId: Long): NoticeState = copy(reads = reads + NoticeRef(userId, notifyId))
    fun markPopped(userId: Long, notifyId: Long): NoticeState = copy(popups = popups + NoticeRef(userId, notifyId))
}

data class Notice(
    val notifyId: Long,
    val notifyType: String = "",
    val displayMode: String = "",
    val uploadUserId: Long = 0,
    val maintenanceStartTime: String = "",
    val materialId: Long = 0,
    val title: String = "",
    val content: String = "",
    val scrollTimeInterval: Int = 0,
)

enum class NoticeChannel(val wire: String) {
    Popup("popup"),
    Scroll("scroll"),
}

object NotificationLogic {

    /** type 4（资料下架）只给材料上传者本人；其余类型来了就展示。 */
    fun shouldShow(notice: Notice, channel: NoticeChannel, userId: Long): Boolean {
        if (notice.notifyType != "4") return true
        return notice.uploadUserId == userId
    }

    /** 弹窗队列：沿用服务端顺序，type 1 / 4 一次性的跳过已弹过的，type 2 / 3 / 5 每次都弹。 */
    fun popupPlan(rows: List<Notice>, seen: Set<NoticeRef>, userId: Long): List<Long> {
        val picked = LinkedHashSet<Long>()
        rows.forEach { notice ->
            if (!shouldShow(notice, NoticeChannel.Popup, userId)) return@forEach
            val oneShot = notice.notifyType == "1" || notice.notifyType == "4"
            if (oneShot && NoticeRef(userId, notice.notifyId) in seen) return@forEach
            picked += notice.notifyId
        }
        return picked.toList()
    }

    sealed interface FeedAction {
        data class Keep(val hash: String) : FeedAction
        data class Replace(val hash: String) : FeedAction
        data object Refetch : FeedAction
        data object KeepAndWarn : FeedAction
    }

    fun feedAction(previousHash: String, hasPayload: Boolean, unchanged: Boolean, newHash: String, data: JsonArray?): FeedAction =
        when {
            unchanged && hasPayload -> FeedAction.Keep(previousHash.ifBlank { newHash })
            unchanged -> FeedAction.Refetch
            data == null -> FeedAction.KeepAndWarn
            else -> FeedAction.Replace(newHash.ifBlank { previousHash })
        }

    /** 维护通知在 maintenanceStartTime 之后不可关闭；其余都能关。 */
    fun isDismissible(notice: Notice, now: String): Boolean {
        if (notice.notifyType != "3") return true
        if (notice.maintenanceStartTime.isBlank()) return true
        return now < notice.maintenanceStartTime
    }

    /** 文件坏了当空集合：通知状态丢了顶多再弹一次，比一打开就报错轻。 */
    fun decodeState(raw: String?): NoticeState {
        val obj = raw?.let { parseJsonOrNull(it) as? JsonObject } ?: return NoticeState()
        return NoticeState(
            reads = decodeRefs(obj["reads"]),
            popups = decodeRefs(obj["popups"]),
        )
    }

    private fun decodeRefs(element: kotlinx.serialization.json.JsonElement?): Set<NoticeRef> =
        (element as? JsonArray)?.mapNotNull { row ->
            val obj = row as? JsonObject ?: return@mapNotNull null
            val userId = obj.longOr("userId")
            val notifyId = obj.longOr("notifyId")
            if (userId <= 0 || notifyId <= 0) null else NoticeRef(userId, notifyId)
        }?.toSet() ?: emptySet()

    fun encodeState(state: NoticeState): String = buildString {
        append("{\"reads\":")
        append(refsJson(state.reads))
        append(",\"popups\":")
        append(refsJson(state.popups))
        append('}')
    }

    private fun refsJson(refs: Set<NoticeRef>): String = JsonArray(
        refs.map { ref ->
            JsonObject(
                mapOf(
                    "userId" to JsonPrimitive(ref.userId),
                    "notifyId" to JsonPrimitive(ref.notifyId),
                ),
            )
        },
    ).toString()

    /** payload → 模型。字段缺省按空 / 0，不因上游加字段失败。 */
    fun noticeFromJson(obj: JsonObject): Notice = Notice(
        notifyId = obj.longOr("notifyId"),
        notifyType = obj.stringOrNull("notifyType").orEmpty(),
        displayMode = obj.stringOrNull("displayMode").orEmpty(),
        uploadUserId = obj.longOr("uploadUserId"),
        maintenanceStartTime = obj.stringOrNull("maintenanceStartTime").orEmpty(),
        materialId = obj.longOr("materialId"),
        title = obj.stringOrNull("title").orEmpty(),
        content = obj.stringOrNull("content").orEmpty(),
        scrollTimeInterval = obj.stringOrNull("scrollTimeInterval")?.toIntOrNull() ?: 0,
    )

    fun noticesFromArray(array: JsonArray?): List<Notice> =
        array?.mapNotNull { it as? JsonObject }?.map(::noticeFromJson) ?: emptyList()
}
