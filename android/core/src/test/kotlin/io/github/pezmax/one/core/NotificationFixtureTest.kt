package io.github.pezmax.one.core

import io.github.pezmax.one.core.SpecFixtures.expect
import io.github.pezmax.one.core.SpecFixtures.input
import io.github.pezmax.one.core.json.arrayOrNull
import io.github.pezmax.one.core.json.booleanOrNull
import io.github.pezmax.one.core.json.intOrNull
import io.github.pezmax.one.core.json.longOrNull
import io.github.pezmax.one.core.json.objectOrNull
import io.github.pezmax.one.core.json.stringOrNull
import io.github.pezmax.one.core.notification.Notice
import io.github.pezmax.one.core.notification.NoticeChannel
import io.github.pezmax.one.core.notification.NoticeRef
import io.github.pezmax.one.core.notification.NoticeState
import io.github.pezmax.one.core.notification.NotificationLogic
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** spec/fixtures/notification/cases.json —— 通道 / 弹窗队列 / 已读 / 缓存 / 可关闭 / 存储。 */
class NotificationFixtureTest {

    private val cases = SpecFixtures.cases("fixtures/notification/cases.json")

    @Test
    fun `channel 用例`() {
        cases.filter { it.input().str("kind") == "channel" }.forEach { case ->
            val input = case.input()
            val notice = input.obj("notification").toNotice()
            val show = NotificationLogic.shouldShow(
                notice = notice,
                channel = NoticeChannel.entries.first { it.wire == input.str("channel") },
                userId = input.longOrNull("userId") ?: 0L,
            )
            assertEquals(case.str("id"), case.expect().booleanOrNull("show") ?: false, show)
        }
    }

    @Test
    fun `popup-plan 用例`() {
        cases.filter { it.input().str("kind") == "popup-plan" }.forEach { case ->
            val input = case.input()
            val rows = (input["rows"] as JsonArray).map { (it as JsonObject).toNotice() }
            val seen = (input["seen"] as JsonArray).map {
                val obj = it as JsonObject
                NoticeRef(obj.longOrNull("userId") ?: 0L, obj.longOrNull("notifyId") ?: 0L)
            }.toSet()
            val plan = NotificationLogic.popupPlan(rows, seen, input.longOrNull("userId") ?: 0L)
            val expected = (case.expect()["notifyIds"] as JsonArray).map { (it as JsonPrimitive).content.toLong() }
            assertEquals(case.str("id"), expected, plan)
        }
    }

    @Test
    fun `read-set 用例`() {
        cases.filter { it.input().str("kind") == "read-set" }.forEach { case ->
            val input = case.input()
            val reads = (input["reads"] as JsonArray).map {
                val obj = it as JsonObject
                NoticeRef(obj.longOrNull("userId") ?: 0L, obj.longOrNull("notifyId") ?: 0L)
            }.toSet()
            val read = NoticeState(reads = reads).isRead(
                userId = input.longOrNull("userId") ?: 0L,
                notifyId = input.longOrNull("notifyId") ?: 0L,
            )
            assertEquals(case.str("id"), case.expect().booleanOrNull("read") ?: false, read)
        }
    }

    @Test
    fun `feed-cache 用例`() {
        cases.filter { it.input().str("kind") == "feed-cache" }.forEach { case ->
            val input = case.input()
            val previous = input.obj("previous")
            val response = input.obj("response")
            val action = NotificationLogic.feedAction(
                previousHash = previous.stringOrNull("hash").orEmpty(),
                hasPayload = previous.booleanOrNull("hasPayload") ?: false,
                unchanged = response.booleanOrNull("unchanged") ?: false,
                newHash = response.stringOrNull("hash").orEmpty(),
                data = response["data"] as? JsonArray,
            )
            val expect = case.expect()
            val wire = when (action) {
                is NotificationLogic.FeedAction.Keep -> "keep"
                is NotificationLogic.FeedAction.Replace -> "replace"
                NotificationLogic.FeedAction.Refetch -> "refetch"
                NotificationLogic.FeedAction.KeepAndWarn -> "keep-and-warn"
            }
            assertEquals(case.str("id"), expect.stringOrNull("action"), wire)
            expect.stringOrNull("hash")?.let { expectedHash ->
                val actual = when (action) {
                    is NotificationLogic.FeedAction.Keep -> action.hash
                    is NotificationLogic.FeedAction.Replace -> action.hash
                    else -> null
                }
                assertEquals(case.str("id") + " hash", expectedHash, actual)
            }
        }
    }

    @Test
    fun `dismissible 用例`() {
        cases.filter { it.input().str("kind") == "dismissible" }.forEach { case ->
            val input = case.input()
            val dismissible = NotificationLogic.isDismissible(
                notice = input.obj("notification").toNotice(),
                now = input.str("now"),
            )
            assertEquals(case.str("id"), case.expect().booleanOrNull("dismissible") ?: false, dismissible)
        }
    }

    @Test
    fun `storage 用例`() {
        cases.filter { it.input().str("kind") == "storage" }.forEach { case ->
            val raw = case.input()["raw"]?.takeUnless { it is JsonNull }?.let { (it as JsonPrimitive).content }
            val state = NotificationLogic.decodeState(raw)
            val expect = case.expect()
            assertEquals(case.str("id") + " reads", expect.intOrNull("reads"), state.reads.size)
            assertEquals(case.str("id") + " popups", expect.intOrNull("popups"), state.popups.size)
            expect.arrayOrNull("hasRead")?.let { pair ->
                val userId = (pair[0] as JsonPrimitive).content.toLong()
                val notifyId = (pair[1] as JsonPrimitive).content.toLong()
                assertTrue(case.str("id"), state.isRead(userId, notifyId))
            }
            expect.arrayOrNull("hasPopped")?.let { pair ->
                val userId = (pair[0] as JsonPrimitive).content.toLong()
                val notifyId = (pair[1] as JsonPrimitive).content.toLong()
                assertTrue(case.str("id"), state.hasPopped(userId, notifyId))
            }
            expect.arrayOrNull("notRead")?.let { pair ->
                val userId = (pair[0] as JsonPrimitive).content.toLong()
                val notifyId = (pair[1] as JsonPrimitive).content.toLong()
                assertTrue(case.str("id"), !state.isRead(userId, notifyId))
            }
        }
    }

    @Test
    fun `本地状态可以往返编码`() {
        val state = NoticeState()
            .markRead(7, 10)
            .markPopped(7, 11)
        val decoded = NotificationLogic.decodeState(NotificationLogic.encodeState(state))
        assertEquals(state, decoded)
    }

    private fun JsonObject.toNotice(): Notice = Notice(
        notifyId = longOrNull("notifyId") ?: 0L,
        notifyType = stringOrNull("notifyType").orEmpty(),
        displayMode = stringOrNull("displayMode").orEmpty(),
        uploadUserId = longOrNull("uploadUserId") ?: 0L,
        maintenanceStartTime = stringOrNull("maintenanceStartTime").orEmpty(),
        materialId = longOrNull("materialId") ?: 0L,
    )

    private fun JsonObject.obj(key: String): JsonObject = objectOrNull(key) ?: error("missing $key")
}
