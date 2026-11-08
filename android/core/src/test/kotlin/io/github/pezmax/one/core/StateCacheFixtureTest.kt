package io.github.pezmax.one.core

import io.github.pezmax.one.core.SpecFixtures.expect
import io.github.pezmax.one.core.SpecFixtures.input
import io.github.pezmax.one.core.cache.CacheAction
import io.github.pezmax.one.core.cache.CacheEntry
import io.github.pezmax.one.core.cache.LocalCacheState
import io.github.pezmax.one.core.cache.StateResponse
import io.github.pezmax.one.core.cache.cacheHit
import io.github.pezmax.one.core.cache.decideCacheAction
import io.github.pezmax.one.core.cache.loadCache
import io.github.pezmax.one.core.cache.shouldApplyResponse
import io.github.pezmax.one.core.json.arrayOrNull
import io.github.pezmax.one.core.json.booleanOrNull
import io.github.pezmax.one.core.json.intOrNull
import io.github.pezmax.one.core.json.stringOrNull
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Test

/** spec/fixtures/state-cache/cases.json —— 键 / 可用性 / 落地 / 过期。 */
class StateCacheFixtureTest {

    @Test
    fun `key 用例`() {
        SpecFixtures.cases("fixtures/state-cache/cases.json")
            .filter { it.input().str("kind") == "key" }
            .forEach { case ->
                val input = case.input()
                val entries = (input["entries"] as? kotlinx.serialization.json.JsonArray)?.map { it as JsonObject }?.map { obj ->
                    CacheEntry(
                        serverUrl = obj.str("serverUrl"),
                        resource = obj.str("resource"),
                        hash = obj.str("hash"),
                        payload = obj["payload"],
                    )
                }.orEmpty()
                val hit = cacheHit(input.str("serverUrl"), input.str("resource"), entries)
                assertEquals(case.str("id"), case.expect().booleanOrNull("hit") ?: false, hit)
            }
    }

    @Test
    fun `load 用例`() {
        SpecFixtures.cases("fixtures/state-cache/cases.json")
            .filter { it.input().str("kind") == "load" }
            .forEach { case ->
                val entry = case.input()["entry"]
                val usable = when (entry) {
                    null, is JsonNull -> false
                    is JsonObject -> loadCache(entry.toString()) != null
                    else -> false
                }
                assertEquals("${case.str("id")} usable", case.expect().booleanOrNull("usable") ?: false, usable)
            }
    }

    @Test
    fun `apply 用例`() {
        SpecFixtures.cases("fixtures/state-cache/cases.json")
            .filter { it.input().str("kind") == "apply" }
            .forEach { case ->
                val input = case.input()
                val previous = input.obj("previous")
                val response = input.obj("response")
                val action = decideCacheAction(
                    previous = LocalCacheState(
                        hash = previous.stringOrNull("hash").orEmpty(),
                        usable = previous.booleanOrNull("usable") ?: false,
                    ),
                    response = StateResponse(
                        success = response.str("kind") == "success",
                        unchanged = response.booleanOrNull("unchanged") ?: false,
                        hash = response.stringOrNull("hash").orEmpty(),
                        data = response["data"]?.takeUnless { it is JsonNull },
                    ),
                )
                val expected = case.expect()
                when (val wire = expected.str("action")) {
                    "store" -> {
                        assertEquals(case.str("id"), true, action is CacheAction.Store)
                        assertEquals(case.str("id") + " hash", expected.stringOrNull("hash"), (action as CacheAction.Store).hash)
                    }
                    "keep" -> {
                        assertEquals(case.str("id"), true, action is CacheAction.Keep)
                        assertEquals(
                            case.str("id") + " hash",
                            expected.stringOrNull("hash"),
                            (action as CacheAction.Keep).hash,
                        )
                    }
                    "refetch" -> assertEquals(case.str("id"), CacheAction.Refetch, action)
                    else -> error("未知 action: $wire")
                }
            }
    }

    @Test
    fun `stale 用例`() {
        SpecFixtures.cases("fixtures/state-cache/cases.json")
            .filter { it.input().str("kind") == "stale" }
            .forEach { case ->
                val input = case.input()
                val apply = shouldApplyResponse(
                    appliedSeq = (input.intOrNull("appliedSeq") ?: 0).toLong(),
                    incomingSeq = (input.intOrNull("incomingSeq") ?: 0).toLong(),
                )
                assertEquals(case.str("id"), case.expect().booleanOrNull("apply") ?: false, apply)
            }
    }

    @Test
    fun `可用缓存的编码解码可以往返`() {
        val entry = CacheEntry("https://pz.example.com", "tree", "h1", JsonNull)
        assertNull(loadCache(io.github.pezmax.one.core.cache.encodeCache(entry)))
        val good = CacheEntry(
            "https://pz.example.com",
            "tree",
            "h1",
            kotlinx.serialization.json.JsonArray(emptyList<JsonElement>()),
        )
        assertNotNull(loadCache(io.github.pezmax.one.core.cache.encodeCache(good)))
    }

    @Test
    fun `坏缓存不当空列表`() {
        assertNull(loadCache("{\"hash\":\"h1\",\"payload\":[]"))
        assertNull(loadCache(""))
        assertNull(loadCache("nonsense"))
        assertFalse(shouldApplyResponse(5, 4))
    }

    @Test
    fun `空 payload 数组是可用缓存`() {
        val loaded = loadCache("{\"serverUrl\":\"x\",\"resource\":\"tree\",\"hash\":\"h\",\"payload\":[]}")
        assertNotNull(loaded)
        assertEquals(0, (loaded!!.payload as kotlinx.serialization.json.JsonArray).size)
    }
}
