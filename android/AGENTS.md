# AGENTS.md —— PezMax-One for Android（Kotlin + Compose + Kanesumi-sec-a）

本文件是 `android/` 目录的权威指南。仓库级约定见仓根 `AGENTS.md`，跨端架构见 `docs/ARCHITECTURE.md`。

**状态**（2026-10-19）：规划稿，尚无代码；排在 Windows 与 Linux 之后（D14）。

> 样板是兄弟仓库 Ncrust 的 Android 端（`../../Ncrust/AGENTS.md`），本文没写的工程细节按 Ncrust 的做法办。

## 立项决策

| 决策 | 结论 | 原因 |
|---|---|---|
| UI | Jetpack Compose + **Kanesumi-sec-a**；**不依赖 `material3`**，Material 只允许 `material-icons-extended`（图标） | 与 Ncrust 一致；Kanesumi 组件已覆盖外壳、列表、对话框、底部表单 |
| 引用 Kanesumi | composite build：`android/settings.gradle.kts` 里 `includeBuild("../../Kanesumi-sec-a")` + 显式 `dependencySubstitution`；Kanesumi 上 Maven Central 后删除该块 | Ncrust 已验证；注意本项目 Gradle 根在 `android/`，相对路径比 Ncrust 多一层 |
| 工具链 | 与 Kanesumi-sec-a / Ncrust 对齐（compileSdk 36、Kotlin 1.9.24、Compose BOM 2024.12.01、AGP 8.5.0、Java 11），minSdk 24（D7） | composite build 要求版本相容；先跟随，不单独升级 |
| 业务核心 | `core/` 包（纯 Kotlin，无 Android 依赖），JVM 单测读 `../../spec/fixtures/` | 与另两端对称 |
| 状态 | 无 DI 框架，单例服务定位；ViewModel + `StateFlow`；高频值在叶子 composable 或 draw scope 里收集 | 同 Ncrust |
| 动画 | Kanesumi `sokuou` 预设（`SokuouTweens` / `SokuouPresets`）；单一 progress + `graphicsLayer`，不用 `animateFloatAsState` 驱动视觉 | GPU 零重组 |
| 网络 | OkHttp（+ Retrofit 可选）；Bearer 拦截器；401 → 会话失效事件 | |
| i18n | 运行时 `Strings` 数据类 + `LocalStrings`（Ncrust 同款），M1 只做 zh-CN | 界面文字不许写死 |

## 布局

- 窄屏（< 600dp）：`MetroShell` + `MetroBottomNav`（首页 / 资料库 / 社区 / 我的），资料库内用 `MetroTabRow` 切资料、电子书、书签。
- 宽屏（≥ 600dp）：200dp `MetroSidebar`，内容区 `ResponsiveContent` 居中限宽；结构与桌面端导航面板一致。
- 下载进度是底部叠层（`MetroBottomStack`），所有可滚动内容用统一的底部 inset 作 `contentPadding`，**不在列表末尾手加 Spacer**。
- 加载沿用 Ncrust 模式：有缓存先渲染、后台刷新写回、`Crossfade(SokuouTweens.CoverFade)`，不整屏替换为加载器。

## 阅读器

| 格式 | 实现 |
|---|---|
| PDF | `android.graphics.pdf.PdfRenderer`（需本地文件，先下载到 `cacheDir`）；网格 / 连续两视图；位图按显示尺寸渲染、LRU 内存缓存 + 磁盘缓存 |
| 图片 | Compose 图片加载（选型与 Ncrust 一致） |
| TXT / Markdown | `Text` 分段；Markdown 复用 Ncrust `MarkdownText` 的思路（自研，不引大型库） |
| EPUB | M0 二选一：Readium Kotlin Toolkit（BSD-3）或 WebView + 本地打包阅读器页（与 Windows 端同一套页面，可共享前端资产） |
| MOBI / AZW3 | 下载后 `Intent.ACTION_VIEW` + `FileProvider` 交给外部应用 |

## 平台集成

| 需求 | 实现 |
|---|---|
| 凭据 | Android Keystore 生成的 AES-GCM 密钥加密后存私有文件（`androidx.security:security-crypto` 已弃用，不再引入） |
| 下载 | 自管 OkHttp 流式 + `.part` + `Range`（状态机在 core），长任务用前台服务或 WorkManager；保存到用户选的目录（SAF `OpenDocumentTree`）或公共「下载」（MediaStore） |
| 上传 | SAF `OpenMultipleDocuments` / `OpenDocumentTree`（文件夹上传），multipart 流式上传 |
| 通知 | 下载进度与完成通知（API 33+ 请求 `POST_NOTIFICATIONS`）；服务端弹窗通知在应用内展示，不转系统通知 |
| 更新 | 照搬 Ncrust `update/`：CrateStack 资产 `pezmax-one-android-universal-<版本>.apk`、断点下载、签名与 `versionCode` 校验、`FileProvider` 安装 |
| 外链 | 书签外链用 Custom Tabs 打开 |

## M0 验证项（Android）

| # | 验证项 | 通过标准 |
|---|---|---|
| A1 | composite build | `android/` 下 `./gradlew assembleDebug` 引用兄弟 `Kanesumi-sec-a` 编过 |
| A2 | `PdfRenderer` 大文件 | 100 页以上 PDF 连续视图在中端机滚动流畅、无 OOM |
| A3 | EPUB 方案 | 选定方案能离线打开 EPUB，翻页 / 目录 / CJK 排版正常，APK 增量可接受 |
