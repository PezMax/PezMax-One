plugins {
    alias(libs.plugins.kotlin.jvm)
    alias(libs.plugins.kotlin.serialization)
}

// 纯 Kotlin / JVM：不依赖 Android，不依赖 Kanesumi。业务判定与状态机都在这里，
// 单测直接读 ../../spec/fixtures（相对 android/core，即仓根 spec/），不复制夹具。
kotlin {
    jvmToolchain(17)
    compilerOptions {
        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_11)
    }
}

java {
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = JavaVersion.VERSION_11
}

dependencies {
    api(libs.kotlinx.serialization.json)
    api(libs.kotlinx.coroutines.core)
    api(libs.okhttp)

    testImplementation(libs.junit)
    testImplementation(kotlin("test"))
}

tasks.test {
    // 夹具路径按模块目录解析（Gradle 默认工作目录即模块目录）。
    systemProperty("pezmax.specDir", rootProject.file("../spec").absolutePath)
    testLogging {
        events("failed")
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
    }
}
