plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
}

val defaultBaseUrl: String = providers.gradleProperty("pezmax.baseUrl").getOrElse("")

android {
    namespace = "io.github.pezmax.one"
    compileSdk = 36

    defaultConfig {
        applicationId = "io.github.pezmax.one"
        minSdk = 24
        targetSdk = 36
        // 版本号唯一来源（docs/RELEASES.md「版本号」）；1.1.0 起，与 egui 1.0.0 区分。
        versionCode = 1
        versionName = "1.1.0"
        // D13：默认后端地址随构建写入，设置里可改，改后清会话。地址不写死在 Kotlin 里。
        buildConfigField("String", "DEFAULT_BASE_URL", "\"$defaultBaseUrl\"")
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    kotlinOptions {
        jvmTarget = "11"
    }
    buildFeatures {
        compose = true
        buildConfig = true
    }
    composeOptions {
        kotlinCompilerExtensionVersion = libs.versions.composeCompiler.get()
    }
    packaging {
        resources.excludes += setOf("/META-INF/{AL2.0,LGPL2.1}", "/META-INF/versions/9/previous-compilation-data.bin")
    }
}

dependencies {
    implementation(project(":core"))

    // Kanesumi —— Kanesumi Design 组件库，组合构建自 ../../Kanesumi-sec-a。
    // controls/structure 的 api 依赖会带出 core/anim，这里只列上游两件。
    implementation("io.github.takahashirinta:kanesumi-controls:0.1.0-SNAPSHOT")
    implementation("io.github.takahashirinta:kanesumi-structure:0.1.0-SNAPSHOT")

    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.lifecycle.viewmodel.compose)
    implementation(libs.androidx.lifecycle.runtime.compose)

    implementation(platform(libs.compose.bom))
    implementation(libs.compose.foundation)
    implementation(libs.compose.ui)
    implementation(libs.compose.ui.graphics)
    implementation(libs.compose.ui.tooling.preview)
    implementation(libs.compose.material.icons.extended)
    debugImplementation(libs.compose.ui.tooling)

    implementation(libs.coil.compose)
    implementation(libs.okhttp)

    testImplementation(libs.junit)
}
