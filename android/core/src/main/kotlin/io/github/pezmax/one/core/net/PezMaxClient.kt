package io.github.pezmax.one.core.net

import io.github.pezmax.one.core.json.PezJson
import io.github.pezmax.one.core.json.stringOrNull
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.JsonObject
import okhttp3.HttpUrl
import okhttp3.HttpUrl.Companion.toHttpUrlOrNull
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.MultipartBody
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.Response
import java.io.IOException
import java.util.concurrent.TimeUnit

/**
 * `/datum/…` 的 HTTP 适配层：拼 URL、带 Bearer、把响应交给 [Envelope]。
 * 业务判定一行都不在这里——这里只做「发出去、读回来」。
 *
 * BaseUrl 与 token 都是 provider 而不是值：改服务器地址 / 登录 / 登出后
 * 不需要重建客户端，也不会有一份旧 token 缓存在别处。
 */
class PezMaxClient(
    private val baseUrlProvider: () -> String,
    private val tokenProvider: () -> String?,
    private val onSessionExpired: () -> Unit = {},
    private val http: OkHttpClient = defaultHttp(),
) {
    companion object {
        private val JSON_MEDIA = "application/json; charset=utf-8".toMediaType()

        fun defaultHttp(): OkHttpClient = OkHttpClient.Builder()
            .connectTimeout(15, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .writeTimeout(60, TimeUnit.SECONDS)
            .followRedirects(true)
            .build()
    }

    val raw: OkHttpClient get() = http

    fun baseUrl(): String = normalizeBaseUrl(baseUrlProvider())

    fun url(path: String, query: Map<String, String?> = emptyMap()): HttpUrl? {
        val base = baseUrl().takeIf { it.isNotBlank() } ?: return null
        val builder = (base + path).toHttpUrlOrNull()?.newBuilder() ?: return null
        query.forEach { (key, value) -> if (value != null && value.isNotEmpty()) builder.addQueryParameter(key, value) }
        return builder.build()
    }

    suspend fun get(path: String, query: Map<String, String?> = emptyMap()): ApiResult = call("GET", path, query = query)

    suspend fun post(path: String, body: JsonObject? = null, query: Map<String, String?> = emptyMap()): ApiResult =
        call("POST", path, query = query, jsonBody = body)

    suspend fun put(path: String, body: JsonObject? = null, query: Map<String, String?> = emptyMap()): ApiResult =
        call("PUT", path, query = query, jsonBody = body)

    suspend fun delete(path: String, query: Map<String, String?> = emptyMap()): ApiResult = call("DELETE", path, query = query)

    suspend fun call(
        method: String,
        path: String,
        query: Map<String, String?> = emptyMap(),
        jsonBody: JsonObject? = null,
    ): ApiResult = withContext(Dispatchers.IO) {
        val url = url(path, query) ?: return@withContext ApiResult.TransportError(0, "", true)
        val body: RequestBody? = jsonBody?.toString()?.toRequestBody(JSON_MEDIA)
        val request = requestBuilder(method, url, body).build()
        try {
            http.newCall(request).execute().use { response ->
                val text = response.body?.string().orEmpty()
                // 401 由 Envelope 判定；这里只把「清会话」回调出去，保证并发 401 只触发一次。
                if (response.code == 401) onSessionExpired()
                Envelope.parse(
                    status = response.code,
                    contentType = response.header("Content-Type").orEmpty(),
                    body = text,
                )
            }
        } catch (e: IOException) {
            ApiResult.TransportError(0, e.message.orEmpty(), true)
        }
    }

    /**
     * 文件流（下载 / 预览）。调用方负责 `use { }` 关闭；401 同样立刻清会话。
     * 不在这里读 body，避免把 200MB 的试卷读进内存。
     */
    suspend fun stream(
        path: String,
        query: Map<String, String?> = emptyMap(),
        headers: Map<String, String> = emptyMap(),
    ): Response? = withContext(Dispatchers.IO) {
        val url = url(path, query) ?: return@withContext null
        val builder = requestBuilder("GET", url, null)
        headers.forEach { (key, value) -> builder.header(key, value) }
        val response = try {
            http.newCall(builder.build()).execute()
        } catch (_: IOException) {
            return@withContext null
        }
        if (response.code == 401) onSessionExpired()
        response
    }

    /** multipart 上传：字段顺序固定，文件在最后（服务端按字段名读，不依赖顺序，但便于抓包比对）。 */
    suspend fun upload(
        path: String,
        fields: Map<String, String>,
        fileName: String,
        contentType: String,
        source: () -> okhttp3.RequestBody,
    ): ApiResult = withContext(Dispatchers.IO) {
        val url = url(path) ?: return@withContext ApiResult.TransportError(0, "", true)
        val multipart = MultipartBody.Builder().setType(MultipartBody.FORM).apply {
            fields.forEach { (key, value) -> addFormDataPart(key, value) }
            addFormDataPart("file", fileName, source())
        }.build()
        val request = requestBuilder("POST", url, multipart).build()
        try {
            http.newCall(request).execute().use { response ->
                val text = response.body?.string().orEmpty()
                if (response.code == 401) onSessionExpired()
                Envelope.parse(response.code, response.header("Content-Type").orEmpty(), text)
            }
        } catch (e: IOException) {
            ApiResult.TransportError(0, e.message.orEmpty(), true)
        }
    }

    private fun requestBuilder(method: String, url: HttpUrl, body: RequestBody?): Request.Builder {
        val builder = Request.Builder()
            .url(url)
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
        tokenProvider()?.takeIf { it.isNotBlank() }?.let { builder.header("Authorization", "Bearer $it") }
        return builder.method(method, body)
    }

    /** 登录 / 注册 / 找回密码的 401 不是「会话失效」：这些请求不带 token，也不该弹回登录层。 */
    suspend fun anonymousCall(method: String, path: String, jsonBody: JsonObject? = null, query: Map<String, String?> = emptyMap()): ApiResult =
        withContext(Dispatchers.IO) {
            val url = url(path, query) ?: return@withContext ApiResult.TransportError(0, "", true)
            val body = jsonBody?.toString()?.toRequestBody(JSON_MEDIA)
            val request = Request.Builder()
                .url(url)
                .header("Accept", "application/json")
                .method(method, body)
                .build()
            try {
                http.newCall(request).execute().use { response ->
                    val text = response.body?.string().orEmpty()
                    val contentType = response.header("Content-Type").orEmpty()
                    // 匿名端点上的 401 按普通业务错误显示，不触发会话失效。
                    if (response.code == 401) {
                        ApiResult.BusinessError(401, failureMessage(text), false)
                    } else {
                        Envelope.parse(response.code, contentType, text)
                    }
                }
            } catch (e: IOException) {
                ApiResult.TransportError(0, e.message.orEmpty(), true)
            }
        }

    private fun failureMessage(body: String): String {
        val obj = io.github.pezmax.one.core.json.parseJsonOrNull(body) as? JsonObject ?: return ""
        return obj.stringOrNull("msg") ?: obj.stringOrNull("message") ?: ""
    }
}

/** BaseUrl 去掉末尾斜杠；空串表示还没配置，调用方出「请先设置服务器」。 */
fun normalizeBaseUrl(raw: String): String = raw.trim().trimEnd('/')

/** 把任意 JSON 文本转成对象；构造请求体时用，失败时给一个空对象（服务端会回 400）。 */
fun jsonObject(text: String): JsonObject =
    (io.github.pezmax.one.core.json.parseJsonOrNull(text) as? JsonObject) ?: JsonObject(emptyMap())

fun jsonText(obj: JsonObject): String = PezJson.encodeToString(JsonObject.serializer(), obj)
