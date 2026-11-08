package io.github.pezmax.one

import android.content.Context
import io.github.pezmax.one.core.net.PezMaxClient
import io.github.pezmax.one.data.AppSettings
import io.github.pezmax.one.data.CredentialStore
import io.github.pezmax.one.data.LocalStore

/**
 * 单例服务定位器：与 Ncrust 一致，不引 DI 框架。
 * 只装生命周期等于进程的东西；按账号 / 按请求的状态不要放这里。
 */
object ServiceLocator {
    lateinit var appContext: Context
        private set

    lateinit var settings: AppSettings
        private set

    lateinit var credentials: CredentialStore
        private set

    lateinit var localStore: LocalStore
        private set

    lateinit var client: PezMaxClient
        private set

    fun install(context: Context) {
        if (::client.isInitialized) return
        appContext = context.applicationContext
        localStore = LocalStore(appContext)
        settings = AppSettings(appContext)
        credentials = CredentialStore(appContext)
        client = PezMaxClient(
            baseUrlProvider = { settings.baseUrl },
            tokenProvider = { credentials.token },
            onSessionExpired = { credentials.clear() },
        )
    }
}
