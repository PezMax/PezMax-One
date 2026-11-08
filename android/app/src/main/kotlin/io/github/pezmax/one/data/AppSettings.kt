package io.github.pezmax.one.data

import android.content.Context
import android.content.SharedPreferences
import io.github.pezmax.one.BuildConfig

/**
 * 非敏感设置。凭据不进这里（见 [CredentialStore]）。
 * 改服务器地址后必须清会话：D13 要求，且旧站点的树 / 排行缓存不能复用（state-cache 夹具）。
 */
class AppSettings(private val prefs: SharedPreferences) {

    constructor(context: Context) : this(
        context.getSharedPreferences("pezmax.settings", Context.MODE_PRIVATE),
    )

    var baseUrl: String
        get() = prefs.getString(KEY_BASE_URL, null)?.takeIf { it.isNotBlank() }
            ?: BuildConfig.DEFAULT_BASE_URL
        set(value) {
            prefs.edit().putString(KEY_BASE_URL, value.trim().trimEnd('/')).apply()
        }

    /** 已登录用户的 id：仅用于 `/datum/desktop/…/{userId}` 路径与本地已读隔离。 */
    var userId: Long
        get() = prefs.getLong(KEY_USER_ID, 0L)
        set(value) = prefs.edit().putLong(KEY_USER_ID, value).apply()

    var userName: String
        get() = prefs.getString(KEY_USER_NAME, "").orEmpty()
        set(value) = prefs.edit().putString(KEY_USER_NAME, value).apply()

    /** 强调色预设 id：cobalt / spruce / crimson / amber / violet（D11）。 */
    var accent: String
        get() = prefs.getString(KEY_ACCENT, "cobalt") ?: "cobalt"
        set(value) = prefs.edit().putString(KEY_ACCENT, value).apply()

    /** 深浅色：system / dark / light。 */
    var themeMode: String
        get() = prefs.getString(KEY_THEME, "dark") ?: "dark"
        set(value) = prefs.edit().putString(KEY_THEME, value).apply()

    var mirrorEnabled: Boolean
        get() = prefs.getBoolean(KEY_MIRROR, false)
        set(value) = prefs.edit().putBoolean(KEY_MIRROR, value).apply()

    var mirrorBase: String
        get() = prefs.getString(KEY_MIRROR_BASE, "").orEmpty()
        set(value) = prefs.edit().putString(KEY_MIRROR_BASE, value.trim()).apply()

    /** 更新检查节流：24 小时一次（docs/RELEASES.md「应用内更新」）。 */
    var lastUpdateCheck: Long
        get() = prefs.getLong(KEY_LAST_UPDATE, 0L)
        set(value) = prefs.edit().putLong(KEY_LAST_UPDATE, value).apply()

    /** 用户点过「稍后」的版本：同版本不再提示。 */
    var skippedVersion: String
        get() = prefs.getString(KEY_SKIPPED_VERSION, "").orEmpty()
        set(value) = prefs.edit().putString(KEY_SKIPPED_VERSION, value).apply()

    fun clearSession() {
        prefs.edit().remove(KEY_USER_ID).remove(KEY_USER_NAME).apply()
    }

    private companion object {
        const val KEY_BASE_URL = "baseUrl"
        const val KEY_USER_ID = "userId"
        const val KEY_USER_NAME = "userName"
        const val KEY_ACCENT = "accent"
        const val KEY_THEME = "theme"
        const val KEY_MIRROR = "mirrorEnabled"
        const val KEY_MIRROR_BASE = "mirrorBase"
        const val KEY_LAST_UPDATE = "lastUpdateCheck"
        const val KEY_SKIPPED_VERSION = "skippedVersion"
    }
}
