package io.github.pezmax.one.core.model

import io.github.pezmax.one.core.json.arrayOrNull
import io.github.pezmax.one.core.json.asObjectOrNull
import kotlinx.serialization.json.contentOrNull
import io.github.pezmax.one.core.json.boolOr
import io.github.pezmax.one.core.json.idLike
import io.github.pezmax.one.core.json.intOr
import io.github.pezmax.one.core.json.longOr
import io.github.pezmax.one.core.json.objectOrNull
import io.github.pezmax.one.core.json.stringOrNull
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject

/**
 * `/datum/…` 的 payload 模型。字段名就是契约（spec/api/endpoints.md），缺字段按缺省处理；
 * 上游加字段不影响解析——这是不用反射映射的原因。
 */

data class FilePayload(
    val id: Long = 0,
    val fileId: Long = 0,
    val fileName: String = "",
    val fileUrl: String = "",
    val fileSize: Long = 0,
    val fileFormat: String = "",
    val fileYear: Int = 0,
    val fileType: Int = 0,
    val fileSchool: String = "",
    val fileSubject: String = "",
    val fileStatus: Int = 0,
    val userId: Long = 0,
    val remark: String = "",
    val createTime: String = "",
) {
    /** 列表 / 详情里的 id 是数字；树的叶子会把 id 覆盖成 "file-N"，所以优先读 fileId。 */
    val key: Long get() = if (fileId > 0) fileId else id

    val extensionHint: String
        get() = fileFormat.ifBlank { fileName.substringAfterLast('.', "").lowercase() }

    companion object {
        fun fromJson(obj: JsonObject): FilePayload = FilePayload(
            id = obj.longOr("id"),
            fileId = obj.longOr("fileId"),
            fileName = obj.stringOrNull("fileName") ?: obj.stringOrNull("label").orEmpty(),
            fileUrl = obj.stringOrNull("fileUrl").orEmpty(),
            fileSize = obj.longOr("fileSize"),
            fileFormat = obj.stringOrNull("fileFormat").orEmpty(),
            fileYear = obj.intOr("fileYear"),
            fileType = obj.intOr("fileType"),
            fileSchool = obj.stringOrNull("fileSchool").orEmpty(),
            fileSubject = obj.stringOrNull("fileSubject").orEmpty(),
            fileStatus = obj.intOr("fileStatus"),
            userId = obj.longOr("userId"),
            remark = obj.stringOrNull("remark").orEmpty(),
            createTime = obj.stringOrNull("createTime").orEmpty(),
        )
    }
}

data class EbookPayload(
    val id: Long = 0,
    val ebookId: Long = 0,
    val ebookName: String = "",
    val author: String = "",
    val publisher: String = "",
    val coverUrl: String = "",
    val ebookUrl: String = "",
    val ebookSize: Long = 0,
    val ebookFormat: String = "",
    val ebookSubject: String = "",
    val ebookType: Int = 0,
    val ebookStatus: Int = 0,
    val userId: Long = 0,
    val remark: String = "",
) {
    val key: Long get() = if (ebookId > 0) ebookId else id

    companion object {
        fun fromJson(obj: JsonObject): EbookPayload = EbookPayload(
            id = obj.longOr("id"),
            ebookId = obj.longOr("ebookId"),
            ebookName = obj.stringOrNull("ebookName") ?: obj.stringOrNull("label").orEmpty(),
            author = obj.stringOrNull("author").orEmpty(),
            publisher = obj.stringOrNull("publisher").orEmpty(),
            coverUrl = obj.stringOrNull("coverUrl").orEmpty(),
            ebookUrl = obj.stringOrNull("ebookUrl").orEmpty(),
            ebookSize = obj.longOr("ebookSize"),
            ebookFormat = obj.stringOrNull("ebookFormat").orEmpty(),
            ebookSubject = obj.stringOrNull("ebookSubject").orEmpty(),
            ebookType = obj.intOr("ebookType"),
            ebookStatus = obj.intOr("ebookStatus"),
            userId = obj.longOr("userId"),
            remark = obj.stringOrNull("remark").orEmpty(),
        )
    }
}

data class BookmarkPayload(
    val id: Long = 0,
    val bookmarkId: Long = 0,
    val userId: Long = 0,
    val url: String = "",
    val title: String = "",
    val description: String = "",
    val coverImage: String = "",
    val subject: String = "",
    val resourceType: String = "",
    val collection: String = "",
    val status: Int = 0,
) {
    val key: Long get() = if (bookmarkId > 0) bookmarkId else id

    companion object {
        fun fromJson(obj: JsonObject): BookmarkPayload = BookmarkPayload(
            id = obj.longOr("id"),
            bookmarkId = obj.idLike("bookmarkId") ?: 0L,
            userId = obj.longOr("userId"),
            url = obj.stringOrNull("url").orEmpty(),
            title = obj.stringOrNull("title").orEmpty(),
            description = obj.stringOrNull("description").orEmpty(),
            coverImage = obj.stringOrNull("coverImage").orEmpty(),
            subject = obj.stringOrNull("subject").orEmpty(),
            resourceType = obj.stringOrNull("resourceType").orEmpty(),
            collection = obj.stringOrNull("collection").orEmpty(),
            status = obj.intOr("status"),
        )
    }
}

data class UserProfile(
    val userId: Long = 0,
    val userName: String = "",
    val nickName: String = "",
    val avatar: String = "",
    val count: Int = 0,
    val status: String = "1",
) {
    /** 上游 nickName 恒等于 userName，界面不要假设有独立昵称。 */
    val displayName: String get() = nickName.ifBlank { userName }

    companion object {
        fun fromJson(obj: JsonObject): UserProfile = UserProfile(
            userId = obj.longOr("userId"),
            userName = obj.stringOrNull("userName").orEmpty(),
            nickName = obj.stringOrNull("nickName").orEmpty(),
            avatar = obj.stringOrNull("avatar").orEmpty(),
            count = obj.intOr("count"),
            status = obj.stringOrNull("status") ?: "1",
        )
    }
}

data class ProfileStats(
    val uploadCount: Int = 0,
    val downloadCount: Int = 0,
    val favoriteCount: Int = 0,
    val fileFavoriteCount: Int = 0,
    val bookmarkFavoriteCount: Int = 0,
) {
    companion object {
        fun fromJson(obj: JsonObject): ProfileStats = ProfileStats(
            uploadCount = obj.intOr("uploadCount"),
            downloadCount = obj.intOr("downloadCount"),
            favoriteCount = obj.intOr("favoriteCount"),
            fileFavoriteCount = obj.intOr("fileFavoriteCount"),
            bookmarkFavoriteCount = obj.intOr("bookmarkFavoriteCount"),
        )
    }
}

data class RankEntry(
    val userId: Long = 0,
    val userName: String = "",
    val avatar: String = "",
    val count: Int = 0,
    val uploads: Int = 0,
) {
    companion object {
        fun fromJson(obj: JsonObject): RankEntry = RankEntry(
            userId = obj.longOr("userId"),
            userName = obj.stringOrNull("userName").orEmpty(),
            avatar = obj.stringOrNull("avatar").orEmpty(),
            count = obj.intOr("count"),
            uploads = obj.intOr("uploads"),
        )
    }
}

data class SubjectCount(val subject: String = "", val total: Int = 0) {
    companion object {
        // 注意上游字段首字母大写（结构体没有 JSON tag）。
        fun fromJson(obj: JsonObject): SubjectCount = SubjectCount(
            subject = obj.stringOrNull("Subject") ?: obj.stringOrNull("subject").orEmpty(),
            total = obj.intOr("Total").takeIf { it > 0 } ?: obj.intOr("total"),
        )
    }
}

data class CaptchaInfo(
    val captchaEnabled: Boolean = true,
    val uuid: String = "",
    val img: String = "",
) {
    companion object {
        fun fromJson(element: JsonElement?): CaptchaInfo? {
            val obj = element.asObjectOrNull() ?: return null
            return CaptchaInfo(
                captchaEnabled = obj.boolOr("captchaEnabled", true),
                uuid = obj.stringOrNull("uuid").orEmpty(),
                img = obj.stringOrNull("img").orEmpty(),
            )
        }
    }
}

data class LoginResult(val token: String = "", val expiresAt: Long = 0) {
    companion object {
        fun fromJson(element: JsonElement?): LoginResult? {
            val obj = element.asObjectOrNull() ?: return null
            val token = obj.stringOrNull("token").orEmpty()
            if (token.isBlank()) return null
            return LoginResult(token = token, expiresAt = obj.longOr("expiresAt"))
        }
    }
}

data class SessionUser(val profile: UserProfile, val roles: List<String> = emptyList()) {
    companion object {
        fun fromJson(element: JsonElement?): SessionUser? {
            val root = element.asObjectOrNull() ?: return null
            val user = root.objectOrNull("user") ?: return null
            val roles = root.arrayOrNull("roles")?.mapNotNull { (it as? kotlinx.serialization.json.JsonPrimitive)?.contentOrNull } ?: emptyList()
            return SessionUser(UserProfile.fromJson(user), roles)
        }
    }
}

/** 下载记录行的服务端部分（`/datum/desktop/download/list/{userId}`）。 */
data class ServerDownloadRow(
    val file: FilePayload,
    val firstDownloadTime: String = "",
) {
    companion object {
        fun fromJson(obj: JsonObject): ServerDownloadRow = ServerDownloadRow(
            file = FilePayload.fromJson(obj),
            firstDownloadTime = obj.stringOrNull("firstDownloadTime").orEmpty(),
        )
    }
}

fun JsonObject.toFilePayload(): FilePayload = FilePayload.fromJson(this)
fun JsonObject.toEbookPayload(): EbookPayload = EbookPayload.fromJson(this)

