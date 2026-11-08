pluginManagement {
    repositories {
        google {
            content {
                includeGroupByRegex("com\\.android.*")
                includeGroupByRegex("com\\.google.*")
                includeGroupByRegex("androidx.*")
            }
        }
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "PezMaxOne"
include(":app")
include(":core")

// Kanesumi-sec-a 通过组合构建接入（与 Ncrust 同一手法）：本仓 Gradle 根在 android/，
// 所以相对路径比 Ncrust 多一层。显式 dependencySubstitution 让 Kanesumi 不需要
// maven-publish 或 group/version 配置。Kanesumi 上 Maven Central 后整块删除，
// app/build.gradle.kts 里的坐标一字不改就能切（android/AGENTS.md「引用 Kanesumi」）。
includeBuild("../../Kanesumi-sec-a") {
    dependencySubstitution {
        substitute(module("io.github.takahashirinta:kanesumi-core"))
            .using(project(":kanesumi-core"))
        substitute(module("io.github.takahashirinta:kanesumi-anim"))
            .using(project(":kanesumi-anim"))
        substitute(module("io.github.takahashirinta:kanesumi-controls"))
            .using(project(":kanesumi-controls"))
        substitute(module("io.github.takahashirinta:kanesumi-structure"))
            .using(project(":kanesumi-structure"))
    }
}
