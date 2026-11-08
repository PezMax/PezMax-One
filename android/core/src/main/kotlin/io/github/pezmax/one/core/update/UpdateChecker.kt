package io.github.pezmax.one.core.update

import io.github.pezmax.one.core.json.asArrayOrNull
import io.github.pezmax.one.core.json.arrayOrNull
import io.github.pezmax.one.core.json.booleanOrNull
import io.github.pezmax.one.core.json.asObjectOrNull
import io.github.pezmax.one.core.json.stringOrNull
import kotlinx.serialization.json.JsonElement
import java.net.URI

/**
 * 应用内更新检查（spec/fixtures/update/、docs/RELEASES.md）。
 *
 * CrateStack 每个班次装载各平台的最新资产，所以只能按资产名扫：不看 Latest 徽章、
 * 不看数组顺序、不借别的版本的附属文件。版本只比严格更高——原地换包不会被发现，
 * 这是刻意设计。
 */
data class AppVersion(val segments: List<Int>) : Comparable<AppVersion> {
    val normalized: String
        get() {
            val padded = segments.toMutableList()
            while (padded.size < 3) padded += 0
            while (padded.size > 3 && padded.last() == 0) padded.removeAt(padded.lastIndex)
            return padded.joinToString(".")
        }

    override fun compareTo(other: AppVersion): Int {
        for (i in 0 until 4) {
            val left = segments.getOrElse(i) { 0 }
            val right = other.segments.getOrElse(i) { 0 }
            if (left != right) return left.compareTo(right)
        }
        return 0
    }

    companion object {
        /** 最多 4 段、允许前缀 v；解析不出来返回 null（绝不猜）。 */
        fun parse(raw: String): AppVersion? {
            val text = raw.trim().removePrefix("v").removePrefix("V")
            if (text.isEmpty()) return null
            val parts = text.split('.')
            if (parts.size !in 1..4) return null
            val numbers = parts.map { it.toIntOrNull() ?: return null }
            if (numbers.any { it < 0 }) return null
            return AppVersion(numbers)
        }
    }
}

enum class UpdatePlatform(val wire: String, val extensions: List<String>) {
    Windows("windows", listOf("appinstaller", "msix", "cer")),
    Linux("linux", listOf("deb", "pkg.tar.zst")),
    Android("android", listOf("apk")),
}

data class ReleaseAsset(val name: String, val downloadUrl: String)

data class ReleaseInfo(
    val tagName: String,
    val htmlUrl: String,
    val draft: Boolean,
    val prerelease: Boolean,
    val assets: List<ReleaseAsset>,
)

data class UpdateInfo(
    val version: AppVersion,
    val releaseTag: String,
    val releasePageUrl: String,
    /** 键是扩展名；该版本没有这类资产时为 null，绝不借用别的版本。 */
    val assets: Map<String, String?>,
)

object UpdateChecker {

    private const val PREFIX = "pezmax-one-"

    fun parseReleases(element: JsonElement?): List<ReleaseInfo> =
        element.asArrayOrNull()?.mapNotNull { row ->
            val obj = row.asObjectOrNull() ?: return@mapNotNull null
            ReleaseInfo(
                tagName = obj.stringOrNull("tag_name").orEmpty(),
                htmlUrl = obj.stringOrNull("html_url").orEmpty(),
                draft = obj.booleanOrNull("draft") ?: false,
                prerelease = obj.booleanOrNull("prerelease") ?: false,
                assets = obj.arrayOrNull("assets")?.mapNotNull { asset ->
                    val a = asset.asObjectOrNull() ?: return@mapNotNull null
                    ReleaseAsset(
                        name = a.stringOrNull("name").orEmpty(),
                        downloadUrl = a.stringOrNull("browser_download_url").orEmpty(),
                    )
                } ?: emptyList(),
            )
        } ?: emptyList()

    data class ParsedAsset(val version: AppVersion, val extension: String, val url: String)

    /** 解析单个资产名；不合规范（旧命名、签名文件、架构不符）返回 null。 */
    fun parseAsset(name: String, platform: UpdatePlatform, arch: String): ParsedAsset? {
        if (!name.startsWith(PREFIX)) return null
        val extension = platform.extensions
            .sortedByDescending { it.length }
            .firstOrNull { name.endsWith(".$it") } ?: return null
        val stem = name.substring(PREFIX.length, name.length - extension.length - 1)
        val parts = stem.split('-')
        if (parts.size != 3) return null
        val (assetPlatform, assetArch, rawVersion) = parts
        if (assetPlatform != platform.wire || assetArch != arch) return null
        val version = AppVersion.parse(rawVersion) ?: return null
        return ParsedAsset(version, extension, "")
    }

    /**
     * 在所有可见 release 里挑本平台、本架构、可安装扩展名的资产，取严格大于当前版本的最高者。
     * @param mirrorBase 非空时把下载地址换成「镜像基址 + 原 URL 的 path 与 query」。
     */
    fun select(
        releases: List<ReleaseInfo>,
        currentVersion: AppVersion,
        platform: UpdatePlatform,
        arch: String,
        mirrorBase: String? = null,
    ): UpdateInfo? {
        data class Candidate(val version: AppVersion, val release: ReleaseInfo, val assets: Map<String, String>)

        val candidates = releases.asSequence()
            .filter { !it.draft && !it.prerelease }
            .mapNotNull { release ->
                val matched = release.assets.mapNotNull { asset ->
                    val parsed = parseAsset(asset.name, platform, arch) ?: return@mapNotNull null
                    Triple(parsed.version, parsed.extension, asset.downloadUrl)
                }
                if (matched.isEmpty()) return@mapNotNull null
                val best = matched.maxOf { it.first }
                val urls = matched.filter { it.first == best }.associate { it.second to it.third }
                Candidate(best, release, urls)
            }
            .filter { it.version > currentVersion }
            .sortedByDescending { it.version }
            .firstOrNull() ?: return null

        val assets = platform.extensions.associateWith { extension ->
            candidates.assets[extension]?.let { url -> mirror(url, mirrorBase) }
        }
        return UpdateInfo(
            version = candidates.version,
            releaseTag = candidates.release.tagName,
            releasePageUrl = candidates.release.htmlUrl,
            assets = assets,
        )
    }

    /** 镜像只改「从哪儿下」：丢掉原 scheme/host，接在镜像基址后。 */
    fun mirror(url: String, mirrorBase: String?): String {
        if (mirrorBase.isNullOrBlank()) return url
        val uri = runCatching { URI(url) }.getOrNull() ?: return url
        val base = mirrorBase.trim().trimEnd('/')
        val path = uri.rawPath.orEmpty()
        val query = uri.rawQuery?.let { "?$it" }.orEmpty()
        return base + path + query
    }
}

/** GitHub 响应顶层是数组；`assets` 里的其他字段（size / content_type）不参与判定。 */
fun parseReleasesOrEmpty(element: JsonElement?): List<ReleaseInfo> = UpdateChecker.parseReleases(element)
