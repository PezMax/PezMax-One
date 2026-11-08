package io.github.pezmax.one.core

import io.github.pezmax.one.core.SpecFixtures.expect
import io.github.pezmax.one.core.SpecFixtures.input
import io.github.pezmax.one.core.json.arrayOrNull
import io.github.pezmax.one.core.json.objectOrNull
import io.github.pezmax.one.core.json.stringOrNull
import io.github.pezmax.one.core.update.AppVersion
import io.github.pezmax.one.core.update.ReleaseAsset
import io.github.pezmax.one.core.update.ReleaseInfo
import io.github.pezmax.one.core.update.UpdateChecker
import io.github.pezmax.one.core.update.UpdatePlatform
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** spec/fixtures/update/cases.json —— 资产挑选、版本比较、镜像前缀。 */
class UpdateFixtureTest {

    @Test
    fun `更新挑选逐条一致`() {
        val cases = SpecFixtures.cases("fixtures/update/cases.json")
        assertTrue(cases.size >= 23)
        cases.forEach { case ->
            val input = case.input()
            val platform = UpdatePlatform.entries.first { it.wire == input.str("platform") }
            val releases = (input["releases"] as kotlinx.serialization.json.JsonArray).map { row ->
                val obj = row as JsonObject
                ReleaseInfo(
                    tagName = obj.stringOrNull("tag_name").orEmpty(),
                    htmlUrl = obj.stringOrNull("html_url").orEmpty(),
                    draft = obj["draft"]?.let { (it as JsonPrimitive).content.toBoolean() } ?: false,
                    prerelease = obj["prerelease"]?.let { (it as JsonPrimitive).content.toBoolean() } ?: false,
                    assets = obj.arrayOrNull("assets")?.map { asset ->
                        val a = asset as JsonObject
                        ReleaseAsset(a.stringOrNull("name").orEmpty(), a.stringOrNull("browser_download_url").orEmpty())
                    } ?: emptyList(),
                )
            }
            val info = UpdateChecker.select(
                releases = releases,
                currentVersion = AppVersion.parse(input.str("currentVersion"))!!,
                platform = platform,
                arch = input.str("arch"),
                mirrorBase = input.stringOrNull("mirrorBase"),
            )
            val expectElement = case["expect"]
            if (expectElement == null || expectElement is kotlinx.serialization.json.JsonNull) {
                assertNull(case.str("id"), info)
                return@forEach
            }
            assertTrue(case.str("id") + " 应该找到更新", info != null)
            val expectObj = expectElement as JsonObject
            assertEquals(case.str("id") + " version", expectObj.stringOrNull("version"), info!!.version.normalized)
            assertEquals(case.str("id") + " tag", expectObj.stringOrNull("releaseTag"), info.releaseTag)
            assertEquals(case.str("id") + " page", expectObj.stringOrNull("releasePageUrl"), info.releasePageUrl)
            val expectedAssets = expectObj["assets"] as JsonObject
            expectedAssets.forEach { (key, value) ->
                val expectedUrl = (value as? JsonPrimitive)?.contentOrNull()
                assertEquals("${case.str("id")} asset $key", expectedUrl, info.assets[key])
            }
            // 该平台每一类扩展名都要出现在结果里（没有时为 null），不得缺键。
            platform.extensions.forEach { ext ->
                assertTrue("${case.str("id")} 缺少扩展名 $ext", info.assets.containsKey(ext))
            }
        }
    }

    @Test
    fun `版本归一与比较`() {
        assertEquals("1.2.0", AppVersion.parse("v1.2")!!.normalized)
        assertEquals("1.2.0", AppVersion.parse("1.2.0.0")!!.normalized)
        assertEquals("1.2.0.5", AppVersion.parse("1.2.0.5")!!.normalized)
        assertNull(AppVersion.parse("1.2.0.3.4"))
        assertNull(AppVersion.parse("abc"))
        assertTrue(AppVersion.parse("1.10")!! > AppVersion.parse("1.9.0")!!)
        assertEquals(0, AppVersion.parse("1.2")!!.compareTo(AppVersion.parse("1.2.0.0")!!))
    }

    @Test
    fun `旧命名与签名文件一律不算资产`() {
        assertNull(UpdateChecker.parseAsset("pezmax-one-1.0.0-windows-x86_64.msi", UpdatePlatform.Windows, "x86_64"))
        assertNull(UpdateChecker.parseAsset("pezmax-one-windows-x86_64-9.0.0.msix.asc", UpdatePlatform.Windows, "x86_64"))
        assertNull(UpdateChecker.parseAsset("app-release.apk", UpdatePlatform.Android, "universal"))
        assertNull(UpdateChecker.parseAsset("pezmax-one-windows-amd64-2.0.0.msix", UpdatePlatform.Windows, "x86_64"))
        assertTrue(UpdateChecker.parseAsset("pezmax-one-linux-x86_64-1.3.0.pkg.tar.zst", UpdatePlatform.Linux, "x86_64") != null)
    }

    @Test
    fun `镜像只换下载源不改选中版本`() {
        assertEquals(
            "https://mirror.example.net/gh/a/b.apk",
            UpdateChecker.mirror("https://github.com/a/b.apk", "https://mirror.example.net/gh/"),
        )
        assertEquals(
            "https://github.com/a/b.apk",
            UpdateChecker.mirror("https://github.com/a/b.apk", null),
        )
    }

    private fun JsonObject.str(key: String): String = stringOrNull(key).orEmpty()
    private fun JsonObject.obj(key: String): JsonObject = objectOrNull(key) ?: error("missing $key")

    private fun kotlinx.serialization.json.JsonPrimitive.contentOrNull(): String? =
        if (this is kotlinx.serialization.json.JsonNull) null else content
}
