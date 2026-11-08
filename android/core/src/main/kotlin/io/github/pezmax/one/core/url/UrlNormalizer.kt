package io.github.pezmax.one.core.url

import java.net.URI

/**
 * 文件 URL 过渡归一（spec/fixtures/url/README.md）。
 *
 * 规则自上而下、先命中先返回：空白 → 空串；data:/blob: 原样；minio:// 拼 BaseUrl；
 * 相对路径拼 BaseUrl；内网 host 的绝对地址换 origin；公网地址一个字符都不动。
 *
 * 这是**过渡层**：上游 ETL 把库里的内网 host 与 minio:// 清完之后，规则 3、5 整段删除，
 * 1、2、4、6 保留（属于正常的 URL 拼装）。
 */
object UrlNormalizer {

    private val INTERNAL_HOSTS = setOf("localhost", "127.0.0.1", "minio", "server")

    fun normalize(rawUrl: String, baseUrl: String): String {
        val url = rawUrl.trim()
        if (url.isEmpty()) return ""
        val lower = url.lowercase()

        // data:/blob: 是本端生成的对象，拼 BaseUrl 只会拼坏。
        if (lower.startsWith("data:") || lower.startsWith("blob:")) return rawUrl

        if (lower.startsWith("minio://")) {
            val rest = url.substring("minio://".length)
            return joinBase(baseUrl, rest)
        }

        val isAbsolute = lower.startsWith("http://") || lower.startsWith("https://")
        if (!isAbsolute) return joinBase(baseUrl, url)

        val uri = runCatching { URI(url) }.getOrNull() ?: return rawUrl
        val host = uri.host?.lowercase() ?: return rawUrl
        if (!isInternalHost(host)) return rawUrl

        // 只换 origin：path / query（含签名参数）必须逐字保留，且不重新编码。
        val base = runCatching { URI(baseUrl.trim()) }.getOrNull() ?: return rawUrl
        val scheme = base.scheme ?: return rawUrl
        val authority = base.rawAuthority ?: return rawUrl
        val path = uri.rawPath.orEmpty().let { if (it.startsWith("/")) it else "/$it" }
        val query = uri.rawQuery?.let { "?$it" }.orEmpty()
        return "$scheme://$authority$path$query"
    }

    private fun joinBase(baseUrl: String, path: String): String {
        val base = baseUrl.trim().trimEnd('/')
        if (base.isEmpty()) return path
        return "$base/${path.trimStart('/')}"
    }

    private fun isInternalHost(host: String): Boolean =
        host in INTERNAL_HOSTS || host.startsWith("172.") || host.startsWith("10.") || host.startsWith("192.168.")
}

/** 静态资源（封面 / 头像）与直链下载共用同一套归一；流式端点不走这里。 */
fun normalizeFileUrl(rawUrl: String, baseUrl: String): String = UrlNormalizer.normalize(rawUrl, baseUrl)
