package io.github.pezmax.one.core.net

/**
 * 端点目录（spec/api/endpoints.md）。客户端只拼 `/datum/…`，不追加 `/api`。
 * 每个函数返回相对路径，BaseUrl 由 [PezMaxClient] 拼接。
 */
object Endpoints {
    // 账户
    const val CAPTCHA = "/datum/user/captchaImage"
    const val LOGIN = "/datum/user/login"
    const val REGISTER = "/datum/user/register"
    const val LOGOUT = "/datum/user/logout"
    const val USER_INFO = "/datum/user/getInfo"
    const val SECURITY_QUESTIONS = "/datum/user/securityQuestions"
    const val RESET_BY_SECURITY = "/datum/user/resetPasswordBySecurity"

    // 个人中心
    const val PROFILE = "/datum/desktop/user/profile"
    const val PROFILE_STATS = "/datum/desktop/user/profile/stats"
    const val PROFILE_USERNAME = "/datum/desktop/user/profile/username"
    const val PROFILE_AVATAR = "/datum/desktop/user/profile/avatar"
    const val PROFILE_PASSWORD_VERIFY = "/datum/desktop/user/profile/password/verify"
    const val PROFILE_PASSWORD = "/datum/desktop/user/profile/password"

    // 资料
    const val FILE_TREE = "/datum/file/tree"
    const val FILE_LIST = "/datum/file/list"
    const val FILE_SEARCH = "/datum/file/search"
    const val FILE_SUBJECTS = "/datum/file/subjects"
    const val FILE_SCHOOLS = "/datum/file/schools"
    const val FILE_CONTENT = "/datum/file/content"
    const val FILE = "/datum/file"
    fun fileDetail(fileId: Long) = "/datum/file/$fileId"

    // 电子书
    const val EBOOK_LIST = "/datum/ebook/list"
    const val EBOOK_SUBJECTS = "/datum/ebook/subjects"
    const val EBOOK_CONTENT = "/datum/ebook/content"
    const val EBOOK_FAVORITE = "/datum/ebook/favorite"
    const val EBOOK_UPLOAD = "/datum/ebook/upload"
    fun ebookDetail(ebookId: Long) = "/datum/ebook/$ebookId"

    // 书签
    const val BOOKMARK_LIST = "/datum/bookmark/list"
    const val BOOKMARK = "/datum/bookmark"
    fun bookmarkDetail(bookmarkId: Long) = "/datum/bookmark/$bookmarkId"

    // 收藏
    const val FAVORITE = "/datum/favorite"
    fun favoriteStatus(fileId: Long) = "/datum/favorite/$fileId"
    fun myFileFavorites(userId: Long) = "/datum/desktop/favorite/list/$userId"
    fun removeFileFavorite(userId: Long, fileId: Long) = "/datum/desktop/favorite/$userId/$fileId"
    fun myEbookFavorites(userId: Long) = "/datum/desktop/ebook/favorite/list/$userId"

    // 下载
    const val DOWNLOAD_FILE = "/datum/download/file"
    const val DOWNLOAD_EBOOK = "/datum/download/ebook"
    fun myDownloads(userId: Long) = "/datum/desktop/download/list/$userId"
    fun deleteDownload(userId: Long, fileId: Long) = "/datum/desktop/download/$userId/$fileId"

    // 社区 / 通知
    const val RANK = "/datum/user/rank"
    fun userPublic(userId: Long) = "/datum/user/$userId"
    const val NOTIFY_POPUP = "/system/notification/user/popup"
    const val NOTIFY_SCROLL = "/system/notification/user/scroll"
    const val NOTIFY_LIST = "/datum/notification/list"

    // 应用更新（不走 BaseUrl）
    const val GITHUB_RELEASES = "https://api.github.com/repos/PezMax/PezMax-One/releases"
}
