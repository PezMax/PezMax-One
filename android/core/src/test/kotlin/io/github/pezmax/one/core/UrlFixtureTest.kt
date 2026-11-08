package io.github.pezmax.one.core

import io.github.pezmax.one.core.SpecFixtures.expect
import io.github.pezmax.one.core.SpecFixtures.input
import io.github.pezmax.one.core.json.stringOrNull
import io.github.pezmax.one.core.url.UrlNormalizer
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** spec/fixtures/url/cases.json —— 过渡归一必须逐字符相等。 */
class UrlFixtureTest {

    @Test
    fun `21 条 URL 归一用例逐字符一致`() {
        val cases = SpecFixtures.cases("fixtures/url/cases.json")
        assertTrue(cases.size >= 20)
        cases.forEach { case ->
            val actual = UrlNormalizer.normalize(case.input().str("url"), case.input().str("baseUrl"))
            assertEquals(case.str("id"), case.expect().stringOrNull("url").orEmpty(), actual)
        }
    }

    @Test
    fun `流式端点不走这条规则`() {
        // 下载 / 预览带 Bearer 与 Range，由客户端直接拼 /datum/download/*，不经 normalize。
        assertEquals("", UrlNormalizer.normalize("", "https://pz.example.com"))
    }
}
