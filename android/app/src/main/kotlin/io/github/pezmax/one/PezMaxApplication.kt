package io.github.pezmax.one

import android.app.Application

class PezMaxApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        ServiceLocator.install(this)
    }
}
